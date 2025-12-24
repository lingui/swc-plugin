use std::collections::HashSet;
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::*;
use swc_core::ecma::atoms::Atom;
use swc_core::ecma::utils::quote_ident;

/// Helper function to panic with a helpful message when encountering unknown AST nodes
#[cfg(swc_ast_unknown)]
pub fn panic_unknown_node(node_type: &str, node: &dyn std::fmt::Debug) -> ! {
    panic!(
        "{} v{}: Unknown {} variant encountered.\n\
         Node: {:?}\n\n\
         This likely means your SWC version is newer than this plugin supports.\n\
         Please try:\n\
         1. Update this plugin to the latest version\n\
         2. Or downgrade @swc/core to a compatible version\n\
         3. Or report this issue at: {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        node_type,
        node,
        env!("CARGO_PKG_REPOSITORY")
    )
}

pub fn get_jsx_attr<'a>(el: &'a JSXOpeningElement, name: &str) -> Option<&'a JSXAttr> {
    for attr in &el.attrs {
        if let JSXAttrOrSpread::JSXAttr(attr) = attr {
            if let JSXAttrName::Ident(ident) = &attr.name {
                if ident.sym == *name {
                    return Some(attr);
                }
            }
        }
    }

    None
}

// get_local_ident_from_object_pat_prop(prop, "t")
// const {t} = useLingui() // => Ident("t")
// const {t: _} = useLingui() // => Ident("_")
pub fn get_local_ident_from_object_pat_prop(
    prop: &ObjectPatProp,
    imported_symbol: &str,
) -> Option<BindingIdent> {
    match prop {
        ObjectPatProp::KeyValue(key_value)
            if key_value
                .key
                .as_ident()
                .is_some_and(|ident| ident.sym == imported_symbol) =>
        {
            Some(key_value.value.as_ident().unwrap().clone())
        }
        ObjectPatProp::Assign(assign) if assign.key.sym == imported_symbol => {
            Some(assign.key.clone())
        }
        // Known variants that we don't support
        ObjectPatProp::KeyValue(_) => None, // Non-matching key
        ObjectPatProp::Assign(_) => None,   // Non-matching key
        ObjectPatProp::Rest(_) => None,

        #[cfg(swc_ast_unknown)]
        _ => panic_unknown_node("ObjectPatProp", prop),
    }
}

pub fn get_jsx_attr_value_as_string(val: &JSXAttrValue) -> Option<String> {
    match val {
        // offset="5"
        JSXAttrValue::Str(Str { value, .. }) => Some(value.to_string_lossy().into_owned()),
        // offset={..}
        JSXAttrValue::JSXExprContainer(JSXExprContainer {
            expr: JSXExpr::Expr(expr),
            ..
        }) => {
            match expr.as_ref() {
                // offset={"5"}
                Expr::Lit(Lit::Str(Str { value, .. })) => {
                    Some(value.to_string_lossy().into_owned())
                }
                // offset={5}
                Expr::Lit(Lit::Num(Number { value, .. })) => Some(value.to_string()),
                // All other known Expr variants that we don't support
                _ => None,
            }
        }
        // Known JSXAttrValue variants that we don't support
        JSXAttrValue::JSXElement(_) => None,
        JSXAttrValue::JSXFragment(_) => None,
        JSXAttrValue::JSXExprContainer(_) => None, // Non-Expr variants

        #[cfg(swc_ast_unknown)]
        _ => panic_unknown_node("JSXAttrValue", val),
    }
}

pub fn get_expr_as_string(val: &Expr) -> Option<String> {
    match val {
        // "Hello"
        Expr::Lit(Lit::Str(Str { value, .. })) => Some(value.to_string_lossy().into_owned()),

        // `Hello`
        Expr::Tpl(Tpl { quasis, .. }) => {
            if quasis.len() == 1 {
                Some(quasis.first().unwrap().raw.to_string())
            } else {
                None
            }
        }

        // All other known Expr variants that we don't support
        Expr::This(_)
        | Expr::Array(_)
        | Expr::Object(_)
        | Expr::Fn(_)
        | Expr::Unary(_)
        | Expr::Update(_)
        | Expr::Bin(_)
        | Expr::Assign(_)
        | Expr::Member(_)
        | Expr::SuperProp(_)
        | Expr::Cond(_)
        | Expr::Call(_)
        | Expr::New(_)
        | Expr::Seq(_)
        | Expr::Ident(_)
        | Expr::Lit(_)
        | Expr::Paren(_)
        | Expr::JSXMember(_)
        | Expr::JSXNamespacedName(_)
        | Expr::JSXEmpty(_)
        | Expr::JSXElement(_)
        | Expr::JSXFragment(_)
        | Expr::TsTypeAssertion(_)
        | Expr::TsConstAssertion(_)
        | Expr::TsNonNull(_)
        | Expr::TsAs(_)
        | Expr::TsInstantiation(_)
        | Expr::TsSatisfies(_)
        | Expr::PrivateName(_)
        | Expr::OptChain(_)
        | Expr::Invalid(_)
        | Expr::Yield(_)
        | Expr::Arrow(_)
        | Expr::Class(_)
        | Expr::Await(_)
        | Expr::MetaProp(_)
        | Expr::TaggedTpl(_) => None,

        #[cfg(swc_ast_unknown)]
        _ => panic_unknown_node("Expr", val),
    }
}

pub fn pick_jsx_attrs(
    mut attrs: Vec<JSXAttrOrSpread>,
    names: HashSet<&str>,
) -> Vec<JSXAttrOrSpread> {
    attrs.retain(|attr| {
        if let JSXAttrOrSpread::JSXAttr(attr) = attr {
            if let JSXAttrName::Ident(ident) = &attr.name {
                let name: &str = &ident.sym;
                return names.contains(name);
            }
        }
        false
    });

    attrs
}

pub fn create_jsx_attribute(name: &str, exp: Box<Expr>) -> JSXAttrOrSpread {
    JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(IdentName::new(name.into(), DUMMY_SP)),
        value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            span: DUMMY_SP,
            expr: JSXExpr::Expr(exp),
        })),
    })
}

pub fn match_callee_name<F: Fn(&Ident) -> bool>(call: &CallExpr, predicate: F) -> Option<&Ident> {
    if let Callee::Expr(expr) = &call.callee {
        if let Expr::Ident(ident) = expr.as_ref() {
            if predicate(ident) {
                return Some(ident);
            }
        }
    }

    None
}

pub fn to_key_value_prop(prop_or_spread: &PropOrSpread) -> Option<&KeyValueProp> {
    if let PropOrSpread::Prop(prop) = prop_or_spread {
        if let Prop::KeyValue(prop) = prop.as_ref() {
            return Some(prop);
        }
    }

    None
}

pub fn get_object_prop<'a>(props: &'a [PropOrSpread], name: &str) -> Option<&'a KeyValueProp> {
    props
        .iter()
        .filter_map(|prop_or_spread| to_key_value_prop(prop_or_spread))
        .find(|prop| {
            get_prop_key(prop)
                .map(|key| key.as_str() == name)
                .unwrap_or(false)
        })
}

pub fn get_prop_key(prop: &KeyValueProp) -> Option<Atom> {
    match &prop.key {
        PropName::Ident(IdentName { sym, .. }) => Some(sym.clone()),
        PropName::Str(Str { value, .. }) => Some(value.to_string_lossy().into_owned().into()),
        // Known PropName variants that we don't support
        PropName::Num(_) => None,
        PropName::Computed(_) => None,
        PropName::BigInt(_) => None,

        #[cfg(swc_ast_unknown)]
        _ => panic_unknown_node("PropName", &prop.key),
    }
}

// recursively expands TypeScript's as expressions until it reaches a real value
pub fn expand_ts_as_expr(mut expr: Box<Expr>) -> Box<Expr> {
    while let Expr::TsAs(TsAsExpr {
        expr: inner_expr, ..
    }) = *expr
    {
        expr = inner_expr;
    }
    expr
}

pub fn create_key_value_prop(key: &str, value: Box<Expr>) -> PropOrSpread {
    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
        key: PropName::Ident(quote_ident!(key)),
        value,
    })))
}

pub fn create_import(source: Atom, imported: IdentName, local: IdentName) -> ModuleItem {
    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
        span: DUMMY_SP,
        phase: ImportPhase::default(),
        specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
            span: DUMMY_SP,
            local: local.into(),
            imported: Some(ModuleExportName::Ident(imported.into())),
            is_type_only: false,
        })],
        src: Box::new(Str {
            span: DUMMY_SP,
            value: source.to_string().into(),
            raw: None,
        }),
        with: None,
        type_only: false,
    }))
}
