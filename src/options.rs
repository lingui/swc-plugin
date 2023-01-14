use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LinguiJsOptions {
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
pub struct RuntimeModulesConfigMap {
    i18n: Option<RuntimeModulesConfig>,
    trans: Option<RuntimeModulesConfig>,
}

#[derive(Debug)]
pub struct RuntimeModulesConfigMapNormalized {
    pub i18n: (String, String),
    pub trans: (String, String),
}

impl LinguiJsOptions {
    pub fn to_options(self, env_name: &str) -> LinguiOptions {
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
pub struct LinguiOptions {
    pub strip_non_essential_fields: bool,
    pub runtime_modules: RuntimeModulesConfigMapNormalized,
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

#[cfg(test)]
mod lib_tests {
    use super::*;

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
