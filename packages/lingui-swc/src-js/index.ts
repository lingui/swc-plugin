import binding = require('../binding')
import type {ParserConfig} from "@swc/types"
import type {ExtractorType} from "@lingui/conf"
import {LinguiMacroOptions, mapOptions} from "./macro-src/map-options"

export type {LinguiMacroOptions};

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
        onMessageExtracted({
          id: msg.id,
          origin: msg.origin
            ? [msg.origin.filename, msg.origin.line, msg.origin.column]
            : undefined,
          placeholders: msg.placeholders,
          context: msg.context,
          comment: msg.comment,
          message: msg.message,
        })
      })
    },
  }
}


