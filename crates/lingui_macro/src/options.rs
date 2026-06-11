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

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct LinguiJsOptions {
    #[serde(default)]
    pub runtime_modules: Option<RuntimeModulesConfigMap>,
    pub core_package: Option<Vec<String>>,
    #[serde(default)]
    pub jsx_package: Option<Vec<String>>,
    #[serde(default)]
    pub descriptor_fields: Option<DescriptorFields>,
    #[serde(default)]
    pub use_lingui_v5_id_generation: Option<bool>,
    #[serde(default)]
    pub id_prefix_leader: Option<String>,
    #[serde(default)]
    pub jsx_placeholder_attribute: Option<String>,
    #[serde(default)]
    pub jsx_placeholder_defaults: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Default)]
pub struct RuntimeModulesConfig(pub String, #[serde(default)] pub Option<String>);

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeModulesConfigMap {
    pub i18n: Option<RuntimeModulesConfig>,
    #[serde(alias = "Trans")]
    pub trans: Option<RuntimeModulesConfig>,
    pub use_lingui: Option<RuntimeModulesConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RuntimeModulesConfigMapNormalized {
    pub i18n: (String, String),
    pub trans: (String, String),
    pub use_lingui: (String, String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MacroPackagesConfig {
    pub core: Vec<String>,
    pub jsx: Vec<String>,
}

impl MacroPackagesConfig {
    pub fn contains(&self, value: &str) -> bool {
        self.core.iter().any(|s| value == s.as_str())
            || self.jsx.iter().any(|s| value == s.as_str())
    }
}

impl Default for MacroPackagesConfig {
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

        LinguiOptions {
            descriptor_fields,
            use_lingui_v5_id_generation: self.use_lingui_v5_id_generation.unwrap_or(false),
            id_prefix_leader: self.id_prefix_leader.clone(),
            jsx_placeholder_attribute: self.jsx_placeholder_attribute.clone(),
            jsx_placeholder_defaults: self.jsx_placeholder_defaults.clone(),
            macro_packages: MacroPackagesConfig {
                core: self
                    .core_package
                    .unwrap_or_else(|| MacroPackagesConfig::default().core),
                jsx: self
                    .jsx_package
                    .unwrap_or_else(|| MacroPackagesConfig::default().jsx),
            },
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
    pub macro_packages: MacroPackagesConfig,
    #[serde(skip_serializing_if = "is_default")]
    pub runtime_modules: RuntimeModulesConfigMapNormalized,
    #[serde(skip_serializing_if = "is_default")]
    pub use_lingui_v5_id_generation: bool,
}

impl Default for LinguiOptions {
    fn default() -> LinguiOptions {
        LinguiOptions {
            descriptor_fields: DescriptorFields::All,
            use_lingui_v5_id_generation: false,
            id_prefix_leader: None,
            jsx_placeholder_attribute: None,
            jsx_placeholder_defaults: None,
            macro_packages: Default::default(),
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
    fn test_config_capital_trans() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {
                    "Trans": ["@lingui/react", "Trans"]
                }
               }"#,
        )
        .expect("invalid config for lingui-plugin");

        assert_eq!(
            config,
            LinguiJsOptions {
                runtime_modules: Some(RuntimeModulesConfigMap {
                    i18n: None,
                    trans: Some(RuntimeModulesConfig(
                        "@lingui/react".into(),
                        Some("Trans".into())
                    )),
                    use_lingui: None,
                }),
                descriptor_fields: None,
                use_lingui_v5_id_generation: None,
                id_prefix_leader: None,
                jsx_placeholder_attribute: None,
                jsx_placeholder_defaults: None,
                core_package: None,
                jsx_package: None,
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
        let config = serde_json::from_str::<LinguiJsOptions>(r#"{}"#).unwrap();

        let options = config.into_options("development");
        assert_eq!(
            options.macro_packages,
            MacroPackagesConfig {
                core: vec!["@lingui/macro".into(), "@lingui/core/macro".into()],
                jsx: vec!["@lingui/macro".into(), "@lingui/react/macro".into()],
            }
        );

        assert!(options.macro_packages.contains("@lingui/macro"));

        assert!(options.macro_packages.contains("@lingui/core/macro"));

        assert!(options.macro_packages.contains("@lingui/react/macro"));
    }

    #[test]
    fn test_macro_package_overrides() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "corePackage": ["@acme/core/macro"],
                "jsxPackage": ["@acme/react/macro"]
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert_eq!(
            options.macro_packages,
            MacroPackagesConfig {
                core: vec!["@acme/core/macro".into()],
                jsx: vec!["@acme/react/macro".into()],
            }
        );

        assert!(options.macro_packages.contains("@acme/core/macro"));

        assert!(options.macro_packages.contains("@acme/react/macro"));
    }

    #[test]
    fn test_id_prefix_leader_config() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "idPrefixLeader": "."
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
                "descriptorFields": "id-only"
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
                "descriptorFields": "all"
               }"#,
        )
        .unwrap();

        let options = config.into_options("production");
        assert!(matches!(options.descriptor_fields, DescriptorFields::All));

        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "descriptorFields": "message"
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
        let config = serde_json::from_str::<LinguiJsOptions>(r#"{}"#).unwrap();

        let options = config.into_options("development");
        assert!(matches!(options.descriptor_fields, DescriptorFields::All));

        let config = serde_json::from_str::<LinguiJsOptions>(r#"{}"#).unwrap();

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
                "descriptorFields": "auto"
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert!(matches!(options.descriptor_fields, DescriptorFields::All));

        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "descriptorFields": "auto"
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
                "useLinguiV5IdGeneration": true
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert!(options.use_lingui_v5_id_generation);

        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "useLinguiV5IdGeneration": false
               }"#,
        )
        .unwrap();

        let options = config.into_options("production");
        assert!(!options.use_lingui_v5_id_generation);
    }

    #[test]
    fn test_use_lingui_v5_id_generation_default() {
        let config = serde_json::from_str::<LinguiJsOptions>(r#"{}"#).unwrap();

        let options = config.into_options("development");
        assert!(!options.use_lingui_v5_id_generation);

        let config = serde_json::from_str::<LinguiJsOptions>(r#"{}"#).unwrap();

        let options = config.into_options("production");
        assert!(!options.use_lingui_v5_id_generation);
    }

    #[test]
    fn test_empty_config() {
        let config = serde_json::from_str::<LinguiJsOptions>(r#"{}"#).unwrap();

        let options = config.into_options("development");
        assert_eq!(
            options.runtime_modules,
            RuntimeModulesConfigMapNormalized::default()
        );
        assert_eq!(options.macro_packages, MacroPackagesConfig::default());
    }

    #[test]
    fn test_runtime_modules_fully_optional() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert_eq!(
            options.runtime_modules,
            RuntimeModulesConfigMapNormalized::default()
        );
    }

    #[test]
    fn test_runtime_modules_partial() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {
                    "i18n": ["my-core", "myI18n"]
                }
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert_eq!(
            options.runtime_modules.i18n,
            ("my-core".into(), "myI18n".into())
        );
        assert_eq!(
            options.runtime_modules.trans,
            ("@lingui/react".into(), "Trans".into())
        );
        assert_eq!(
            options.runtime_modules.use_lingui,
            ("@lingui/react".into(), "useLingui".into())
        );
    }
}
