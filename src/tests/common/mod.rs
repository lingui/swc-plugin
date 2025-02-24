#[macro_export]
macro_rules! to {
    ($name:ident, $from:expr, $to:expr) => {
        swc_core::ecma::transforms::testing::test_inline!(
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
                    swc_core::ecma::visit::fold_pass($crate::LinguiMacroFolder::default()),
                )
            },
            $name,
            $from,
            $to
        );
    };

    (production, $name:ident, $from:expr, $to:expr) => {
        swc_core::ecma::transforms::testing::test_inline!(
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
                    swc_core::ecma::visit::fold_pass($crate::LinguiMacroFolder::new(
                        $crate::LinguiOptions {
                            strip_non_essential_fields: true,
                            ..Default::default()
                        },
                    )),
                )
            },
            $name,
            $from,
            $to
        );
    };
}
