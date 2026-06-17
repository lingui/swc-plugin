use data_encoding::BASE64;
use lingui_extractor::detect_parser_config;
use lingui_macro::{LinguiJsOptions, LinguiMacroFolder, LinguiOptions};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::Deserialize;
use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::errors::{DiagnosticBuilder, Handler, HandlerFlags};
use swc_core::common::{sync::Lrc, BytePos, LineCol};
use swc_core::common::{FileName, Globals, Mark, SourceMap, GLOBALS};
use swc_core::ecma::codegen::text_writer::JsWriter;
use swc_core::ecma::codegen::Emitter;
use swc_core::ecma::parser::{Parser, StringInput, Syntax};
use swc_core::ecma::transforms::base::{fixer, hygiene, resolver};
use swc_core::ecma::visit::fold_pass;
use swc_sourcemap as sourcemap;

use std::sync::{Arc, Mutex};

struct StringEmitter {
  buffer: Arc<Mutex<String>>,
}

impl swc_core::common::errors::Emitter for StringEmitter {
  fn emit(&mut self, db: &mut DiagnosticBuilder<'_>) {
    let msg: String = db
      .message
      .iter()
      .map(|m| m.0.as_str())
      .collect::<Vec<_>>()
      .join("");
    let mut buf = self.buffer.lock().unwrap();
    if !buf.is_empty() {
      buf.push('\n');
    }
    buf.push_str(&msg);
  }
}

#[napi(object)]
pub struct TransformResult {
  pub code: String,
  pub map: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum SourceMapsOption {
  Bool(bool),
  Str(String),
}

impl Default for SourceMapsOption {
  fn default() -> Self {
    SourceMapsOption::Bool(true)
  }
}

#[derive(Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformOptionsInternal {
  pub parser: Option<Syntax>,
  #[serde(default, rename = "macro")]
  pub macro_options: Option<LinguiJsOptions>,
  #[serde(default)]
  pub source_maps: SourceMapsOption,
}

fn extract_inline_sourcemap(source_code: &str) -> Option<sourcemap::SourceMap> {
  let source_mapping_prefix = "sourceMappingURL=";

  if let Some(idx) = source_code.rfind(source_mapping_prefix) {
    let url_part = &source_code[idx + source_mapping_prefix.len()..];
    let url = url_part.lines().next().unwrap_or(url_part).trim();

    if url.starts_with("data:application/json;base64,") {
      if let Some(base64_start) = url.find("base64,") {
        let base64_content = &url[base64_start + "base64,".len()..].trim();

        if let Ok(decoded) = BASE64.decode(base64_content.as_bytes()) {
          if let Ok(source_map) = sourcemap::SourceMap::from_slice(&decoded) {
            return Some(source_map);
          }
        }
      }
    }
  }

  None
}

fn do_transform(
  code: &str,
  filename: &str,
  parser_syntax: Option<Syntax>,
  macro_options: Option<LinguiJsOptions>,
  input_source_map: Option<sourcemap::SourceMap>,
  source_maps: &SourceMapsOption,
) -> std::result::Result<TransformResult, String> {
  let error_buffer = Arc::new(Mutex::new(String::new()));
  let cm: Lrc<SourceMap> = Default::default();
  let comments = SingleThreadedComments::default();

  let handler = Handler::with_emitter_and_flags(
    Box::new(StringEmitter {
      buffer: error_buffer.clone(),
    }),
    HandlerFlags {
      can_emit_warnings: true,
      ..Default::default()
    },
  );

  let syntax = detect_parser_config(filename, parser_syntax);

  let source_file = cm.new_source_file(Lrc::new(FileName::Real(filename.into())), code.to_string());

  let mut parser = Parser::new(syntax, StringInput::from(&*source_file), Some(&comments));

  let module = parser
    .parse_module()
    .map_err(|e| format!("Parse error: {e:?}"))?;

  let program = swc_core::ecma::ast::Program::Module(module);

  let lingui_options = match macro_options {
    Some(opts) => opts.into_options(""),
    None => LinguiOptions::default(),
  };

  let globals = Globals::default();

  let result = GLOBALS.set(&globals, || {
    swc_core::common::errors::HANDLER.set(&handler, || {
      let unresolved_mark = Mark::new();
      let top_level_mark = Mark::new();

      let lingui_macro = LinguiMacroFolder::new(lingui_options, Some(&comments), cm.clone());

      let program = program
        .apply(&mut resolver(
          unresolved_mark,
          top_level_mark,
          syntax.typescript(),
        ))
        .apply(fold_pass(lingui_macro))
        .apply(hygiene::hygiene())
        .apply(fixer::fixer(Some(&comments)));

      let generate_maps = !matches!(source_maps, SourceMapsOption::Bool(false));
      let inline_maps = matches!(source_maps, SourceMapsOption::Str(ref s) if s == "inline");

      let mut buf = Vec::new();
      let mut src_map_buf: Vec<(BytePos, LineCol)> = Vec::new();

      {
        let mut emitter = Emitter {
          cfg: Default::default(),
          cm: cm.clone(),
          comments: Some(&comments),
          wr: JsWriter::new(
            cm.clone(),
            "\n",
            &mut buf,
            if generate_maps { Some(&mut src_map_buf) } else { None },
          ),
        };
        emitter
          .emit_program(&program)
          .map_err(|e| format!("Emit error: {e:?}"))?;
      }

      let mut output_code = String::from_utf8(buf).map_err(|e| format!("UTF-8 error: {e:?}"))?;

      if !generate_maps {
        return Ok(TransformResult {
          code: output_code,
          map: None,
        });
      }

      let output_source_map = cm.build_source_map(
        &src_map_buf,
        input_source_map,
        CustomSourceMapConfig { filename },
      );

      if inline_maps {
        let data_url = output_source_map
          .to_data_url()
          .map_err(|e| format!("Source map encode error: {e:?}"))?;
        output_code.push_str("\n//# sourceMappingURL=");
        output_code.push_str(&data_url);
        output_code.push('\n');

        Ok(TransformResult {
          code: output_code,
          map: None,
        })
      } else {
        let mut map_buf = Vec::new();
        output_source_map
          .to_writer(&mut map_buf)
          .map_err(|e| format!("Source map write error: {e:?}"))?;

        let map_string =
          String::from_utf8(map_buf).map_err(|e| format!("Source map UTF-8 error: {e:?}"))?;

        Ok(TransformResult {
          code: output_code,
          map: Some(map_string),
        })
      }
    })
  });

  let errors = error_buffer.lock().unwrap().clone();
  if !errors.is_empty() {
    return Err(errors);
  }

  result
}

struct CustomSourceMapConfig<'a> {
  filename: &'a str,
}

impl swc_core::common::source_map::SourceMapGenConfig for CustomSourceMapConfig<'_> {
  fn file_name_to_source(&self, _f: &FileName) -> String {
    self.filename.to_string()
  }

  fn inline_sources_content(&self, _f: &FileName) -> bool {
    true
  }
}

pub struct TransformTask {
  code: String,
  filename: String,
  options: String,
}

impl Task for TransformTask {
  type Output = TransformResult;
  type JsValue = TransformResult;

  fn compute(&mut self) -> Result<Self::Output> {
    let options: TransformOptionsInternal = if self.options.is_empty() {
      TransformOptionsInternal::default()
    } else {
      serde_json::from_str(&self.options)
        .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid options: {e}")))?
    };

    let input_source_map = if matches!(options.source_maps, SourceMapsOption::Bool(false)) {
      None
    } else {
      extract_inline_sourcemap(&self.code)
    };

    do_transform(
      &self.code,
      &self.filename,
      options.parser,
      options.macro_options,
      input_source_map,
      &options.source_maps,
    )
    .map_err(|e| Error::new(Status::GenericFailure, e))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

#[napi(ts_return_type = "Promise<TransformResult>")]
pub fn transform(
  code: String,
  filename: String,
  options: Option<Buffer>,
) -> AsyncTask<TransformTask> {
  let options = options
    .map(|buf| String::from_utf8_lossy(buf.as_ref()).into_owned())
    .unwrap_or_default();

  AsyncTask::new(TransformTask {
    code,
    filename,
    options,
  })
}
