use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DescriptorFields {
    Auto,
    #[default]
    All,
    IdOnly,
    Message,
}

impl DescriptorFields {
    pub fn should_keep_message(&self) -> bool {
        matches!(self, DescriptorFields::All | DescriptorFields::Message)
    }

    pub fn should_keep_context(&self) -> bool {
        matches!(self, DescriptorFields::All | DescriptorFields::Message)
    }

    pub fn should_keep_comment(&self) -> bool {
        matches!(self, DescriptorFields::All)
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LinguiJsOptions {
    runtime_modules: Option<RuntimeModulesConfigMap>,
    #[serde(default)]
    core_package: Option<Vec<String>>,
    #[serde(default)]
    jsx_package: Option<Vec<String>>,
    #[serde(default)]
    descriptor_fields: Option<DescriptorFields>,
    #[serde(default)]
    use_lingui_v5_id_generation: Option<bool>,
    #[serde(default)]
    id_prefix_leader: Option<String>,
    #[serde(default)]
    jsx_placeholder_attribute: Option<String>,
    #[serde(default)]
    jsx_placeholder_defaults: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct RuntimeModulesConfig(String, #[serde(default)] Option<String>);

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeModulesConfigMap {
    i18n: Option<RuntimeModulesConfig>,
    trans: Option<RuntimeModulesConfig>,
    use_lingui: Option<RuntimeModulesConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RuntimeModulesConfigMapNormalized {
    pub i18n: (String, String),
    pub trans: (String, String),
    pub use_lingui: (String, String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MacroPackagesConfigNormalized {
    pub core: Vec<String>,
    pub jsx: Vec<String>,
}

impl MacroPackagesConfigNormalized {
    pub(crate) fn all_macro_packages(&self) -> Vec<String> {
        let mut all_macro_packages = Vec::with_capacity(self.core.len() + self.jsx.len());

        for package in self.core.iter().chain(self.jsx.iter()) {
            if all_macro_packages
                .iter()
                .any(|existing| existing == package)
            {
                continue;
            }

            all_macro_packages.push(package.clone());
        }

        all_macro_packages
    }
}

impl Default for MacroPackagesConfigNormalized {
    fn default() -> Self {
        Self {
            core: vec!["@lingui/macro".into(), "@lingui/core/macro".into()],
            jsx: vec!["@lingui/macro".into(), "@lingui/react/macro".into()],
        }
    }
}

impl Default for RuntimeModulesConfigMapNormalized {
    fn default() -> Self {
        Self {
            i18n: ("@lingui/core".into(), "i18n".into()),
            trans: ("@lingui/react".into(), "Trans".into()),
            use_lingui: ("@lingui/react".into(), "useLingui".into()),
        }
    }
}

impl LinguiOptions {
    pub fn sync_derived_fields(&mut self) {
        self.all_macro_packages = self.macro_packages.all_macro_packages();
    }
}

impl LinguiJsOptions {
    pub fn into_options(self, env_name: &str) -> LinguiOptions {
        let descriptor_fields = match self.descriptor_fields.unwrap_or(DescriptorFields::Auto) {
            DescriptorFields::Auto => {
                if matches!(env_name, "production") {
                    DescriptorFields::IdOnly
                } else {
                    DescriptorFields::All
                }
            }
            other => other,
        };

        let macro_packages = MacroPackagesConfigNormalized {
            core: self
                .core_package
                .unwrap_or_else(|| MacroPackagesConfigNormalized::default().core),
            jsx: self
                .jsx_package
                .unwrap_or_else(|| MacroPackagesConfigNormalized::default().jsx),
        };
        let all_macro_packages = macro_packages.all_macro_packages();

        LinguiOptions {
            descriptor_fields,
            use_lingui_v5_id_generation: self.use_lingui_v5_id_generation.unwrap_or(false),
            id_prefix_leader: self.id_prefix_leader.clone(),
            jsx_placeholder_attribute: self.jsx_placeholder_attribute.clone(),
            jsx_placeholder_defaults: self.jsx_placeholder_defaults.clone(),
            macro_packages,
            all_macro_packages,
            runtime_modules: RuntimeModulesConfigMapNormalized {
                i18n: (
                    self.runtime_modules
                        .as_ref()
                        .and_then(|o| o.i18n.as_ref())
                        .map(|o| o.0.clone())
                        .unwrap_or("@lingui/core".into()),
                    self.runtime_modules
                        .as_ref()
                        .and_then(|o| o.i18n.as_ref())
                        .and_then(|o| o.1.clone())
                        .unwrap_or("i18n".into()),
                ),
                trans: (
                    self.runtime_modules
                        .as_ref()
                        .and_then(|o| o.trans.as_ref())
                        .map(|o| o.0.clone())
                        .unwrap_or("@lingui/react".into()),
                    self.runtime_modules
                        .as_ref()
                        .and_then(|o| o.trans.as_ref())
                        .and_then(|o| o.1.clone())
                        .unwrap_or("Trans".into()),
                ),
                use_lingui: (
                    self.runtime_modules
                        .as_ref()
                        .and_then(|o| o.use_lingui.as_ref())
                        .map(|o| o.0.clone())
                        .unwrap_or("@lingui/react".into()),
                    self.runtime_modules
                        .as_ref()
                        .and_then(|o| o.use_lingui.as_ref())
                        .and_then(|o| o.1.clone())
                        .unwrap_or("useLingui".into()),
                ),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LinguiOptions {
    #[serde(skip_serializing_if = "is_default")]
    pub descriptor_fields: DescriptorFields,
    #[serde(skip_serializing_if = "is_default")]
    pub id_prefix_leader: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub jsx_placeholder_attribute: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub jsx_placeholder_defaults: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "is_default")]
    pub macro_packages: MacroPackagesConfigNormalized,
    #[serde(skip)]
    pub all_macro_packages: Vec<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub runtime_modules: RuntimeModulesConfigMapNormalized,
    #[serde(skip_serializing_if = "is_default")]
    pub use_lingui_v5_id_generation: bool,
}

impl Default for LinguiOptions {
    fn default() -> LinguiOptions {
        let macro_packages = MacroPackagesConfigNormalized::default();

        LinguiOptions {
            descriptor_fields: DescriptorFields::All,
            use_lingui_v5_id_generation: false,
            id_prefix_leader: None,
            jsx_placeholder_attribute: None,
            jsx_placeholder_defaults: None,
            all_macro_packages: macro_packages.all_macro_packages(),
            macro_packages,
            runtime_modules: Default::default(),
        }
    }
}

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn test_config() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {
                    "i18n": ["my-core", "myI18n"],
                    "trans": ["my-react", "myTrans"],
                    "useLingui": ["my-react", "myUseLingui"]
                }
               }"#,
        )
        .expect("invalid config for lingui-plugin");

        assert_eq!(
            config,
            LinguiJsOptions {
                runtime_modules: Some(RuntimeModulesConfigMap {
                    i18n: Some(RuntimeModulesConfig(
                        "my-core".into(),
                        Some("myI18n".into())
                    )),
                    trans: Some(RuntimeModulesConfig(
                        "my-react".into(),
                        Some("myTrans".into())
                    )),
                    use_lingui: Some(RuntimeModulesConfig(
                        "my-react".into(),
                        Some("myUseLingui".into())
                    )),
                }),
                core_package: None,
                jsx_package: None,
                descriptor_fields: None,
                use_lingui_v5_id_generation: None,
                id_prefix_leader: None,
                jsx_placeholder_attribute: None,
                jsx_placeholder_defaults: None,
            }
        )
    }

    #[test]
    fn test_config_optional() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {
                    "i18n": ["@lingui/core"]
                }
               }"#,
        )
        .expect("invalid config for lingui-plugin");

        assert_eq!(
            config,
            LinguiJsOptions {
                runtime_modules: Some(RuntimeModulesConfigMap {
                    i18n: Some(RuntimeModulesConfig("@lingui/core".into(), None)),
                    trans: None,
                    use_lingui: None,
                }),
                core_package: None,
                jsx_package: None,
                descriptor_fields: None,
                use_lingui_v5_id_generation: None,
                id_prefix_leader: None,
                jsx_placeholder_attribute: None,
                jsx_placeholder_defaults: None,
            }
        )
    }

    #[test]
    fn test_macro_package_defaults() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert_eq!(
            options.macro_packages,
            MacroPackagesConfigNormalized {
                core: vec!["@lingui/macro".into(), "@lingui/core/macro".into()],
                jsx: vec!["@lingui/macro".into(), "@lingui/react/macro".into()],
            }
        );
        assert_eq!(
            options.all_macro_packages,
            vec!["@lingui/macro", "@lingui/core/macro", "@lingui/react/macro"]
        );
    }

    #[test]
    fn test_macro_package_overrides() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "corePackage": ["@acme/core/macro"],
                "jsxPackage": ["@acme/react/macro"],
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert_eq!(
            options.macro_packages,
            MacroPackagesConfigNormalized {
                core: vec!["@acme/core/macro".into()],
                jsx: vec!["@acme/react/macro".into()],
            }
        );
        assert_eq!(
            options.all_macro_packages,
            vec!["@acme/core/macro", "@acme/react/macro"]
        );
    }

    #[test]
    fn test_id_prefix_leader_config() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "idPrefixLeader": ".",
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert_eq!(options.id_prefix_leader.as_deref(), Some("."));
    }

    #[test]
    fn test_descriptor_fields_config() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "descriptorFields": "id-only",
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert!(matches!(
            options.descriptor_fields,
            DescriptorFields::IdOnly
        ));

        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "descriptorFields": "all",
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("production");
        assert!(matches!(options.descriptor_fields, DescriptorFields::All));

        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "descriptorFields": "message",
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("production");
        assert!(matches!(
            options.descriptor_fields,
            DescriptorFields::Message
        ));
    }

    #[test]
    fn test_descriptor_fields_auto_default() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert!(matches!(options.descriptor_fields, DescriptorFields::All));

        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("production");
        assert!(matches!(
            options.descriptor_fields,
            DescriptorFields::IdOnly
        ));
    }

    #[test]
    fn test_jsx_placeholder_config() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "jsxPlaceholderAttribute": "_t",
                "jsxPlaceholderDefaults": {
                    "a": "link",
                    "em": "emphasis"
                }
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert_eq!(options.jsx_placeholder_attribute.unwrap(), "_t");

        let defaults = options.jsx_placeholder_defaults.unwrap();
        assert_eq!(defaults.get("a").unwrap(), "link");
        assert_eq!(defaults.get("em").unwrap(), "emphasis");
    }

    #[test]
    fn test_descriptor_fields_explicit_auto() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "descriptorFields": "auto",
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert!(matches!(options.descriptor_fields, DescriptorFields::All));

        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "descriptorFields": "auto",
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("production");
        assert!(matches!(
            options.descriptor_fields,
            DescriptorFields::IdOnly
        ));
    }

    #[test]
    fn test_use_lingui_v5_id_generation_config() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "useLinguiV5IdGeneration": true,
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert!(options.use_lingui_v5_id_generation);

        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "useLinguiV5IdGeneration": false,
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("production");
        assert!(!options.use_lingui_v5_id_generation);
    }

    #[test]
    fn test_use_lingui_v5_id_generation_default() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert!(!options.use_lingui_v5_id_generation);

        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("production");
        assert!(!options.use_lingui_v5_id_generation);
    }
}
