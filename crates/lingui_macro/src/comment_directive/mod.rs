use swc_core::common::{BytePos, Span};
use swc_core::plugin::errors::HANDLER;

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

impl From<DirectiveValueUpdate> for Option<String> {
    fn from(update: DirectiveValueUpdate) -> Self {
        match update {
            DirectiveValueUpdate::Set(value) => Some(value),
            DirectiveValueUpdate::Unset => None,
        }
    }
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
            self.context = value.into();
        }
        if let Some(value) = update.comment {
            self.comment = value.into();
        }
        if let Some(value) = update.id_prefix {
            self.id_prefix = value.into();
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommentKind {
    Line,
    Block,
}

#[derive(Debug, PartialEq, Eq)]
struct LocatedDirective<'a> {
    /// Byte offset of the comment opener (`//`, `/*` or `/**`) introducing it.
    opener_start: usize,
    /// Byte offset just past the comment (after `*/`, the newline, or EOF).
    comment_end: usize,
    reset: bool,
    /// Raw parameter text between the directive name and the comment end.
    params: &'a str,
}

/// Common prefix of both directives (`lingui-set` / `lingui-reset`); used as
/// the substring anchor for the scan and the cheap "any directives?" check.
const LINGUI_PREFIX: &str = "lingui-";

fn locate_directives(source: &str) -> Vec<LocatedDirective<'_>> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut from = 0usize;

    while let Some(rel) = source[from..].find(LINGUI_PREFIX) {
        let keyword = from + rel;
        let after = keyword + LINGUI_PREFIX.len();
        // Always advance past this occurrence so the loop terminates regardless
        // of whether it turns out to be a real directive.
        from = after;

        let (reset, name_end) = if source[after..].starts_with("reset") {
            (true, after + "reset".len())
        } else if source[after..].starts_with("set") {
            (false, after + "set".len())
        } else {
            continue;
        };

        // Word boundary: `lingui-settings` is not a `lingui-set` directive.
        if let Some(&b) = bytes.get(name_end) {
            if b.is_ascii_alphanumeric() || b == b'_' {
                continue;
            }
        }

        let Some((opener_start, kind)) = find_comment_opener(source, keyword) else {
            continue;
        };

        let (params, comment_end) = match kind {
            CommentKind::Line => {
                let end = line_end(bytes, name_end);
                (&source[name_end..end], end)
            }
            CommentKind::Block => match block_comment_close(bytes, name_end) {
                Some(star) => (&source[name_end..star], star + 2),
                None => (&source[name_end..], bytes.len()),
            },
        };

        out.push(LocatedDirective {
            opener_start,
            comment_end,
            reset,
            params,
        });
    }

    out
}

/// Walk backwards from a `lingui-…` keyword over the horizontal whitespace
/// separating it from its comment opener. Returns the opener's start offset and
/// kind, or `None` if the keyword is not at the start of a `//`, `/*` or `/**`
/// comment on the same line.
fn find_comment_opener(source: &str, keyword: usize) -> Option<(usize, CommentKind)> {
    let bytes = source.as_bytes();

    // The directive keyword must sit on the opener's line, separated from it by
    // spaces/tabs only — no intervening newline.
    let mut j = keyword;
    while j > 0 && matches!(bytes[j - 1], b' ' | b'\t') {
        j -= 1;
    }
    if source[..j].ends_with("//") {
        return Some((j - 2, CommentKind::Line));
    }
    if source[..j].ends_with("/**") {
        return Some((j - 3, CommentKind::Block));
    }
    if source[..j].ends_with("/*") {
        return Some((j - 2, CommentKind::Block));
    }

    None
}

fn line_end(bytes: &[u8], from: usize) -> usize {
    bytes[from..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(bytes.len(), |offset| from + offset)
}

/// Find the closing `*/` of a block comment. Returns the offset of the `*`, or
/// `None` for an unterminated block comment (parameters then run to EOF).
fn block_comment_close(bytes: &[u8], from: usize) -> Option<usize> {
    bytes[from..]
        .windows(2)
        .position(|pair| pair == b"*/")
        .map(|offset| from + offset)
}

fn parse_lingui_directive(reset: bool, params: &str) -> Result<ParsedDirective, String> {
    let directive_name = if reset { "lingui-reset" } else { "lingui-set" };
    let rest = params.trim();
    let bytes = rest.as_bytes();

    let invalid_syntax =
        || format!("`{directive_name}` directive has invalid syntax: {directive_name} {rest}");

    let mut values = DirectiveUpdate::default();
    let mut has_params = false;
    let mut pos = 0;

    while pos < bytes.len() {
        // Skip whitespace between params
        if bytes[pos].is_ascii_whitespace() {
            pos += 1;
            continue;
        }

        // Parse the key (word chars)
        let key_start = pos;
        while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
            pos += 1;
        }
        if pos == key_start {
            return Err(invalid_syntax());
        }
        let key = &rest[key_start..pos];

        // Expect `="`
        let requires_value = || {
            format!("`{directive_name}` directive: \"{key}\" requires a value, e.g. {key}=\"...\"")
        };
        if bytes.get(pos) != Some(&b'=') {
            return Err(requires_value());
        }
        pos += 1;
        if bytes.get(pos) != Some(&b'"') {
            return Err(requires_value());
        }
        pos += 1;

        // Read the value up to the closing `"`
        let value_start = pos;
        while pos < bytes.len() && bytes[pos] != b'"' {
            pos += 1;
        }
        if pos >= bytes.len() {
            return Err(invalid_syntax());
        }
        let value = &rest[value_start..pos];
        pos += 1; // closing quote

        let field = match key {
            "context" => &mut values.context,
            "comment" => &mut values.comment,
            "idPrefix" => &mut values.id_prefix,
            _ => {
                return Err(format!(
                    "`{directive_name}` directive has unknown param \"{key}\". Valid params: context, comment, idPrefix"
                ));
            }
        };
        *field = Some(parse_value_update(value));
        has_params = true;
    }

    if !has_params && !reset {
        return Err(format!(
            "`{directive_name}` directive requires at least one param. Valid params: context, comment, idPrefix"
        ));
    }

    Ok(ParsedDirective { reset, values })
}

fn find_directive_for_pos(directives: &[DirectiveEntry], pos: BytePos) -> Option<&DirectiveValues> {
    // `directives` is sorted by `pos` ascending, so the closest directive at or
    // before `pos` is the one just before the first entry that starts after it.
    let after = directives.partition_point(|entry| entry.pos <= pos);
    after.checked_sub(1).map(|i| &directives[i].values)
}

fn collect_lingui_directives_from_source(source: &str, start_pos: BytePos) -> Vec<DirectiveEntry> {
    if !source.contains(LINGUI_PREFIX) {
        return Vec::new();
    }

    let mut directives = Vec::new();
    let mut accumulated = DirectiveValues::default();

    for located in locate_directives(source) {
        let comment_start = BytePos(start_pos.0 + located.opener_start as u32);

        match parse_lingui_directive(located.reset, located.params) {
            Ok(parsed) => {
                // A reset starts from a clean slate; otherwise updates layer on
                // top of the values accumulated from preceding directives.
                if parsed.reset {
                    accumulated = DirectiveValues::default();
                }
                accumulated.apply_update(parsed.values);

                directives.push(DirectiveEntry {
                    pos: comment_start,
                    values: accumulated.clone(),
                });
            }
            Err(message) => {
                let span = Span::new(
                    comment_start,
                    BytePos(start_pos.0 + located.comment_end as u32),
                );
                HANDLER.with(|handler| handler.struct_span_err(span, &message).emit());
            }
        }
    }

    directives
}

#[cfg(test)]
mod tests;
