# <div align="center">Lingui Rust Tooling</div>

<div align="center">

Rust tooling for [Lingui](https://lingui.dev) i18n - compiles to WebAssembly and runs inside SWC, Next.js, and Vite build pipelines.

[![codecov](https://codecov.io/gh/lingui/swc-plugin/branch/main/graph/badge.svg)](https://codecov.io/gh/lingui/swc-plugin)
[![GitHub contributors](https://img.shields.io/github/contributors/lingui/swc-plugin?cacheSeconds=1000)](https://github.com/lingui/swc-plugin/graphs/contributors)
[![GitHub](https://img.shields.io/github/license/lingui/swc-plugin)](https://github.com/lingui/swc-plugin/blob/main/LICENSE)

</div>

## Overview

This monorepo hosts the Rust-based tooling for [Lingui](https://lingui.dev). Its flagship is an SWC plugin that transforms `@lingui/macro` and `@lingui/react/macro` calls into optimized i18n runtime code at build time - a faster, Rust-powered alternative to the Babel macro.

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

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](./CONTRIBUTING.md) for instructions on setting up Rust, building the WASM plugin, running tests, and submitting pull requests.

## License

The project is licensed under the [MIT](./LICENSE) license.
