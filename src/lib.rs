use std::collections::HashSet;
use swc_core::common::{Spanned, DUMMY_SP};
use swc_core::{
    ecma::{
        ast::*,
        utils::quote_ident,
        visit::{Fold, FoldWith, VisitWith},
    },
    plugin::{
        metadata::TransformPluginMetadataContextKind, plugin_transform,
        proxies::TransformPluginProgramMetadata,
    },
};

mod ast_utils;
mod builder;
mod generate_id;
mod js_macro_folder;
mod jsx_visitor;
mod macro_utils;
mod options;
mod tests;
mod tokens;

use crate::generate_id::*;
use crate::macro_utils::*;
use crate::options::*;
use ast_utils::*;
use builder::*;
use js_macro_folder::JsMacroFolder;
use jsx_visitor::TransJSXVisitor;

#[derive(Default)]
pub struct LinguiMacroFolder {
    has_lingui_macro_imports: bool,
    ctx: MacroCtx,
}

impl LinguiMacroFolder {
    pub fn new(options: LinguiOptions) -> LinguiMacroFolder {
        LinguiMacroFolder {
            has_lingui_macro_imports: false,
            ctx: MacroCtx::new(options),
        }
    }

    // <Trans>Message</Trans>
    // <Plural />
    fn transform_jsx_macro(&mut self, el: JSXElement, is_trans_el: bool) -> JSXElement {
        let mut trans_visitor = TransJSXVisitor::new(&self.ctx);

        if is_trans_el {
            el.children.visit_children_with(&mut trans_visitor);
        } else {
            el.visit_children_with(&mut trans_visitor);
        }

        let parsed = MessageBuilder::parse(trans_visitor.tokens);
        let id_attr = get_jsx_attr(&el.opening, "id");

        let context_attr_val = get_jsx_attr(&el.opening, "context")
            .and_then(|attr| attr.value.as_ref())
            .and_then(|value| get_jsx_attr_value_as_string(value));

        let mut attrs = vec![create_jsx_attribute("message".into(), parsed.message)];

        if !id_attr.is_some() {
            attrs.push(create_jsx_attribute(
                "id",
                generate_message_id(&parsed.message_str, &context_attr_val.unwrap_or_default())
                    .into(),
            ));
        }

        if let Some(exp) = parsed.values {
            attrs.push(create_jsx_attribute("values", exp));
        }

        if let Some(exp) = parsed.components {
            attrs.push(create_jsx_attribute("components", exp));
        }

        attrs.extend(pick_jsx_attrs(
            el.opening.attrs,
            HashSet::from(["id", "render", "i18n"]),
        ));

        if self.ctx.options.strip_non_essential_fields {
            attrs = pick_jsx_attrs(
                attrs,
                HashSet::from(["id", "render", "i18n", "values", "components"]),
            )
        }

        self.ctx.should_add_trans_import = true;

        let (_, trans_export) = self.ctx.options.runtime_modules.trans.clone();

        return JSXElement {
            span: el.span,
            children: vec![],
            closing: None,
            opening: JSXOpeningElement {
                self_closing: true,
                span: el.opening.span,
                name: JSXElementName::Ident(Ident::new(trans_export.into(), el.opening.span)),
                type_args: None,
                attrs,
            },
        };
    }
}

impl<'a> Fold for LinguiMacroFolder {
    fn fold_module_items(&mut self, mut n: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let mut has_i18n_import = false;
        let mut has_trans_import = false;

        let (i18n_source, i18n_export) = self.ctx.options.runtime_modules.i18n.clone();
        let (trans_source, trans_export) = self.ctx.options.runtime_modules.trans.clone();

        let mut insert_index: usize = 0;
        let mut index = 0;

        n.retain(|m| {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(imp)) = m {
                // drop macro imports
                if &imp.src.value == "@lingui/macro" || &imp.src.value == "@lingui/core/macro" || &imp.src.value == "@lingui/react/macro" {
                    self.has_lingui_macro_imports = true;
                    self.ctx.register_macro_import(imp);
                    insert_index = index;
                    return false;
                }

                if &imp.src.value == &i18n_source && !imp.type_only {
                    for spec in &imp.specifiers {
                        if let ImportSpecifier::Named(spec) = spec {
                            has_i18n_import = if !has_i18n_import {
                                &spec.local.sym == &i18n_export
                            } else {
                                true
                            };
                        }
                    }
                }

                if &imp.src.value == &trans_source && !imp.type_only {
                    for spec in &imp.specifiers {
                        if let ImportSpecifier::Named(spec) = spec {
                            has_trans_import = if !has_trans_import {
                                &spec.local.sym == &trans_export
                            } else {
                                true
                            };
                        }
                    }
                }
            }

            index += 1;
            true
        });

        n = n.fold_children_with(self);

        if !has_i18n_import && self.ctx.should_add_18n_import {
            n.insert(
                insert_index,
                create_import(i18n_source.into(), quote_ident!(i18n_export[..])),
            );
        }

        if !has_trans_import && self.ctx.should_add_trans_import {
            n.insert(
                insert_index,
                create_import(trans_source.into(), quote_ident!(trans_export[..])),
            );
        }

        n
    }
    fn fold_arrow_expr(&mut self, n: ArrowExpr) -> ArrowExpr {
        println!("arrow expr");
        n.fold_children_with(self)
    }
    // fn fold_fn_decl(&mut self, n: FnDecl) -> FnDecl {
    //     println!("fold_fn_decl");
    //     n.fold_children_with(self)
    // }
    //
    fn fold_fn_expr(&mut self, n: FnExpr) -> FnExpr {
        println!("fold_fn_expr");
        n.fold_children_with(self)
    }
    fn fold_function(&mut self, n: Function) -> Function {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return n;
        }

        let mut ctx = self.ctx.clone();

        if let Some(body) = n.body {
            let stmts: Vec<Stmt> = body
                .stmts
                .into_iter()
                .map(|stmt| {
                    return match stmt {
                        Stmt::Decl(Decl::Var(var_decl)) => {
                            let decl = *var_decl;

                            let decls: Vec<VarDeclarator> = decl.decls.into_iter().map(|declarator| {
                                if let Some(init) = &declarator.init {
                                    let expr = init.as_ref();

                                    if let Expr::Call(call) = &expr {
                                        if match_callee_name(call, |n| {
                                            self.ctx.is_lingui_ident("useLingui", n)
                                        })
                                        .is_some()
                                        {
                                            if let Pat::Object(obj_pat) = declarator.name {
                                                let mew_props: Vec<ObjectPatProp> =
                                                    obj_pat.props.into_iter().map(|prop| {
                                                        match prop {
                                                            ObjectPatProp::Assign(assign)
                                                                if &assign.key.sym == "t" =>
                                                            {
                                                                ctx.register_reference(
                                                                    &assign.key.sym,
                                                                    &assign.key.to_id(),
                                                                );

                                                                return ObjectPatProp::Assign(
                                                                    AssignPatProp {
                                                                        span: assign.span,
                                                                        value: None,
                                                                        key: BindingIdent {
                                                                            id: Ident {
                                                                                span: assign
                                                                                    .key
                                                                                    .span,
                                                                                sym: "i18n".into(),
                                                                                optional: false,
                                                                            },
                                                                            type_ann: None,
                                                                        },
                                                                    },
                                                                );
                                                            }
                                                            _ => {
                                                                return prop;
                                                                // panic: useLingui could be used only with object desctructuring
                                                            }
                                                        }
                                                    }).collect();

                                                return VarDeclarator {
                                                    init: declarator.init,
                                                    definite: true,
                                                    span:  declarator.span,
                                                    name: Pat::Object(ObjectPat {
                                                        optional: false,
                                                        type_ann: None,
                                                        span: DUMMY_SP,
                                                        props: mew_props
                                                        
                                                    }),
                                                }
                                            } else {
                                                //panic: useLingui could be used only with object desctructuring
                                            }
                                        }
                                    }
                                }
                                
                                return declarator;
                            }).collect();

                            return Stmt::Decl(Decl::Var(Box::new(VarDecl {
                                span: decl.span,
                                decls,
                                declare: false,
                                kind: decl.kind,
                            })))
                        }
                        _ => stmt,
                    };
                })
                .collect();

            let new_func = Function {
                params: n.params,
                body: Some(BlockStmt {
                    span: body.span,
                    stmts,
                }),
                decorators: n.decorators,
                span: n.span,
                is_async: n.is_async,
                is_generator: n.is_generator,
                return_type: n.return_type,
                type_params: n.type_params,
            };

            let mut folder = JsMacroFolder::new(&mut ctx);
            return new_func
                .fold_children_with(&mut folder)
                .fold_children_with(self);
            // folder.fold_expr(n).fold_children_with(self)
        }

        n.fold_children_with(self)
    }

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return expr;
        }

        let mut folder = JsMacroFolder::new(&mut self.ctx);

        folder.fold_expr(expr).fold_children_with(self)
    }

    fn fold_call_expr(&mut self, expr: CallExpr) -> CallExpr {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return expr;
        }

        let mut folder = JsMacroFolder::new(&mut self.ctx);

        folder.fold_call_expr(expr).fold_children_with(self)
    }

    fn fold_jsx_element(&mut self, mut el: JSXElement) -> JSXElement {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return el;
        }

        // apply JS Macro transformations to jsx elements
        // before they will be extracted as message components
        el = el.fold_with(&mut JsMacroFolder::new(&mut self.ctx));

        if let JSXElementName::Ident(ident) = &el.opening.name {
            if self.ctx.is_lingui_ident("Trans", &ident) {
                return self.transform_jsx_macro(el, true);
            }

            if self.ctx.is_lingui_jsx_choice_cmp(&ident) {
                return self.transform_jsx_macro(el, false);
            }
        }

        el.fold_children_with(self)
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<LinguiJsOptions>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for lingui-plugin"),
    )
    .expect("invalid config for lingui-plugin");

    let config = config.to_options(
        &metadata
            .get_context(&TransformPluginMetadataContextKind::Env)
            .unwrap_or_default(),
    );

    program.fold_with(&mut LinguiMacroFolder::new(config))
}
