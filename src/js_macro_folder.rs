use swc_core::{
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::ExprFactory,
        visit::{Fold, FoldWith},
    },
};
use swc_core::ecma::atoms::JsWord;
use crate::ast_utils::{*};
use crate::builder::MessageBuilder;
use crate::macro_utils::{*};
use crate::tokens::MsgToken;

#[derive(Debug)]
pub struct JsMacroFolder<'a> {
    pub should_add_18n_import: &'a mut bool,
    pub i18_callee_name: JsWord,
}

impl<'a> JsMacroFolder<'a> {
    fn create_i18n_fn_call_from_tokens(&mut self, callee_obj: Option<Box<Expr>>, tokens: Vec<MsgToken>) -> CallExpr {
        let parsed = MessageBuilder::parse(tokens, false);

        let mut args: Vec<ExprOrSpread> = vec![parsed.message.as_arg()];

        if let Some(v) = parsed.values {
            args.push(v.as_arg())
        }

        return self.create_i18n_fn_call(callee_obj, args);
    }

    fn create_i18n_fn_call(&mut self, callee_obj: Option<Box<Expr>>, args: Vec<ExprOrSpread>) -> CallExpr {
        let t = CallExpr {
            span: DUMMY_SP,
            callee: Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: callee_obj.unwrap_or_else(|| {
                    (*self.should_add_18n_import) = true;

                    return Box::new(Ident::new(self.i18_callee_name.clone().into(), DUMMY_SP).into());
                }),
                prop: MemberProp::Ident(Ident::new("_".into(), DUMMY_SP)),
            }).as_callee(),
            args,
            type_args: None,
        };

        t
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


impl<'a> Fold for JsMacroFolder<'a> {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
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
}
