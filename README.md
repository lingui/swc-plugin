# <div align="center">Lingui Rust Tooling</div>

<div align="center">

Rust tooling for [Lingui](https://lingui.dev) i18n - an SWC plugin for the macro transform and a native Node.js addon for message extraction.

[![CI - Rust](https://github.com/lingui/swc-plugin/actions/workflows/ci-rust.yml/badge.svg?branch=main)](https://github.com/lingui/swc-plugin/actions/workflows/ci-rust.yml)
[![codecov](https://codecov.io/gh/lingui/swc-plugin/branch/main/graph/badge.svg)](https://codecov.io/gh/lingui/swc-plugin)
[![GitHub contributors](https://img.shields.io/github/contributors/lingui/swc-plugin?cacheSeconds=1000)](https://github.com/lingui/swc-plugin/graphs/contributors)
[![GitHub](https://img.shields.io/github/license/lingui/swc-plugin)](https://github.com/lingui/swc-plugin/blob/main/LICENSE)

</div>

## Overview

This monorepo hosts the Rust-based tooling for [Lingui](https://lingui.dev). It ships two npm packages built from the same Rust crates:

- [`@lingui/swc-plugin`](#linguiswc-plugin) - an SWC plugin (WebAssembly) that transforms `@lingui/core/macro` and `@lingui/react/macro` calls into optimized i18n runtime code at build time. A faster, Rust-powered alternative to the Babel macro that runs inside SWC, Next.js, and Vite build pipelines.
- [`@lingui/native-tools`](#linguinative-tools) - a native Node.js addon (NAPI-RS) with a Rust message extractor for `lingui extract` and a standalone macro transformer for custom build pipelines.

## Packages

### [`@lingui/swc-plugin`](./packages/lingui-macro/)

[![npm](https://img.shields.io/npm/v/@lingui/swc-plugin?logo=npm&cacheSeconds=1800)](https://www.npmjs.com/package/@lingui/swc-plugin)
[![npm](https://img.shields.io/npm/dt/@lingui/swc-plugin?cacheSeconds=500)](https://www.npmjs.com/package/@lingui/swc-plugin)
[![CI](https://github.com/lingui/swc-plugin/actions/workflows/ci-macro.yml/badge.svg?branch=main)](https://github.com/lingui/swc-plugin/actions/workflows/ci-macro.yml)

SWC macro transform plugin for Lingui. Transforms `@lingui/macro` and `@lingui/react/macro` calls into optimized i18n runtime code. Compiles to WebAssembly (`wasm32-wasip1`).

#### Documentation

- Installation, usage & options - [`@lingui/swc-plugin` README](./packages/lingui-macro/README.md)
- `swc_core` compatibility table - [Compatibility](./packages/lingui-macro/README.md#compatibility)
- [Lingui macro reference](https://lingui.dev/ref/macro)

### [`@lingui/native-tools`](./packages/native-tools/)

[![npm](https://img.shields.io/npm/v/@lingui/native-tools?logo=npm&cacheSeconds=1800)](https://www.npmjs.com/package/@lingui/native-tools)
[![npm](https://img.shields.io/npm/dt/@lingui/native-tools?cacheSeconds=500)](https://www.npmjs.com/package/@lingui/native-tools)
[![CI](https://github.com/lingui/swc-plugin/actions/workflows/ci-native-tools.yml/badge.svg?branch=main)](https://github.com/lingui/swc-plugin/actions/workflows/ci-native-tools.yml)

Native Node.js addon (NAPI-RS) with a Rust message extractor and a standalone macro transformer. Provides `createSwcExtractor()` for `lingui extract` and `transform()` for custom build pipelines that only need the Lingui macro step. Prebuilt binaries are published for macOS, Linux (glibc and musl), Windows, and Android.

> **Note**
> `@lingui/native-tools` is pre-1.0. The API may change between minor versions until the 1.0 release.

#### Documentation

- Installation, usage & options - [`@lingui/native-tools` README](./packages/native-tools/README.md)

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](./CONTRIBUTING.md) for instructions on setting up Rust, building the WASM plugin, running tests, and submitting pull requests.

## License

The project is licensed under the [MIT](./LICENSE) license.
