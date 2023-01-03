use swc_common::DUMMY_SP;
use swc_core::ecma::ast::{*};

pub fn get_jsx_attr_value<'a>(el: &'a JSXOpeningElement, name: &str) -> &'a Option<JSXAttrValue> {
    for attr in &el.attrs {
        if let JSXAttrOrSpread::JSXAttr(attr) = &attr {
            if let JSXAttrName::Ident(ident) = &attr.name {
                if (&ident.sym) == name {
                    return &attr.value
                }
            }
        }
    }

    return &Option::None;
}

pub fn create_jsx_attribute(name: &str, exp: Expr) -> JSXAttrOrSpread {
    JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(Ident {
            span: DUMMY_SP,
            sym: name.into(),
            optional: false,
        }),
        value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            span: DUMMY_SP,
            expr: JSXExpr::Expr(Box::new(exp)),
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

pub fn match_jsx_name(el: &JSXOpeningElement, name: &str) -> bool {
    if let JSXElementName::Ident(ident) = &el.name {
        return ident.sym.to_string() == name;
    }
    return false;
}