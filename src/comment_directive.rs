use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use swc_core::common::comments::{Comment, Comments};
use swc_core::common::{BytePos, Span, Spanned};
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{Visit, VisitWith};
use swc_core::plugin::errors::HANDLER;

static DIRECTIVE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(lingui-(?:set|reset))(?:\s|$)(.*)").unwrap());
static TOKEN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\s+|(\w+)(?:="([^"]*)")?"#).unwrap());

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DirectiveValues {
    pub context: Option<String>,
    pub comment: Option<String>,
    pub id_prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectiveEntry {
    pub pos: BytePos,
    pub values: DirectiveValues,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectiveLineEntry {
    pub line: usize,
    pub values: DirectiveValues,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawDirectiveEntry {
    pos: BytePos,
    reset: bool,
    values: DirectiveUpdate,
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

#[cfg(test)]
pub(crate) fn parse_lingui_directive(
    comment_value: &str,
) -> Result<Option<(bool, DirectiveValues)>, String> {
    parse_lingui_directive_raw(comment_value).map(|parsed| {
        parsed.map(|parsed| {
            let mut values = DirectiveValues::default();
            values.apply_update(parsed.values);
            (parsed.reset, values)
        })
    })
}

fn parse_lingui_directive_raw(comment_value: &str) -> Result<Option<ParsedDirective>, String> {
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

pub fn collect_lingui_directives_from_comments(comments: &[Comment]) -> Vec<DirectiveEntry> {
    let mut directives: Vec<RawDirectiveEntry> = comments
        .iter()
        .filter_map(
            |comment| match parse_lingui_directive_raw(comment.text.as_ref()) {
                Ok(Some(parsed)) => Some(RawDirectiveEntry {
                    pos: comment.span.lo,
                    reset: parsed.reset,
                    values: parsed.values,
                }),
                Ok(None) => None,
                Err(message) => {
                    HANDLER.with(|handler| {
                        handler.struct_span_err(comment.span, &message).emit();
                    });
                    None
                }
            },
        )
        .collect();

    directives.sort_by_key(|directive| directive.pos);

    let mut accumulated = DirectiveValues::default();

    directives
        .into_iter()
        .map(|directive| {
            let mut values = if directive.reset {
                DirectiveValues::default()
            } else {
                accumulated.clone()
            };

            values.apply_update(directive.values);
            accumulated = values.clone();

            DirectiveEntry {
                pos: directive.pos,
                values,
            }
        })
        .collect()
}

pub fn find_directive_for_pos(
    directives: &[DirectiveEntry],
    pos: BytePos,
) -> Option<&DirectiveValues> {
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

pub fn collect_lingui_directives_from_source(
    source: &str,
    start_line: usize,
) -> Vec<DirectiveLineEntry> {
    let mut directives: Vec<(usize, bool, DirectiveUpdate)> = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let comment = extract_directive_comment(line)?;
            match parse_lingui_directive_raw(comment) {
                Ok(Some(parsed)) => Some((index + start_line, parsed.reset, parsed.values)),
                Ok(None) => None,
                Err(message) => {
                    HANDLER.with(|handler| handler.struct_err(&message).emit());
                    None
                }
            }
        })
        .collect();

    directives.sort_by_key(|directive| directive.0);

    let mut accumulated = DirectiveValues::default();

    directives
        .into_iter()
        .map(|(line, reset, values)| {
            if reset {
                accumulated = DirectiveValues::default();
            }

            accumulated.apply_update(values);

            DirectiveLineEntry {
                line,
                values: accumulated.clone(),
            }
        })
        .collect()
}

pub fn find_directive_for_line(
    directives: &[DirectiveLineEntry],
    line: usize,
) -> Option<&DirectiveValues> {
    if directives.is_empty() {
        return None;
    }

    let mut lo = 0usize;
    let mut hi = directives.len();

    while lo < hi {
        let mid = (lo + hi) / 2;
        if directives[mid].line <= line {
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

fn extract_directive_comment(line: &str) -> Option<&str> {
    let trimmed = line.trim();

    if let Some(comment) = trimmed.strip_prefix("//") {
        return Some(comment.trim());
    }

    if let Some(comment) = trimmed.strip_prefix("/*") {
        return Some(comment.trim_end_matches("*/").trim());
    }

    if let Some(comment) = trimmed.strip_prefix('*') {
        return Some(comment.trim_end_matches("*/").trim());
    }

    None
}

pub(crate) fn collect_lingui_directives<C: Comments, N>(
    node: &N,
    comments: &Option<C>,
) -> Vec<DirectiveEntry>
where
    for<'a> N: VisitWith<DirectiveCollector<'a, C>>,
{
    let Some(comments) = comments.as_ref() else {
        return vec![];
    };

    let mut collector = DirectiveCollector::new(comments);
    node.visit_with(&mut collector);
    collect_lingui_directives_from_comments(&collector.comments)
}

pub(crate) struct DirectiveCollector<'a, C>
where
    C: Comments,
{
    comments_api: &'a C,
    seen_positions: HashSet<BytePos>,
    comments: Vec<Comment>,
}

impl<'a, C> DirectiveCollector<'a, C>
where
    C: Comments,
{
    fn new(comments_api: &'a C) -> Self {
        Self {
            comments_api,
            seen_positions: HashSet::new(),
            comments: vec![],
        }
    }

    fn record_for_span(&mut self, span: Span) {
        if span.is_dummy() {
            return;
        }

        if let Some(comments) = self.comments_api.get_leading(span.lo()) {
            for comment in comments {
                if self.seen_positions.insert(comment.span.lo) {
                    self.comments.push(comment);
                }
            }
        }
    }
}

impl<C> Visit for DirectiveCollector<'_, C>
where
    C: Comments,
{
    fn visit_expr(&mut self, expr: &Expr) {
        self.record_for_span(expr.span());
        expr.visit_children_with(self);
    }

    fn visit_module_item(&mut self, module_item: &ModuleItem) {
        self.record_for_span(module_item.span());
        module_item.visit_children_with(self);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        self.record_for_span(stmt.span());
        stmt.visit_children_with(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_core::common::comments::CommentKind;

    fn make_comment(text: &str, lo: u32) -> Comment {
        Comment {
            kind: CommentKind::Block,
            span: Span::new(BytePos(lo), BytePos(lo + text.len() as u32)),
            text: text.into(),
        }
    }

    #[test]
    fn parse_should_parse_multiple_keys() {
        let parsed =
            parse_lingui_directive(r#" lingui-set context="ctx" comment="cmt" idPrefix="p." "#)
                .unwrap();

        assert_eq!(
            parsed,
            Some((
                false,
                DirectiveValues {
                    context: Some("ctx".into()),
                    comment: Some("cmt".into()),
                    id_prefix: Some("p.".into()),
                }
            ))
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
            Some((
                false,
                DirectiveValues {
                    context: None,
                    comment: Some("note".into()),
                    id_prefix: None,
                }
            ))
        );
    }

    #[test]
    fn collect_should_merge_and_reset_directives() {
        let directives = collect_lingui_directives_from_comments(&[
            make_comment(r#" lingui-set context="ctx1" "#, 10),
            make_comment(" not a directive", 20),
            make_comment(r#" lingui-set comment="cmt" "#, 30),
            make_comment(r#" lingui-reset context="ctx2" "#, 40),
        ]);

        assert_eq!(
            directives,
            vec![
                DirectiveEntry {
                    pos: BytePos(10),
                    values: DirectiveValues {
                        context: Some("ctx1".into()),
                        comment: None,
                        id_prefix: None,
                    },
                },
                DirectiveEntry {
                    pos: BytePos(30),
                    values: DirectiveValues {
                        context: Some("ctx1".into()),
                        comment: Some("cmt".into()),
                        id_prefix: None,
                    },
                },
                DirectiveEntry {
                    pos: BytePos(40),
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
