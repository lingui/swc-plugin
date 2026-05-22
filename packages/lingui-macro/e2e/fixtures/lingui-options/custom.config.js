export default {
  locales: ["en"],
  sourceLocale: "en",
  runtimeConfigModule: {
    i18n: ["@custom/core", "customI18n"],
    Trans: ["@custom/react", "CustomTrans"],
    useLingui: ["@custom/react", "useCustomLingui"],
  },
  macro: {
    jsxPlaceholderAttribute: "data-i18n",
    jsxPlaceholderDefaults: {
      a: "anchor",
      strong: "bold",
    },
  },
}
