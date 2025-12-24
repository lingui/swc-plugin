// Enum coverage tests for SWC cross-version compatibility
//
// These tests verify that all known enum variants are properly handled in pattern matches.
// This ensures that panic_unknown_node() only triggers for truly unknown variants from
// future SWC versions, not for any currently existing variants.
//
// See: https://swc.rs/docs/plugin/ecmascript/compatibility
//
// Background: Starting from @swc/core v1.15.0, SWC introduced enhanced compatibility
// for Wasm plugins. By enabling the `swc_ast_unknown` cfg flag and explicitly enumerating
// all known enum variants, plugins can work with newer SWC versions until new features
// are actually encountered in the code.

#[test]
fn test_object_pat_prop_variants() {
    use crate::ast_utils::*;
    use swc_core::common::DUMMY_SP;
    use swc_core::ecma::ast::*;
    use swc_core::ecma::utils::quote_ident;

    // Test KeyValue variant
    let key_value = ObjectPatProp::KeyValue(KeyValuePatProp {
        key: PropName::Ident(quote_ident!("test")),
        value: Box::new(Pat::Ident(quote_ident!("value").into())),
    });

    // Should return None for non-matching symbol
    assert_eq!(
        get_local_ident_from_object_pat_prop(&key_value, "other"),
        None
    );

    // Test Assign variant
    let assign = ObjectPatProp::Assign(AssignPatProp {
        span: DUMMY_SP,
        key: quote_ident!("test").into(),
        value: None,
    });

    // Should return Some for matching symbol
    assert!(get_local_ident_from_object_pat_prop(&assign, "test").is_some());

    // Test Rest variant - should return None
    let rest = ObjectPatProp::Rest(RestPat {
        span: DUMMY_SP,
        dot3_token: DUMMY_SP,
        arg: Box::new(Pat::Ident(quote_ident!("rest").into())),
        type_ann: None,
    });

    assert_eq!(get_local_ident_from_object_pat_prop(&rest, "test"), None);
}

#[test]
fn test_jsx_attr_value_variants() {
    use crate::ast_utils::*;
    use swc_core::common::DUMMY_SP;
    use swc_core::ecma::ast::*;

    // Test Str variant
    let str_val = JSXAttrValue::Str(Str {
        span: DUMMY_SP,
        value: "test".into(),
        raw: None,
    });
    assert_eq!(
        get_jsx_attr_value_as_string(&str_val),
        Some("test".to_string())
    );

    // Test JSXExprContainer with Lit::Str
    let expr_str = JSXAttrValue::JSXExprContainer(JSXExprContainer {
        span: DUMMY_SP,
        expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: "test".into(),
            raw: None,
        })))),
    });
    assert_eq!(
        get_jsx_attr_value_as_string(&expr_str),
        Some("test".to_string())
    );

    // Test JSXExprContainer with Lit::Num
    let expr_num = JSXAttrValue::JSXExprContainer(JSXExprContainer {
        span: DUMMY_SP,
        expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num(Number {
            span: DUMMY_SP,
            value: 42.0,
            raw: None,
        })))),
    });
    assert_eq!(
        get_jsx_attr_value_as_string(&expr_num),
        Some("42".to_string())
    );

    // Test JSXElement variant - should return None
    let jsx_elem = JSXAttrValue::JSXElement(Box::new(JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(IdentName::new("div".into(), DUMMY_SP).into()),
            attrs: vec![],
            self_closing: true,
            type_args: None,
        },
        children: vec![],
        closing: None,
    }));
    assert_eq!(get_jsx_attr_value_as_string(&jsx_elem), None);

    // Test JSXFragment variant - should return None
    let jsx_frag = JSXAttrValue::JSXFragment(JSXFragment {
        span: DUMMY_SP,
        opening: JSXOpeningFragment { span: DUMMY_SP },
        children: vec![],
        closing: JSXClosingFragment { span: DUMMY_SP },
    });
    assert_eq!(get_jsx_attr_value_as_string(&jsx_frag), None);
}

#[test]
fn test_expr_as_string_variants() {
    use crate::ast_utils::*;
    use swc_core::common::DUMMY_SP;
    use swc_core::ecma::ast::*;
    // Test Lit::Str variant
    let str_expr = Expr::Lit(Lit::Str(Str {
        span: DUMMY_SP,
        value: "hello".into(),
        raw: None,
    }));
    assert_eq!(get_expr_as_string(&str_expr), Some("hello".to_string()));

    // Test Tpl variant with single quasi
    let tpl_expr = Expr::Tpl(Tpl {
        span: DUMMY_SP,
        exprs: vec![],
        quasis: vec![TplElement {
            span: DUMMY_SP,
            tail: true,
            cooked: Some("template".into()),
            raw: "template".into(),
        }],
    });
    assert_eq!(get_expr_as_string(&tpl_expr), Some("template".to_string()));

    // Test Tpl variant with multiple quasis - should return None
    let tpl_multi = Expr::Tpl(Tpl {
        span: DUMMY_SP,
        exprs: vec![Box::new(Expr::Lit(Lit::Num(Number {
            span: DUMMY_SP,
            value: 1.0,
            raw: None,
        })))],
        quasis: vec![
            TplElement {
                span: DUMMY_SP,
                tail: false,
                cooked: Some("part1".into()),
                raw: "part1".into(),
            },
            TplElement {
                span: DUMMY_SP,
                tail: true,
                cooked: Some("part2".into()),
                raw: "part2".into(),
            },
        ],
    });
    assert_eq!(get_expr_as_string(&tpl_multi), None);
}

#[test]
fn test_prop_name_variants() {
    use crate::ast_utils::*;
    use swc_core::common::DUMMY_SP;
    use swc_core::ecma::ast::*;
    use swc_core::ecma::utils::quote_ident;

    // Test Ident variant
    let ident_prop = KeyValueProp {
        key: PropName::Ident(quote_ident!("test")),
        value: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
    };
    assert_eq!(get_prop_key(&ident_prop), Some("test".into()));

    // Test Str variant
    let str_prop = KeyValueProp {
        key: PropName::Str(Str {
            span: DUMMY_SP,
            value: "test".into(),
            raw: None,
        }),
        value: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
    };
    assert_eq!(get_prop_key(&str_prop), Some("test".into()));

    // Test Num variant - should return None
    let num_prop = KeyValueProp {
        key: PropName::Num(Number {
            span: DUMMY_SP,
            value: 42.0,
            raw: None,
        }),
        value: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
    };
    assert_eq!(get_prop_key(&num_prop), None);

    // Test Computed variant - should return None
    let computed_prop = KeyValueProp {
        key: PropName::Computed(ComputedPropName {
            span: DUMMY_SP,
            expr: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: "computed".into(),
                raw: None,
            }))),
        }),
        value: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
    };
    assert_eq!(get_prop_key(&computed_prop), None);

    // Test BigInt variant - should return None
    let bigint_prop = KeyValueProp {
        key: PropName::BigInt(BigInt {
            span: DUMMY_SP,
            value: Box::new(BigIntValue::from(42)),
            raw: None,
        }),
        value: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
    };
    assert_eq!(get_prop_key(&bigint_prop), None);
}

#[test]
fn test_get_expr_as_string_does_not_panic_on_known_variants() {
    use crate::ast_utils::*;
    use swc_core::common::{SyntaxContext, DUMMY_SP};
    use swc_core::ecma::ast::{Expr, Ident};

    // Test Expr::Ident - a known variant that get_expr_as_string doesn't support
    let expr = Expr::Ident(Ident {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        sym: "idVar".into(),
        optional: false,
    });

    // Should return None for Ident, not panic
    // This is a known variant that we don't support, not an unknown variant
    let result = std::panic::catch_unwind(|| get_expr_as_string(&expr))
        .expect("should not panic on known Expr::Ident variant");

    assert_eq!(result, None);
}

#[test]
fn test_macro_ctx_get_js_choice_case_key_with_unsupported_prop() {
    use crate::macro_utils::MacroCtx;
    use crate::LinguiOptions;
    use swc_core::common::DUMMY_SP;
    use swc_core::ecma::ast::*;

    let ctx = MacroCtx::new(LinguiOptions::default());

    // Test with PropName::Computed - a known variant that get_js_choice_case_key doesn't support
    let computed_prop = KeyValueProp {
        key: PropName::Computed(ComputedPropName {
            span: DUMMY_SP,
            expr: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: "computed".into(),
                raw: None,
            }))),
        }),
        value: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
    };

    // Should return None for unsupported known variants, not panic
    let result = ctx.get_js_choice_case_key(&computed_prop);
    assert_eq!(result, None);

    // Test with PropName::BigInt - another known variant that's not supported
    let bigint_prop = KeyValueProp {
        key: PropName::BigInt(BigInt {
            span: DUMMY_SP,
            value: Box::new(BigIntValue::from(42)),
            raw: None,
        }),
        value: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
    };

    let result = ctx.get_js_choice_case_key(&bigint_prop);
    assert_eq!(result, None);

    // Test with PropName::Num - should return formatted number
    let num_prop = KeyValueProp {
        key: PropName::Num(Number {
            span: DUMMY_SP,
            value: 0.0,
            raw: None,
        }),
        value: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
    };

    let result = ctx.get_js_choice_case_key(&num_prop);
    assert_eq!(result, Some("=0".into()));
}
