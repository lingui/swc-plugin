mod source_scanner;

use source_scanner::{scan_source_comments, CommentKind};
use swc_core::common::{BytePos, Span};
use swc_core::plugin::errors::HANDLER;

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

    let (directive_name, rest) = if let Some(rest) = trimmed.strip_prefix("lingui-set") {
        ("lingui-set", rest)
    } else if let Some(rest) = trimmed.strip_prefix("lingui-reset") {
        ("lingui-reset", rest)
    } else {
        return Ok(None);
    };

    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return Ok(None);
    }

    let reset = directive_name == "lingui-reset";
    let rest = rest.trim();

    let mut values = DirectiveUpdate::default();
    let mut has_params = false;
    let mut pos = 0;
    let rest_bytes = rest.as_bytes();

    while pos < rest_bytes.len() {
        // skip whitespace
        while pos < rest_bytes.len() && rest_bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= rest_bytes.len() {
            break;
        }

        // parse key (word chars)
        let key_start = pos;
        while pos < rest_bytes.len()
            && (rest_bytes[pos].is_ascii_alphanumeric() || rest_bytes[pos] == b'_')
        {
            pos += 1;
        }
        if pos == key_start {
            return Err(format!(
                "`{directive_name}` directive has invalid syntax: {trimmed}"
            ));
        }
        let key = &rest[key_start..pos];

        // expect ="..."
        if pos >= rest_bytes.len() || rest_bytes[pos] != b'=' {
            return Err(format!(
                "`{directive_name}` directive: \"{key}\" requires a value, e.g. {key}=\"...\""
            ));
        }
        pos += 1; // skip '='

        if pos >= rest_bytes.len() || rest_bytes[pos] != b'"' {
            return Err(format!(
                "`{directive_name}` directive: \"{key}\" requires a value, e.g. {key}=\"...\""
            ));
        }
        pos += 1; // skip opening '"'

        let value_start = pos;
        while pos < rest_bytes.len() && rest_bytes[pos] != b'"' {
            pos += 1;
        }
        if pos >= rest_bytes.len() {
            return Err(format!(
                "`{directive_name}` directive has invalid syntax: {trimmed}"
            ));
        }
        let value = &rest[value_start..pos];
        pos += 1; // skip closing '"'

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
    if !source.contains("lingui-set") && !source.contains("lingui-reset") {
        return vec![];
    }

    let mut directives = Vec::new();
    let mut accumulated = DirectiveValues::default();

    for comment in scan_source_comments(source) {
        let comment_start = BytePos(start_pos.0 + comment.byte_offset as u32);
        let trimmed = comment.content.trim();

        if !is_lingui_directive_prefix(trimmed) {
            continue;
        }

        let content_end = match comment.kind {
            CommentKind::Line => BytePos(comment_start.0 + 2 + comment.content.len() as u32),
            CommentKind::Block => BytePos(comment_start.0 + 2 + comment.content.len() as u32 + 2),
        };
        let span = Span::new(comment_start, content_end);

        match parse_lingui_directive(trimmed) {
            Ok(Some(parsed)) => {
                let mut values = if parsed.reset {
                    DirectiveValues::default()
                } else {
                    accumulated.clone()
                };

                values.apply_update(parsed.values);
                accumulated = values.clone();

                directives.push(DirectiveEntry {
                    pos: comment_start,
                    values,
                })
            }
            Ok(None) => {}
            Err(message) => {
                HANDLER.with(|handler| handler.struct_span_err(span, &message).emit());
            }
        }
    }

    directives
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
