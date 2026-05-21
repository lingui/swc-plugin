import {getConfig} from "@lingui/conf"

export type RuntimeModuleConfig = readonly [modulePath: string, exportName?: string];

/** Options accepted by the `@lingui/swc-plugin` WASM plugin. */
export type LinguiMacroOptions = {
  jsxPlaceholderAttribute?: string
  jsxPlaceholderDefaults?: Record<string, string>
  runtimeModules: {
    i18n: RuntimeModuleConfig
    trans: RuntimeModuleConfig
    useLingui: RuntimeModuleConfig
  }
  useLinguiV5IdGeneration?: boolean
  descriptorFields?: 'auto' | 'all' | 'id-only' | 'message'
  idPrefixLeader?: string
}

/** Makes all properties in `T` optional, recursing into nested objects but preserving tuples/arrays as-is. */
export type DeepPartial<T> = {
  [Key in keyof T]?: T[Key] extends readonly unknown[]
    ? T[Key]
    : T[Key] extends object
      ? DeepPartial<T[Key]>
      : T[Key]
}

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
export function linguiMacroSwcPlugin(overrides?: DeepPartial<LinguiMacroOptions>, configOptions: GetConfigOptions = {}) {
  const config = getConfig(
    configOptions,
  )
  const {i18n, Trans, useLingui} = config.runtimeConfigModule

  const macroOptions: LinguiMacroOptions = {
    jsxPlaceholderAttribute: config.macro.jsxPlaceholderAttribute,
    jsxPlaceholderDefaults: config.macro.jsxPlaceholderDefaults,
    ...overrides,
    runtimeModules: {
      i18n,
      trans: Trans,
      useLingui,
      ...overrides?.runtimeModules,
    },
  } satisfies LinguiMacroOptions

  return ["@lingui/swc-plugin", macroOptions];
}
