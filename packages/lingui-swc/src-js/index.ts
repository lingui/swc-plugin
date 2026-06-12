import binding = require('../binding')
import type {ParserConfig} from "@swc/types"
import type {ExtractedMessage, ExtractorCtx, ExtractorType} from "@lingui/conf"
import {LinguiMacroOptions, mapOptions} from "./macro-src/map-options"

export type {LinguiMacroOptions};

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
   * External source map JSON string. If not provided, inline source maps in the code are used.
   */
  sourceMap?: string
}

export type TransformResult = {
  code: string
  map?: string
}

/**
 * Transform source code by applying the Lingui macro transformation.
 *
 * This is a minimal SWC + Lingui transformer built as a single native library
 * for optimal performance. It only transforms Lingui macros and keeps everything
 * else as-is.
 *
 * Parser options are automatically inferred from the filename (.ts, .tsx, .js, .jsx, etc.)
 *
 * @param code - The source code to transform
 * @param filename - The filename (used for parser inference and source maps)
 * @param options - Optional transform options
 * @returns Promise resolving to transformed code and source map
 */
export function transform(code: string, filename: string, options?: TransformOptions): Promise<TransformResult> {
  return binding.transform(code, filename, options ? toBuffer(options) : undefined)
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
export function createSwcExtractor(options: ExtractorOptions = {}): ExtractorType & {
  extractFromFiles: (
    filenames: string[],
    onMessageExtracted: (msg: ExtractedMessage) => void,
    ctx: ExtractorCtx,
  ) => Promise<void>
} {
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

    async extractFromFiles(filenames: string[],
                           onMessageExtracted: (msg: ExtractedMessage) => void,
                           ctx: ExtractorCtx) {
      const {messages} = await extractMessagesFromFiles(filenames, {
        ...options,
        macro: mapOptions(ctx.linguiConfig)
      })

      messages.forEach((msg) => {
        onMessageExtracted(mapMessage(msg))
      })
    }
  }
}


