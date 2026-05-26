import {getConfig} from "@lingui/conf"
import {mapOptions, DeepPartial, LinguiMacroOptions} from "./map-options"

export type {RuntimeModuleConfig, LinguiMacroOptions} from "./map-options"
export {mapOptions} from "./map-options"

/** Controls how the Lingui config is located and loaded. */
export type GetConfigOptions = {
  /** Working directory for config discovery. Defaults to `process.cwd()`. */
  cwd?: string
  /** Explicit path to a Lingui config file, bypassing discovery. */
  configPath?: string
  /** Skip schema validation of the loaded config. */
  skipValidation?: boolean
}

/**
 * Loads the Lingui config, maps relevant options to the SWC plugin format,
 * and returns a ready-to-use `["@lingui/swc-plugin", options]` tuple.
 *
 * @example
 * ```js
 * // next.config.js
 * const nextConfig = {
 *   experimental: {
 *     swcPlugins: [linguiMacroSwcPlugin()],
 *   },
 * };
 * ```
 *
 * @param overrides - Plugin options merged over values derived from the Lingui config.
 * @param configOptions - Controls how the Lingui config is discovered or loaded.
 */
export function linguiMacroSwcPlugin(overrides?: DeepPartial<LinguiMacroOptions>, configOptions: GetConfigOptions = {}): [wasmPackage: string, config: LinguiMacroOptions] {
  const config = getConfig(
    configOptions,
  )

  return ["@lingui/swc-plugin", mapOptions(config, overrides)];
}
