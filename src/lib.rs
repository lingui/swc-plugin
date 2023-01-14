use std::collections::HashSet;
use swc_core::{
    ecma::{
        utils::quote_ident,
        ast::*,
        visit::{Fold, FoldWith, VisitWith},
    },
    plugin::{
        metadata::TransformPluginMetadataContextKind,
        plugin_transform,
        proxies::TransformPluginProgramMetadata,
    },
};

mod tests;
mod normalize_witespaces_jsx;
mod normalize_witespaces_js;
mod builder;
mod tokens;
mod ast_utils;
mod macro_utils;
mod jsx_visitor;
mod js_macro_folder;
mod options;

use builder::*;
use ast_utils::*;
use js_macro_folder::JsMacroFolder;
use jsx_visitor::TransJSXVisitor;
use crate::macro_utils::{*};
use crate::options::{*};


#[derive(Default)]
pub struct LinguiMacroFolder {
    has_lingui_macro_imports: bool,
    ctx: MacroCtx,
}

impl LinguiMacroFolder {
    pub  fn new(options: LinguiOptions) -> LinguiMacroFolder {
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
            pick_jsx_attrs(el.opening.attrs, HashSet::from(["id", "render", "comment", "context", "i18n"]))
        );

        if self.ctx.options.strip_non_essential_fields {
            attrs = pick_jsx_attrs(attrs, HashSet::from(["id", "render", "i18n", "context", "values", "components"]))
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
                name: JSXElementName::Ident(
                    Ident::new(trans_export.into(), el.opening.span)
                ),
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

        n.retain(|m| {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(imp)) = m {
                // drop macro imports
                if &imp.src.value == "@lingui/macro" {
                    self.has_lingui_macro_imports = true;
                    self.ctx.register_macro_import(imp);
                    return false;
                }

                if &imp.src.value == &i18n_source && !imp.type_only {
                    for spec in &imp.specifiers {
                        if let ImportSpecifier::Named(spec) = spec {
                            has_i18n_import = if !has_i18n_import { &spec.local.sym == &i18n_export } else { true };
                        }
                    }
                }

                if &imp.src.value == &trans_source && !imp.type_only {
                    for spec in &imp.specifiers {
                        if let ImportSpecifier::Named(spec) = spec {
                            has_trans_import = if !has_trans_import { &spec.local.sym == &trans_export } else { true };
                        }
                    }
                }
            }

            true
        });

        // println!("{:?}", self.ctx.imports_id_map);

        n = n.fold_children_with(self);

        if !has_i18n_import && self.ctx.should_add_18n_import {
            n.insert(0, create_import(i18n_source.into(), quote_ident!(i18n_export[..])));
        }

        if !has_trans_import && self.ctx.should_add_trans_import {
            n.insert(0, create_import(trans_source.into(), quote_ident!(trans_export[..])));
        }

        n
    }

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return expr;
        }

        let mut folder = JsMacroFolder::new(&mut self.ctx);

        folder
            .fold_expr(expr)
            .fold_children_with(self)
    }

    fn fold_call_expr(&mut self, expr: CallExpr) -> CallExpr {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        if !self.has_lingui_macro_imports {
            return expr;
        }

        let mut folder = JsMacroFolder::new(&mut self.ctx);

        folder
            .fold_call_expr(expr)
            .fold_children_with(self)
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
