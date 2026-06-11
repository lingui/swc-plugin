import type {LinguiConfigNormalized} from "@lingui/conf" with {"resolution-mode": "import"}

export type RuntimeModuleConfig = readonly [modulePath: string, exportName?: string];

/** Options accepted by the `@lingui/swc-plugin` WASM plugin. */
export type LinguiMacroOptions = {
  /** Module specifiers treated as core macro imports, such as `t`, `msg`, and `defineMessage`. */
  corePackage?: string[]
  /** Module specifiers treated as JSX macro imports, such as `Trans` and `useLingui`. */
  jsxPackage?: string[]
  /** JSX attribute name used to provide explicit placeholder names inside `<Trans>` content. */
  jsxPlaceholderAttribute?: string
  /** Default placeholder names for JSX tags when no explicit placeholder attribute is present. */
  jsxPlaceholderDefaults?: Record<string, string>
  /** Overrides the runtime imports used by the plugin. Unlike the Babel macro configuration, must be passed as an object. */
  runtimeModules: {
    i18n: RuntimeModuleConfig
    Trans: RuntimeModuleConfig
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
  /**
   * Emits the extraction marker as a JSDoc comment (`/** i18n *\/`) instead of a regular block comment (`/* i18n *\/`).
   * JSDoc comments survive bundler code generation (e.g. Rolldown), ensuring the extractor can still locate messages in bundled output.
   *
   * - `false` (default) — `/* i18n *\/` (compatible with all `@lingui/cli` versions).
   * - `true` — `/** i18n *\/` (requires `@lingui/cli` >= 6.4.0).
   */
  useJsdocI18nComment?: boolean
}

/** Makes all properties in `T` optional, recursing into nested objects but preserving tuples/arrays as-is. */
export type DeepPartial<T> = {
  [Key in keyof T]?: T[Key] extends readonly unknown[]
    ? T[Key]
    : T[Key] extends object
      ? DeepPartial<T[Key]>
      : T[Key]
}

/**
 * Maps relevant options from the Lingui config to the SWC plugin format
 */
export function mapOptions(linguiConfig: LinguiConfigNormalized, overrides?: DeepPartial<LinguiMacroOptions>): LinguiMacroOptions {
  return {
    corePackage: linguiConfig.macro.corePackage,
    jsxPackage: linguiConfig.macro.jsxPackage,
    jsxPlaceholderAttribute: linguiConfig.macro.jsxPlaceholderAttribute,
    jsxPlaceholderDefaults: linguiConfig.macro.jsxPlaceholderDefaults,
    idPrefixLeader: linguiConfig.macro.idPrefixLeader,
    ...overrides,
    runtimeModules: {
      ...linguiConfig.runtimeConfigModule,
      ...overrides?.runtimeModules,
    },
  }
}
