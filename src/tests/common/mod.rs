#[macro_export]
macro_rules! to {
    ($name:ident, $from:expr, $to:expr) => {
        swc_core::ecma::transforms::testing::test!(
            swc_core::ecma::parser::Syntax::Typescript(swc_core::ecma::parser::TsConfig {
                tsx: true,
                ..Default::default()
            }),
            |_| {
                if let Err(_) = swc_core::plugin::errors::HANDLER.inner.set(
                        swc_core::common::errors::Handler::with_tty_emitter(
                            swc_core::common::errors::ColorConfig::Auto,
                            true,
                            false,
                            None,
                        )
                ) {
                    // set on a previous run
                }

                swc_core::common::chain!(
                    swc_core::ecma::transforms::base::resolver(swc_core::common::Mark::new(), swc_core::common::Mark::new(), true),
                    $crate::LinguiMacroFolder::default()
                )
            },
            $name,
            $from,
            $to,
            ok_if_code_eq
        );
    };

    (production, $name:ident, $from:expr, $to:expr) => {
        swc_core::ecma::transforms::testing::test!(
            swc_core::ecma::parser::Syntax::Typescript(swc_core::ecma::parser::TsConfig {
                tsx: true,
                ..Default::default()
            }),
            |_| {
                if let Err(_) = swc_core::plugin::errors::HANDLER.inner.set(
                        swc_core::common::errors::Handler::with_tty_emitter(
                            swc_core::common::errors::ColorConfig::Auto,
                            true,
                            false,
                            None,
                        )
                ) {
                    // set on a previous run
                }

                swc_core::common::chain!(
                    swc_core::ecma::transforms::base::resolver(swc_core::common::Mark::new(), swc_core::common::Mark::new(), true),
                    $crate::LinguiMacroFolder::new(
                         $crate::LinguiOptions {
                        strip_non_essential_fields: true,
                        ..Default::default()
                    })
                )
            },
            $name,
            $from,
            $to,
            ok_if_code_eq
        );
    }
}
