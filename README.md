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
            // Lingui strips non-essential fields in production builds for performance.
            // You can override the default behavior with:
            // "stripNonEssentialFields": false/true
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

So you need to select an appropriate version of the plugin to match compatible `swc_core` using a https://plugins.swc.rs/.

> **Note**
> next `v13.2.4` ~ `v13.3.1` cannot execute SWC Wasm plugins, due to a [bug of next-swc](https://github.com/vercel/next.js/issues/46989#issuecomment-1486989081).
>
> next `v13.4.3` ~ `v13.4.5-canary.7` cannot execute SWC Wasm plugins, due to [missing filesystem cache](https://github.com/vercel/next.js/pull/50651).

- Version `0.1.0` ~ `0.*` compatible with `@lingui/core@3.*`
- Version `4.*` compatible with `@lingui/core@4.*`
- Version `5.*` compatible with `@lingui/core@5.*`

## License

The project is licensed under the [MIT](https://github.com/lingui/swc-plugin/blob/main/LICENSE) license.
