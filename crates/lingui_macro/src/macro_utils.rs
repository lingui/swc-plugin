use crate::ast_utils::*;
use crate::comment_directive::{DirectiveValues, LinguiCommentDirectives};
use crate::tokens::*;
use crate::LinguiOptions;
use std::collections::{HashMap, HashSet};
use swc_core::common::{BytePos, Spanned};
use swc_core::ecma::utils::quote_ident;
use swc_core::ecma::{ast::*, atoms::Atom};
use swc_core::plugin::errors::HANDLER;

fn expression_to_name(expr: &Expr, get_index: &mut impl FnMut() -> usize) -> String {
    let expr = unwrap_ts_as_expr(expr);

    match expr {
        Expr::Ident(ident) => ident.sym.to_string(),
        Expr::Object(object) => {
            if let Some(PropOrSpread::Prop(prop)) = object.props.first() {
                // {foo}
                if let Some(short) = prop.as_shorthand() {
                    return short.sym.to_string();
                }
                // assume this is labeled expression, {label: value}
                if let Prop::KeyValue(kv) = prop.as_ref() {
                    if let PropName::Ident(ident) = &kv.key {
                        return ident.sym.to_string();
                    }
                }
            }
            get_index().to_string()
        }
        _ => get_index().to_string(),
    }
}

fn expression_to_value(expr: Box<Expr>) -> Box<Expr> {
    let unwrapped = unwrap_ts_as_expr(&expr);

    match unwrapped {
        Expr::Object(object) => {
            if object.props.len() > 1 {
                HANDLER.with(|h| {
                    h.struct_span_err(
                        object.span,
                        "Incorrect usage of a labeled expression. Expected exactly one property as `{variableName: variableValue}`.",
                    )
                    .emit();
                });
            }

            if let Some(PropOrSpread::Prop(prop)) = object.props.first() {
                if let Some(short) = prop.as_shorthand() {
                    return Box::new(Expr::Ident(Ident {
                        span: swc_core::common::DUMMY_SP,
                        sym: short.sym.clone(),
                        ctxt: swc_core::common::SyntaxContext::empty(),
                        optional: false,
                    }));
                }
                if let Prop::KeyValue(kv) = prop.as_ref() {
                    return kv.value.clone();
                }
            }
            expr
        }
        _ => expr,
    }
}

// recursively expands TypeScript's as expressions until it reaches a real value
fn unwrap_ts_as_expr(expr: &Expr) -> &Expr {
    let mut current = expr;
    while let Expr::TsAs(TsAsExpr {
        expr: inner_expr, ..
    }) = current
    {
        current = inner_expr;
    }
    current
}

fn tokenize_expression(expr: Box<Expr>, get_index: &mut impl FnMut() -> usize) -> MsgArg {
    let name = expression_to_name(&expr, get_index);
    let value = expression_to_value(expr);
    MsgArg {
        name,
        value,
        format: None,
        cases: None,
    }
}

/// Take KeyValueProp and return Key as string if parsable
/// If key is numeric, return an exact match syntax `={number}`
fn get_js_choice_case_key(prop: &KeyValueProp) -> Option<Atom> {
    match &prop.key {
        // {one: ""}
        PropName::Ident(IdentName { sym, .. }) => Some(sym.clone()),
        // {"one": ""}
        PropName::Str(Str { value, .. }) => Some(value.to_string_lossy().into_owned().into()),
        // {0: ""} -> `={number}`
        PropName::Num(Number { value, .. }) => Some(format!("={value}").into()),
        _ => None,
    }
}

fn unwrap_ph_call(ctx: &MacroCtx, expr: Box<Expr>) -> Box<Expr> {
    if let Expr::Call(call) = expr.as_ref() {
        if call.callee.as_expr().is_some_and(|c| {
            c.as_ident()
                .is_some_and(|i| ctx.transform.is_lingui_placeholder_expr(i))
        }) {
            if let Some(first) = call.args.first() {
                if !first.expr.is_object() {
                    HANDLER.with(|h| {
                        h.struct_span_err(
                            first.expr.span(),
                            "Incorrect usage of `ph` macro. First argument should be an object expression like `ph({name: value})`.",
                        )
                        .emit();
                    });
                    return expr;
                }
                return first.expr.clone();
            }
        }
    }
    expr
}

pub fn tokenize_expr_to_arg(ctx: &mut MacroCtx, expr: Box<Expr>) -> MsgArg {
    let expr = unwrap_ph_call(ctx, expr);
    let idx = &mut ctx.expression_index;
    let mut get_index = || {
        let i = *idx;
        *idx += 1;
        i
    };
    tokenize_expression(expr, &mut get_index)
}

/// Receive TemplateLiteral with variables and return MsgTokens
pub fn tokenize_tpl(ctx: &mut MacroCtx, tpl: &Tpl) -> Vec<MsgToken> {
    let mut tokens: Vec<MsgToken> = Vec::with_capacity(tpl.quasis.len());

    for (i, tpl_element) in tpl.quasis.iter().enumerate() {
        let value = tpl_element
            .cooked
            .as_ref()
            .map(|c| c.to_string_lossy().into_owned())
            .unwrap_or_else(|| tpl_element.raw.to_string());
        tokens.push(MsgToken::String(value));

        if let Some(exp) = tpl.exprs.get(i) {
            if let Expr::Call(call) = exp.as_ref() {
                if let Some(call_tokens) = try_tokenize_call_expr_as_choice_cmp(ctx, call) {
                    tokens.extend(call_tokens);
                    continue;
                }
            }

            let arg = tokenize_expr_to_arg(ctx, exp.clone());
            tokens.push(MsgToken::Arg(arg));
        }
    }

    tokens
}

/// Try to tokenize call expression as ICU Choice macro
/// Return None if this call is not related to macros or is not parsable
pub fn try_tokenize_call_expr_as_choice_cmp(
    ctx: &mut MacroCtx,
    expr: &CallExpr,
) -> Option<Vec<MsgToken>> {
    if let Some(ident) = match_callee_name(expr, |name| ctx.transform.is_lingui_fn_choice_cmp(name))
    {
        if expr.args.len() != 2 {
            // malformed plural call, exit
            return None;
        }

        // ICU value
        let arg = expr.args.first().unwrap();
        let icu_value = arg.expr.clone();

        // ICU Choice Cases
        let arg = expr.args.get(1).unwrap();
        if let Expr::Object(object) = &arg.expr.as_ref() {
            let format = ctx
                .transform
                .get_ident_export_name(ident)
                .unwrap()
                .to_lowercase();
            let mut token_arg = tokenize_expr_to_arg(ctx, icu_value);
            let cases = get_choice_cases_from_obj(ctx, &object.props, &format);
            token_arg.format = Some(format.into());
            token_arg.cases = Some(cases);

            return Some(vec![MsgToken::Arg(token_arg)]);
        } else {
            // todo passed not an ObjectLiteral,
            //      we should panic here or just skip this call
        }
    }

    None
}

pub fn try_tokenize_expr(ctx: &mut MacroCtx, expr: &Expr) -> Option<Vec<MsgToken>> {
    match expr {
        // String Literal: "has # friend"
        Expr::Lit(Lit::Str(str)) => Some(vec![MsgToken::String(
            str.value.to_string_lossy().into_owned(),
        )]),
        // Template Literal: `${name} has # friend`
        Expr::Tpl(tpl) => Some(tokenize_tpl(ctx, tpl)),

        // ParenthesisExpression: ("has # friend")
        Expr::Paren(ParenExpr { expr, .. }) => try_tokenize_expr(ctx, expr),

        // Call Expression: {one: plural(numArticles, {...})}
        Expr::Call(expr) => try_tokenize_call_expr_as_choice_cmp(ctx, expr),
        _ => None,
    }
}

/// receive ObjectLiteral {few: "..", many: "..", other: ".."} and create tokens
/// If messages passed as TemplateLiterals with variables, it extracts variables
pub fn get_choice_cases_from_obj(
    ctx: &mut MacroCtx,
    props: &Vec<PropOrSpread>,
    icu_format: &str,
) -> Vec<CaseOrOffset> {
    // todo: there might be more props then real choices. Id for example
    let mut choices: Vec<CaseOrOffset> = Vec::with_capacity(props.len());

    for prop_or_spread in props {
        if let PropOrSpread::Prop(prop) = prop_or_spread {
            if let Prop::KeyValue(prop) = prop.as_ref() {
                if let Some(key) = get_js_choice_case_key(prop) {
                    if &key == "offset" && icu_format != "select" {
                        if let Expr::Lit(Lit::Num(Number { value, .. })) = prop.value.as_ref() {
                            choices.push(CaseOrOffset::Offset(value.to_string()))
                        } else {
                            // todo: panic offset might be only a number, other forms is not supported
                        }
                    } else {
                        let tokens = try_tokenize_expr(ctx, &prop.value).unwrap_or_else(|| {
                            let arg = tokenize_expr_to_arg(ctx, prop.value.clone());
                            vec![MsgToken::Arg(arg)]
                        });

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

const LINGUI_T: &str = "t";

pub fn build_prefixed_id(
    options: &LinguiOptions,
    id: &str,
    defaults: Option<&DirectiveValues>,
) -> Option<String> {
    let id_prefix = defaults.and_then(|defaults| defaults.id_prefix.as_deref())?;

    if let Some(leader) = options.id_prefix_leader.as_deref() {
        if !id.starts_with(leader) {
            return None;
        }
    }

    Some(format!("{id_prefix}{id}"))
}

/// Global context for the entire plugin transform run on a file.
/// Tracks macro imports, options, directives, and runtime identifiers.
#[derive(Default, Clone)]
pub struct TransformCtx {
    // export name -> local name
    symbol_to_id_map: HashMap<Atom, HashSet<Id>>,
    // local name -> export name
    id_to_symbol_map: HashMap<Id, Atom>,

    pub should_add_18n_import: bool,
    pub should_add_trans_import: bool,
    pub should_add_uselingui_import: bool,

    pub options: LinguiOptions,
    pub directives: LinguiCommentDirectives,
    pub runtime_idents: RuntimeIdents,
}

#[derive(Clone)]
pub struct RuntimeIdents {
    pub i18n: Ident,
    pub trans: IdentName,
    pub use_lingui: IdentName,
}

impl Default for RuntimeIdents {
    fn default() -> RuntimeIdents {
        RuntimeIdents {
            i18n: quote_ident!("$_i18n").into(),
            trans: quote_ident!("Trans_"),
            use_lingui: quote_ident!("$_useLingui"),
        }
    }
}

impl TransformCtx {
    pub fn new(options: LinguiOptions) -> TransformCtx {
        TransformCtx {
            options,
            ..Default::default()
        }
    }

    pub fn set_directives(&mut self, directives: LinguiCommentDirectives) {
        self.directives = directives;
    }

    pub fn get_comment_directive(&self, pos: BytePos) -> Option<&DirectiveValues> {
        self.directives.find_for_pos(pos)
    }

    /// is given ident exported from @lingui/macro? and one of choice functions?
    pub fn is_lingui_fn_choice_cmp(&self, ident: &Ident) -> bool {
        self.is_lingui_ident("plural", ident)
            || self.is_lingui_ident("select", ident)
            || self.is_lingui_ident("selectOrdinal", ident)
    }

    pub fn is_lingui_placeholder_expr(&self, ident: &Ident) -> bool {
        self.is_lingui_ident("ph", ident)
    }

    /// is given ident exported from @lingui/macro?
    pub fn is_lingui_ident(&self, name: &str, ident: &Ident) -> bool {
        let name_atom: Atom = name.into();
        self.symbol_to_id_map
            .get(&name_atom)
            .and_then(|refs| refs.get(&ident.to_id()))
            .is_some()
    }

    pub fn is_define_message_ident(&self, ident: &Ident) -> bool {
        self.is_lingui_ident("defineMessage", ident) || self.is_lingui_ident("msg", ident)
    }

    /// given import {plural as i18nPlural} from "@lingui/macro";
    /// get_ident_export_name("i18nPlural") would return `plural`
    pub fn get_ident_export_name(&self, ident: &Ident) -> Option<&Atom> {
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

    pub fn register_reference(&mut self, symbol: &Atom, id: &Id) {
        self.symbol_to_id_map
            .entry(symbol.clone())
            .or_default()
            .insert(id.clone());

        self.id_to_symbol_map.insert(id.clone(), symbol.clone());
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
    pub fn is_lingui_t_call_expr(&self, callee_expr: &Expr) -> (bool, Option<Box<Expr>>) {
        match callee_expr {
            // t(i18n)...
            Expr::Call(call)
                if match_callee_name(call, |n| self.is_lingui_ident(LINGUI_T, n)).is_some() =>
            {
                if let Some(v) = call.args.first() {
                    (true, Some(v.expr.clone()))
                } else {
                    (false, None)
                }
            }
            // t..
            Expr::Ident(ident) if self.is_lingui_ident(LINGUI_T, ident) => (true, None),
            _ => (false, None),
        }
    }
}

/// Local context for a single macro invocation.
/// Owns a fresh expression index counter and borrows the global TransformCtx.
pub struct MacroCtx<'a> {
    pub transform: &'a mut TransformCtx,
    expression_index: usize,
}

impl<'a> MacroCtx<'a> {
    pub fn new(transform: &'a mut TransformCtx) -> MacroCtx<'a> {
        MacroCtx {
            transform,
            expression_index: 0,
        }
    }

    /// Re-borrow this context with a shorter lifetime, sharing the same
    /// counter and TransformCtx. Use when passing to a child visitor
    /// that continues the same macro invocation.
    pub fn reborrow(&mut self) -> MacroCtx<'_> {
        MacroCtx {
            transform: self.transform,
            expression_index: self.expression_index,
        }
    }
}
