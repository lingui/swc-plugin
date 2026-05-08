use std::io::Write;
use std::sync::{Arc, Mutex};

use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::errors::{EmitterWriter, Handler, HANDLER};
use swc_core::common::sync::Lrc;
use swc_core::common::{FileName, Globals, Mark, SourceMap, GLOBALS};
use swc_core::ecma::ast::Pass;
use swc_core::ecma::codegen::to_code_default;
use swc_core::ecma::parser::{Parser, StringInput, Syntax, TsSyntax};
use swc_core::ecma::transforms::base::{fixer, hygiene, resolver};

struct SharedWriter(Arc<Mutex<Vec<u8>>>);

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

pub fn transform<P: Pass>(
    input: &str,
    transform_cb: impl FnOnce(&SingleThreadedComments) -> P,
) -> Result<String, String> {
    let error_buffer = Arc::new(Mutex::new(Vec::new()));
    let cm: Lrc<SourceMap> = Default::default();
    let comments = SingleThreadedComments::default();

    let emitter = EmitterWriter::new(
        Box::new(SharedWriter(error_buffer.clone())),
        Some(cm.clone()),
        false,
        false,
    );

    let handler = Handler::with_emitter(true, false, Box::new(emitter));

    let syntax = Syntax::Typescript(TsSyntax {
        tsx: true,
        ..Default::default()
    });

    let output = GLOBALS.set(&Globals::new(), || {
        HANDLER.set(&handler, || {
            let fm =
                cm.new_source_file(FileName::Real("input.tsx".into()).into(), input.to_string());

            let mut parser = Parser::new(syntax, StringInput::from(&*fm), Some(&comments));

            let program = match parser.parse_program() {
                Ok(program) => program,
                Err(e) => {
                    e.into_diagnostic(&handler).emit();
                    for e in parser.take_errors() {
                        e.into_diagnostic(&handler).emit();
                    }
                    return None;
                }
            };
            for e in parser.take_errors() {
                e.into_diagnostic(&handler).emit();
            }

            let program = program
                .apply(resolver(Mark::new(), Mark::new(), true))
                .apply(transform_cb(&comments))
                .apply(hygiene::hygiene())
                .apply(fixer::fixer(Some(&comments)));

            Some(to_code_default(cm.clone(), Some(&comments), &program))
        })
    });

    let errors = error_buffer.lock().unwrap().clone();
    if !errors.is_empty() {
        Err(String::from_utf8(errors).expect("Error output is not valid UTF-8"))
    } else {
        Ok(output.expect("Transform produced no output and no errors"))
    }
}

pub fn dedent(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();

    // Skip empty first/last lines (from r#" ... "# formatting)
    let start = if lines.first().is_some_and(|l| l.trim().is_empty()) {
        1
    } else {
        0
    };
    let end = if lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.len() - 1
    } else {
        lines.len()
    };

    if start >= end {
        return String::new();
    }

    let lines = &lines[start..end];

    let min_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    lines
        .iter()
        .map(|l| {
            if l.len() >= min_indent {
                &l[min_indent..]
            } else {
                l.trim()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[macro_export]
macro_rules! to {
    ($name:ident, $input:expr) => {
        #[test]
        fn $name() {
            let source = common::dedent($input);
            let output = common::transform(source.as_str(), |comments| {
                swc_core::ecma::visit::fold_pass(lingui_macro_plugin::LinguiMacroFolder::new(
                    Default::default(),
                    Some(comments.clone()),
                ))
            })
            .expect("Transform produced unexpected errors");
            insta::with_settings!({
                omit_expression => true,
            }, {
                insta::assert_snapshot!(format!("{}\n\n↓ ↓ ↓ ↓ ↓ ↓\n\n{}", source, output));
            });
        }
    };
    ($name:ident, $options:expr, $input:expr) => {
        #[test]
        fn $name() {
            let options: lingui_macro_plugin::LinguiOptions = $options;
            let source = common::dedent($input);

            let output = common::transform(source.as_str(), |comments| {
                swc_core::ecma::visit::fold_pass(lingui_macro_plugin::LinguiMacroFolder::new(
                    options.clone(),
                    Some(comments.clone()),
                ))
            })
            .expect("Transform produced unexpected errors");
            insta::with_settings!({
                info => &options,
                omit_expression => true,
            }, {
                insta::assert_snapshot!(format!("{}\n\n↓ ↓ ↓ ↓ ↓ ↓\n\n{}", source, output));
            });
        }
    };
}

#[macro_export]
macro_rules! to_panic {
    ($name:ident, $options:expr, $input:expr) => {
        #[test]
        fn $name() {
            let options: lingui_macro_plugin::LinguiOptions = $options;
            let source = common::dedent($input);
            let err = common::transform(source.as_str(), |comments| {
                swc_core::ecma::visit::fold_pass(lingui_macro_plugin::LinguiMacroFolder::new(
                    options.clone(),
                    Some(comments.clone()),
                ))
            })
            .expect_err("Expected transform to produce an error, but it succeeded");
            insta::with_settings!({
                info => &options,
                omit_expression => true,
            }, {
                insta::assert_snapshot!(format!("{}\n\n↓ ↓ ↓ ↓ ↓ ↓\n\n{}", source, err));
            });
        }
    };
}
