use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LinguiJsOptions {
    runtime_modules: Option<RuntimeModulesConfigMap>,
    #[serde(default)]
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
        LinguiOptions {
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
    pub strip_non_essential_fields: bool,
    pub jsx_placeholder_attribute: Option<String>,
    pub jsx_placeholder_defaults: Option<HashMap<String, String>>,
    pub runtime_modules: RuntimeModulesConfigMapNormalized,
}

impl Default for LinguiOptions {
    fn default() -> LinguiOptions {
        LinguiOptions {
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
                strip_non_essential_fields: None,
                jsx_placeholder_attribute: None,
                jsx_placeholder_defaults: None,
            }
        )
    }

    #[test]
    fn test_strip_non_essential_fields_config() {
        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "stripNonEssentialFields": true,
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("development");
        assert!(options.strip_non_essential_fields);

        let config = serde_json::from_str::<LinguiJsOptions>(
            r#"{
                "stripNonEssentialFields": false,
                "runtimeModules": {}
               }"#,
        )
        .unwrap();

        let options = config.into_options("production");
        assert!(!options.strip_non_essential_fields);
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
}
