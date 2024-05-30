# <div align="center">A SWC Plugin For LinguiJS</div>

<div align="center">

A Rust versions of [LinguiJS Macro](https://lingui.dev/ref/macro) [<img src="https://img.shields.io/badge/beta-yellow"/>](https://github.com/lingui/swc-plugin)

[![npm](https://img.shields.io/npm/v/@lingui/swc-plugin?logo=npm&cacheSeconds=1800)](https://www.npmjs.com/package/@lingui/swc-plugin)
[![npm](https://img.shields.io/npm/dt/@lingui/swc-plugin?cacheSeconds=500)](https://www.npmjs.com/package/@lingui/swc-plugin)
[![CI](https://github.com/lingui/swc-plugin/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/lingui/swc-plugin/actions/workflows/ci.yml)
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

`.swcrc`
https://swc.rs/docs/configuration/swcrc

```json5
{
  "$schema": "https://json.schemastore.org/swcrc",
  "jsc": {
    "experimental": {
      "plugins": [
        [
          "@lingui/swc-plugin",
          {
            // Optional
            // Unlike the JS version this option must be passed as object only.
            // Docs https://lingui.dev/ref/conf#runtimeconfigmodule
            // "runtimeModules": {
            //   "i18n": ["@lingui/core", "i18n"],
            //   "trans": ["@lingui/react", "Trans"]
            // }
          },
        ],
      ],
    },
  },
}
```

Or Next JS Usage:

`next.config.js`
```js
/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  experimental: {
    swcPlugins: [
      ['@lingui/swc-plugin', {
       // the same options as in .swcrc
      }],
    ],
  },
};

module.exports = nextConfig;
```

> **Note**
> Consult with full working example for NextJS in the `/examples` folder in this repo.


## Compatibility
SWC Plugin support is still experimental. They do not guarantee a semver backwards compatibility between different `swc-core` versions.

So you need to select an appropriate version of the plugin to match compatible `swc_core`.

| Plugin Version                           | used `swc_core` | Compatibility                                                                                        |
|------------------------------------------|-----------------|------------------------------------------------------------------------------------------------------|
| `0.1.0`, `4.0.0-next.0`                  | `0.52.8`        | `next@13.0.0` ~ `next@13.2.3`                                                                        |
| `0.2.*`, `4.0.0-next.1` ~ `4.0.0-next.3` | `0.56.1`        | `@swc/core@1.3.29` ~ `@swc/core@1.3.37` <br/> `next@13.2.4-canary.0` ~ `next@13.2.5-canary.5`        |
| `4.0.0`                                  | `0.75.33`       | `@swc/core@1.3.49` ~ `@swc/core@1.3.57` <br/> `next@v13.3.1-canary.12` ~ `next@v13.4.3-canary.1`     |
| `4.0.1`                                  | `0.76.0`        | broken due to [`lto = true`](https://github.com/swc-project/swc/issues/7470#issuecomment-1571585905) |
| `4.0.2`                                  | `0.76.41`       | `@swc/core@1.3.58` ~ `@swc/core@1.3.62` <br/> `next@v13.4.3-canary.2` ~                              |
| `4.0.3`                                  | `0.78.28`       | `@swc/core@1.3.63` ~ `@swc/core@1.3.67` <br/>  `next@v13.4.8 ~ next@v13.4.10-canary.0`               |
| `4.0.4`                                  | `0.79.x`        | `@swc/core@1.3.68` ~ `@swc/core@1.3.80`  <br/> `next@v13.4.10-canary.1` ~                            |
| `4.0.5`                                  | `0.87.x`        | broken due incorrect version of `swc_common`                                                         |
| `4.0.6`                                  | `0.87.x`        | `@swc/core@1.3.81 ~ @swc/core@1.3.105` <br /> `~ next@v14.1.0`                                       |
| `4.0.7`, `4.0.8`                         | `0.90.35`       | `@swc/core@1.4.0 ~` <br /> `next@14.1.1-canary.52 ~` <br /> `@rspack/core@0.6.0 ~`                   |

This table may become outdated. If you don't see a particular version of `@swc/core` or `next` check the compatibility by referring to the upstream's [Selecting the version](https://swc.rs/docs/plugin/selecting-swc-core) article.
This will help you select the appropriate plugin version for your project.

> **Note**
> next `v13.2.4` ~ `v13.3.1` cannot execute SWC Wasm plugins, due to a [bug of next-swc](https://github.com/vercel/next.js/issues/46989#issuecomment-1486989081).
>
> next `v13.4.3` ~ `v13.4.5-canary.7` cannot execute SWC Wasm plugins, due to [missing filesystem cache](https://github.com/vercel/next.js/pull/50651).

- Version `0.1.0` ~ `0.*` compatible with `@lingui/core@3.*`
- Version `4.*` compatible with `@lingui/core@4.*`

## License

The project is licensed under the [MIT](https://github.com/lingui/swc-plugin/blob/main/LICENSE) license.
