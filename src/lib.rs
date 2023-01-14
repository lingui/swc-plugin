use std::collections::{HashSet};
use serde::{Deserialize};
use swc_common::plugin::metadata::TransformPluginMetadataContextKind;
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

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct LinguiJsOptions {
    runtime_modules: Option<RuntimeModulesConfigMap>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct RuntimeModulesConfig(
    String,
    #[serde(default)]
    Option<String>,
);

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct RuntimeModulesConfigMap {
    i18n: Option<RuntimeModulesConfig>,
    trans: Option<RuntimeModulesConfig>,
}

#[derive(Debug)]
struct RuntimeModulesConfigMapNormalized {
    i18n: (String, String),
    trans: (String, String),
}

impl LinguiJsOptions {
    fn to_options(self, env_name: &str) -> LinguiOptions {
        LinguiOptions {
            strip_non_essential_fields: !(matches!(env_name, "development")),
            runtime_modules: RuntimeModulesConfigMapNormalized {
                i18n: (
                    self.runtime_modules.as_ref()
                        .and_then(|o| o.i18n.as_ref())
                        .and_then(|o| Some(o.0.clone()))
                        .unwrap_or("@lingui/core".into()),
                    self.runtime_modules.as_ref()
                        .and_then(|o| o.i18n.as_ref())
                        .and_then(|o| o.1.clone())
                        .unwrap_or("i18n".into()),
                ),
                trans: (
                    self.runtime_modules.as_ref()
                        .and_then(|o| o.trans.as_ref())
                        .and_then(|o| Some(o.0.clone()))
                        .unwrap_or("@lingui/react".into()),
                    self.runtime_modules.as_ref()
                        .and_then(|o| o.trans.as_ref())
                        .and_then(|o| o.1.clone())
                        .unwrap_or("Trans".into()),
                ),
            },
        }
    }
}

#[derive(Debug)]
struct LinguiOptions {
    strip_non_essential_fields: bool,
    runtime_modules: RuntimeModulesConfigMapNormalized,
}

impl Default for LinguiOptions {
    fn default() -> LinguiOptions {
        LinguiOptions {
            strip_non_essential_fields: false,
            runtime_modules: RuntimeModulesConfigMapNormalized {
                i18n: ("@lingui/core".into(), "i18n".into()),
                trans: ("@lingui/react".into(), "Trans".into()),
            },
        }
    }
}

#[derive(Default)]
pub struct LinguiMacroFolder {
    options: LinguiOptions,
    has_lingui_macro_imports: bool,
    should_add_18n_import: bool,
    should_add_trans_import: bool,

    ctx: MacroCtx,
}

impl LinguiMacroFolder {
    fn new(options: LinguiOptions) -> LinguiMacroFolder {
        LinguiMacroFolder {
            options,
            ..Default::default()
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

        if self.options.strip_non_essential_fields {
            attrs = pick_jsx_attrs(attrs, HashSet::from(["id", "render", "i18n", "context", "values", "components"]))
        }

        self.should_add_trans_import = true;

        let (_, trans_export) = self.options.runtime_modules.trans.clone();

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

impl Fold for LinguiMacroFolder {
    fn fold_module_items(&mut self, mut n: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let mut has_i18n_import = false;
        let mut has_trans_import = false;

        let (i18n_source, i18n_export) = self.options.runtime_modules.i18n.clone();
        let (trans_source, trans_export) = self.options.runtime_modules.trans.clone();

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

        if !has_i18n_import && self.should_add_18n_import {
            n.insert(0, create_import(i18n_source.into(), quote_ident!(i18n_export[..])));
        }

        if !has_trans_import && self.should_add_trans_import {
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

        let (_, i18n_export) = self.options.runtime_modules.i18n.clone();

        let mut folder = JsMacroFolder {
            strip_non_essential_fields: self.options.strip_non_essential_fields,
            should_add_18n_import: &mut self.should_add_18n_import,
            i18_callee_name: i18n_export.clone().into(),
            ctx: &self.ctx
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

        let (_, i18n_export) = self.options.runtime_modules.i18n.clone();

        let mut folder = JsMacroFolder {
            strip_non_essential_fields: self.options.strip_non_essential_fields,
            should_add_18n_import: &mut self.should_add_18n_import,
            i18_callee_name: i18n_export.clone().into(),
            ctx: &self.ctx
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

        let (_, i18n_export) = self.options.runtime_modules.i18n.clone();

        // apply JS Macro transformations to jsx elements
        // before they will be extracted as message components
        el = el.fold_with(&mut JsMacroFolder {
            strip_non_essential_fields: self.options.strip_non_essential_fields,
            should_add_18n_import: &mut self.should_add_18n_import,
            i18_callee_name: i18n_export.clone().into(),
            ctx: &self.ctx
        });

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

#[cfg(test)]
mod lib_tests {
    use super::{*};

    #[test]
    fn test_config() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {
                    "i18n": ["@lingui/core", "i18n"],
                    "trans": ["@lingui/react", "Trans"]
                }
               }"#
        )
            .expect("invalid config for lingui-plugin");

        assert_eq!(config, LinguiJsOptions {
            runtime_modules: Some(RuntimeModulesConfigMap {
                i18n: Some(RuntimeModulesConfig("@lingui/core".into(), Some("i18n".into()))),
                trans: Some(RuntimeModulesConfig("@lingui/react".into(), Some("Trans".into()))),
            })
        })
    }

    #[test]
    fn test_config_optional() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {
                    "i18n": ["@lingui/core"]
                }
               }"#
        )
            .expect("invalid config for lingui-plugin");

        assert_eq!(config, LinguiJsOptions {
            runtime_modules: Some(RuntimeModulesConfigMap {
                i18n: Some(RuntimeModulesConfig("@lingui/core".into(), None)),
                trans: None,
            })
        })
    }
}
