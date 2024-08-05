use swc_core::{
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::ExprFactory,
        visit::{Fold, FoldWith},
    },
};
use crate::ast_utils::{*};
use crate::builder::MessageBuilder;
use crate::macro_utils::{*};
use crate::tokens::MsgToken;
use crate::generate_id::generate_message_id;

pub struct JsMacroFolder<'a> {
    pub ctx: &'a mut MacroCtx,
}

impl<'a> JsMacroFolder<'a> {
    pub fn new(ctx: &'a mut MacroCtx) -> JsMacroFolder<'a> {
        JsMacroFolder {
            ctx,
        }
    }

    fn create_message_descriptor_from_tokens(&mut self, tokens: Vec<MsgToken>) -> Expr {
      let parsed = MessageBuilder::parse(tokens);

      let mut props: Vec<PropOrSpread> = vec![
        create_key_value_prop("id", generate_message_id(&parsed.message_str, "").into()),

      ];

      if !self.ctx.options.strip_non_essential_fields {
        props.push(
          create_key_value_prop("message", parsed.message),
        );
      }

      if let Some(v) = parsed.values {
        props.push(
          create_key_value_prop("values", v),
        )
      }

      let message_descriptor = Expr::Object(ObjectLit {
        span: DUMMY_SP,
        props,
      });

      return message_descriptor;
    }

    fn create_i18n_fn_call_from_tokens(&mut self, callee_obj: Option<Box<Expr>>, tokens: Vec<MsgToken>) -> CallExpr {
      let message_descriptor = Box::new(self.create_message_descriptor_from_tokens(tokens));
      return self.create_i18n_fn_call(callee_obj, vec![message_descriptor.as_arg()]);
    }

    fn create_i18n_fn_call(&mut self, callee_obj: Option<Box<Expr>>, args: Vec<ExprOrSpread>) -> CallExpr {
      CallExpr {
        span: DUMMY_SP,
        callee: Expr::Member(MemberExpr {
          span: DUMMY_SP,
          obj: callee_obj.unwrap_or_else(|| {
            self.ctx.should_add_18n_import = true;

            return Box::new(self.ctx.runtime_idents.i18n.clone().into());
          }),
          prop: MemberProp::Ident(Ident::new("_".into(), DUMMY_SP)),
        }).as_callee(),
        args,
        type_args: None,
      }
    }

    // take {message: "", id: "", ...} object literal, process message and return updated props
    fn update_msg_descriptor_props(&self, expr: Box<Expr>) -> Box<Expr> {
      if let Expr::Object(obj) = *expr {
        let id_prop = get_object_prop(&obj.props, "id");

        let context_val = get_object_prop(&obj.props, "context")
          .and_then(|prop| get_expr_as_string(&prop.value));

        let message_prop = get_object_prop(&obj.props, "message");

        let mut new_props: Vec<PropOrSpread> = vec![];

        if let Some(prop) = id_prop {
          if let Some(value) = get_expr_as_string(&prop.value) {
            new_props.push(create_key_value_prop(
              "id",
              value.into(),
            ));
          }
        }

        if let Some(prop) = message_prop {
          let tokens = self.ctx.try_tokenize_expr(&prop.value).unwrap_or_else(|| Vec::new());

          let parsed = MessageBuilder::parse(tokens);

          if !id_prop.is_some() {
            new_props.push(
              create_key_value_prop(
                "id",
                generate_message_id(
                  &parsed.message_str,
                  &(context_val.unwrap_or_default()),
                ).into(),
              ),
            )
          }

          if !self.ctx.options.strip_non_essential_fields {
            new_props.push(
              create_key_value_prop("message", parsed.message),
            );
          }

          if let Some(v) = parsed.values {
            new_props.push(
              create_key_value_prop("values", v),
            )
          }
        }

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
        // t`Message`
        if let Expr::TaggedTpl(tagged_tpl) = &expr {
            let (is_t, callee) = self.ctx.is_lingui_t_call_expr(&tagged_tpl.tag);

            if is_t {
                return Expr::Call(self.create_i18n_fn_call_from_tokens(
                    callee,
                    self.ctx.tokenize_tpl(&tagged_tpl.tpl),
                ));
            }
        }

        // defineMessage`Message`
        if let Expr::TaggedTpl(tagged_tpl) = &expr {
          if let Expr::Ident(ident) = tagged_tpl.tag.as_ref() {
            if self.ctx.is_define_message_ident(&ident) {
              let tokens = self.ctx.tokenize_tpl(&tagged_tpl.tpl);
              return self.create_message_descriptor_from_tokens(tokens);
            }
          }
        }

        // defineMessage({message: "Message"})
        if let Expr::Call(call) = &expr {
            if let Some(_) = match_callee_name(&call, |n| self.ctx.is_define_message_ident(n)) {
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
            let (is_t, callee) = self.ctx.is_lingui_t_call_expr(callee);

            if is_t && expr.args.len() == 1 {
                let descriptor = self.update_msg_descriptor_props(
                    expr.args.into_iter().next().unwrap().expr
                );

                return self.create_i18n_fn_call(callee, vec![descriptor.as_arg()]);
            }
        }

        // plural / selectOrdinal / select
        if let Some(tokens) = self.ctx.try_tokenize_call_expr_as_choice_cmp( &expr) {
            return self.create_i18n_fn_call_from_tokens(
                None,
                tokens,
            );
        }

        expr
    }
}
