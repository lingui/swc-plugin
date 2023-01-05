use swc_core::ecma::{
    visit::{VisitWith},
};
use std::collections::HashSet;

use swc_core::{
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::ExprFactory,
        visit::{Fold, FoldWith},
    },
    plugin::{
        plugin_transform,
        proxies::TransformPluginProgramMetadata,
    },
};
use swc_core::ecma::utils::quote_ident;

mod tests;
mod utils;
mod builder;
mod tokens;
mod ecma_utils;
mod jsx_visitor;

use builder::*;
use ecma_utils::*;
use jsx_visitor::TransJSXVisitor;
use crate::tokens::{Icu, IcuChoice, MsgToken};

const LINGUI_T: &str = &"t";

fn is_lingui_fn(name: &str) -> bool {
    // todo: i didn't find a better way to create a constant hashmap
    match name {
        "plural" | "select" | "selectOrdinal" => true,
        _ => false,
    }
}

fn is_lingui_jsx_el(name: &str) -> bool {
    // todo: i didn't find a better way to create a constant hashmap
    match name {
        "Plural" | "Select" | "SelectOrdinal" => true,
        _ => false,
    }
}

#[derive(Default)]
pub struct TransformVisitor {
    has_lingui_macro_imports: bool,
    should_add_18n_import: bool,
    should_add_trans_import: bool,
}

impl TransformVisitor {
    // Receive TemplateLiteral with variables and return MsgTokens
    fn tokenize_tpl(&self, tpl: &Tpl) -> Vec<MsgToken> {
        let mut tokens: Vec<MsgToken> = Vec::with_capacity(tpl.quasis.len());

        for (i, tpl_element) in tpl.quasis.iter().enumerate() {
            tokens.push(MsgToken::String(tpl_element.raw.to_string()));

            if let Some(exp) = tpl.exprs.get(i) {
                if let Expr::Call(call) = exp.as_ref() {
                    if let Some(call_tokens) = self.try_tokenize_call_expr_as_icu(call) {
                        tokens.extend(call_tokens);
                        continue;
                    }
                }

                tokens.push(MsgToken::Expression(exp.clone()));
            }
        }

        tokens
    }

    fn create_i18n_fn_call_from_tokens(&mut self, callee_obj: Option<Box<Expr>>, tokens: Vec<MsgToken>) -> CallExpr {
        let parsed = MessageBuilder::parse(tokens);

        let mut args: Vec<ExprOrSpread> = vec![parsed.message.as_arg()];

        if let Some(v) = parsed.values {
            args.push(v.as_arg())
        }

        return self.create_i18n_fn_call(callee_obj, args);
    }

    fn create_i18n_fn_call(&mut self, callee_obj: Option<Box<Expr>>, args: Vec<ExprOrSpread>) -> CallExpr {
        return CallExpr {
            span: DUMMY_SP,
            callee: Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: callee_obj.unwrap_or_else(|| {
                    self.should_add_18n_import = true;
                    return Box::new(Ident::new("i18n".into(), DUMMY_SP).into());
                }),
                prop: MemberProp::Ident(Ident::new("_".into(), DUMMY_SP)),
            }).as_callee(),
            args,
            type_args: None,
        };
    }

    fn try_tokenize_call_expr_as_icu(&self, expr: &CallExpr) -> Option<Vec<MsgToken>> {
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
                let choices = self.get_choices_from_obj(&object.props);

                return Some(vec![MsgToken::Icu(Icu {
                    icu_method: ident.sym.to_lowercase(),
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

    fn try_tokenize_expr(&self, expr: &Box<Expr>) -> Option<Vec<MsgToken>> {
        match expr.as_ref() {
            // String Literal: "has # friend"
            Expr::Lit(Lit::Str(str)) => {
                Some(vec!(MsgToken::String(str.clone().value.to_string())))
            }
            // Template Literal: `${name} has # friend`
            Expr::Tpl(tpl) => {
                Some(self.tokenize_tpl(tpl))
            }

            // ParenthesisExpression: ("has # friend")
            Expr::Paren(ParenExpr { expr, .. }) => {
                self.try_tokenize_expr(expr)
            }

            // Call Expression: {one: plural(numArticles, {...})}
            Expr::Call(expr) => {
                self.try_tokenize_call_expr_as_icu(expr)
            }
            _ => None
        }
    }

    fn is_lingui_t_call_expr(&self, callee_expr: &Box<Expr>) -> (bool, Option<Box<Expr>>) {
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

    // receive ObjectLiteral {few: "..", many: "..", other: ".."} and create tokens
    // If messages passed as TemplateLiterals with variables, it extracts variables
    fn get_choices_from_obj(&self, props: &Vec<PropOrSpread>) -> Vec<IcuChoice> {
        // todo: there might be more props then real choices. Id for example
        let mut choices: Vec<IcuChoice> = Vec::with_capacity(props.len());

        for prop_or_spread in props {
            if let PropOrSpread::Prop(prop) = prop_or_spread {
                if let Prop::KeyValue(prop) = prop.as_ref() {
                    match &prop.key {
                        // {one: ""}
                        // {"one": ""}
                        PropName::Ident(Ident { sym, .. })
                        | PropName::Str(Str { value: sym, .. }) => {
                            let tokens = self
                                .try_tokenize_expr(&prop.value)
                                .unwrap_or(Vec::new());

                            choices.push(IcuChoice {
                                tokens,
                                key: sym.to_string(),
                            })
                        }
                        _ => {}
                    }

                    if let PropName::Ident(ident) = &prop.key {} else {
                        // todo panic
                    }
                    // icuParts.push_str(prop.key)
                } else {
                    // todo: panic here we could not parse anything else then KeyValue pair
                }
            } else {
                // todo: panic here, we could not parse spread
            }
        }

        choices
    }

    // <Trans>Message</Trans>
    // <Plural />
    fn transform_jsx_macro(&mut self, el: JSXElement, is_trans_el: bool) -> JSXElement {
        let mut trans_visitor = TransJSXVisitor::new();

        if is_trans_el {
            el.children.visit_children_with(&mut trans_visitor);
        } else {
            el.visit_children_with(&mut trans_visitor);
        }

        let parsed = MessageBuilder::parse(trans_visitor.tokens);
        let id_attr = get_jsx_attr(&el.opening, "id");

        let mut attrs = vec![
            create_jsx_attribute(
                if let Some(_) = id_attr { "message" } else { "id" }.into(),
                parsed.message,
            ),
        ];

        if let Some(exp) = parsed.values {
            attrs.push(create_jsx_attribute(
                "values",
                exp,
            ));
        }

        if let Some(exp) = parsed.components {
            attrs.push(create_jsx_attribute(
                "components",
                exp,
            ));
        }

        attrs.extend(
            pick_jsx_attrs(el.opening.attrs, HashSet::from(["id", "render"]))
        );

        self.should_add_trans_import = true;

        return JSXElement {
            span: el.span,
            children: vec![],
            closing: None,
            opening: JSXOpeningElement {
                self_closing: true,
                span: el.opening.span,
                name: JSXElementName::Ident(
                    Ident::new("Trans".into(), el.opening.span)
                ),
                type_args: None,
                attrs,
            },
        };
    }

    // take {message: "", ...} object literal, process message and return updated props
    fn update_msg_descriptor_props(&self, expr: Box<Expr>) -> Box<Expr> {
        if let Expr::Object(obj) = *expr {
            let has_id = has_object_prop(&obj.props, "id");

            let new_props: Vec<PropOrSpread> = obj.props.into_iter().map(|prop_or_spread| {
                if let PropOrSpread::Prop(prop1) = &prop_or_spread {
                    if let Prop::KeyValue(prop) = prop1.as_ref() {
                        if match_prop_key(prop, "message") {
                            let tokens = self.try_tokenize_expr(&prop.value).unwrap();

                            let parsed = MessageBuilder::parse(tokens);

                            let mut args: Vec<PropOrSpread> = vec![
                                create_key_value_prop(if has_id { "message" } else { "id" }, parsed.message),
                            ];

                            if let Some(v) = parsed.values {
                                args.push(
                                    create_key_value_prop("values", v),
                                )
                            }

                            return args;
                        }
                    }
                }

                return vec![prop_or_spread];
            }).flatten().collect();

            return Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: new_props,
            }));
        }

        expr
    }
}


impl Fold for TransformVisitor {
    fn fold_module_items(&mut self, mut n: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let mut has_i18n_import = false;
        let mut has_trans_import = false;

        n.retain(|m| {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(imp)) = m {
                // drop macro imports
                if &imp.src.value == "@lingui/macro" {
                    self.has_lingui_macro_imports = true;
                    return false;
                }

                if &imp.src.value == "@lingui/core" && !imp.type_only {
                    for spec in &imp.specifiers {
                        if let ImportSpecifier::Named(spec) = spec {
                            has_i18n_import = if !has_i18n_import { &spec.local.sym == "i18n" } else { true };
                        }
                    }
                }

                if &imp.src.value == "@lingui/react" && !imp.type_only {
                    for spec in &imp.specifiers {
                        if let ImportSpecifier::Named(spec) = spec {
                            has_trans_import = if !has_trans_import { &spec.local.sym == "Trans" } else { true };
                        }
                    }
                }
            }

            true
        });

        n = n.fold_children_with(self);

        if !has_i18n_import && self.should_add_18n_import {
            n.insert(0, create_import("@lingui/core".into(), quote_ident!("i18n")));
        }

        if !has_trans_import && self.should_add_trans_import {
            n.insert(0, create_import("@lingui/react".into(), quote_ident!("Trans")));
        }

        n
    }

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return expr;
        }

        if let Expr::TaggedTpl(tagged_tpl) = &expr {
            let (is_t, callee) = self.is_lingui_t_call_expr(&tagged_tpl.tag);

            if is_t {
                return Expr::Call(self.create_i18n_fn_call_from_tokens(
                    callee,
                    self.tokenize_tpl(&tagged_tpl.tpl),
                ));
            }
        }

        expr.fold_children_with(self)
    }


    fn fold_call_expr(&mut self, expr: CallExpr) -> CallExpr {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return expr;
        }

        // t({}) / t(i18n)({})
        if let Callee::Expr(callee) = &expr.callee {
            let (is_t, callee) = self.is_lingui_t_call_expr(callee);

            if is_t && expr.args.len() == 1 {
                let descriptor = self.update_msg_descriptor_props(
                    expr.args.into_iter().next().unwrap().expr
                );

                return self.create_i18n_fn_call(callee, vec![descriptor.as_arg()]);
            }
        }

        // plural / selectOrdinal / select
        if let Some(tokens) = self.try_tokenize_call_expr_as_icu(&expr) {
            return self.create_i18n_fn_call_from_tokens(
                None,
                tokens,
            );
        }

        expr
    }

    fn fold_jsx_element(&mut self, el: JSXElement) -> JSXElement {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return el;
        }

        if let JSXElementName::Ident(ident) = &el.opening.name {
            if &ident.sym == "Trans" {
                return self.transform_jsx_macro(el, true);
            }

            if is_lingui_jsx_el(&ident.sym) {
                return self.transform_jsx_macro(el, false);
            }
        }

        el
    }
}


/// An example plugin function with macro support.
/// `plugin_transform` macro interop pointers into deserialized structs, as well
/// as returning ptr back to host.
///
/// It is possible to opt out from macro by writing transform fn manually
/// if plugin need to handle low-level ptr directly via
/// `__transform_plugin_process_impl(
///     ast_ptr: *const u8, ast_ptr_len: i32,
///     unresolved_mark: u32, should_enable_comments_proxy: i32) ->
///     i32 /*  0 for success, fail otherwise.
///             Note this is only for internal pointer interop result,
///             not actual transform result */`
///
/// This requires manual handling of serialization / deserialization from ptrs.
/// Refer swc_plugin_macro to see how does it work internally.
#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut TransformVisitor::default())
}

