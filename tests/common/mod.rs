#[macro_export]
macro_rules! lingui_test {
    ($name:ident, $input:expr) => {
        swc_core::ecma::transforms::testing::test!(
            swc_core::ecma::parser::Syntax::Typescript(swc_core::ecma::parser::TsSyntax {
                tsx: true,
                ..Default::default()
            }),
            |_| {
                (
                    swc_core::ecma::transforms::base::resolver(
                        swc_core::common::Mark::new(),
                        swc_core::common::Mark::new(),
                        true,
                    ),
                    swc_core::ecma::visit::fold_pass(
                        lingui_macro_plugin::LinguiMacroFolder::default(),
                    ),
                )
            },
            $name,
            $input
        );
    };
    (production, $name:ident, $input:expr) => {
        swc_core::ecma::transforms::testing::test!(
            swc_core::ecma::parser::Syntax::Typescript(swc_core::ecma::parser::TsSyntax {
                tsx: true,
                ..Default::default()
            }),
            |_| {
                (
                    swc_core::ecma::transforms::base::resolver(
                        swc_core::common::Mark::new(),
                        swc_core::common::Mark::new(),
                        true,
                    ),
                    swc_core::ecma::visit::fold_pass(lingui_macro_plugin::LinguiMacroFolder::new(
                        lingui_macro_plugin::LinguiOptions {
                            strip_non_essential_fields: true,
                            ..Default::default()
                        },
                    )),
                )
            },
            $name,
            $input
        );
    };
}
