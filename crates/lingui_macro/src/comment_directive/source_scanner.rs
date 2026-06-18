pub struct SourceComment<'a> {
    pub byte_offset: usize,
    pub content: &'a str,
    pub kind: CommentKind,
}

pub enum CommentKind {
    Line,
    Block,
}

/// Lexer context. JSX must be tracked because a quote or backtick inside JSX
/// text (e.g. `<p>'</p>` or ``<p>`</p>``) is literal content, not a string or
/// template literal. Treating it as a literal opener would scan ahead to the
/// next matching delimiter and swallow any comments — including lingui
/// directives — in between.
enum Ctx {
    /// Ordinary JS/TS code. `brace_depth` counts nested `{`; when this frame is
    /// a JSX expression container (`jsx_expr`), the `}` at depth 0 returns to
    /// the enclosing JSX children.
    Code { brace_depth: u32, jsx_expr: bool },
    /// Inside a JSX tag, between `<` and its closing `>` / `/>`.
    JsxTag,
    /// Inside JSX element children (text). Quotes, backticks and comment
    /// markers are all literal text here.
    JsxChildren,
}

/// Extracts every line and block comment from `source`, in source order, with
/// byte offsets relative to the start of `source`.
///
/// # Why this is a hand-rolled scanner
///
/// Comments are needed to locate `lingui-set` / `lingui-reset` directives and
/// associate each with the byte position it precedes. SWC has of course already
/// parsed the file and knows every comment — but the `Comments` trait (and the
/// `PluginCommentsProxy` the plugin actually receives in the WASM host) only
/// supports *lookup by position*, not *enumeration*. There is no API to ask
/// "give me all comments in order", so we cannot reuse SWC's comment table here
/// and must recover the comments from the raw source ourselves.
///
/// # Why it can't be a naive `//` / `/*` search
///
/// Comment markers also appear inside string literals (`"https://…"`), template
/// literals, and regular text — so the scanner has to understand enough lexical
/// structure to skip those. The tricky part is JSX: a quote or backtick in JSX
/// *text* (`<p>'</p>`, ``<p>`</p>``) is literal content, not a string/template
/// opener. A flat scanner that skips every `'`/`"`/`` ` `` as a literal would,
/// on hitting such a character, scan ahead to the next matching delimiter and
/// swallow any comments — including directives — in between. That is exactly the
/// bug this scanner exists to avoid, so it tracks JSX context (see [`Ctx`]).
///
/// # Limitations
///
/// This is a pragmatic lexer, not a full parser. It does not handle regular
/// expression literals (a `/regex/` is treated as division, which at worst
/// misreads its contents but never the surrounding directives), and JSX
/// detection is heuristic (see [`is_jsx_tag_start`]). These trade-offs are
/// acceptable because the only consumers are lingui directive comments, and a
/// misclassification can at most cause a directive to be missed — never applied
/// to the wrong code.
pub fn scan_source_comments(source: &str) -> Vec<SourceComment<'_>> {
    let bytes = source.as_bytes();
    let mut comments = Vec::new();
    let mut index = 0usize;
    let mut stack: Vec<Ctx> = vec![Ctx::Code {
        brace_depth: 0,
        jsx_expr: false,
    }];
    // Last significant (non-whitespace, non-comment) code byte. Used to decide
    // whether a `<` opens a JSX element (expression position) or is a
    // comparison / TS generic (after a value).
    let mut last_sig = 0u8;

    while index < bytes.len() {
        match stack.last_mut().expect("scanner stack is never empty") {
            Ctx::Code {
                brace_depth,
                jsx_expr,
            } => match bytes[index] {
                b'\'' | b'"' => {
                    index = skip_string_literal(bytes, index);
                    last_sig = b'"';
                }
                b'`' => {
                    index = skip_template_literal(bytes, index);
                    last_sig = b'`';
                }
                b'/' if bytes.get(index + 1) == Some(&b'/') => {
                    let comment_start = index;
                    let content_start = index + 2;
                    index = content_start;

                    while index < bytes.len() && bytes[index] != b'\n' {
                        index += 1;
                    }

                    comments.push(SourceComment {
                        byte_offset: comment_start,
                        content: &source[content_start..index],
                        kind: CommentKind::Line,
                    });
                }
                b'/' if bytes.get(index + 1) == Some(&b'*') => {
                    let comment_start = index;
                    let content_start = index + 2;
                    index = content_start;

                    while index < bytes.len() {
                        if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                            break;
                        }
                        index += 1;
                    }

                    let content_end = index;
                    if index < bytes.len() {
                        index += 2;
                    }

                    comments.push(SourceComment {
                        byte_offset: comment_start,
                        content: &source[content_start..content_end],
                        kind: CommentKind::Block,
                    });
                }
                b'{' => {
                    *brace_depth += 1;
                    last_sig = b'{';
                    index += 1;
                }
                b'}' => {
                    if *brace_depth > 0 {
                        *brace_depth -= 1;
                    } else if *jsx_expr {
                        // Closes a JSX expression container; back to children.
                        stack.pop();
                    }
                    last_sig = b'}';
                    index += 1;
                }
                b'<' if is_jsx_tag_start(bytes, index, last_sig) => {
                    stack.push(Ctx::JsxTag);
                    index += 1;
                }
                b' ' | b'\t' | b'\r' | b'\n' => {
                    index += 1;
                }
                other => {
                    last_sig = other;
                    index += 1;
                }
            },
            Ctx::JsxTag => match bytes[index] {
                b'\'' | b'"' => {
                    index = skip_string_literal(bytes, index);
                }
                b'{' => {
                    stack.push(Ctx::Code {
                        brace_depth: 0,
                        jsx_expr: true,
                    });
                    index += 1;
                }
                b'/' if bytes.get(index + 1) == Some(&b'>') => {
                    // Self-closing tag: no children.
                    stack.pop();
                    index += 2;
                    last_sig = b']';
                }
                b'>' => {
                    stack.pop();
                    stack.push(Ctx::JsxChildren);
                    index += 1;
                }
                b',' | b';' => {
                    // Not a real JSX tag (e.g. a TS generic such as `<T, U>`):
                    // treat the `<` as an operator and resume scanning as code.
                    stack.pop();
                    stack.push(Ctx::Code {
                        brace_depth: 0,
                        jsx_expr: false,
                    });
                }
                _ => {
                    index += 1;
                }
            },
            Ctx::JsxChildren => match bytes[index] {
                b'{' => {
                    stack.push(Ctx::Code {
                        brace_depth: 0,
                        jsx_expr: true,
                    });
                    index += 1;
                }
                b'<' if bytes.get(index + 1) == Some(&b'/') => {
                    // Closing tag `</...>`: skip to `>` and leave children.
                    index += 2;
                    while index < bytes.len() && bytes[index] != b'>' {
                        index += 1;
                    }
                    if index < bytes.len() {
                        index += 1;
                    }
                    stack.pop();
                    last_sig = b']';
                }
                b'<' => {
                    // Nested element open tag.
                    stack.push(Ctx::JsxTag);
                    index += 1;
                }
                _ => {
                    // Text content — quotes/backticks/`//` are literal here.
                    index += 1;
                }
            },
        }
    }

    comments
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Keywords after which a `<` begins an expression, so it may open a JSX element
/// (`return <div>`, `export default <App/>`, `yield <x>`, …). In a comparison or
/// generic the byte before `<` is an operand identifier, never one of these, so
/// allowing them does not reintroduce false positives.
fn is_expression_keyword(word: &[u8]) -> bool {
    matches!(
        word,
        b"return"
            | b"yield"
            | b"await"
            | b"void"
            | b"typeof"
            | b"delete"
            | b"throw"
            | b"do"
            | b"else"
            | b"case"
            | b"default"
            | b"in"
            | b"of"
            | b"instanceof"
    )
}

/// A byte that ends a value/expression, after which a `<` is a comparison or TS
/// generic rather than the start of a JSX element.
fn is_value_ending(b: u8) -> bool {
    is_ident_byte(b) || matches!(b, b')' | b']' | b'.' | b'"' | b'\'' | b'`')
}

/// Whether the `<` at `index` (in code context) opens a JSX element. Requires an
/// expression position (per `last_sig`) and a following tag-name char or `>`
/// (fragment). This matches `.tsx` semantics, where a bare `<T>` is JSX and a
/// generic arrow must be written `<T,>` (handled by the `,` bail in `JsxTag`).
///
/// When the preceding byte is identifier-like it is normally a value (so `<` is
/// a comparison/generic), unless that identifier is an expression-starting
/// keyword such as `return` — handled by inspecting the full preceding word.
fn is_jsx_tag_start(bytes: &[u8], index: usize, last_sig: u8) -> bool {
    let next_is_tag = match bytes.get(index + 1) {
        Some(&b) => b.is_ascii_alphabetic() || b == b'_' || b == b'$' || b == b'>',
        None => false,
    };
    if !next_is_tag {
        return false;
    }

    if !is_value_ending(last_sig) {
        return true;
    }

    // The preceding byte ends a value. Only an identifier ending might actually
    // be an expression-starting keyword (e.g. `return`); other value endings
    // (`)`, `]`, `.`, quotes) never are.
    if !is_ident_byte(last_sig) {
        return false;
    }

    is_expression_keyword(preceding_word(bytes, index))
}

/// The identifier immediately preceding `index`, skipping whitespace. Returns an
/// empty slice if the preceding token is not a bare identifier (e.g. a member
/// access like `obj.return`, which is a property name, not the keyword).
fn preceding_word(bytes: &[u8], index: usize) -> &[u8] {
    let mut end = index;
    while end > 0 && matches!(bytes[end - 1], b' ' | b'\t' | b'\r' | b'\n') {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }
    if start > 0 && bytes[start - 1] == b'.' {
        return &[];
    }
    &bytes[start..end]
}

fn skip_string_literal(bytes: &[u8], start: usize) -> usize {
    let delim = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
        } else if bytes[i] == delim {
            return i + 1;
        } else if bytes[i] == b'\n' {
            // A single/double-quoted JS string cannot contain an unescaped
            // newline. Hitting one means the opening quote was not a string
            // delimiter (e.g. an apostrophe in JSX text like `<Trans>'</Trans>`).
            // Treat it as a plain character so scanning resumes — otherwise we
            // would skip ahead to the next stray quote and swallow any comments
            // (including lingui directives) in between.
            return start + 1;
        } else {
            i += 1;
        }
    }
    i
}

fn skip_template_literal(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
        } else if bytes[i] == b'`' {
            return i + 1;
        } else {
            i += 1;
        }
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_comments(source: &str) -> Vec<(usize, &str)> {
        scan_source_comments(source)
            .into_iter()
            .filter(|c| matches!(c.kind, CommentKind::Line))
            .map(|c| (c.byte_offset, c.content))
            .collect()
    }

    fn block_comments(source: &str) -> Vec<(usize, &str)> {
        scan_source_comments(source)
            .into_iter()
            .filter(|c| matches!(c.kind, CommentKind::Block))
            .map(|c| (c.byte_offset, c.content))
            .collect()
    }

    fn all_comments(source: &str) -> Vec<(usize, &str)> {
        scan_source_comments(source)
            .into_iter()
            .map(|c| (c.byte_offset, c.content))
            .collect()
    }

    #[test]
    fn empty_source() {
        assert_eq!(all_comments(""), Vec::<(usize, &str)>::new());
    }

    #[test]
    fn no_comments() {
        assert_eq!(all_comments("const x = 1;\nlet y = 2;"), vec![]);
    }

    #[test]
    fn single_line_comment() {
        assert_eq!(line_comments("// hello world"), vec![(0, " hello world")]);
    }

    #[test]
    fn line_comment_after_code() {
        assert_eq!(
            line_comments("const x = 1; // inline"),
            vec![(13, " inline")]
        );
    }

    #[test]
    fn multiple_line_comments() {
        let source = "// first\n// second\ncode\n// third";
        assert_eq!(
            line_comments(source),
            vec![(0, " first"), (9, " second"), (24, " third")]
        );
    }

    #[test]
    fn single_block_comment() {
        assert_eq!(block_comments("/* block */"), vec![(0, " block ")]);
    }

    #[test]
    fn multiline_block_comment() {
        let source = "/* line1\n   line2 */";
        assert_eq!(block_comments(source), vec![(0, " line1\n   line2 ")]);
    }

    #[test]
    fn block_comment_after_code() {
        assert_eq!(
            block_comments("x = 1; /* note */ y = 2;"),
            vec![(7, " note ")]
        );
    }

    #[test]
    fn ignores_comment_syntax_in_single_quoted_string() {
        assert_eq!(all_comments("const x = '// not a comment';"), vec![]);
        assert_eq!(all_comments("const x = '/* not a comment */';"), vec![]);
    }

    #[test]
    fn ignores_comment_syntax_in_double_quoted_string() {
        assert_eq!(all_comments(r#"const x = "// not a comment";"#), vec![]);
        assert_eq!(all_comments(r#"const x = "/* not a comment */";"#), vec![]);
    }

    #[test]
    fn ignores_comment_syntax_in_template_literal() {
        assert_eq!(all_comments("const x = `// not a comment`;"), vec![]);
        assert_eq!(all_comments("const x = `/* not a comment */`;"), vec![]);
    }

    #[test]
    fn handles_escaped_quotes_in_single_quoted_string() {
        assert_eq!(
            all_comments(r"const x = 'it\'s'; // after"),
            vec![(19, " after")]
        );
    }

    #[test]
    fn handles_escaped_quotes_in_double_quoted_string() {
        assert_eq!(
            all_comments(r#"const x = "say \"hi\""; // after"#),
            vec![(24, " after")]
        );
    }

    #[test]
    fn handles_escaped_backtick_in_template_literal() {
        assert_eq!(
            all_comments(r"const x = `\`template\``; // after"),
            vec![(26, " after")]
        );
    }

    #[test]
    fn handles_backslash_at_end_of_string() {
        // String ending with escape at EOF (unterminated)
        assert_eq!(all_comments(r"const x = '\"), vec![]);
    }

    #[test]
    fn handles_backslash_at_end_of_template() {
        assert_eq!(all_comments("const x = `\\"), vec![]);
    }

    #[test]
    fn unterminated_single_quoted_string() {
        // No closing quote — scanner shouldn't panic
        assert_eq!(all_comments("const x = 'unterminated // nope"), vec![]);
    }

    #[test]
    fn unterminated_double_quoted_string() {
        assert_eq!(all_comments(r#"const x = "unterminated // nope"#), vec![]);
    }

    #[test]
    fn unterminated_template_literal() {
        assert_eq!(all_comments("const x = `unterminated // nope"), vec![]);
    }

    #[test]
    fn unterminated_block_comment() {
        // Block comment that never closes — content runs to end
        assert_eq!(
            block_comments("/* never closed"),
            vec![(0, " never closed")]
        );
    }

    #[test]
    fn mixed_comment_types() {
        let source = "// line\n/* block */\ncode // inline";
        let comments = all_comments(source);
        assert_eq!(
            comments,
            vec![(0, " line"), (8, " block "), (25, " inline")]
        );
    }

    #[test]
    fn slash_not_followed_by_slash_or_star() {
        // Division operator should not be mistaken for comment
        assert_eq!(all_comments("const x = 10 / 2;"), vec![]);
    }

    #[test]
    fn empty_line_comment() {
        assert_eq!(line_comments("//\ncode"), vec![(0, "")]);
    }

    #[test]
    fn empty_block_comment() {
        assert_eq!(block_comments("/**/"), vec![(0, "")]);
    }

    #[test]
    fn block_comment_with_star_inside() {
        assert_eq!(block_comments("/* a * b */"), vec![(0, " a * b ")]);
    }

    #[test]
    fn consecutive_block_comments() {
        // "/* a */" = 7 bytes, so second comment starts at offset 7
        assert_eq!(
            block_comments("/* a *//* b */"),
            vec![(0, " a "), (7, " b ")]
        );
    }

    #[test]
    fn line_comment_at_eof_without_newline() {
        assert_eq!(line_comments("// eof"), vec![(0, " eof")]);
    }

    #[test]
    fn comment_after_template_literal_with_expressions() {
        // Template with ${} — the simplified scanner treats it as text until closing backtick
        let source = "const x = `hello ${world}`; // after";
        assert_eq!(line_comments(source), vec![(28, " after")]);
    }

    #[test]
    fn multiline_template_literal_with_comment_like_content() {
        let source = "const x = `\n// fake\n/* also fake */\n`;\n// real";
        assert_eq!(line_comments(source), vec![(39, " real")]);
    }

    #[test]
    fn quote_in_jsx_text_does_not_swallow_following_comment() {
        // The apostrophe is JSX text, not a string opener. A real string can't
        // span the newline, so the following comment must still be found.
        let source = "const x = <Trans>'</Trans>;\n// real";
        assert_eq!(line_comments(source), vec![(28, " real")]);
    }

    #[test]
    fn quote_in_jsx_text_after_return_keyword_does_not_swallow_comment() {
        // `return` is an expression-starting keyword, so the `<` opens JSX even
        // though the byte before it is identifier-like.
        let source = "function f() { return <div>'</div>; }\n// real";
        assert_eq!(line_comments(source), vec![(38, " real")]);
    }

    #[test]
    fn jsx_after_export_default_is_detected() {
        let source = "export default <App>'</App>;\n// real";
        assert_eq!(line_comments(source), vec![(29, " real")]);
    }

    #[test]
    fn identifier_ending_in_keyword_is_not_treated_as_jsx() {
        // `myreturn` is a value identifier, so `<` is a comparison (the lack of
        // a space still forces the keyword check via `next_is_tag`), and the
        // quoted text after it is a normal string literal (not JSX text).
        let source = "const a = myreturn<x; const b = 'hi';\n// real";
        assert_eq!(line_comments(source), vec![(38, " real")]);
    }

    #[test]
    fn backtick_in_jsx_text_does_not_swallow_following_comment() {
        // A backtick is JSX text, not a template-literal opener. Real templates
        // span newlines, so only JSX awareness (not a newline bail) saves us.
        let source = "const x = () => <p>`</p>;\n// real";
        assert_eq!(line_comments(source), vec![(26, " real")]);
    }

    #[test]
    fn template_literal_inside_jsx_expression_is_still_skipped() {
        // Inside a JSX expression container `{...}` we are back in code, so a
        // real template literal must still be skipped (its `//` is not a comment).
        let source = "const x = <p>{`// not a comment`}</p>;\n// real";
        assert_eq!(line_comments(source), vec![(39, " real")]);
    }

    #[test]
    fn ts_generic_is_not_mistaken_for_jsx() {
        // `Record<string, ...>` is a generic, not JSX; a following comment and
        // its quotes must scan normally.
        let source = "const x: Record<string, T> = {};\n// real";
        assert_eq!(line_comments(source), vec![(33, " real")]);
    }

    #[test]
    fn lone_quote_before_newline_is_not_a_string() {
        // A lone quote followed by a newline is not a string literal, so a
        // comment on a later line must still be discovered.
        let source = "a = ';\nb = '; // c\n";
        assert_eq!(line_comments(source), vec![(14, " c")]);
    }

    #[test]
    fn string_containing_backslash_n_is_not_newline() {
        // The literal text \n in a string (escaped), not a real newline
        assert_eq!(all_comments("const x = '\\n'; // yes"), vec![(16, " yes")]);
    }
}
