#[cfg(test)]
use crate::{LinguiOptions, RuntimeModulesConfigMapNormalized};

macro_rules! to {
    ($name:ident, $options:expr, $from:expr, $to:expr) => {
        swc_core::ecma::transforms::testing::test_inline!(
            swc_core::ecma::parser::Syntax::Typescript(swc_core::ecma::parser::TsSyntax {
                tsx: true,
                ..Default::default()
            }),
            |_| {
              $crate::LinguiMacroFolder::new($options)
            },
            $name,
            $from,
            $to
        );
    };
}

to!(
    should_use_provided_runtime_modules,
    LinguiOptions {
        runtime_modules: RuntimeModulesConfigMapNormalized {
            i18n: ("./custom-core".into(), "customI18n".into()),
            trans: ("./custom-react".into(), "CustomTrans".into())
        },
        ..Default::default()
    },
     r#"
     import { t, Trans } from "@lingui/macro";

     t`Refresh inbox`;
     const exp2 = <Trans id="custom.id">Refresh inbox</Trans>;
     "#,
    r#"
    import { CustomTrans } from "./custom-react";
    import { customI18n } from "./custom-core";

    customI18n._({
        id: "EsCV2T",
        message: "Refresh inbox"
    });

    const exp2 = <CustomTrans message={"Refresh inbox"} id="custom.id"/>;
    "#
);
