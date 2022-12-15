use std::any::Any;
use swc_core::common::Spanned;
use swc_core::ecma::{
    // ast::Program,
    ast::*,
    transforms::testing::test,
    visit::{as_folder, VisitMut, VisitMutWith},
};
use std::path::{Path, PathBuf};
use swc_core::{
    common::{FileName, DUMMY_SP, util::take::Take},
    ecma::{
        ast::*,
        atoms::JsWord,
        utils::{quote_ident, ExprFactory},
        visit::{Fold, FoldWith},
    },
    plugin::{
        metadata::TransformPluginMetadataContextKind, plugin_transform,
        proxies::TransformPluginProgramMetadata,
    },
};
use swc_core::ecma::utils::ExprExt;
// use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};


pub struct TransformVisitor;

impl TransformVisitor {
    fn create_args_from_tagged_tpl(&self, tagged_tpl: &mut Tpl) -> Vec<ExprOrSpread> {
        let mut args: Vec<ExprOrSpread> = Vec::with_capacity(2);

        let mut message = String::new();
        let mut values: Vec<(JsWord, Option<&Box<Expr>>)> = Vec::with_capacity(tagged_tpl.exprs.len());

        for (i, tplElement) in tagged_tpl.quasis.iter().enumerate() {
            message.push_str(&tplElement.raw);

            if let Some(e) = tagged_tpl.exprs.get(i) {
                match e.as_ref() {
                    // `text {foo} bar`
                    Expr::Ident(ident) => {
                        values.push((ident.sym.clone(), None));
                        message.push_str(&format!("{{{}}}", &ident.sym));
                    }
                    // everything else, e.q.
                    // `text {executeFn()} bar`
                    // `text {bar.baz} bar`
                    _ => {
                        // would be a positional argument
                        let index_str = &i.to_string()[..];
                        let test: JsWord = index_str.into();

                        values.push((test, Some(e)));
                        message.push_str(&format!("{{{}}}", i))
                    }
                }
            }
        }

        let mut props = vec![];

        for (label, expr) in values {
            props.push(PropOrSpread::Prop(Box::new(
                if let Some(e) = expr {
                    Prop::KeyValue(KeyValueProp {
                        key: PropName::Ident(
                            Ident::new(label, DUMMY_SP)
                        ),
                        value: e.clone(),
                    })
                } else {
                    Prop::Shorthand(
                        Ident::new(label, DUMMY_SP)
                    )
                }
            )))
        }

        vec![
            message.as_arg(),
            Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props,
            }).as_arg(),
        ]
    }
    fn create_i18n_fn_call(&self, callee_obj: &Box<Expr>, args: Vec<ExprOrSpread>) -> Expr {
        return Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: callee_obj.clone(),
                prop: MemberProp::Ident(Ident::new("_".into(), DUMMY_SP)),
            }).as_callee(),
            args,
            type_args: None,
        });
    }
}

impl Fold for TransformVisitor {
    fn fold_expr(&mut self, mut expr: Expr) -> Expr {
        if let Expr::TaggedTpl(tagged_tpl) = &mut expr {
            match tagged_tpl.tag.as_mut() {
                // t(i18n)``
                Expr::Call(call) => {
                    if let Some(v) = call.args.get(0) {
                        return self.create_i18n_fn_call(
                            &v.expr,
                            self.create_args_from_tagged_tpl(&mut tagged_tpl.tpl),
                        );
                    }
                }
                // t``
                Expr::Ident(i) => {
                    return self.create_i18n_fn_call(
                        &Box::new(Ident::new("i18n".into(), DUMMY_SP).into()),
                        self.create_args_from_tagged_tpl(&mut tagged_tpl.tpl),
                    );
                }
                _ => {}
            }
        }

        expr.fold_children_with(self)
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
    program.fold_with(&mut TransformVisitor)
}

// An example to test plugin transform.
// Recommended strategy to test plugin's transform is verify
// the Visitor's behavior, instead of trying to run `process_transform` with mocks
// unless explicitly required to do so.

test!(
    Default::default(),
    |_| TransformVisitor,
    substitution_in_tpl_literal1,
    // input
     r#"
     t`Refresh inbox`
     t`Refresh ${foo} inbox ${bar}`
     t`Refresh ${foo.bar} inbox ${bar}`
     t`Refresh ${expr()}`
     "#,
    // output after transform
    r#"
    i18n._("Refresh inbox", {})
    i18n._("Refresh {foo} inbox {bar}", {foo, bar})
    i18n._("Refresh {0} inbox {bar}", {0: foo.bar, bar})
    i18n._("Refresh {0}", {0: expr()})
    "#
);

test!(
    Default::default(),
    |_| TransformVisitor,
    custom_i18n_passed,
    // input
     r#"
     t(custom_i18n)`Refresh inbox`
     t(custom_i18n)`Refresh ${foo} inbox ${bar}`
     t(custom_i18n)`Refresh ${foo.bar} inbox ${bar}`
     t(custom_i18n)`Refresh ${expr()}`
     "#,
    // output after transform
    r#"
    custom_i18n._("Refresh inbox", {})
    custom_i18n._("Refresh {foo} inbox {bar}", {foo, bar})
    custom_i18n._("Refresh {0} inbox {bar}", {0: foo.bar, bar})
    custom_i18n._("Refresh {0}", {0: expr()})
    "#
);