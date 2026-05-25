import {defineConfig} from "@lingui/conf";

export default defineConfig({
  locales: ["en"],
  sourceLocale: "en",
  runtimeConfigModule: {
    i18n: ["@custom/core", "customI18n"],
    Trans: ["@custom/react", "CustomTrans"],
    useLingui: ["@custom/react", "useCustomLingui"],
  },
  macro: {
    idPrefixLeader: '.',
    corePackage: ["@custom/core/macro"],
    jsxPackage: ["@custom/react/macro"],
    jsxPlaceholderAttribute: "data-i18n",
    jsxPlaceholderDefaults: {
      a: "anchor",
      strong: "bold",
    },
  },
})
