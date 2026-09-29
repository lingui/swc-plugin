import binding = require('../binding')
import type {ParserConfig} from "@swc/types"
import type {ExtractedMessage, ExtractorType} from "@lingui/conf"
import {LinguiMacroOptions, mapOptions} from "./macro-src/map-options"

export type {LinguiMacroOptions};
export {mapOptions as mapMacroOptions};

export type TransformOptions = {
  /**
   * The same options as in `jsc.parser` in `.swcrc`
   * https://swc.rs/docs/configuration/compilation#jscparser
   *
   * The syntax (ecmascript/typescript) and jsx support is automatically inferred from the filename,
   * you don't need to specify it manually
   */
  parser?: ParserConfig
  /**
   * Options for Lingui Macro
   */
  macro?: LinguiMacroOptions
  /**
   * Controls source map generation:
   * - `true` (default) — source map returned in `result.map`
   * - `"inline"` — source map appended to code as a base64 data URL, `result.map` is undefined
   * - `false` — no source map generated, `result.map` is undefined
   */
  sourceMaps?: "inline" | boolean
}

export type TransformResult = {
  code: string
  map?: string
}

/**
 * Error thrown by `transform`.
 *
 * The message is prefixed with `filename:line:column` (1-based) when the location is known,
 * e.g. `src/App.tsx:12:7: Incorrect usage of \`ph\` macro...`. Multiple errors are joined with `\n`.
 */
export type TransformError = Error & {
  /**
   * Location of the first error. `line` is 1-based, `column` is 0-based,
   * following the Rollup / Vite convention so bundlers can render a code frame from it.
   */
  loc?: {file: string; line: number; column: number}
}

/**
 * Transform source code by applying the Lingui macro transformation.
 *
 * This is a minimal SWC + Lingui transformer built as a single native library
 * for optimal performance. It transforms Lingui macros and, for TypeScript input,
 * strips TypeScript syntax: TS files come out as JS with JSX preserved.
 * Everything else is kept as-is.
 *
 * Parser options are automatically inferred from the filename (.ts, .tsx, .js, .jsx, etc.),
 * with decorators enabled. Pass `parser` to take full control of the parser config.
 *
 * `macro.descriptorFields` defaults to `"auto"`: `"id-only"` when `process.env.NODE_ENV`
 * is `"production"`, `"all"` otherwise.
 *
 * Rejects with a {@link TransformError} on parse or macro errors.
 *
 * @param code - The source code to transform
 * @param filename - The filename (used for parser inference, error messages and source maps)
 * @param options - Optional transform options
 * @returns Promise resolving to transformed code and source map
 */
export async function transform(code: string, filename: string, options?: TransformOptions): Promise<TransformResult> {
  const internalOptions = {...options, envName: process.env.NODE_ENV}

  try {
    return await binding.transform(code, filename, toBuffer(internalOptions))
  } catch (error) {
    throw withLocation(error, filename)
  }
}

/**
 * Parses the `filename:line:column: ` prefix produced by the native side
 * and exposes it as `error.loc` (1-based line, 0-based column).
 */
function withLocation(error: unknown, filename: string): unknown {
  if (!(error instanceof Error) || !error.message.startsWith(filename)) {
    return error
  }

  const match = /^:(\d+):(\d+): /.exec(error.message.slice(filename.length))
  if (match) {
    (error as TransformError).loc = {
      file: filename,
      line: Number(match[1]),
      column: Number(match[2]) - 1,
    }
  }

  return error
}

export type ExtractorOptions = {
  /**
   * The same options as in `jsc.parser` in `.swcrc`
   * https://swc.rs/docs/configuration/compilation#jscparser
   *
   * The syntax (ecmascript/typescript) and jsx support is automatically inferred from the filename,
   * you don't need to specify it manually
   */
  parser?: ParserConfig
  /**
   * Options for Lingui Macro
   *
   * Except of `descriptorFields` property which is always set to `All` in extraction
   */
  macro?: Omit<LinguiMacroOptions, 'descriptorFields'>
}

function toBuffer(t: any): Buffer {
  return Buffer.from(JSON.stringify(t));
}

export function extractMessages(sourceCode: string, filename: string, options?: ExtractorOptions) {
  return binding.extractMessages(sourceCode, filename, toBuffer(options || {}))
}

export function extractMessagesFromFiles(filePaths: string[], options?: ExtractorOptions) {
  return binding.extractMessagesFromFiles(filePaths, toBuffer(options || {}))
}

const mapMessage = (msg: binding.ExtractedMessage): ExtractedMessage => {
  return {
    id: msg.id,
    origin: msg.origin
      ? [msg.origin.filename, msg.origin.line, msg.origin.column]
      : undefined,
    placeholders: msg.placeholders,
    context: msg.context,
    comment: msg.comment,
    message: msg.message,
  };
}

/**
 * Creates pluggable SWC Lingui Extractor implementation.
 *
 * Example:
 *
 * ```ts
 * // lingui.config.ts
 * defineConfig({
 *    extractors: [createSwcExtractor()],
 * })
 * ```
 *
 * Macro options automatically inherited from the Lingui Config.
 */
export function createSwcExtractor(options: ExtractorOptions = {}): ExtractorType {
  const matchRe = new RegExp(
    "\\.(" +
    [".ts", ".mts", ".cts", ".tsx", ".js", ".mjs", ".cjs", ".jsx"]
      .map((ext) => ext.slice(1))
      .join("|") +
    ")$",
    "i",
  )

  return {
    match(filename) {
      return matchRe.test(filename)
    },

    async extract(filename, code, onMessageExtracted, ctx) {
      const {messages} = await extractMessages(code, filename, {
        ...options,
        macro: mapOptions(ctx.linguiConfig)
      })

      messages.forEach((msg) => {
        onMessageExtracted(mapMessage(msg))
      })
    },
  }
}


