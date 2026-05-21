import type { LinguiConfigNormalized } from "@lingui/conf"

/** SWC plugin options derived from the shared Lingui config. */
export type LinguiSwcOptions = {
  jsxPlaceholderAttribute?: string
  jsxPlaceholderDefaults?: Record<string, string>
  runtimeModules: {
    i18n: LinguiConfigNormalized["runtimeConfigModule"]["i18n"]
    trans: LinguiConfigNormalized["runtimeConfigModule"]["Trans"]
    useLingui: LinguiConfigNormalized["runtimeConfigModule"]["useLingui"]
  }
}

/** Recursively marks all properties in `T` as optional. */
export type DeepPartial<T> = {
  [Key in keyof T]?: T[Key] extends readonly unknown[]
    ? T[Key]
    : T[Key] extends object
      ? DeepPartial<T[Key]>
      : T[Key]
}

/**
 * Load a Lingui config file and map its shared options to the SWC plugin format.
 *
 * `overrides` are merged over the mapped config. Omit `linguiConfigPath` to use
 * the default Lingui config discovery logic.
 */
export declare function linguiSwcOptions(
  overrides?: DeepPartial<LinguiSwcOptions>,
  linguiConfigPath?: string,
): LinguiSwcOptions
