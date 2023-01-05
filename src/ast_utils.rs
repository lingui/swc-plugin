use std::collections::HashSet;
use swc_common::DUMMY_SP;
use swc_core::ecma::ast::{*};
use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::utils::quote_ident;

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

pub fn get_jsx_attr_value_as_string(val: &JSXAttrValue) -> Option<String> {
    match val {
        // offset="5"
        JSXAttrValue::Lit(Lit::Str(Str {value, ..})) => {
            return Some(value.to_string());
        }
        // offset={..}
        JSXAttrValue::JSXExprContainer(JSXExprContainer {expr: JSXExpr::Expr(expr), ..}) => {
            match expr.as_ref() {
                // offset={"5"}
                Expr::Lit(Lit::Str(Str {value, ..})) => {
                    return Some(value.to_string());
                }
                // offset={5}
                Expr::Lit(Lit::Num(Number {value, ..})) => {
                    return Some(value.to_string());
                }
                _ => None
            }
        }
        _ => None
    }
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

pub fn match_callee_name<F: Fn(&JsWord) -> bool>(call: &CallExpr, predicate: F) -> Option<&Ident> {
    if let Callee::Expr(expr) = &call.callee {
        if let Expr::Ident(ident) = expr.as_ref() {
            if predicate(&ident.sym) {
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

pub fn has_object_prop(props: &Vec<PropOrSpread>, name: &str) -> bool {
   for prop_or_spread in props {
       if let Some(prop) = to_key_value_prop(prop_or_spread) {
          if  match_prop_key(prop, name) {
              return true;
          }
       }
   }

    false
}

pub fn match_prop_key(prop: &KeyValueProp, name: &str) -> bool {
    match &prop.key {
        PropName::Ident(Ident { sym, .. })
        | PropName::Str(Str { value: sym, .. }) => {
            sym.to_string() == name
        }
        _ => {
            false
        }
    }
}

// pub fn match_jsx_name(el: &JSXOpeningElement, name: &str) -> bool {
//     if let JSXElementName::Ident(ident) = &el.name {
//         return ident.sym.to_string() == name;
//     }
//     return false;
// }

pub fn create_key_value_prop(key: &str, value: Box<Expr>) -> PropOrSpread {
    return PropOrSpread::Prop(Box::new(Prop::KeyValue(
        KeyValueProp {
            key: PropName::Ident(quote_ident!(key)),
            value,
        }
    )));
}

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