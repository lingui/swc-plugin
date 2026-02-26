import binding = require('../binding')
import type {ParserConfig} from "@swc/types"

export type ExtractorOptions = {
  parser?: ParserConfig
}

function toBuffer(t: any): Buffer {
  return Buffer.from(JSON.stringify(t));
}

/**
 * Extract messages from source code
 *
 * This function parses JavaScript/TypeScript code and extracts internationalization
 * messages from lingui macro calls and components.
 *
 * # Arguments
 *
 * * `source_code` - The source code to analyze
 * * `filename` - The filename (used for error reporting and source maps)
 *
 * # Returns
 *
 * A Promise that resolves to an ExtractionResult containing:
 * * `messages` - Array of extracted messages
 * * `warnings` - Array of warning messages encountered during extraction
 *
 * # Example
 *
 * ```javascript
 * const result = await extractMessages(sourceCode, 'app.tsx');
 * console.log(result.messages);
 * ```
 */
export function extractMessages(sourceCode: string, filename: string, options?: ExtractorOptions) {
  options = options || {
    parser: {syntax: "ecmascript"}
  };

  options!.parser!.syntax = options!.parser!.syntax || "ecmascript";

  return binding.extractMessages(sourceCode, filename, toBuffer(options))
}

/**
 * Extract messages from multiple files in parallel
 *
 * This function reads multiple files and extracts internationalization
 * messages from all of them in parallel using all available CPU cores.
 *
 * # Arguments
 *
 * * `filePaths` - Array of file paths to process
 * * `options` - Extraction options (parser configuration)
 *
 * # Returns
 *
 * A Promise that resolves to an ExtractionResult containing:
 * * `messages` - Array of all extracted messages from all files
 * * `warnings` - Array of warning messages (including file read errors)
 *
 * # Example
 *
 * ```javascript
 * const result = await extractMessagesFromFiles(['app.tsx', 'components/Header.tsx']);
 * console.log(result.messages);
 * ```
 */
export function extractMessagesFromFiles(filePaths: string[], options?: ExtractorOptions) {
  options = options || {
    parser: {syntax: "ecmascript"}
  };

  options!.parser!.syntax = options!.parser!.syntax || "ecmascript";

  return binding.extractMessagesFromFiles(filePaths, toBuffer(options))
}
