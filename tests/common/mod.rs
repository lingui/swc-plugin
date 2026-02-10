#[macro_export]
macro_rules! to {
    ($name:ident, $input:expr) => {
        swc_core::ecma::transforms::testing::test!(
            swc_core::ecma::parser::Syntax::Typescript(swc_core::ecma::parser::TsSyntax {
                tsx: true,
                ..Default::default()
            }),
            |tester| {
                (
                    swc_core::ecma::transforms::base::resolver(
                        swc_core::common::Mark::new(),
                        swc_core::common::Mark::new(),
                        true,
                    ),
                    swc_core::ecma::visit::fold_pass(lingui_macro_plugin::LinguiMacroFolder::new(
                        Default::default(),
                        Some(tester.comments.clone()),
                    )),
                )
            },
            $name,
            $input
        );
    };
    ($name:ident, $options:expr, $input:expr) => {
        swc_core::ecma::transforms::testing::test!(
            swc_core::ecma::parser::Syntax::Typescript(swc_core::ecma::parser::TsSyntax {
                tsx: true,
                ..Default::default()
            }),
            |tester| {
                swc_core::ecma::visit::fold_pass(lingui_macro_plugin::LinguiMacroFolder::new(
                    $options,
                    Some(tester.comments.clone()),
                ))
            },
            $name,
            $input
        );
    };
}
