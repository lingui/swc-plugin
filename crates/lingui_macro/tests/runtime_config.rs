#[macro_use]
mod common;

to!(
    should_use_provided_runtime_modules,
    lingui_macro::LinguiOptions {
        runtime_modules: lingui_macro::RuntimeModulesConfigMapNormalized {
            i18n: ("./custom-core".into(), "customI18n".into()),
            trans: ("./custom-react".into(), "CustomTrans".into()),
            use_lingui: ("./custom-react".into(), "useLingui2".into())
        },
        ..Default::default()
    },
    r#"
     import { t } from "@lingui/core/macro";
     import { Trans } from "@lingui/react/macro";

     t`Refresh inbox`;
     const exp2 = <Trans id="custom.id">Refresh inbox</Trans>;
     "#
);
