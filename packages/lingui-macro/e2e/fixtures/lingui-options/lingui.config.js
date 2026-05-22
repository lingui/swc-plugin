export default {
  locales: ["en"],
  sourceLocale: "en",
  runtimeConfigModule: {
    i18n: ["@acme/core", "i18n"],
    Trans: ["@acme/react", "Trans"],
    useLingui: ["@acme/react", "useLingui"],
  },
  macro: {
    jsxPlaceholderAttribute: "_t",
    jsxPlaceholderDefaults: {
      a: "link",
    },
  },
}
