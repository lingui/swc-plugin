import {defineConfig} from "@lingui/conf";

export default defineConfig({
  locales: ["en"],
  sourceLocale: "en",
  runtimeConfigModule: {
    Trans: ["@lingui/solid", "Trans"],
  },
  macro: {
    jsxRuntime: "solid",
  },
})
