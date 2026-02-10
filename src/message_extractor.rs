use crate::message_extractor_visitor::{ExtractionResult, MessageExtractorVisitor};
use crate::{LinguiMacroFolder, LinguiOptions};
use data_encoding::BASE64;
use std::sync::Arc;

use swc_sourcemap as sourcemap;

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
        parser::{Parser, StringInput, Syntax, TsSyntax},
        visit::VisitWith,
    },
};

// static COMPILER: Lazy<Arc<Compiler>> = Lazy::new(|| {
//     let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
//
//     Arc::new(Compiler::new(cm))
// });
//
// fn get_compiler() -> Arc<Compiler> {
//   COMPILER.clone()
// }

/// Extract inline source map from source code
/// Looks for sourceMappingURL comments with inline base64 data
fn extract_inline_sourcemap(source_code: &str) -> Option<sourcemap::SourceMap> {
    // get_compiler().transform()
    // Look for sourceMappingURL comment (typically at the end of the file)
    let source_mapping_prefix = "sourceMappingURL=";

    // Search for the comment in the source code
    if let Some(idx) = source_code.rfind(source_mapping_prefix) {
        let url_part = &source_code[idx + source_mapping_prefix.len()..];

        // Find the end of the URL (typically end of line or end of file)
        let url = url_part.lines().next().unwrap_or(url_part).trim();

        // Check if it's an inline base64 data URL
        if url.starts_with("data:application/json;base64,") {
            // Extract the base64 content
            if let Some(base64_start) = url.find("base64,") {
                let base64_content = &url[base64_start + "base64,".len()..].trim();

                // Decode base64
                if let Ok(decoded) = BASE64.decode(base64_content.as_bytes()) {
                    // Parse the source map
                    if let Ok(source_map) = sourcemap::SourceMap::from_slice(&decoded) {
                        return Some(source_map);
                    }
                }
            }
        }
    }

    None
}

/// Extract messages from source code
pub fn extract_messages(
    source_code: &str,
    filename: &str,
) -> Result<ExtractionResult, Box<dyn std::error::Error>> {
    // Set up parser with JSX support
    let syntax = Syntax::Typescript(TsSyntax {
        tsx: true,
        ..Default::default()
    });

    // let fm = c.cm.new_source_file(filename.into(), src);

    let source_map = Lrc::new(SourceMap::default());
    // source_map.new_source_file()
    // let source_file = source_map.new_source_file(FileName::Anon, src.into());

    let source_file = source_map.new_source_file(
        Arc::new(FileName::Custom(filename.to_string())),
        source_code.to_string(),
    );

    let comments = Lrc::new(SingleThreadedComments::default());

    let mut parser = Parser::new(syntax, StringInput::from(&*source_file), Some(&comments));

    let module = parser
        .parse_module()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    let program = Program::Module(module);

    // Extract inline source map if present
    let inline_source_map = extract_inline_sourcemap(source_code);

    // Extract messages directly from parsed module
    let mut extractor_visitor = MessageExtractorVisitor::new(
        source_map.clone(),
        &comments as &dyn Comments,
        filename.to_string(),
        inline_source_map,
    );

    let lingui_macro = LinguiMacroFolder::new(
        LinguiOptions {
            strip_non_essential_fields: false,
            ..Default::default()
        },
        Some(&comments as &dyn Comments),
    );

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
