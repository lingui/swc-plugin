import binding = require('../binding')
import type {ParserConfig} from "@swc/types"
import type {LinguiMacroOptions} from "@lingui/swc-plugin/types"

export type ExtractorOptions = {
  parser?: ParserConfig
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
