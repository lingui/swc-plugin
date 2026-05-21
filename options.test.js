import { describe, expect, it } from "vitest"
import { resolve } from "node:path"

import { linguiSwcOptions } from "@lingui/swc-plugin/options"

const fixturesDir = resolve(import.meta.dirname, "e2e/fixtures/lingui-options")

describe("linguiSwcOptions", () => {
  it("discovers the default Lingui config from cwd", () => {
    const previousCwd = process.cwd()

    try {
      process.chdir(fixturesDir)

      expect(linguiSwcOptions()).toEqual({
        jsxPlaceholderAttribute: "_t",
        jsxPlaceholderDefaults: {
          a: "link",
        },
        runtimeModules: {
          i18n: ["@acme/core", "i18n"],
          trans: ["@acme/react", "Trans"],
          useLingui: ["@acme/react", "useLingui"],
        },
      })
    } finally {
      process.chdir(previousCwd)
    }
  })

  it("maps shared options from an explicit config path", () => {
    expect(linguiSwcOptions({}, resolve(fixturesDir, "custom.config.js"))).toEqual(
      {
        jsxPlaceholderAttribute: "data-i18n",
        jsxPlaceholderDefaults: {
          a: "anchor",
          strong: "bold",
        },
        runtimeModules: {
          i18n: ["@custom/core", "customI18n"],
          trans: ["@custom/react", "CustomTrans"],
          useLingui: ["@custom/react", "useCustomLingui"],
        },
      },
    )
  })

  it("merges overrides over mapped config", () => {
    expect(
      linguiSwcOptions(
        {
          jsxPlaceholderAttribute: "data-test",
          runtimeModules: {
            trans: ["@override/react", "OverrideTrans"],
          },
        },
        resolve(fixturesDir, "custom.config.js"),
      ),
    ).toEqual({
      jsxPlaceholderAttribute: "data-test",
      jsxPlaceholderDefaults: {
        a: "anchor",
        strong: "bold",
      },
      runtimeModules: {
        i18n: ["@custom/core", "customI18n"],
        trans: ["@override/react", "OverrideTrans"],
        useLingui: ["@custom/react", "useCustomLingui"],
      },
    })
  })
})
