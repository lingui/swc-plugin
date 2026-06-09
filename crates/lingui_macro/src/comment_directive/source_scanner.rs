pub struct SourceComment<'a> {
    pub byte_offset: usize,
    pub content: &'a str,
    pub kind: CommentKind,
}

pub enum CommentKind {
    Line,
    Block,
}

pub fn scan_source_comments(source: &str) -> Vec<SourceComment<'_>> {
    let bytes = source.as_bytes();
    let mut comments = Vec::new();
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'\'' | b'"' => {
                index = skip_string_literal(bytes, index);
            }
            b'`' => {
                index = skip_template_literal(bytes, index);
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
            _ => {
                index += 1;
            }
        }
    }

    comments
}

fn skip_string_literal(bytes: &[u8], start: usize) -> usize {
    let delim = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
        } else if bytes[i] == delim {
            return i + 1;
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
    fn string_containing_backslash_n_is_not_newline() {
        // The literal text \n in a string (escaped), not a real newline
        assert_eq!(all_comments("const x = '\\n'; // yes"), vec![(16, " yes")]);
    }
}
