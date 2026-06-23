use super::*;

// ---------------------------------------------------------------------------
// locate_directives — the substring scanner
// ---------------------------------------------------------------------------

/// Compact view of a located directive for assertions: (reset, params).
fn located(source: &str) -> Vec<(bool, &str)> {
    locate_directives(source)
        .into_iter()
        .map(|d| (d.reset, d.params))
        .collect()
}

#[test]
fn locate_line_comment_directive() {
    assert_eq!(
        located("// lingui-set context=\"a\"\ncode"),
        vec![(false, " context=\"a\"")]
    );
}

#[test]
fn locate_line_comment_at_eof_without_newline() {
    assert_eq!(
        located("// lingui-set context=\"a\""),
        vec![(false, " context=\"a\"")]
    );
}

#[test]
fn locate_block_comment_directive() {
    assert_eq!(
        located("/* lingui-set context=\"a\" */"),
        vec![(false, " context=\"a\" ")]
    );
}

#[test]
fn locate_jsdoc_block_comment_directive() {
    assert_eq!(
        located("/** lingui-set context=\"a\" */"),
        vec![(false, " context=\"a\" ")]
    );
}

#[test]
fn locate_block_comment_with_trailing_code_on_same_line() {
    // This is the case the regex scanner broke on: the directive must end at
    // `*/`, not swallow the rest of the line.
    assert_eq!(
        located("/* lingui-set context=\"a\" */ const x = 1;"),
        vec![(false, " context=\"a\" ")]
    );
}

#[test]
fn locate_jsx_expression_container_block_comment() {
    // `{/* lingui-reset */}` — the only valid block-comment form inside JSX.
    assert_eq!(
        located("<div>{/* lingui-reset */}<Trans>Hi</Trans></div>"),
        vec![(true, " ")]
    );
}

#[test]
fn locate_jsx_expression_container_set_directive() {
    assert_eq!(
        located("<div>{/* lingui-set context=\"x\" */}<Trans>Hi</Trans></div>"),
        vec![(false, " context=\"x\" ")]
    );
}

#[test]
fn locate_rejects_directive_on_line_after_block_opener() {
    // The directive must be on the opener's line; a newline between `/*` and
    // the keyword means it is not recognised.
    assert_eq!(
        located("/*\n  lingui-set context=\"a\"\n*/"),
        Vec::<(bool, &str)>::new()
    );
}

#[test]
fn locate_reset_directive() {
    assert_eq!(located("// lingui-reset\ncode"), vec![(true, "")]);
}

#[test]
fn locate_unterminated_block_comment_runs_to_eof() {
    assert_eq!(
        located("/* lingui-set context=\"a\""),
        vec![(false, " context=\"a\"")]
    );
}

#[test]
fn locate_word_boundary_rejects_longer_identifier() {
    // `lingui-settings` / `lingui-resetter` are not directives.
    assert_eq!(
        located("// lingui-settings here"),
        Vec::<(bool, &str)>::new()
    );
    assert_eq!(
        located("// lingui-resetter here"),
        Vec::<(bool, &str)>::new()
    );
}

#[test]
fn locate_requires_a_comment_opener() {
    // A bare `lingui-set` not introduced by a comment is ignored.
    assert_eq!(
        located("const lingui_set = 1; lingui-set"),
        Vec::<(bool, &str)>::new()
    );
}

#[test]
fn locate_multiple_directives_in_order() {
    assert_eq!(
        located(
            "// lingui-set context=\"a\"\ncode\n/* lingui-reset */\n// lingui-set comment=\"c\""
        ),
        vec![
            (false, " context=\"a\""),
            (true, " "),
            (false, " comment=\"c\"")
        ]
    );
}

#[test]
fn locate_reports_full_span_and_fields() {
    // Comment start points at the `/` of the introducing comment (even when
    // preceded by code), and comment end points just past the closing `*/`.
    assert_eq!(
        locate_directives("<div>{/* lingui-reset */}</div>"),
        vec![LocatedDirective {
            comment_start: 6, // the `/*`
            comment_end: 24,  // just past `*/`
            reset: true,
            params: " ",
        }]
    );
}

#[test]
fn locate_ignores_lingui_prefix_not_followed_by_a_directive() {
    // `lingui-` followed by neither `set` nor `reset` is not a directive.
    assert_eq!(located("// lingui-format here"), Vec::<(bool, &str)>::new());
}

#[test]
fn locate_ignores_division_and_plain_comments() {
    assert_eq!(
        located("const x = 10 / 2; // just a note"),
        Vec::<(bool, &str)>::new()
    );
}

#[test]
fn locate_block_comment_directive_without_space_before_terminator() {
    assert_eq!(
        located("/* lingui-set context=\"a\"*/"),
        vec![(false, " context=\"a\"")]
    );
}

// ---------------------------------------------------------------------------
// block_comment_close
// ---------------------------------------------------------------------------

#[test]
fn block_close_finds_terminator() {
    assert_eq!(block_comment_close(b" x */", 0), Some(3));
}

#[test]
fn block_close_stops_at_first_terminator() {
    // No string awareness: the first `*/` wins, matching JS comment lexing.
    assert_eq!(block_comment_close(b" \"a */ b\" */", 0), Some(4));
}

#[test]
fn block_close_unterminated_is_none() {
    assert_eq!(block_comment_close(b" no terminator", 0), None);
}

// ---------------------------------------------------------------------------
// parse_lingui_directive
// ---------------------------------------------------------------------------

#[test]
fn parse_should_parse_multiple_keys() {
    let parsed =
        parse_lingui_directive(false, r#" context="ctx" comment="cmt" idPrefix="p." "#).unwrap();

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
fn parse_should_accept_bare_reset() {
    let parsed = parse_lingui_directive(true, "").unwrap();
    assert_eq!(
        parsed,
        ParsedDirective {
            reset: true,
            values: DirectiveUpdate::default(),
        }
    );
}

#[test]
fn parse_reset_may_carry_new_values() {
    let parsed = parse_lingui_directive(true, r#" context="fresh" "#).unwrap();
    assert_eq!(
        parsed,
        ParsedDirective {
            reset: true,
            values: DirectiveUpdate {
                context: Some(DirectiveValueUpdate::Set("fresh".into())),
                ..Default::default()
            }
        }
    );
}

#[test]
fn parse_should_reject_invalid_syntax() {
    let error = parse_lingui_directive(false, " context=single ")
        .expect_err("expected parser to reject invalid syntax");
    assert!(error.contains("requires a value"));
}

#[test]
fn parse_should_reject_unknown_params() {
    let error = parse_lingui_directive(false, r#" unknown="value" "#)
        .expect_err("expected parser to reject unknown params");
    assert!(error.contains("unknown param \"unknown\""));
}

#[test]
fn parse_should_reject_set_without_params() {
    let error = parse_lingui_directive(false, "  ")
        .expect_err("expected parser to reject set with no params");
    assert!(error.contains("requires at least one param"));
}

#[test]
fn parse_should_treat_empty_strings_as_unset() {
    let parsed = parse_lingui_directive(false, r#" context="" comment="note" "#).unwrap();
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
fn parse_should_reject_missing_equals() {
    let error = parse_lingui_directive(false, "context")
        .expect_err("expected parser to reject a key with no `=value`");
    assert!(error.contains("requires a value"));
}

#[test]
fn parse_should_reject_empty_key() {
    // A param position that does not begin with a word char yields no key.
    let error = parse_lingui_directive(false, "=\"x\"")
        .expect_err("expected parser to reject a missing key");
    assert!(error.contains("invalid syntax"));
}

#[test]
fn parse_should_reject_unterminated_value() {
    let error = parse_lingui_directive(false, "context=\"unterminated")
        .expect_err("expected parser to reject an unterminated quoted value");
    assert!(error.contains("invalid syntax"));
}

// ---------------------------------------------------------------------------
// collect_lingui_directives_from_source — accumulation + positions
// ---------------------------------------------------------------------------

#[test]
fn collect_returns_empty_when_no_directive_substring() {
    assert_eq!(
        collect_lingui_directives_from_source("const x = 1; // hi", BytePos(1)),
        vec![]
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
fn collect_block_comment_with_trailing_code_parses_cleanly() {
    // Regression for the regex scanner: trailing code after `*/` must not leak
    // into the directive parameters and cause a syntax error.
    let directives = collect_lingui_directives_from_source(
        "{/* lingui-set context=\"a\" */}\n<Trans>Hi</Trans>",
        BytePos(1),
    );
    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].values.context.as_deref(), Some("a"));
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

    let values: Vec<_> = directives.iter().map(|d| d.values.clone()).collect();
    assert_eq!(
        values,
        vec![
            DirectiveValues {
                context: Some("ctx1".into()),
                comment: None,
                id_prefix: None,
            },
            DirectiveValues {
                context: Some("ctx1".into()),
                comment: Some("cmt".into()),
                id_prefix: None,
            },
            DirectiveValues {
                context: Some("ctx2".into()),
                comment: None,
                id_prefix: None,
            },
        ]
    );
    // Positions must be strictly increasing (binary search in find_for_pos
    // relies on this).
    assert!(directives.windows(2).all(|w| w[0].pos < w[1].pos));
}

#[test]
fn collect_reset_then_set_accumulates_from_reset() {
    let directives = collect_lingui_directives_from_source(
        "/* lingui-set context=\"a\" comment=\"c\" */\n/* lingui-reset */\n/* lingui-set context=\"b\" */",
        BytePos(1),
    );
    let last = &directives.last().unwrap().values;
    assert_eq!(last.context.as_deref(), Some("b"));
    assert_eq!(
        last.comment, None,
        "reset must have cleared the inherited comment"
    );
}

// ---------------------------------------------------------------------------
// find_directive_for_pos
// ---------------------------------------------------------------------------

#[test]
fn find_returns_none_for_empty_directives() {
    assert_eq!(find_directive_for_pos(&[], BytePos(5)), None);
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
