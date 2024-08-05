use std::collections::HashSet;
use swc_core::common::DUMMY_SP;
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
        JSXAttrValue::Lit(Lit::Str(Str { value, .. })) => {
            return Some(value.to_string());
        }
        // offset={..}
        JSXAttrValue::JSXExprContainer(JSXExprContainer { expr: JSXExpr::Expr(expr), .. }) => {
            match expr.as_ref() {
                // offset={"5"}
                Expr::Lit(Lit::Str(Str { value, .. })) => {
                    return Some(value.to_string());
                }
                // offset={5}
                Expr::Lit(Lit::Num(Number { value, .. })) => {
                    return Some(value.to_string());
                }
                _ => None
            }
        }
        _ => None
    }
}

pub fn get_expr_as_string(val: &Box<Expr>) -> Option<String> {
  match val.as_ref() {
    // "Hello"
    Expr::Lit(Lit::Str(Str { value, .. })) => {
      return Some(value.to_string());
    }

    // `Hello`
    Expr::Tpl(Tpl {quasis, ..}) => {
      if quasis.len() == 1 {
        return Some(quasis.get(0).unwrap().raw.to_string());
      } else { None }
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

pub fn match_callee_name<F: Fn(&Ident) -> bool>(call: &CallExpr, predicate: F) -> Option<&Ident> {
    if let Callee::Expr(expr) = &call.callee {
        if let Expr::Ident(ident) = expr.as_ref() {
            if predicate(&ident) {
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

pub fn get_object_prop<'a>(props: &'a Vec<PropOrSpread>, name: &str) -> Option<&'a KeyValueProp> {
  props.iter()
    .filter_map(|prop_or_spread| to_key_value_prop(prop_or_spread))
    .find(|prop| {
      get_prop_key(prop)
        .and_then(|key| {
          if key == name { Some(key) } else { None }
        }).is_some()
    })
}

pub fn get_prop_key(prop: &KeyValueProp) -> Option<&JsWord> {
    match &prop.key {
        PropName::Ident(Ident { sym, .. })
        | PropName::Str(Str { value: sym, .. }) => {
            Some(sym)
        }
        _ => {
            None
        }
    }
}

pub fn create_key_value_prop(key: &str, value: Box<Expr>) -> PropOrSpread {
    return PropOrSpread::Prop(Box::new(Prop::KeyValue(
        KeyValueProp {
            key: PropName::Ident(quote_ident!(key)),
            value,
        }
    )));
}

pub fn create_import(source: JsWord, imported: Ident, local: Ident) -> ModuleItem {
    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
        span: DUMMY_SP,
        phase: ImportPhase::default(),
        specifiers: vec![
            ImportSpecifier::Named(ImportNamedSpecifier {
                span: DUMMY_SP,
                local,
                imported: Some(ModuleExportName::Ident(imported)),
                is_type_only: false,
            })
        ],
        src: Box::new(Str {
            span: DUMMY_SP,
            value: source,
            raw: None,
        }),
        with: None,
        type_only: false,
    }))
}
