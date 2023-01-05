use crate::ast_utils::*;
use crate::tokens::*;
use swc_core::{
    ecma::{
        ast::*,
        atoms::JsWord
    },
};

const LINGUI_T: &str = &"t";

fn is_lingui_fn(name: &str) -> bool {
    // todo: i didn't find a better way to create a constant hashmap
    match name {
        "plural" | "select" | "selectOrdinal" => true,
        _ => false,
    }
}

pub fn is_lingui_jsx_el(name: &str) -> bool {
    // todo: i didn't find a better way to create a constant hashmap
    match name {
        "Plural" | "Select" | "SelectOrdinal" => true,
        _ => false,
    }
}

/// Take a callee expression and detect is it a lingui t`` macro call
/// Returns a callee object depending whether custom i18n instance was passed or not
pub fn is_lingui_t_call_expr(callee_expr: &Box<Expr>) -> (bool, Option<Box<Expr>>) {
    match callee_expr.as_ref() {
        // t(i18n)...
        Expr::Call(call) if matches!(match_callee_name(call, |n| n == LINGUI_T), Some(_)) => {
            if let Some(v) = call.args.get(0) {
                (true, Some(v.expr.clone()))
            } else {
                (false, None)
            }
        }
        // t..
        Expr::Ident(ident) if &ident.sym == LINGUI_T => {
            (true, None)
        }
        _ => {
            (false, None)
        }
    }
}

/// Receive TemplateLiteral with variables and return MsgTokens
pub fn tokenize_tpl(tpl: &Tpl) -> Vec<MsgToken> {
    let mut tokens: Vec<MsgToken> = Vec::with_capacity(tpl.quasis.len());

    for (i, tpl_element) in tpl.quasis.iter().enumerate() {
        tokens.push(MsgToken::String(tpl_element.raw.to_string()));

        if let Some(exp) = tpl.exprs.get(i) {
            if let Expr::Call(call) = exp.as_ref() {
                if let Some(call_tokens) = try_tokenize_call_expr_as_icu(call) {
                    tokens.extend(call_tokens);
                    continue;
                }
            }

            tokens.push(MsgToken::Expression(exp.clone()));
        }
    }

    tokens
}

/// Try to tokenize call expression as IVU Choice macro
/// Return None if this call is not related to macros or is not parsable
pub fn try_tokenize_call_expr_as_icu(expr: &CallExpr) -> Option<Vec<MsgToken>> {
    if let Some(ident) = match_callee_name(&expr, |name| is_lingui_fn(name)) {
        if expr.args.len() != 2 {
            // malformed plural call, exit
            return None;
        }

        // ICU value
        let arg = expr.args.get(0).unwrap();
        let icu_value = arg.expr.clone();

        // ICU Choices
        let arg = expr.args.get(1).unwrap();
        if let Expr::Object(object) = &arg.expr.as_ref() {
            let icu_method = ident.sym.to_lowercase();
            let choices = get_choices_from_obj(&object.props, &icu_method);

            return Some(vec![MsgToken::Icu(Icu {
                icu_method,
                value: icu_value,
                choices,
            })]);
        } else {
            // todo passed not an ObjectLiteral,
            //      we should panic here or just skip this call
        }
    }

    return None;
}

pub fn try_tokenize_expr(expr: &Box<Expr>) -> Option<Vec<MsgToken>> {
    match expr.as_ref() {
        // String Literal: "has # friend"
        Expr::Lit(Lit::Str(str)) => {
            Some(vec!(MsgToken::String(str.clone().value.to_string())))
        }
        // Template Literal: `${name} has # friend`
        Expr::Tpl(tpl) => {
            Some(tokenize_tpl(tpl))
        }

        // ParenthesisExpression: ("has # friend")
        Expr::Paren(ParenExpr { expr, .. }) => {
            try_tokenize_expr(expr)
        }

        // Call Expression: {one: plural(numArticles, {...})}
        Expr::Call(expr) => {
            try_tokenize_call_expr_as_icu(expr)
        }
        _ => None
    }
}

/// Take KeyValueProp and return Key as string if parsable
/// If key is numeric, return an exact match syntax `={number}`
pub fn get_js_choice_key(prop: &KeyValueProp ) -> Option<JsWord> {
    match &prop.key {
        // {one: ""}
        PropName::Ident(Ident { sym, .. })
        // {"one": ""}
        | PropName::Str(Str { value: sym, .. }) => {
            Some(sym.clone())
        }
        // {0: ""} -> `={number}`
        PropName::Num(Number {value, ..}) => {
            Some(format!("={value}").into())
        }
        _ => {
            None
        }
    }
}


/// receive ObjectLiteral {few: "..", many: "..", other: ".."} and create tokens
/// If messages passed as TemplateLiterals with variables, it extracts variables
pub fn get_choices_from_obj(props: &Vec<PropOrSpread>, icu_format: &str) -> Vec<IcuChoiceOrOffset> {
    // todo: there might be more props then real choices. Id for example
    let mut choices: Vec<IcuChoiceOrOffset> = Vec::with_capacity(props.len());

    for prop_or_spread in props {
        if let PropOrSpread::Prop(prop) = prop_or_spread {
            if let Prop::KeyValue(prop) = prop.as_ref() {
                if let Some(key) = get_js_choice_key(prop) {
                    if &key == "offset" && icu_format != "select" {
                        if let Expr::Lit(Lit::Num(Number {value, ..})) = prop.value.as_ref() {
                            choices.push(IcuChoiceOrOffset::Offset(value.to_string()))
                        } else {
                            // todo: panic offset might be only a number, other forms is not supported
                        }
                    } else {
                        let tokens = try_tokenize_expr(&prop.value)
                            .unwrap_or(Vec::new());

                        choices.push(IcuChoiceOrOffset::IcuChoice(IcuChoice {
                            tokens,
                            key: key.to_string(),
                        }));
                    }
                }
            } else {
                // todo: panic here we could not parse anything else then KeyValue pair
            }
        } else {
            // todo: panic here, we could not parse spread
        }
    }

    choices
}