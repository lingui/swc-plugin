#![deny(clippy::all)]

use lingui_extractor::{ExtractionResult, ExtractorOptions};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rayon::prelude::*;
use std::fs;
use anyhow::Result as AnyhowResult;

use swc_core::node::get_deserialized;

/// Task for extracting messages asynchronously
pub struct ExtractMessagesTask {
  source_code: String,
  filename: String,
  pub options: String,
}

impl Task for ExtractMessagesTask {
  type Output = ExtractionResult;
  type JsValue = ExtractionResult;

  fn compute(&mut self) -> Result<Self::Output> {
    let options: ExtractorOptions = get_deserialized(&self.options)?;

    let result = lingui_extractor::extract_messages(&self.source_code, &self.filename, &options)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    Ok(result)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

/// Extract messages from source code
///
/// This function parses JavaScript/TypeScript code and extracts internationalization
/// messages from lingui macro calls and components.
///
/// # Arguments
///
/// * `source_code` - The source code to analyze
/// * `filename` - The filename (used for error reporting and source maps)
///
/// # Returns
///
/// A Promise that resolves to an ExtractionResult containing:
/// * `messages` - Array of extracted messages
/// * `warnings` - Array of warning messages encountered during extraction
///
/// # Example
///
/// ```javascript
/// const result = await extractMessages(sourceCode, 'app.tsx');
/// console.log(result.messages);
/// ```
#[napi(ts_return_type = "Promise<ExtractionResult>")]
pub fn extract_messages(
  source_code: String,
  filename: String,
  options: Buffer,
) -> AsyncTask<ExtractMessagesTask> {
  let options = String::from_utf8_lossy(options.as_ref()).into_owned();

  AsyncTask::new(ExtractMessagesTask {
    source_code,
    filename,
    options,
  })
}

/// Task for extracting messages from multiple files in parallel
pub struct ExtractMessagesFromFilesTask {
  file_paths: Vec<String>,
  pub options: String,
}

impl Task for ExtractMessagesFromFilesTask {
  type Output = ExtractionResult;
  type JsValue = ExtractionResult;

  fn compute(&mut self) -> Result<Self::Output> {
    let options: ExtractorOptions = get_deserialized(&self.options)?;

    // Process files in parallel using rayon
    let results: Vec<(String, AnyhowResult<ExtractionResult>)> = self
      .file_paths
      .par_iter()
      .map(|file_path| {
        let result = fs::read_to_string(file_path)
          .map_err(|e| anyhow::anyhow!("Failed to read file '{file_path}': {e}"))
          .and_then(|source_code| {
            lingui_extractor::extract_messages(&source_code, file_path, &options)
              .map_err(|e| anyhow::anyhow!("Failed to extract from '{file_path}': {e}"))
          });

        (file_path.clone(), result)
      })
      .collect();

    // Aggregate all messages and warnings
    let mut all_messages = Vec::new();
    let mut all_warnings = Vec::new();

    for (_file_path, result) in results {
      match result {
        Ok(extraction_result) => {
          all_messages.extend(extraction_result.messages);
          all_warnings.extend(extraction_result.warnings);
        }
        Err(err) => {
          all_warnings.push(err.to_string());
        }
      }
    }

    Ok(ExtractionResult {
      messages: all_messages,
      warnings: all_warnings,
    })
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

/// Extract messages from multiple files in parallel
///
/// This function reads multiple files and extracts internationalization
/// messages from all of them in parallel using all available CPU cores.
///
/// # Arguments
///
/// * `file_paths` - Array of file paths to process
/// * `options` - Extraction options (parser configuration)
///
/// # Returns
///
/// A Promise that resolves to an ExtractionResult containing:
/// * `messages` - Array of all extracted messages from all files
/// * `warnings` - Array of warning messages (including file read errors)
///
/// # Example
///
/// ```javascript
/// const result = await extractMessagesFromFiles(['app.tsx', 'components/Header.tsx']);
/// console.log(result.messages);
/// ```
#[napi(ts_return_type = "Promise<ExtractionResult>")]
pub fn extract_messages_from_files(
  file_paths: Vec<String>,
  options: Buffer,
) -> AsyncTask<ExtractMessagesFromFilesTask> {
  let options = String::from_utf8_lossy(options.as_ref()).into_owned();

  AsyncTask::new(ExtractMessagesFromFilesTask {
    file_paths,
    options,
  })
}
