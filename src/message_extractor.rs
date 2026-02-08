use crate::message_extractor_visitor::{ExtractionResult, MessageExtractorVisitor};
use crate::{LinguiMacroFolder, LinguiOptions};
use std::sync::Arc;
// use once_cell::sync::Lazy;
// use swc::Compiler;
// use swc_core::common::{sync::Lazy, FilePathMapping};
use swc_core::common::{Globals, Mark, GLOBALS};
use swc_core::ecma::transforms::base::resolver;
use swc_core::{
    common::{
        comments::{Comments, SingleThreadedComments},
        sync::Lrc,
        FileName, SourceMap,
    },
    ecma::{
        ast::*,
        parser::{EsSyntax, Parser, StringInput, Syntax},
        visit::VisitWith,
    },
};

/// Extract messages from source code
pub fn extract_messages(
    source_code: &str,
    filename: &str,
) -> Result<ExtractionResult, Box<dyn std::error::Error>> {
    // Set up parser with JSX support
    let syntax = Syntax::Es(EsSyntax {
        jsx: true,
        ..Default::default()
    });

    let source_map = Lrc::new(SourceMap::default());
    // let source_file = source_map.new_source_file(FileName::Anon, src.into());

    let source_file = source_map.new_source_file(
        Arc::new(FileName::Custom(filename.to_string())),
        source_code.to_string(),
    );

    let comments = Lrc::new(SingleThreadedComments::default());

    let mut parser = Parser::new(syntax, StringInput::from(&*source_file), Some(&comments));

    // let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    let program = Program::Module(module);

    // let res = p
    //   .parse_module()
    //   .map_err(|e| e.into_diagnostic(tester.handler).emit());
    //
    // for e in p.take_errors() {
    //   e.into_diagnostic(tester.handler).emit()
    // }

    // Extract messages directly from parsed module
    let mut extractor_visitor = MessageExtractorVisitor::new(
        source_map.clone(),
        &comments as &dyn Comments,
        source_code.to_string(),
        filename.to_string(),
    );

    let lingui_macro = LinguiMacroFolder::new(LinguiOptions {
        strip_non_essential_fields: false,
        ..Default::default()
    });

    let globals = Globals::default();

    GLOBALS.set(&globals, || {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        program
            .apply(&mut resolver(unresolved_mark, top_level_mark, false))
            .apply(swc_core::ecma::visit::fold_pass(lingui_macro))
            .visit_with(&mut extractor_visitor);
    });

    Ok(ExtractionResult {
        messages: extractor_visitor.messages,
        warnings: extractor_visitor.warnings,
    })
}
