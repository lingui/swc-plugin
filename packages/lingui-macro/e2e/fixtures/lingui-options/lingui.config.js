import {defineConfig} from "@lingui/conf";

export default defineConfig({
  locales: ["en"],
  sourceLocale: "en",
  runtimeConfigModule: {
    i18n: ["@acme/core", "i18n"],
    Trans: ["@acme/react", "Trans"],
    useLingui: ["@acme/react", "useLingui"],
  },
  macro: {
    idPrefixLeader: '.',
    corePackage: ["@acme/core/macro"],
    jsxPackage: ["@acme/jsx/macro"],
    jsxPlaceholderAttribute: "_t",
    jsxPlaceholderDefaults: {
      a: "link",
    },
  },
})
