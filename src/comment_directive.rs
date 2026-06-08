use once_cell::sync::Lazy;
use regex::Regex;
use swc_core::common::{BytePos, Span};
use swc_core::plugin::errors::HANDLER;

static DIRECTIVE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(lingui-(?:set|reset))(?:\s|$)(.*)").unwrap());
static TOKEN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\s+|(\w+)(?:="([^"]*)")?"#).unwrap());

fn is_lingui_directive_prefix(comment: &str) -> bool {
    comment.starts_with("lingui-set") || comment.starts_with("lingui-reset")
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DirectiveValues {
    pub context: Option<String>,
    pub comment: Option<String>,
    pub id_prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LinguiCommentDirectives {
    directives: Vec<DirectiveEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirectiveEntry {
    pos: BytePos,
    values: DirectiveValues,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DirectiveValueUpdate {
    Set(String),
    Unset,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct DirectiveUpdate {
    context: Option<DirectiveValueUpdate>,
    comment: Option<DirectiveValueUpdate>,
    id_prefix: Option<DirectiveValueUpdate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedDirective {
    reset: bool,
    values: DirectiveUpdate,
}

impl LinguiCommentDirectives {
    pub fn from_source_text(source: &str, start_pos: BytePos) -> Self {
        Self {
            directives: collect_lingui_directives_from_source(source, start_pos),
        }
    }

    pub fn find_for_pos(&self, pos: BytePos) -> Option<&DirectiveValues> {
        find_directive_for_pos(&self.directives, pos)
    }

    pub fn is_empty(&self) -> bool {
        self.directives.is_empty()
    }
}

impl DirectiveValues {
    fn apply_update(&mut self, update: DirectiveUpdate) {
        if let Some(value) = update.context {
            self.context = match value {
                DirectiveValueUpdate::Set(value) => Some(value),
                DirectiveValueUpdate::Unset => None,
            };
        }

        if let Some(value) = update.comment {
            self.comment = match value {
                DirectiveValueUpdate::Set(value) => Some(value),
                DirectiveValueUpdate::Unset => None,
            };
        }

        if let Some(value) = update.id_prefix {
            self.id_prefix = match value {
                DirectiveValueUpdate::Set(value) => Some(value),
                DirectiveValueUpdate::Unset => None,
            };
        }
    }
}

fn parse_value_update(value: &str) -> DirectiveValueUpdate {
    if value.is_empty() {
        DirectiveValueUpdate::Unset
    } else {
        DirectiveValueUpdate::Set(value.into())
    }
}

fn parse_lingui_directive(comment_value: &str) -> Result<Option<ParsedDirective>, String> {
    let trimmed = comment_value.trim();

    let Some(directive_match) = DIRECTIVE_RE.captures(trimmed) else {
        return Ok(None);
    };

    let directive_name = directive_match.get(1).unwrap().as_str();
    let reset = directive_name == "lingui-reset";
    let rest = directive_match
        .get(2)
        .map(|m| m.as_str().trim())
        .unwrap_or_default();

    let mut consumed = 0usize;
    let mut values = DirectiveUpdate::default();
    let mut has_params = false;

    for capture in TOKEN_RE.captures_iter(rest) {
        let full = capture.get(0).unwrap();
        if full.start() != consumed {
            return Err(format!(
                "`{directive_name}` directive has invalid syntax: {trimmed}"
            ));
        }
        consumed = full.end();

        let Some(key) = capture.get(1).map(|m| m.as_str()) else {
            continue;
        };

        let Some(value) = capture.get(2).map(|m| m.as_str()) else {
            return Err(format!(
                "`{directive_name}` directive: \"{key}\" requires a value, e.g. {key}=\"...\""
            ));
        };

        has_params = true;
        let update = parse_value_update(value);

        match key {
            "context" => values.context = Some(update),
            "comment" => values.comment = Some(update),
            "idPrefix" => values.id_prefix = Some(update),
            _ => {
                return Err(format!(
                    "`{directive_name}` directive has unknown param \"{key}\". Valid params: context, comment, idPrefix"
                ));
            }
        }
    }

    if consumed != rest.len() {
        return Err(format!(
            "`{directive_name}` directive has invalid syntax: {trimmed}"
        ));
    }

    if !has_params && !reset {
        return Err(format!(
            "`{directive_name}` directive requires at least one param. Valid params: context, comment, idPrefix"
        ));
    }

    Ok(Some(ParsedDirective { reset, values }))
}

fn find_directive_for_pos(directives: &[DirectiveEntry], pos: BytePos) -> Option<&DirectiveValues> {
    if directives.is_empty() {
        return None;
    }

    let mut lo = 0usize;
    let mut hi = directives.len();

    while lo < hi {
        let mid = (lo + hi) / 2;
        if directives[mid].pos <= pos {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    if lo == 0 {
        None
    } else {
        Some(&directives[lo - 1].values)
    }
}

fn collect_lingui_directives_from_source(source: &str, start_pos: BytePos) -> Vec<DirectiveEntry> {
    let mut directives = Vec::new();
    let mut accumulated = DirectiveValues::default();
    let bytes = source.as_bytes();
    let mut index = 0usize;
    let mut mode = SourceScanMode::Code;
    let mut template_expr_depths: Vec<usize> = vec![];

    while index < bytes.len() {
        match mode {
            SourceScanMode::Code => match bytes[index] {
                b'\'' => {
                    mode = SourceScanMode::SingleQuoted;
                    index += 1;
                }
                b'"' => {
                    mode = SourceScanMode::DoubleQuoted;
                    index += 1;
                }
                b'`' => {
                    mode = SourceScanMode::TemplateText;
                    index += 1;
                }
                b'/' if bytes.get(index + 1) == Some(&b'/') => {
                    let comment_start = BytePos(start_pos.0 + index as u32);
                    let content_start = index + 2;
                    index = content_start;

                    while index < bytes.len() && bytes[index] != b'\n' {
                        index += 1;
                    }

                    parse_source_directive(
                        source[content_start..index].trim(),
                        Span::new(comment_start, BytePos(start_pos.0 + index as u32)),
                        &mut accumulated,
                        &mut directives,
                    );
                }
                b'/' if bytes.get(index + 1) == Some(&b'*') => {
                    let comment_start = BytePos(start_pos.0 + index as u32);
                    let content_start = index + 2;
                    index = content_start;

                    while index + 1 < bytes.len()
                        && !(bytes[index] == b'*' && bytes[index + 1] == b'/')
                    {
                        index += 1;
                    }

                    let content_end = index;
                    if index + 1 < bytes.len() {
                        index += 2;
                    } else {
                        index = bytes.len();
                    }

                    parse_block_directives(
                        &source[content_start..content_end],
                        comment_start,
                        &mut accumulated,
                        &mut directives,
                    );
                }
                b'{' => {
                    if let Some(depth) = template_expr_depths.last_mut() {
                        *depth += 1;
                    }
                    index += 1;
                }
                b'}' => {
                    if let Some(depth) = template_expr_depths.last_mut() {
                        if *depth == 0 {
                            template_expr_depths.pop();
                            mode = SourceScanMode::TemplateText;
                        } else {
                            *depth -= 1;
                        }
                    }
                    index += 1;
                }
                _ => {
                    index += 1;
                }
            },
            SourceScanMode::SingleQuoted => {
                if bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                } else {
                    if bytes[index] == b'\'' {
                        mode = SourceScanMode::Code;
                    }
                    index += 1;
                }
            }
            SourceScanMode::DoubleQuoted => {
                if bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                } else {
                    if bytes[index] == b'"' {
                        mode = SourceScanMode::Code;
                    }
                    index += 1;
                }
            }
            SourceScanMode::TemplateText => match bytes[index] {
                b'\\' => {
                    index = (index + 2).min(bytes.len());
                }
                b'`' => {
                    mode = SourceScanMode::Code;
                    index += 1;
                }
                b'$' if bytes.get(index + 1) == Some(&b'{') => {
                    template_expr_depths.push(0);
                    mode = SourceScanMode::Code;
                    index += 2;
                }
                _ => {
                    index += 1;
                }
            },
        }
    }

    directives
}

enum SourceScanMode {
    Code,
    SingleQuoted,
    DoubleQuoted,
    TemplateText,
}

fn parse_source_directive(
    text: &str,
    span: Span,
    accumulated: &mut DirectiveValues,
    directives: &mut Vec<DirectiveEntry>,
) {
    if !is_lingui_directive_prefix(text) {
        return;
    }

    match parse_lingui_directive(text) {
        Ok(Some(parsed)) => directives.push(apply_directive(parsed, span.lo, accumulated)),
        Ok(None) => {}
        Err(message) => {
            HANDLER.with(|handler| handler.struct_span_err(span, &message).emit());
        }
    }
}

fn apply_directive(
    parsed: ParsedDirective,
    pos: BytePos,
    accumulated: &mut DirectiveValues,
) -> DirectiveEntry {
    let mut values = if parsed.reset {
        DirectiveValues::default()
    } else {
        accumulated.clone()
    };

    values.apply_update(parsed.values);
    *accumulated = values.clone();

    DirectiveEntry { pos, values }
}

fn parse_block_directives(
    content: &str,
    comment_start: BytePos,
    accumulated: &mut DirectiveValues,
    directives: &mut Vec<DirectiveEntry>,
) {
    let mut line_offset = 0u32;

    for segment in content.split_inclusive('\n') {
        let line_with_cr = segment.strip_suffix('\n').unwrap_or(segment);
        let line = line_with_cr.trim_end_matches('\r');
        let trimmed = line.trim_start();
        let leading_ws = (line.len() - trimmed.len()) as u32;

        if line_offset == 0 {
            parse_source_directive(
                trimmed,
                Span::new(
                    comment_start,
                    BytePos(comment_start.0 + 2 + line_with_cr.len() as u32),
                ),
                accumulated,
                directives,
            );
        } else if let Some(after_star) = trimmed.strip_prefix('*') {
            let text = after_star.trim_start();
            let marker_pos = BytePos(comment_start.0 + 2 + line_offset + leading_ws);

            parse_source_directive(
                text,
                Span::new(
                    marker_pos,
                    BytePos(comment_start.0 + 2 + line_offset + line_with_cr.len() as u32),
                ),
                accumulated,
                directives,
            );
        }

        line_offset += segment.len() as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_should_parse_multiple_keys() {
        let parsed =
            parse_lingui_directive(r#" lingui-set context="ctx" comment="cmt" idPrefix="p." "#)
                .unwrap();

        assert_eq!(
            parsed,
            Some(ParsedDirective {
                reset: false,
                values: DirectiveUpdate {
                    context: Some(DirectiveValueUpdate::Set("ctx".into())),
                    comment: Some(DirectiveValueUpdate::Set("cmt".into())),
                    id_prefix: Some(DirectiveValueUpdate::Set("p.".into())),
                }
            })
        );
    }

    #[test]
    fn parse_should_return_none_for_non_directive_comments() {
        assert_eq!(parse_lingui_directive(" some comment ").unwrap(), None);
        assert_eq!(parse_lingui_directive(" i18n ").unwrap(), None);
    }

    #[test]
    fn parse_should_reject_invalid_syntax() {
        let error = parse_lingui_directive(" lingui-set context=single ")
            .expect_err("expected parser to reject invalid syntax");

        assert!(error.contains("requires a value"));
    }

    #[test]
    fn parse_should_reject_unknown_params() {
        let error = parse_lingui_directive(r#" lingui-set unknown="value" "#)
            .expect_err("expected parser to reject unknown params");

        assert!(error.contains("unknown param \"unknown\""));
    }

    #[test]
    fn parse_should_treat_empty_strings_as_unset() {
        let parsed = parse_lingui_directive(r#" lingui-set context="" comment="note" "#).unwrap();

        assert_eq!(
            parsed,
            Some(ParsedDirective {
                reset: false,
                values: DirectiveUpdate {
                    context: Some(DirectiveValueUpdate::Unset),
                    comment: Some(DirectiveValueUpdate::Set("note".into())),
                    id_prefix: None,
                }
            })
        );
    }

    #[test]
    fn collect_from_source_should_handle_crlf_block_comments() {
        let directives = collect_lingui_directives_from_source(
            "/* lingui-set context=\"ctx\" */\r\nconst msg = t`Hello`;\r\n",
            BytePos(10),
        );

        assert_eq!(
            directives,
            vec![DirectiveEntry {
                pos: BytePos(10),
                values: DirectiveValues {
                    context: Some("ctx".into()),
                    comment: None,
                    id_prefix: None,
                },
            }]
        );
    }

    #[test]
    fn collect_from_source_should_ignore_template_text_that_looks_like_comment() {
        let directives = collect_lingui_directives_from_source(
            "const msg = `\n// lingui-set context=\"ctx\"\n`;\n",
            BytePos(10),
        );

        assert_eq!(directives, vec![]);
    }

    #[test]
    fn collect_from_source_should_support_starred_block_comment_lines() {
        let directives = collect_lingui_directives_from_source(
            "/**\n * lingui-set context=\"ctx\"\n */\nconst msg = t`Hello`;\n",
            BytePos(10),
        );

        assert_eq!(
            directives,
            vec![DirectiveEntry {
                pos: BytePos(15),
                values: DirectiveValues {
                    context: Some("ctx".into()),
                    comment: None,
                    id_prefix: None,
                },
            }]
        );
    }

    #[test]
    fn collect_should_merge_and_reset_directives() {
        let directives = collect_lingui_directives_from_source(
            r#"
      // lingui-set context="ctx1"
      // not a directive
      // lingui-set comment="cmt"
      // lingui-reset context="ctx2"
      "#,
            BytePos(10),
        );

        assert_eq!(
            directives,
            vec![
                DirectiveEntry {
                    pos: BytePos(17),
                    values: DirectiveValues {
                        context: Some("ctx1".into()),
                        comment: None,
                        id_prefix: None,
                    },
                },
                DirectiveEntry {
                    pos: BytePos(77),
                    values: DirectiveValues {
                        context: Some("ctx1".into()),
                        comment: Some("cmt".into()),
                        id_prefix: None,
                    },
                },
                DirectiveEntry {
                    pos: BytePos(111),
                    values: DirectiveValues {
                        context: Some("ctx2".into()),
                        comment: None,
                        id_prefix: None,
                    },
                },
            ]
        );
    }

    #[test]
    fn find_should_return_closest_preceding_directive() {
        let directives = vec![
            DirectiveEntry {
                pos: BytePos(3),
                values: DirectiveValues {
                    context: Some("first".into()),
                    ..Default::default()
                },
            },
            DirectiveEntry {
                pos: BytePos(10),
                values: DirectiveValues {
                    context: Some("second".into()),
                    ..Default::default()
                },
            },
            DirectiveEntry {
                pos: BytePos(20),
                values: DirectiveValues {
                    comment: Some("third".into()),
                    ..Default::default()
                },
            },
        ];

        assert_eq!(find_directive_for_pos(&directives, BytePos(1)), None);
        assert_eq!(
            find_directive_for_pos(&directives, BytePos(7)),
            Some(&DirectiveValues {
                context: Some("first".into()),
                ..Default::default()
            })
        );
        assert_eq!(
            find_directive_for_pos(&directives, BytePos(15)),
            Some(&DirectiveValues {
                context: Some("second".into()),
                ..Default::default()
            })
        );
    }
}
