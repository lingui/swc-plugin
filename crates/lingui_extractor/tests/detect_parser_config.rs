use lingui_extractor::detect_parser_config;
use swc_core::ecma::parser::{EsSyntax, Syntax, TsSyntax};

#[test]
fn tsx_file_no_user_config() {
    let result = detect_parser_config("component.tsx", None);
    assert_eq!(
        result,
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        })
    );
}

#[test]
fn jsx_file_no_user_config() {
    let result = detect_parser_config("component.jsx", None);
    assert_eq!(
        result,
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        })
    );
}

#[test]
fn ts_file_no_user_config() {
    let result = detect_parser_config("utils.ts", None);
    assert_eq!(
        result,
        Syntax::Typescript(TsSyntax {
            tsx: false,
            ..Default::default()
        })
    );
}

#[test]
fn mts_file_no_user_config() {
    let result = detect_parser_config("utils.mts", None);
    assert_eq!(
        result,
        Syntax::Typescript(TsSyntax {
            tsx: false,
            ..Default::default()
        })
    );
}

#[test]
fn cts_file_no_user_config() {
    let result = detect_parser_config("utils.cts", None);
    assert_eq!(
        result,
        Syntax::Typescript(TsSyntax {
            tsx: false,
            ..Default::default()
        })
    );
}

#[test]
fn js_file_no_user_config() {
    let result = detect_parser_config("utils.js", None);
    assert_eq!(
        result,
        Syntax::Es(EsSyntax {
            jsx: false,
            ..Default::default()
        })
    );
}

#[test]
fn mjs_file_no_user_config() {
    let result = detect_parser_config("utils.mjs", None);
    assert_eq!(
        result,
        Syntax::Es(EsSyntax {
            jsx: false,
            ..Default::default()
        })
    );
}

#[test]
fn cjs_file_no_user_config() {
    let result = detect_parser_config("utils.cjs", None);
    assert_eq!(
        result,
        Syntax::Es(EsSyntax {
            jsx: false,
            ..Default::default()
        })
    );
}

#[test]
fn unknown_extension_defaults_to_ecmascript() {
    let result = detect_parser_config("file.svelte", None);
    assert_eq!(
        result,
        Syntax::Es(EsSyntax {
            jsx: false,
            ..Default::default()
        })
    );
}

#[test]
fn no_extension_defaults_to_ecmascript() {
    let result = detect_parser_config("Makefile", None);
    assert_eq!(
        result,
        Syntax::Es(EsSyntax {
            jsx: false,
            ..Default::default()
        })
    );
}

#[test]
fn full_path_extracts_extension() {
    let result = detect_parser_config("/src/components/App.tsx", None);
    assert_eq!(
        result,
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        })
    );
}

#[test]
fn user_typescript_on_js_file_keeps_typescript() {
    let result = detect_parser_config("utils.js", Some(Syntax::Typescript(Default::default())));
    assert_eq!(
        result,
        Syntax::Typescript(TsSyntax {
            tsx: false,
            ..Default::default()
        })
    );
}

#[test]
fn user_typescript_on_tsx_file_gets_tsx_from_detection() {
    let result = detect_parser_config("app.tsx", Some(Syntax::Typescript(Default::default())));
    assert_eq!(
        result,
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        })
    );
}

#[test]
fn user_ecmascript_on_ts_file_respects_user_choice() {
    // User explicitly passed Es — we respect their variant choice
    let result = detect_parser_config("utils.ts", Some(Syntax::Es(Default::default())));
    assert_eq!(
        result,
        Syntax::Es(EsSyntax {
            jsx: false,
            ..Default::default()
        })
    );
}

#[test]
fn user_ecmascript_on_tsx_file_gets_jsx_from_detection() {
    // User passed Es on a .tsx file — jsx gets OR'd in from detection
    let result = detect_parser_config("app.tsx", Some(Syntax::Es(Default::default())));
    assert_eq!(
        result,
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        })
    );
}

#[test]
fn user_jsx_true_on_js_file() {
    let user = Syntax::Es(EsSyntax {
        jsx: true,
        ..Default::default()
    });
    let result = detect_parser_config("component.js", Some(user));
    assert_eq!(
        result,
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        })
    );
}

#[test]
fn user_jsx_false_on_jsx_file_gets_jsx_from_detection() {
    let user = Syntax::Es(EsSyntax {
        jsx: false,
        ..Default::default()
    });
    let result = detect_parser_config("component.jsx", Some(user));
    assert_eq!(
        result,
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        })
    );
}

#[test]
fn user_tsx_false_on_tsx_file_gets_tsx_from_detection() {
    let user = Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
    });
    let result = detect_parser_config("component.tsx", Some(user));
    assert_eq!(
        result,
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        })
    );
}

#[test]
fn user_decorators_preserved_on_js_file() {
    let user = Syntax::Es(EsSyntax {
        decorators: true,
        ..Default::default()
    });
    let result = detect_parser_config("app.js", Some(user));
    assert_eq!(
        result,
        Syntax::Es(EsSyntax {
            decorators: true,
            jsx: false,
            ..Default::default()
        })
    );
}

#[test]
fn user_decorators_preserved_on_ts_file() {
    let user = Syntax::Typescript(TsSyntax {
        decorators: true,
        ..Default::default()
    });
    let result = detect_parser_config("app.ts", Some(user));
    assert_eq!(
        result,
        Syntax::Typescript(TsSyntax {
            decorators: true,
            tsx: false,
            ..Default::default()
        })
    );
}

#[test]
fn user_tsx_true_on_ts_file_stays_true() {
    let user = Syntax::Typescript(TsSyntax {
        tsx: true,
        ..Default::default()
    });
    let result = detect_parser_config("utils.ts", Some(user));
    assert_eq!(
        result,
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        })
    );
}
