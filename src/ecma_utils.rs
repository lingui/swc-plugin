use std::collections::HashSet;
use swc_common::DUMMY_SP;
use swc_core::ecma::ast::{*};
use swc_core::ecma::atoms::JsWord;

pub fn get_jsx_attr_value<'a>(el: &'a JSXOpeningElement, name: &str) -> &'a Option<JSXAttrValue> {
    for attr in &el.attrs {
        if let JSXAttrOrSpread::JSXAttr(attr) = &attr {
            if let JSXAttrName::Ident(ident) = &attr.name {
                if (&ident.sym) == name {
                    return &attr.value;
                }
            }
        }
    }

    return &None;
}

pub fn pick_jsx_attrs(mut attrs: Vec<JSXAttrOrSpread>, names: HashSet<&str>) -> Vec<JSXAttrOrSpread> {
    attrs.retain(|attr| {
        if let JSXAttrOrSpread::JSXAttr(attr) = attr {
            if let JSXAttrName::Ident(ident) = &attr.name {
                let name: &str = &ident.sym.to_string();
                if let Some(_) = names.get(name) {
                    return true;
                }
            }
        }
        return false;
    });

    attrs
}

pub fn get_jsx_attr<'a>(el: &'a JSXOpeningElement, name: &str) -> Option<&'a JSXAttr> {
    for attr in &el.attrs {
        if let JSXAttrOrSpread::JSXAttr(attr) = &attr {
            if let JSXAttrName::Ident(ident) = &attr.name {
                if (&ident.sym) == name {
                    return Some(attr);
                }
            }
        }
    }

    return None;
}

pub fn create_jsx_attribute(name: &str, exp: Box<Expr>) -> JSXAttrOrSpread {
    JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(Ident {
            span: DUMMY_SP,
            sym: name.into(),
            optional: false,
        }),
        value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            span: DUMMY_SP,
            expr: JSXExpr::Expr(exp),
        })),
    })
}


pub fn match_callee_name(call: &CallExpr, fn_name: &str) -> bool {
    match &call.callee {
        Callee::Expr(expr) => {
            if let Expr::Ident(ident) = expr.as_ref() {
                return &ident.sym == fn_name;
            }
        }
        _ => {}
    }

    false
}

// pub fn match_jsx_name(el: &JSXOpeningElement, name: &str) -> bool {
//     if let JSXElementName::Ident(ident) = &el.name {
//         return ident.sym.to_string() == name;
//     }
//     return false;
// }

pub fn create_import(source: JsWord, specifier: Ident) -> ModuleItem {
    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
        span: DUMMY_SP,
        specifiers: vec![
            ImportSpecifier::Named(ImportNamedSpecifier {
                span: DUMMY_SP,
                local: specifier,
                imported: None,
                is_type_only: false,
            })
        ],
        src: Box::new(Str {
            span: DUMMY_SP,
            value: source,
            raw: None,
        }),
        asserts: None,
        type_only: false,
    }))
}