use std::collections::HashSet;
use swc_core::common::{Span, Spanned, SyntaxContext, DUMMY_SP};

use swc_core::common::comments::*;
use swc_core::ecma::utils::private_ident;
use swc_core::plugin::errors::HANDLER;
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
pub mod message_extractor;
pub mod message_extractor_visitor;
mod options;
mod tokens;

use crate::generate_id::*;
use crate::macro_utils::*;
use crate::options::*;
use ast_utils::*;
use builder::*;
use js_macro_folder::JsMacroFolder;
use jsx_visitor::TransJSXVisitor;
pub use message_extractor_visitor::ExtractedMessage;
pub use message_extractor_visitor::ExtractionResult;

pub struct IdentReplacer {
    from: Id,
    to: Ident,
}
impl IdentReplacer {
    // pub fn new(from: Id, to: Ident) -> {}
}
impl Fold for IdentReplacer {
    fn fold_ident(&mut self, n: Ident) -> Ident {
        if n.to_id() == self.from {
            return self.to.clone();
        }

        n
    }
}

pub struct LinguiMacroFolder<C>
where
    C: Comments + Clone,
{
    has_lingui_macro_imports: bool,
    ctx: MacroCtx,
    comments: Option<C>,
}

impl<C> LinguiMacroFolder<C>
where
    C: Comments + Clone,
{
    pub fn new(options: LinguiOptions, comments: Option<C>) -> LinguiMacroFolder<C> {
        LinguiMacroFolder {
            has_lingui_macro_imports: false,
            ctx: MacroCtx::new(options),
            comments,
        }
    }

    // <Trans>Message</Trans>
    // <Plural />
    fn transform_jsx_macro(&mut self, el: JSXElement, is_trans_el: bool) -> JSXElement {
        let mut trans_visitor = TransJSXVisitor::new(&self.ctx);

        let message_dscrptr_span: Span;

        // SWC attaches comments based on the `span` position of a node.
        // If a synthesized node reuses the span of an existing node,
        // the comment may end up in an unexpected place.
        //
        // Example:
        // `MessageDescriptor`’s `ObjectLit` can inherit the span of `<Trans>`.
        // In that case the comment `/* i18n */` would be emitted as:
        //
        //   /* i18n */ <Trans {... {desc}}>
        //
        // instead of:
        //
        //   <Trans {... /* i18n */ {desc}}>
        //
        // To avoid this, the span must come from a real source element that no
        // longer exists after the transformation, so that source maps stay accurate and
        // comments are placed correctly.
        //
        // Accurate sourcemaps ensures that the extractor can correctly report the original
        // line and column of the macro invocation.
        //
        // Span selection strategy:
        // - For a regular `<Trans>`, use the span of its first child.
        // - For `<Plural>`, use the span of the `value` attribute, since this element has no children.
        if is_trans_el {
            // Trans
            message_dscrptr_span = el.children.first().span();
            el.children.visit_children_with(&mut trans_visitor);
        } else {
            let value_attr =
                get_jsx_attr(&el.opening, "value").and_then(|attr| attr.value.as_ref());

            // <Plural />, etc
            message_dscrptr_span = value_attr.span();
            el.visit_children_with(&mut trans_visitor);
        }

        let parsed = MessageBuilder::parse(trans_visitor.tokens);
        let id_attr = get_jsx_attr(&el.opening, "id").and_then(|attr| attr.value.as_ref());

        let context_attr =
            get_jsx_attr(&el.opening, "context").and_then(|attr| attr.value.as_ref());

        let mut message_descriptor_props: Vec<PropOrSpread> = vec![];

        if let Some(attr) = id_attr {
            message_descriptor_props.push(create_key_value_prop(
                "id",
                get_jsx_attr_value_as_string(attr)
                    .unwrap_or_default()
                    .into(),
            ));
        } else {
            let context_attr_val = context_attr.and_then(get_jsx_attr_value_as_string);

            message_descriptor_props.push(create_key_value_prop(
                "id",
                generate_message_id(&parsed.message_str, &context_attr_val.unwrap_or_default())
                    .into(),
            ));
        }

        if let Some(exp) = parsed.values {
            message_descriptor_props.push(create_key_value_prop("values", exp));
        }

        if let Some(exp) = parsed.components {
            message_descriptor_props.push(create_key_value_prop("components", exp));
        }

        if !self.ctx.options.strip_non_essential_fields {
            let comment_attr = get_jsx_attr(&el.opening, "comment")
                .and_then(|attr| attr.value.as_ref())
                .and_then(get_jsx_attr_value_as_string);

            if let Some(comment) = comment_attr {
                message_descriptor_props.push(create_key_value_prop("comment", comment.into()));
            }

            message_descriptor_props.push(create_key_value_prop("message", parsed.message));

            if let Some(context_attr) = context_attr {
                let context_attr_val = get_jsx_attr_value_as_string(context_attr).unwrap();

                message_descriptor_props.push(create_key_value_prop(
                    "context",
                    Box::new(Expr::Lit(Lit::Str(Str {
                        span: context_attr.span(),
                        value: context_attr_val.into(),
                        raw: None,
                    }))),
                ));
            }
        }

        let message_descriptor = Expr::Object(ObjectLit {
            span: message_dscrptr_span,
            props: message_descriptor_props,
        });

        add_i18n_comment(&self.comments, message_descriptor.span());

        let mut attrs = vec![JSXAttrOrSpread::SpreadElement(SpreadElement {
            dot3_token: DUMMY_SP,
            expr: Box::new(message_descriptor),
        })];

        attrs.extend(pick_jsx_attrs(
            el.opening.attrs,
            HashSet::from(["component", "render", "i18n"]),
        ));

        self.ctx.should_add_trans_import = true;

        JSXElement {
            span: el.span,
            children: vec![],
            closing: None,
            opening: JSXOpeningElement {
                self_closing: true,
                span: el.opening.span,
                name: JSXElementName::Ident(self.ctx.runtime_idents.trans.clone().into()),
                type_args: None,
                attrs,
            },
        }
    }

    pub fn handle_use_lingui(&mut self, n: BlockStmt) -> BlockStmt {
        let mut ctx = self.ctx.clone();

        let mut ident_replacer: Option<IdentReplacer> = None;

        let stmts: Vec<Stmt> = n
            .stmts
            .into_iter()
            .map(|stmt| match stmt {
                Stmt::Decl(Decl::Var(var_decl)) => {
                    let decl = *var_decl;

                    let underscore_ident = private_ident!("$__");
                    let decls: Vec<VarDeclarator> = decl
                        .decls
                        .into_iter()
                        .map(|declarator| {
                            if let Some(init) = &declarator.init {
                                let expr = init.as_ref();

                                if let Expr::Call(call) = &expr {
                                    if match_callee_name(call, |n| {
                                        self.ctx.is_lingui_ident("useLingui", n)
                                    })
                                    .is_some()
                                    {
                                        self.ctx.should_add_uselingui_import = true;

                                        if let Pat::Object(obj_pat) = declarator.name {
                                            let mut new_props: Vec<ObjectPatProp> = obj_pat
                                                .props
                                                .into_iter()
                                                .map(|prop| {
                                                    get_local_ident_from_object_pat_prop(&prop, "t")
                                                        .map(|ident| {
                                                            ctx.register_reference(
                                                                &"t".into(),
                                                                &ident.to_id(),
                                                            );

                                                            let new_i18n_ident =
                                                                private_ident!("$__i18n");

                                                            ident_replacer = Some(IdentReplacer {
                                                                from: ident.to_id(),
                                                                to: underscore_ident.clone(),
                                                            });

                                                            ctx.runtime_idents.i18n =
                                                                new_i18n_ident.clone();

                                                            ObjectPatProp::KeyValue(
                                                                KeyValuePatProp {
                                                                    value: Box::new(Pat::Ident(
                                                                        new_i18n_ident.into(),
                                                                    )),
                                                                    key: PropName::Ident(
                                                                        quote_ident!("i18n"),
                                                                    ),
                                                                },
                                                            )
                                                        })
                                                        .unwrap_or(prop)
                                                })
                                                .collect();

                                            new_props.push(ObjectPatProp::KeyValue(
                                                KeyValuePatProp {
                                                    value: Box::new(Pat::Ident(
                                                        underscore_ident.clone().into(),
                                                    )),
                                                    key: PropName::Ident(quote_ident!("_")),
                                                },
                                            ));

                                            return VarDeclarator {
                                                init: Some(Box::new(Expr::Call(CallExpr {
                                                    callee: Callee::Expr(Box::new(Expr::Ident(
                                                        ctx.runtime_idents
                                                            .use_lingui
                                                            .clone()
                                                            .into(),
                                                    ))),
                                                    ..call.clone()
                                                }))),

                                                definite: true,
                                                span: declarator.span,
                                                name: Pat::Object(ObjectPat {
                                                    optional: false,
                                                    type_ann: None,
                                                    span: DUMMY_SP,
                                                    props: new_props,
                                                }),
                                            };
                                        } else {
                                            HANDLER.with(|h| {
                                                h.struct_span_warn(decl.span, "Unsupported Syntax")
                                                        .note(
r#"You have to destructure `t` when using the `useLingui` macro, i.e:
 const { t } = useLingui()
 or
 const { t: _ } = useLingui()"#)
                                                        .emit()
                                            });
                                        }
                                    }
                                }
                            }

                            declarator
                        })
                        .collect();

                    Stmt::Decl(Decl::Var(Box::new(VarDecl {
                        span: decl.span,
                        decls,
                        declare: false,
                        kind: decl.kind,
                        ctxt: SyntaxContext::empty(),
                    })))
                }
                _ => stmt,
            })
            .collect();

        let mut block = BlockStmt {
            span: n.span,
            stmts,
            ctxt: SyntaxContext::empty(),
        };

        // use lingui matched above
        if ident_replacer.is_some() {
            block = block
                .fold_children_with(&mut JsMacroFolder::new(&mut ctx, &self.comments))
                // replace other
                .fold_children_with(&mut ident_replacer.unwrap());
        }

        block.fold_children_with(self)
    }
}

impl<C> Fold for LinguiMacroFolder<C>
where
    C: Comments + Clone,
{
    fn fold_module_items(&mut self, mut n: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let (i18n_source, i18n_export) = self.ctx.options.runtime_modules.i18n.clone();
        let (trans_source, trans_export) = self.ctx.options.runtime_modules.trans.clone();
        let (use_lingui_source, use_lingui_export) =
            self.ctx.options.runtime_modules.use_lingui.clone();

        let mut insert_index: usize = 0;
        let mut index = 0;

        n.retain(|m| {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(imp)) = m {
                // drop macro imports
                if &imp.src.value == "@lingui/macro"
                    || &imp.src.value == "@lingui/core/macro"
                    || &imp.src.value == "@lingui/react/macro"
                {
                    self.has_lingui_macro_imports = true;
                    self.ctx.register_macro_import(imp);
                    insert_index = index;
                    return false;
                }
            }

            index += 1;
            true
        });

        n = n.fold_children_with(self);

        if self.ctx.should_add_18n_import {
            n.insert(
                insert_index,
                create_import(
                    i18n_source.into(),
                    quote_ident!(i18n_export[..]),
                    self.ctx.runtime_idents.i18n.clone().into(),
                ),
            );
        }

        if self.ctx.should_add_trans_import {
            n.insert(
                insert_index,
                create_import(
                    trans_source.into(),
                    quote_ident!(trans_export[..]),
                    self.ctx.runtime_idents.trans.clone(),
                ),
            );
        }

        if self.ctx.should_add_uselingui_import {
            n.insert(
                insert_index,
                create_import(
                    use_lingui_source.into(),
                    quote_ident!(use_lingui_export[..]),
                    self.ctx.runtime_idents.use_lingui.clone(),
                ),
            );
        }

        n
    }
    fn fold_arrow_expr(&mut self, n: ArrowExpr) -> ArrowExpr {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return n;
        }

        let mut func = n;

        if func.body.is_block_stmt() {
            let block = func.body.block_stmt().unwrap();

            func = ArrowExpr {
                body: Box::new(BlockStmtOrExpr::BlockStmt(self.handle_use_lingui(block))),
                ..func
            }
        }

        func.fold_children_with(self)
    }

    fn fold_function(&mut self, n: Function) -> Function {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return n;
        }

        let mut func = n;

        if let Some(body) = func.body {
            func = Function {
                body: Some(self.handle_use_lingui(body)),
                ..func
            };
        }

        func.fold_children_with(self)
    }

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return expr;
        }

        if let Expr::Arrow(arrow_expr) = expr {
            return Expr::Arrow(self.fold_arrow_expr(arrow_expr));
        }

        let mut folder = JsMacroFolder::new(&mut self.ctx, &self.comments);

        folder.fold_expr(expr).fold_children_with(self)
    }

    fn fold_call_expr(&mut self, expr: CallExpr) -> CallExpr {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return expr;
        }

        let mut folder = JsMacroFolder::new(&mut self.ctx, &self.comments);

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
        el = el.fold_with(&mut JsMacroFolder::new(&mut self.ctx, &self.comments));

        if let JSXElementName::Ident(ident) = &el.opening.name {
            if self.ctx.is_lingui_ident("Trans", ident) {
                return self.transform_jsx_macro(el, true);
            }

            if self.ctx.is_lingui_jsx_choice_cmp(ident) {
                return self.transform_jsx_macro(el, false);
            }
        }

        el.fold_children_with(self)
    }
}

pub use self::message_extractor::extract_messages;
pub use self::options::{LinguiOptions, RuntimeModulesConfigMapNormalized};

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<LinguiJsOptions>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for lingui-plugin"),
    )
    .expect("invalid config for lingui-plugin");

    let config = config.into_options(
        &metadata
            .get_context(&TransformPluginMetadataContextKind::Env)
            .unwrap_or_default(),
    );

    program.fold_with(&mut LinguiMacroFolder::new(config, metadata.comments))
}
