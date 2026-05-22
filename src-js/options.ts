import {getConfig} from "@lingui/conf"

export type RuntimeModuleConfig = readonly [modulePath: string, exportName?: string];

/** Options accepted by the `@lingui/swc-plugin` WASM plugin. */
export type LinguiMacroOptions = {
  /** JSX attribute name used to provide explicit placeholder names inside `<Trans>` content. */
  jsxPlaceholderAttribute?: string
  /** Default placeholder names for JSX tags when no explicit placeholder attribute is present. */
  jsxPlaceholderDefaults?: Record<string, string>
  /** Overrides the runtime imports used by the plugin. Unlike the Babel macro configuration, must be passed as an object. */
  runtimeModules: {
    i18n: RuntimeModuleConfig
    trans: RuntimeModuleConfig
    useLingui: RuntimeModuleConfig
  }
  /**
   * Compatibility option for using the v6 SWC plugin with `@lingui/cli@5.*`.
   * - `false` (default) — URL-safe Base64 alphabet (Lingui v6).
   * - `true` — Standard Base64 alphabet (Lingui v5).
   *
   * Temporary — will be removed in the next major release.
   */
  useLinguiV5IdGeneration?: boolean
  /**
   * Controls which descriptor fields are preserved in output.
   * - `"auto"` (default) — `"id-only"` in production, `"all"` otherwise.
   * - `"all"` — Keeps id, message, context, and comment.
   * - `"id-only"` — Keeps only id. Most optimized for production bundles.
   * - `"message"` — Keeps id, message, and context (not comment).
   */
  descriptorFields?: 'auto' | 'all' | 'id-only' | 'message'
  /** Restricts directive-based `idPrefix` to explicit ids starting with this leader string. When omitted, `idPrefix` is prepended to all explicit static ids. */
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
