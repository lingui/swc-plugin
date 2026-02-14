#![deny(clippy::all)]

use lingui_extractor::ExtractionResult;
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Task for extracting messages asynchronously
pub struct ExtractMessagesTask {
  source_code: String,
  filename: String,
}

impl Task for ExtractMessagesTask {
  type Output = ExtractionResult;
  type JsValue = ExtractionResult;

  fn compute(&mut self) -> Result<Self::Output> {
    let result = lingui_extractor::extract_messages(&self.source_code, &self.filename)
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
pub fn extract_messages(source_code: String, filename: String) -> AsyncTask<ExtractMessagesTask> {
  AsyncTask::new(ExtractMessagesTask {
    source_code,
    filename,
  })
}
