import { getConfig } from "@lingui/conf"

export function linguiSwcOptions(overrides, linguiConfigPath) {
  const config = getConfig(
    linguiConfigPath ? { configPath: linguiConfigPath } : {},
  )
  const { i18n, Trans, useLingui } = config.runtimeConfigModule

  return {
    jsxPlaceholderAttribute: config.macro.jsxPlaceholderAttribute,
    jsxPlaceholderDefaults: config.macro.jsxPlaceholderDefaults,
    ...overrides,
    runtimeModules: {
      i18n,
      trans: Trans,
      useLingui,
      ...overrides?.runtimeModules,
    },
  }
}
