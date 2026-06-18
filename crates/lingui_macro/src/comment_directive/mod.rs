use std::sync::LazyLock;

use regex::Regex;
use swc_core::common::{BytePos, Span};
use swc_core::plugin::errors::HANDLER;

/// Matches a `lingui-set` / `lingui-reset` directive introduced by a line
/// comment (`//`), block comment (`/*`) or JSDoc comment (`/**`).
/// Group 1 is the directive kind, group 2 the rest of the line (params, along with
/// trailing `*/` for block comments that [`parse_lingui_directive`] strips).
///
/// This is deliberately a plain text scan: it does not understand strings,
/// template literals or JSX, so a directive-looking comment *inside* a string
/// literal is a false positive. This is an intentional trade-off to avoid
/// requiring a full TS+JSX aware lexer pass.
static DIRECTIVE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"/(?:/|\*\*?)\s*lingui-(set|reset)[ ]*([^\n]*)")
        .expect("lingui directive regex is valid")
});

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

fn parse_lingui_directive(reset: bool, params: &str) -> Result<ParsedDirective, String> {
    let directive_name = if reset { "lingui-reset" } else { "lingui-set" };

    // The regex captures everything up to the end of the line, which for a
    // block comment includes the trailing `*/`. Strip it so the params parse
    // cleanly (and `lingui-reset` with no params is recognised as a bare reset).
    let rest = params.trim();
    let rest = rest.strip_suffix("*/").unwrap_or(rest).trim();

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
                "`{directive_name}` directive has invalid syntax: {directive_name} {rest}"
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
                "`{directive_name}` directive has invalid syntax: {directive_name} {rest}"
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

    Ok(ParsedDirective { reset, values })
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

    for caps in DIRECTIVE_RE.captures_iter(source) {
        let matched = caps.get(0).expect("group 0 always matches");
        let comment_start = BytePos(start_pos.0 + matched.start() as u32);
        let span = Span::new(comment_start, BytePos(start_pos.0 + matched.end() as u32));

        let reset = &caps[1] == "reset";
        let params = &caps[2];

        match parse_lingui_directive(reset, params) {
            Ok(parsed) => {
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
            parse_lingui_directive(false, r#"context="ctx" comment="cmt" idPrefix="p." "#).unwrap();

        assert_eq!(
            parsed,
            ParsedDirective {
                reset: false,
                values: DirectiveUpdate {
                    context: Some(DirectiveValueUpdate::Set("ctx".into())),
                    comment: Some(DirectiveValueUpdate::Set("cmt".into())),
                    id_prefix: Some(DirectiveValueUpdate::Set("p.".into())),
                }
            }
        );
    }

    #[test]
    fn parse_should_strip_trailing_block_comment_terminator() {
        let parsed = parse_lingui_directive(false, r#"context="ctx" */"#).unwrap();

        assert_eq!(
            parsed,
            ParsedDirective {
                reset: false,
                values: DirectiveUpdate {
                    context: Some(DirectiveValueUpdate::Set("ctx".into())),
                    ..Default::default()
                }
            }
        );
    }

    #[test]
    fn parse_should_accept_bare_reset() {
        let parsed = parse_lingui_directive(true, "*/").unwrap();

        assert_eq!(
            parsed,
            ParsedDirective {
                reset: true,
                values: DirectiveUpdate::default(),
            }
        );
    }

    #[test]
    fn parse_should_reject_invalid_syntax() {
        let error = parse_lingui_directive(false, "context=single")
            .expect_err("expected parser to reject invalid syntax");

        assert!(error.contains("requires a value"));
    }

    #[test]
    fn parse_should_reject_unknown_params() {
        let error = parse_lingui_directive(false, r#"unknown="value""#)
            .expect_err("expected parser to reject unknown params");

        assert!(error.contains("unknown param \"unknown\""));
    }

    #[test]
    fn parse_should_treat_empty_strings_as_unset() {
        let parsed = parse_lingui_directive(false, r#"context="" comment="note""#).unwrap();

        assert_eq!(
            parsed,
            ParsedDirective {
                reset: false,
                values: DirectiveUpdate {
                    context: Some(DirectiveValueUpdate::Unset),
                    comment: Some(DirectiveValueUpdate::Set("note".into())),
                    id_prefix: None,
                }
            }
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
