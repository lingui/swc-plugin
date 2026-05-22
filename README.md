# <div align="center">A SWC Plugin For LinguiJS</div>

<div align="center">

A Rust version of [LinguiJS Macro](https://lingui.dev/ref/macro) [<img src="https://img.shields.io/badge/beta-yellow"/>](https://github.com/lingui/swc-plugin)

[![npm](https://img.shields.io/npm/v/@lingui/swc-plugin?logo=npm&cacheSeconds=1800)](https://www.npmjs.com/package/@lingui/swc-plugin)
[![npm](https://img.shields.io/npm/dt/@lingui/swc-plugin?cacheSeconds=500)](https://www.npmjs.com/package/@lingui/swc-plugin)
[![CI](https://github.com/lingui/swc-plugin/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/lingui/swc-plugin/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/lingui/swc-plugin/branch/main/graph/badge.svg)](https://codecov.io/gh/lingui/swc-plugin)
[![GitHub contributors](https://img.shields.io/github/contributors/lingui/swc-plugin?cacheSeconds=1000)](https://github.com/lingui/swc-plugin/graphs/contributors)
[![GitHub](https://img.shields.io/github/license/lingui/swc-plugin)](https://github.com/lingui/swc-plugin/blob/main/LICENSE)

</div>

## Installation

Install plugin:
```bash
npm install --save-dev @lingui/swc-plugin
# or
yarn add -D @lingui/swc-plugin
```

You still need to install `@lingui/macro` for typings support:
```bash
npm install @lingui/macro
# or
yarn add @lingui/macro
```

## Usage

If your build tool uses a JS config (Next.js, Vite, etc.), use the `linguiMacroSwcPlugin` helper — it reads your Lingui config and prepares all plugin options automatically.

If you configure SWC directly via `.swcrc` (e.g. the SWC CLI), pass options manually as described in the [Options](#options) section below.

### JS config (recommended)

#### `next.config.js`

```js
const { linguiMacroSwcPlugin } = require("@lingui/swc-plugin/options")

/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  experimental: {
    swcPlugins: [
      linguiMacroSwcPlugin(),
    ],
  },
};

module.exports = nextConfig;
```

> **Note**
> Consult with full working example for NextJS in the `/examples` folder in this repo.

#### `vite.config.ts`

```ts
import { defineConfig } from "vite"
import react from "@vitejs/plugin-react-swc"
import { lingui } from "@lingui/vite-plugin"
import { linguiMacroSwcPlugin } from "@lingui/swc-plugin/options"

export default defineConfig({
  plugins: [
    react({
      plugins: [linguiMacroSwcPlugin()],
    }),
    lingui(),
  ],
})
```

#### `linguiMacroSwcPlugin(overrides?, configOptions?)`

`linguiMacroSwcPlugin` reads your Lingui config and maps relevant options to the SWC plugin format. It returns a `["@lingui/swc-plugin", options]` tuple ready to use in plugin arrays.

```js
import { linguiMacroSwcPlugin } from "@lingui/swc-plugin/options"

// Recommended — reads lingui.config.{js,ts} automatically
linguiMacroSwcPlugin()

// Override specific options
linguiMacroSwcPlugin({
  useLinguiV5IdGeneration: true,
})

// Specify which lingui config to use
linguiMacroSwcPlugin({}, { configPath: '../lingui.config.js' })
```

### `.swcrc`

When using SWC directly via CLI or a JSON-only configuration, pass options manually. All options are optional — if your have a standard setup, an empty object `{}` is sufficient:

```json5
{
  "$schema": "https://json.schemastore.org/swcrc",
  "jsc": {
    "experimental": {
      "plugins": [
        [
          "@lingui/swc-plugin",
          {
            "runtimeModules": {
              "i18n": ["@lingui/core", "i18n"],
              "trans": ["@lingui/react", "Trans"],
              "useLingui": ["@lingui/react", "useLingui"]
            },
            "descriptorFields": "auto",
            "jsxPlaceholderAttribute": "_t",
            "jsxPlaceholderDefaults": {
              "a": "link"
            }
          },
        ],
      ],
    },
  },
}
```

## Options

### `descriptorFields`

Controls which fields are preserved in the transformed message descriptors. Accepts one of:

- **`"auto"`** (default) — In production (`NODE_ENV=production`), behaves like `"id-only"`. Otherwise, behaves like `"all"`.
- **`"all"`** — Keeps `id`, `message`, `context`, and `comment`. Use this for extraction (replaces the old `extract: true` from the Babel plugin).
- **`"id-only"`** — Keeps only the `id`. Most optimized for production bundles.
- **`"message"`** — Keeps `id`, `message`, and `context` (but not `comment`). Useful when you need message content at runtime.

See [Optimizing bundle size](https://lingui.dev/guides/optimizing-bundle-size) for more info about this configuration.

### `idPrefixLeader`

The SWC plugin matches the Babel macro behavior

See [Configuration Doc](https://lingui.dev/ref/conf#macroidprefixleader) and [`lingui-set` / `lingui-reset` Comment Directives Doc](https://lingui.dev/ref/macro#lingui-directive)

### `jsxPlaceholderAttribute`

Sets the JSX attribute name used to provide explicit placeholder names inside `<Trans>` content.

### `jsxPlaceholderDefaults`

Defines default placeholder names for JSX tags when no explicit placeholder attribute is present.

### `runtimeModules`

Overrides the runtime imports used by the plugin. Unlike [the Babel macro configuration](https://lingui.dev/ref/conf#runtimeconfigmodule), this option must be passed as an object.

### `useLinguiV5IdGeneration`

Compatibility option for using the v6 SWC plugin release channel with `@lingui/cli@5.*`.

- **`false`** (default) — Uses the URL-safe Base64 alphabet used by Lingui v6.
- **`true`** — Uses the standard Base64 alphabet used by Lingui v5.

> **Note**
> This option is temporary and will be removed in the next major release.

## Compatibility
SWC Plugin support is still experimental. They do not guarantee a semver backwards compatibility between different `swc-core` versions.

So you need to select an appropriate version of the plugin to match compatible `swc_core` using a https://plugins.swc.rs/.

Below is a table referencing the `swc_core` version used during the plugin build, along with a link to the plugin's site to check compatibility with runtimes for this `swc_core` range.

To learn more about SWC Plugins compatibility check this issue https://github.com/lingui/swc-plugin/issues/179

| Plugin Version                                                                                                                                                                                                                   | used `swc_core`                                                                                                                                                       |
|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `0.1.0`, `4.0.0-next.0`                                                                                                                                                                                                          | `0.52.8`                                                                                                                                                              |
| `0.2.*`, `4.0.0-next.1` ~ `4.0.0-next.3`                                                                                                                                                                                         | `0.56.1`                                                                                                                                                              |
| `4.0.0`                                                                                                                                                                                                                          | `0.75.33`                                                                                                                                                             |
| `4.0.1`                                                                                                                                                                                                                          | `0.76.0`                                                                                                                                                              |
| `4.0.2`                                                                                                                                                                                                                          | `0.76.41`                                                                                                                                                             |
| `4.0.3`                                                                                                                                                                                                                          | `0.78.28`                                                                                                                                                             |
| `4.0.4`                                                                                                                                                                                                                          | `0.79.x`                                                                                                                                                              |
| `4.0.5`, `4.0.6`                                                                                                                                                                                                                 | [`0.87.x`](https://plugins.swc.rs/versions/range/10)                                                                                                                  |
| `4.0.7`, `4.0.8`, `5.0.0-next.0` ~ `5.0.0-next.1`                                                                                                                                                                                | [`0.90.35`](https://plugins.swc.rs/versions/range/12)                                                                                                                 |
| `4.0.9`                                                                                                                                                                                                                          | [`0.96.9`](https://plugins.swc.rs/versions/range/15)                                                                                                                  |
| `4.0.10`                                                                                                                                                                                                                         | [`0.101.4`](https://plugins.swc.rs/versions/range/94)                                                                                                                 |
| `4.1.0`, `5.0.0` ~ `5.2.0`                                                                                                                                                                                                       | [`0.106.3`](https://plugins.swc.rs/versions/range/95)                                                                                                                 |
| `5.3.0`                                                                                                                                                                                                                          | [`5.0.4`](https://plugins.swc.rs/versions/range/116)                                                                                                                  |
| `5.4.0`                                                                                                                                                                                                                          | [`14.1.0`](https://plugins.swc.rs/versions/range/138)                                                                                                                 |
| `5.5.0` ~ `5.5.2`                                                                                                                                                                                                                | [`15.0.1`](https://plugins.swc.rs/versions/range/271)                                                                                                                 |
| `5.6.0` ~ `5.6.1`                                                                                                                                                                                                                | [`27.0.6`](https://plugins.swc.rs/versions/range/364)                                                                                                                 |
| `5.7.0`                                                                                                                                                                                                                          | [`39.0.3`](https://plugins.swc.rs/versions/range/426)                                                                                                                 |
| `5.8.0`                                                                                                                                                                                                                          | [`45.0.2`](https://plugins.swc.rs/versions/range/497)                                                                                                                 |
| `5.9.0`                                                                                                                                                                                                                          | [`46.0.3`](https://plugins.swc.rs/versions/range/713)                                                                                                                 |
| `5.10.0`                                                                                                                                                                                                                         | [`50.2.3`](https://plugins.swc.rs/versions/range/768)                                                                                                                 |
| `5.10.1` ~ `6.1.0`  <br/> Starting from this version Wasm plugins are compatible between `@swc/core` versions to some extent. Read more [here](https://swc.rs/docs/plugin/ecmascript/compatibility#make-your-plugin-compatible). | [`50.2.3`](https://plugins.swc.rs/versions/range/768) with [`--cfg=swc_ast_unknown`](https://swc.rs/docs/plugin/ecmascript/compatibility#make-your-plugin-compatible) |
| `6.2.0` ~ `*`                                                                                                                                                                                                                    | `66.0.3`                                                                                                                 |


> **Note**
>
> next `v13.2.4` ~ `v13.3.1` cannot execute SWC Wasm plugins, due to a [bug of next-swc](https://github.com/vercel/next.js/issues/46989#issuecomment-1486989081).
>
> next `v13.4.3` ~ `v13.4.5-canary.7` cannot execute SWC Wasm plugins, due to [missing filesystem cache](https://github.com/vercel/next.js/pull/50651).

- Version `0.1.0` ~ `0.*` compatible with `@lingui/core@3.*`
- Version `4.*` compatible with `@lingui/core@4.*`
- Version `5.*` compatible with `@lingui/core@5.*`
- Version `6.*` compatible with `@lingui/core@5.*` with `useLinguiV5IdGeneration: true` and `@lingui/core@6.*`

## License

The project is licensed under the [MIT](https://github.com/lingui/swc-plugin/blob/main/LICENSE) license.
