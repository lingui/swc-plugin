use crate::ast_utils::*;
use crate::builder::MessageBuilder;
use crate::generate_id::generate_message_id;
use crate::macro_utils::*;
use crate::tokens::MsgToken;
use swc_core::common::comments::Comments;
use swc_core::common::{Span, Spanned, SyntaxContext};
use swc_core::{
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::ExprFactory,
        visit::{Fold, FoldWith},
    },
};

pub struct JsMacroFolder<'a, C>
where
    C: Comments,
{
    pub ctx: &'a mut MacroCtx,
    pub comments: &'a Option<C>,
}

impl<'a, C> JsMacroFolder<'a, C>
where
    C: Comments,
{
    pub fn new(ctx: &'a mut MacroCtx, comments: &'a Option<C>) -> JsMacroFolder<'a, C> {
        JsMacroFolder { ctx, comments }
    }

    fn create_message_descriptor_from_tokens(&mut self, tokens: Vec<MsgToken>, span: Span) -> Expr {
        let parsed = MessageBuilder::parse(tokens);

        let mut props: Vec<PropOrSpread> = vec![create_key_value_prop(
            "id",
            generate_message_id(&parsed.message_str, "").into(),
        )];

        if !self.ctx.options.strip_non_essential_fields {
            props.push(create_key_value_prop("message", parsed.message));
        }

        if let Some(v) = parsed.values {
            props.push(create_key_value_prop("values", v))
        }

        let message_descriptor = Expr::Object(ObjectLit { span, props });

        add_i18n_comment(self.comments, span);

        message_descriptor
    }

    fn create_i18n_fn_call_from_tokens(
        &mut self,
        callee_obj: Option<Box<Expr>>,
        tokens: Vec<MsgToken>,
        msg_dscrptr_span: Span,
        call_expr_span: Span,
    ) -> CallExpr {
        let message_descriptor =
            Box::new(self.create_message_descriptor_from_tokens(tokens, msg_dscrptr_span));

        self.create_i18n_fn_call(
            callee_obj,
            vec![message_descriptor.as_arg()],
            call_expr_span,
        )
    }

    fn create_i18n_fn_call(
        &mut self,
        callee_obj: Option<Box<Expr>>,
        args: Vec<ExprOrSpread>,
        span: Span,
    ) -> CallExpr {
        CallExpr {
            span,
            callee: Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: callee_obj.unwrap_or_else(|| {
                    self.ctx.should_add_18n_import = true;

                    Box::new(self.ctx.runtime_idents.i18n.clone().into())
                }),
                prop: MemberProp::Ident(IdentName::new("_".into(), DUMMY_SP)),
            })
            .as_callee(),
            args,
            type_args: None,
            ctxt: SyntaxContext::empty(),
        }
    }

    // take {message: "", id: "", ...} object literal, process message and return updated props
    fn update_msg_descriptor_props(&self, expr: Box<Expr>, span: Span) -> Box<Expr> {
        if let Expr::Object(obj) = *expr {
            let id_prop = get_object_prop(&obj.props, "id");

            let context_val = get_object_prop(&obj.props, "context")
                .and_then(|prop| get_expr_as_string(&prop.value));

            let message_prop = get_object_prop(&obj.props, "message");

            let mut new_props: Vec<PropOrSpread> = vec![];

            if let Some(prop) = id_prop {
                if let Some(value) = get_expr_as_string(&prop.value) {
                    new_props.push(create_key_value_prop("id", value.into()));
                }
            }

            if let Some(prop) = message_prop {
                let tokens = self.ctx.try_tokenize_expr(&prop.value).unwrap_or_default();

                let parsed = MessageBuilder::parse(tokens);

                if id_prop.is_none() {
                    new_props.push(create_key_value_prop(
                        "id",
                        generate_message_id(
                            &parsed.message_str,
                            context_val.as_deref().unwrap_or_default(),
                        )
                        .into(),
                    ))
                }

                if !self.ctx.options.strip_non_essential_fields {
                    new_props.push(create_key_value_prop("message", parsed.message));
                }

                if let Some(v) = parsed.values {
                    new_props.push(create_key_value_prop("values", v))
                }
            }

            if !self.ctx.options.strip_non_essential_fields {
                if let Some(context) = context_val {
                    new_props.push(create_key_value_prop("context", context.into()));
                }

                let comment = get_object_prop(&obj.props, "comment")
                    .and_then(|prop| get_expr_as_string(&prop.value));

                if let Some(comment) = comment {
                    new_props.push(create_key_value_prop("comment", comment.into()));
                }
            }

            let message_descriptor = Box::new(Expr::Object(ObjectLit {
                span,
                props: new_props,
            }));

            add_i18n_comment(self.comments, span);

            return message_descriptor;
        }

        expr
    }
}

impl<C> Fold for JsMacroFolder<'_, C>
where
    C: Comments,
{
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        // t`Message`
        if let Expr::TaggedTpl(tagged_tpl) = &expr {
            let (is_t, callee) = self.ctx.is_lingui_t_call_expr(&tagged_tpl.tag);

            if is_t {
                return Expr::Call(self.create_i18n_fn_call_from_tokens(
                    callee,
                    self.ctx.tokenize_tpl(&tagged_tpl.tpl),
                    tagged_tpl.tpl.span(),
                    expr.span(),
                ));
            }
        }

        // defineMessage`Message`
        if let Expr::TaggedTpl(tagged_tpl) = &expr {
            let span = tagged_tpl.span();
            if let Expr::Ident(ident) = tagged_tpl.tag.as_ref() {
                if self.ctx.is_define_message_ident(ident) {
                    let tokens = self.ctx.tokenize_tpl(&tagged_tpl.tpl);
                    return self.create_message_descriptor_from_tokens(tokens, span);
                }
            }
        }

        // defineMessage({message: "Message"})
        if let Expr::Call(call) = &expr {
            if match_callee_name(call, |n| self.ctx.is_define_message_ident(n)).is_some()
                && call.args.len() == 1
            {
                let descriptor = self.update_msg_descriptor_props(
                    call.args.clone().into_iter().next().unwrap().expr,
                    call.span(),
                );

                return *descriptor;
            }
        }

        expr.fold_children_with(self)
    }

    fn fold_call_expr(&mut self, expr: CallExpr) -> CallExpr {
        // t({}) / t(i18n)({})
        if let Callee::Expr(callee) = &expr.callee {
            let (is_t, callee) = self.ctx.is_lingui_t_call_expr(callee);

            let span = expr.span();
            if is_t && expr.args.len() == 1 {
                let msg_dscrpt_expr = expr.args.into_iter().next().unwrap().expr;

                let msg_dscrpt_expr_span = msg_dscrpt_expr.span();
                let descriptor =
                    self.update_msg_descriptor_props(msg_dscrpt_expr, msg_dscrpt_expr_span);

                return self.create_i18n_fn_call(callee, vec![descriptor.as_arg()], span);
            }
        }

        // plural / selectOrdinal / select
        if let Some(tokens) = self.ctx.try_tokenize_call_expr_as_choice_cmp(&expr) {
            let msg_dscrptr_span = expr.args.first().map(|arg| arg.span()).unwrap_or(DUMMY_SP);

            return self.create_i18n_fn_call_from_tokens(
                None,
                tokens,
                msg_dscrptr_span,
                expr.span(),
            );
        }

        expr.fold_children_with(self)
    }
}
