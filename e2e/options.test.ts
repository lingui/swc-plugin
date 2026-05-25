import {describe, expect, it} from "vitest"
import {resolve} from "node:path"

import {linguiMacroSwcPlugin} from "../src-js/options"

const fixturesDir = resolve(import.meta.dirname, "fixtures/lingui-options")

describe("linguiMacroSwcPlugin", () => {
  it("discovers the default Lingui config from cwd", () => {
    const previousCwd = process.cwd()

    try {
      process.chdir(fixturesDir)

      expect(linguiMacroSwcPlugin()).toMatchInlineSnapshot(`
        [
          "@lingui/swc-plugin",
          {
            "corePackage": [
              "@lingui/core/macro",
            ],
            "jsxPackage": [
              "@lingui/react/macro",
            ],
            "jsxPlaceholderAttribute": "_t",
            "jsxPlaceholderDefaults": {
              "a": "link",
            },
            "runtimeModules": {
              "Trans": [
                "@acme/react",
                "Trans",
              ],
              "i18n": [
                "@acme/core",
                "i18n",
              ],
              "useLingui": [
                "@acme/react",
                "useLingui",
              ],
            },
          },
        ]
      `)
    } finally {
      process.chdir(previousCwd)
    }
  })

  it("maps shared options from an explicit config path", () => {
    expect(linguiMacroSwcPlugin({}, {configPath: resolve(fixturesDir, "custom.config.js")})).toMatchInlineSnapshot(
      `
      [
        "@lingui/swc-plugin",
        {
          "corePackage": [
            "@custom/core/macro",
          ],
          "jsxPackage": [
            "@custom/react/macro",
          ],
          "jsxPlaceholderAttribute": "data-i18n",
          "jsxPlaceholderDefaults": {
            "a": "anchor",
            "strong": "bold",
          },
          "runtimeModules": {
            "Trans": [
              "@custom/react",
              "CustomTrans",
            ],
            "i18n": [
              "@custom/core",
              "customI18n",
            ],
            "useLingui": [
              "@custom/react",
              "useCustomLingui",
            ],
          },
        },
      ]
    `)
  })

  it("merges overrides over mapped config", () => {
    expect(
      linguiMacroSwcPlugin(
        {
          corePackage: ["@override/core/macro"],
          jsxPackage: ["@override/react/macro"],
          jsxPlaceholderAttribute: "data-test",
          runtimeModules: {
            Trans: ["@override/react", "OverrideTrans"],
          },
        },
        {configPath: resolve(fixturesDir, "custom.config.js")},
      ),
    ).toMatchInlineSnapshot(`
      [
        "@lingui/swc-plugin",
        {
          "corePackage": [
            "@override/core/macro",
          ],
          "jsxPackage": [
            "@override/react/macro",
          ],
          "jsxPlaceholderAttribute": "data-test",
          "jsxPlaceholderDefaults": {
            "a": "anchor",
            "strong": "bold",
          },
          "runtimeModules": {
            "Trans": [
              "@override/react",
              "OverrideTrans",
            ],
            "i18n": [
              "@custom/core",
              "customI18n",
            ],
            "useLingui": [
              "@custom/react",
              "useCustomLingui",
            ],
          },
        },
      ]
    `)
  })
})
