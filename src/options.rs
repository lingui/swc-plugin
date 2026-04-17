use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum DescriptorFields {
    Auto,
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
    descriptor_fields: Option<DescriptorFields>,
    #[serde(default)]
    use_lingui_v5_id_generation: Option<bool>,
    strip_non_essential_fields: Option<bool>,
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

#[derive(Debug, Clone)]
pub struct RuntimeModulesConfigMapNormalized {
    pub i18n: (String, String),
    pub trans: (String, String),
    pub use_lingui: (String, String),
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
            strip_non_essential_fields: self
                .strip_non_essential_fields
                .unwrap_or(matches!(env_name, "production")),
            jsx_placeholder_attribute: self.jsx_placeholder_attribute.clone(),
            jsx_placeholder_defaults: self.jsx_placeholder_defaults.clone(),
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

#[derive(Debug, Clone)]
pub struct LinguiOptions {
    pub descriptor_fields: DescriptorFields,
    pub strip_non_essential_fields: bool,
    pub jsx_placeholder_attribute: Option<String>,
    pub jsx_placeholder_defaults: Option<HashMap<String, String>>,
    pub runtime_modules: RuntimeModulesConfigMapNormalized,
    pub use_lingui_v5_id_generation: bool,
}

impl Default for LinguiOptions {
    fn default() -> LinguiOptions {
        LinguiOptions {
            descriptor_fields: DescriptorFields::All,
            use_lingui_v5_id_generation: false,
            strip_non_essential_fields: false,
            jsx_placeholder_attribute: None,
            jsx_placeholder_defaults: None,
            runtime_modules: RuntimeModulesConfigMapNormalized {
                i18n: ("@lingui/core".into(), "i18n".into()),
                trans: ("@lingui/react".into(), "Trans".into()),
                use_lingui: ("@lingui/react".into(), "useLingui".into()),
            },
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
                descriptor_fields: None,
                use_lingui_v5_id_generation: None,
                strip_non_essential_fields: None,
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
                descriptor_fields: None,
                use_lingui_v5_id_generation: None,
                strip_non_essential_fields: None,
                jsx_placeholder_attribute: None,
                jsx_placeholder_defaults: None,
            }
        )
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
    fn test_strip_non_essential_fields_default() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert!(!options.strip_non_essential_fields);

        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("production");
        assert!(options.strip_non_essential_fields);
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
