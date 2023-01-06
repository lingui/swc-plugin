use std::collections::HashSet;

use swc_core::{
    ecma::{
        ast::*,
        visit::{Fold, FoldWith, VisitWith},
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
mod js_macro_folder;

use builder::*;
use ast_utils::*;
use js_macro_folder::JsMacroFolder;
use jsx_visitor::TransJSXVisitor;
use crate::macro_utils::{*};


#[derive(Default)]
pub struct LinguiMacroFolder {
    has_lingui_macro_imports: bool,
    should_add_18n_import: bool,
    should_add_trans_import: bool,
}

impl LinguiMacroFolder {
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
            pick_jsx_attrs(el.opening.attrs, HashSet::from(["id", "render", "comment", "context"]))
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
}

impl Fold for LinguiMacroFolder {
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

        let mut folder = JsMacroFolder {
            should_add_18n_import: &mut self.should_add_18n_import,
        };

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

        let mut folder = JsMacroFolder {
            should_add_18n_import: &mut self.should_add_18n_import,
        };

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
        el = el.fold_with(&mut JsMacroFolder {
            should_add_18n_import: &mut self.should_add_18n_import,
        });

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

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut LinguiMacroFolder::default())
}

