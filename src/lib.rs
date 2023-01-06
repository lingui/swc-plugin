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
mod normalize_witespaces_jsx;
mod normalize_witespaces_js;
mod builder;
mod tokens;
mod ast_utils;
mod macro_utils;
mod jsx_visitor;

use builder::*;
use ast_utils::*;
use jsx_visitor::TransJSXVisitor;
use crate::macro_utils::{*};
use crate::tokens::{MsgToken};

#[derive(Default)]
pub struct TransformVisitor {
    has_lingui_macro_imports: bool,
    should_add_18n_import: bool,
    should_add_trans_import: bool,
}

impl TransformVisitor {
    fn create_i18n_fn_call_from_tokens(&mut self, callee_obj: Option<Box<Expr>>, tokens: Vec<MsgToken>) -> CallExpr {
        let parsed = MessageBuilder::parse(tokens, false);

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


    // <Trans>Message</Trans>
    // <Plural />
    fn transform_jsx_macro(&mut self, el: JSXElement, is_trans_el: bool) -> JSXElement {
        let mut trans_visitor = TransJSXVisitor::new();

        if is_trans_el {
            el.children.visit_children_with(&mut trans_visitor);
        } else {
            el.visit_children_with(&mut trans_visitor);
        }

        let parsed = MessageBuilder::parse(trans_visitor.tokens, true);
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

    // take {message: "", id: "", ...} object literal, process message and return updated props
    fn update_msg_descriptor_props(&self, expr: Box<Expr>) -> Box<Expr> {
        if let Expr::Object(obj) = *expr {
            let has_id = has_object_prop(&obj.props, "id");

            let new_props: Vec<PropOrSpread> = obj.props.into_iter().map(|prop_or_spread| {
                if let PropOrSpread::Prop(prop1) = &prop_or_spread {
                    if let Prop::KeyValue(prop) = prop1.as_ref() {
                        if match_prop_key(prop, "message") {
                            let tokens = try_tokenize_expr(&prop.value).unwrap();

                            let parsed = MessageBuilder::parse(tokens, false);

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
            let (is_t, callee) = is_lingui_t_call_expr(&tagged_tpl.tag);

            if is_t {
                return Expr::Call(self.create_i18n_fn_call_from_tokens(
                    callee,
                    tokenize_tpl(&tagged_tpl.tpl),
                ));
            }
        }

        if let Expr::Call(call) = &expr {
            if let Some(_) = match_callee_name(&call, |n| n == "defineMessage") {
                if call.args.len() == 1 {
                    let descriptor = self.update_msg_descriptor_props(
                        call.args.clone().into_iter().next().unwrap().expr
                    );

                    return *descriptor;
                }
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
            let (is_t, callee) = is_lingui_t_call_expr(callee);

            if is_t && expr.args.len() == 1 {
                let descriptor = self.update_msg_descriptor_props(
                    expr.args.into_iter().next().unwrap().expr
                );

                return self.create_i18n_fn_call(callee, vec![descriptor.as_arg()]);
            }
        }

        // plural / selectOrdinal / select
        if let Some(tokens) = try_tokenize_call_expr_as_icu(&expr) {
            return self.create_i18n_fn_call_from_tokens(
                None,
                tokens,
            );
        }

        expr
    }

    fn fold_jsx_element(&mut self, mut el: JSXElement) -> JSXElement {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return el;
        }

        // apply this visitor transformations to inner
        // jsx element before they will be extracted as message components
        el = el.fold_children_with(self);

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

