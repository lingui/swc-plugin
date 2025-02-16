use crate::ast_utils::*;
use crate::tokens::*;
use crate::LinguiOptions;
use std::collections::{HashMap, HashSet};
use swc_core::ecma::{ast::*, atoms::JsWord};
use swc_core::ecma::utils::quote_ident;

const LINGUI_T: &str = &"t";

#[derive(Default, Clone)]
pub struct MacroCtx {
    // export name -> local name
    symbol_to_id_map: HashMap<JsWord, HashSet<Id>>,
    // local name -> export name
    id_to_symbol_map: HashMap<Id, JsWord>,

    pub should_add_18n_import: bool,
    pub should_add_trans_import: bool,
    pub should_add_uselingui_import: bool,

    pub options: LinguiOptions,
    pub runtime_idents: RuntimeIdents,
}

#[derive(Clone)]
pub struct RuntimeIdents {
    pub i18n: IdentName,
    pub trans: IdentName,
    pub use_lingui: IdentName,
}

impl Default for RuntimeIdents {
    fn default() -> RuntimeIdents {
        RuntimeIdents {
            i18n: quote_ident!("$_i18n"),
            trans: quote_ident!("Trans_"),
            use_lingui: quote_ident!("$_useLingui"),
        }
    }
}

impl MacroCtx {
    pub fn new(options: LinguiOptions) -> MacroCtx {
        MacroCtx {
            options,
            ..Default::default()
        }
    }

    /// is given ident exported from @lingui/macro? and one of choice functions?
    fn is_lingui_fn_choice_cmp(&self, ident: &Ident) -> bool {
        // self.symbol_to_id_map.
        self.is_lingui_ident("plural", ident)
            || self.is_lingui_ident("select", ident)
            || self.is_lingui_ident("selectOrdinal", ident)
    }

    pub fn is_lingui_placeholder_expr(&self, ident: &Ident) -> bool {
        return self.is_lingui_fn_choice_cmp(&ident)
            || self.is_lingui_ident("ph", &ident);
    }

    /// is given ident exported from @lingui/macro?
    pub fn is_lingui_ident(&self, name: &str, ident: &Ident) -> bool {
        self.symbol_to_id_map
            .get(&name.into())
            .and_then(|refs| refs.get(&ident.to_id()))
            .is_some()
    }

    pub fn is_define_message_ident(&self, ident: &Ident) -> bool {
        return self.is_lingui_ident("defineMessage", &ident)
            || self.is_lingui_ident("msg", &ident);
    }

    /// given import {plural as i18nPlural} from "@lingui/macro";
    /// get_ident_export_name("i18nPlural") would return `plural`
    pub fn get_ident_export_name(&self, ident: &Ident) -> Option<&JsWord> {
        if let Some(name) = self.id_to_symbol_map.get(&ident.to_id()) {
            return Some(name);
        }

        None
    }

    pub fn is_lingui_jsx_choice_cmp(&self, ident: &Ident) -> bool {
        self.is_lingui_ident("Plural", ident)
            || self.is_lingui_ident("Select", ident)
            || self.is_lingui_ident("SelectOrdinal", ident)
    }

    pub fn register_reference(&mut self, symbol: &JsWord, id: &Id) {
        self.symbol_to_id_map
            .entry(symbol.clone())
            .or_default()
            .insert(id.clone());
        
        self.id_to_symbol_map
            .insert(id.clone(), symbol.clone());
    }
    pub fn register_macro_import(&mut self, imp: &ImportDecl) {
        for spec in &imp.specifiers {
            if let ImportSpecifier::Named(spec) = spec {
                if let Some(ModuleExportName::Ident(ident)) = &spec.imported {
                    self.register_reference(&ident.sym, &spec.local.to_id());
                } else {
                    self.register_reference(&spec.local.sym, &spec.local.to_id());
                }
            }
        }
    }

    /// Take a callee expression and detect is it a lingui t`` macro call
    /// Returns a callee object depending whether custom i18n instance was passed or not
    pub fn is_lingui_t_call_expr(&self, callee_expr: &Box<Expr>) -> (bool, Option<Box<Expr>>) {
        match callee_expr.as_ref() {
            // t(i18n)...
            Expr::Call(call)
                if matches!(
                    match_callee_name(call, |n| self.is_lingui_ident(LINGUI_T, n)),
                    Some(_)
                ) =>
            {
                if let Some(v) = call.args.get(0) {
                    (true, Some(v.expr.clone()))
                } else {
                    (false, None)
                }
            }
            // t..
            Expr::Ident(ident) if self.is_lingui_ident(LINGUI_T, &ident) => (true, None),
            _ => (false, None),
        }
    }

    /// Receive TemplateLiteral with variables and return MsgTokens
    pub fn tokenize_tpl(&self, tpl: &Tpl) -> Vec<MsgToken> {
        let mut tokens: Vec<MsgToken> = Vec::with_capacity(tpl.quasis.len());

        for (i, tpl_element) in tpl.quasis.iter().enumerate() {
            tokens.push(MsgToken::String(tpl_element.cooked.as_ref().unwrap_or(&tpl_element.raw).to_string()));

            if let Some(exp) = tpl.exprs.get(i) {
                if let Expr::Call(call) = exp.as_ref() {
                    if let Some(call_tokens) = self.try_tokenize_call_expr_as_choice_cmp(call) {
                        tokens.extend(call_tokens);
                        continue;
                    }
                    if let Some(placeholder) = self.try_tokenize_call_expr_as_placeholder_call(call) {
                        tokens.push(placeholder);
                        continue;
                    }
                }

                tokens.push(MsgToken::Expression(exp.clone()));
            }
        }

        tokens
    }

    /// Try to tokenize call expression as ICU Choice macro
    /// Return None if this call is not related to macros or is not parsable
    pub fn try_tokenize_call_expr_as_choice_cmp(&self, expr: &CallExpr) -> Option<Vec<MsgToken>> {
        if let Some(ident) = match_callee_name(&expr, |name| self.is_lingui_fn_choice_cmp(name)) {
            if expr.args.len() != 2 {
                // malformed plural call, exit
                return None;
            }

            // ICU value
            let arg = expr.args.get(0).unwrap();
            let icu_value = arg.expr.clone();

            // ICU Choice Cases
            let arg = expr.args.get(1).unwrap();
            if let Expr::Object(object) = &arg.expr.as_ref() {
                let format = self.get_ident_export_name(ident).unwrap().to_lowercase();
                let cases = self.get_choice_cases_from_obj(&object.props, &format);

                return Some(vec![MsgToken::IcuChoice(IcuChoice {
                    format: format.into(),
                    value: icu_value,
                    cases,
                })]);
            } else {
                // todo passed not an ObjectLiteral,
                //      we should panic here or just skip this call
            }
        }

        return None;
    }

    pub fn try_tokenize_call_expr_as_placeholder_call(&self, expr: &CallExpr) -> Option<MsgToken> {
        if expr.callee.as_expr().is_some_and(|c| c.as_ident().map_or(false, |i| self.is_lingui_placeholder_expr(i))) {
            if let Some(first) = expr.args.first() {
                return Some(MsgToken::PlaceholderCall(first.expr.clone()));
            }
        }

        return None;
    }

    pub fn try_tokenize_expr(&self, expr: &Box<Expr>) -> Option<Vec<MsgToken>> {
        match expr.as_ref() {
            // String Literal: "has # friend"
            Expr::Lit(Lit::Str(str)) => Some(vec![MsgToken::String(str.clone().value.to_string())]),
            // Template Literal: `${name} has # friend`
            Expr::Tpl(tpl) => Some(self.tokenize_tpl(tpl)),

            // ParenthesisExpression: ("has # friend")
            Expr::Paren(ParenExpr { expr, .. }) => self.try_tokenize_expr(expr),

            // Call Expression: {one: plural(numArticles, {...})}
            Expr::Call(expr) => self.try_tokenize_call_expr_as_choice_cmp(expr),
            _ => None,
        }
    }

    /// Take KeyValueProp and return Key as string if parsable
    /// If key is numeric, return an exact match syntax `={number}`
    pub fn get_js_choice_case_key(&self, prop: &KeyValueProp) -> Option<JsWord> {
        match &prop.key {
            // {one: ""}
            PropName::Ident(IdentName { sym, .. })
            // {"one": ""}
            | PropName::Str(Str { value: sym, .. }) => {
                Some(sym.clone())
            }
            // {0: ""} -> `={number}`
            PropName::Num(Number { value, .. }) => {
                Some(format!("={value}").into())
            }
            _ => {
                None
            }
        }
    }

    /// receive ObjectLiteral {few: "..", many: "..", other: ".."} and create tokens
    /// If messages passed as TemplateLiterals with variables, it extracts variables
    pub fn get_choice_cases_from_obj(
        &self,
        props: &Vec<PropOrSpread>,
        icu_format: &str,
    ) -> Vec<CaseOrOffset> {
        // todo: there might be more props then real choices. Id for example
        let mut choices: Vec<CaseOrOffset> = Vec::with_capacity(props.len());

        for prop_or_spread in props {
            if let PropOrSpread::Prop(prop) = prop_or_spread {
                if let Prop::KeyValue(prop) = prop.as_ref() {
                    if let Some(key) = self.get_js_choice_case_key(prop) {
                        if &key == "offset" && icu_format != "select" {
                            if let Expr::Lit(Lit::Num(Number { value, .. })) = prop.value.as_ref() {
                                choices.push(CaseOrOffset::Offset(value.to_string()))
                            } else {
                                // todo: panic offset might be only a number, other forms is not supported
                            }
                        } else {
                            let tokens = self
                                .try_tokenize_expr(&prop.value)
                                .unwrap_or(vec![MsgToken::Expression(prop.value.clone())]);

                            choices.push(CaseOrOffset::Case(ChoiceCase { tokens, key }));
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
}

