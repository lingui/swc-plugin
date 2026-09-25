# <div align="center">Lingui Native Tools</div>

<div align="center">

Native (Rust + SWC) [Lingui](https://lingui.dev) tooling: message extractor and standalone macro transformer

[![npm](https://img.shields.io/npm/v/@lingui/native-tools?logo=npm&cacheSeconds=1800)](https://www.npmjs.com/package/@lingui/native-tools)
[![npm](https://img.shields.io/npm/dt/@lingui/native-tools?cacheSeconds=500)](https://www.npmjs.com/package/@lingui/native-tools)
[![CI](https://github.com/lingui/swc-plugin/actions/workflows/ci-native-tools.yml/badge.svg?branch=main)](https://github.com/lingui/swc-plugin/actions/workflows/ci-native-tools.yml)
[![GitHub](https://img.shields.io/github/license/lingui/swc-plugin)](https://github.com/lingui/swc-plugin/blob/main/LICENSE)

</div>

> **Note**
> `@lingui/native-tools` is pre-1.0. The API may change between minor versions until the 1.0 release.

## Installation

```bash
npm install --save-dev @lingui/native-tools
# or
yarn add -D @lingui/native-tools
```

Requirements:

- Node.js `>=22.19.0`
- `@lingui/conf` `^6` (peer dependency, used by `createSwcExtractor()`)

Prebuilt binaries are installed automatically for macOS (x64, arm64), Linux (x64, arm64; glibc and musl), Windows (x64, arm64), and Android (arm64). No Rust toolchain is needed.

## Lingui Extractor Plugin

`createSwcExtractor()` adds the Rust-based extractor implementation to your existing Lingui setup. Macro options are inferred from your Lingui config automatically.

```ts
// lingui.config.ts
import { defineConfig } from '@lingui/conf'
import { createSwcExtractor } from '@lingui/native-tools'

export default defineConfig({
  extractors: [createSwcExtractor()],
})
```

> **Note**
> For the best performance, disable multithreading on the Lingui CLI side with `lingui extract --workers 1`. The native extractor processes files in parallel itself.

`createSwcExtractor()` accepts optional extractor options:

```ts
export type ExtractorOptions = {
  /**
   * The same options as in `jsc.parser` in `.swcrc`
   * https://swc.rs/docs/configuration/compilation#jscparser
   *
   * The syntax (ecmascript/typescript) and jsx support is automatically inferred from the filename,
   * you don't need to specify it manually
   */
  parser?: ParserConfig
  /**
   * Options for Lingui Macro
   *
   * Except of `descriptorFields` property which is always set to `All` in extraction
   */
  macro?: Omit<LinguiMacroOptions, 'descriptorFields'>
}
```

In most cases you don't need to specify anything, unless you use non-standard parser features or have a custom configuration for the macro itself.

### Low-level extraction methods

For custom tooling, the extractor is also exposed directly:

- `extractMessages(code, filename, options?)` - extracts messages from a source string.
- `extractMessagesFromFiles(filePaths, options?)` - reads and extracts messages from many files in parallel.

Both return a `Promise<ExtractionResult>` with the extracted messages (id, message, context, comment, placeholders, and origin). Check the TypeScript types for details.

## Transform

A native Lingui macro transformer that can be used as a standalone alternative to a full SWC or Babel setup.

It is a minimal SWC setup with the Lingui macro transform baked into a single native binary. It skips the SWC plugin system overhead and omits all other SWC transforms - only Lingui macros are processed, everything else is emitted as-is.

The only exception is TypeScript: TS syntax is stripped before the macro runs (the same way SWC does before running plugins), so `.ts` / `.tsx` files come out as JS with JSX preserved. Type-only imports are removed, class fields are kept as-is (`useDefineForClassFields: true`), and `enum` / `namespace` are compiled to JS.

This is useful when you have a custom build pipeline (e.g. esbuild, Rollup, or a dev server) and only need to transform Lingui macros without pulling in the full SWC or Babel toolchain.

```ts
import { transform } from '@lingui/native-tools'

const result = await transform(
  `import { t } from '@lingui/core/macro';
const msg = t\`Hello world\`;`,
  'app.tsx'
)

console.log(result.code)
// => transformed code with Lingui macros compiled to runtime calls
console.log(result.map)
// => source map JSON string
```

The `transform` function accepts an optional third argument with options:

```ts
import { transform, type TransformOptions } from '@lingui/native-tools'

const options: TransformOptions = {
  // SWC parser config (auto-inferred from filename by default)
  parser: { syntax: 'typescript', tsx: true },
  // Lingui macro options, the same as for @lingui/swc-plugin
  macro: {
    runtimeModules: {
      i18n: ['@lingui/core', 'i18n'],
      Trans: ['@lingui/react', 'Trans'],
      useLingui: ['@lingui/react', 'useLingui'],
    },
  },
  // Source map generation:
  // - true (default): source map returned in `result.map`
  // - "inline": source map appended to `result.code` as a base64 data URL
  // - false: no source map
  sourceMaps: true,
}

const result = await transform(code, 'app.tsx', options)
```

If the input code ends with an inline `//# sourceMappingURL=data:...` comment, that source map is consumed and chained into the output map.

The `macro` options are documented in the [`@lingui/swc-plugin` README](https://github.com/lingui/swc-plugin/blob/main/packages/lingui-macro/README.md#options).

### Benchmark

Macro transform benchmark results for the native transformer (lower is better):

```
══════════════════════════════════════════════════════════════
  Lingui Benchmark — Preset: medium
  1000 files · 10.0k messages · 5 locales
  Node v24.13.1 · darwin arm64
  Apple M3 Max (16 cores)
══════════════════════════════════════════════════════════════

Running: Macro Transform...

Babel               █████████████████████████  1.57s  636 files/s ±10.9%
SWC                 ██░░░░░░░░░░░░░░░░░░░░░░░  143ms  6996 files/s
native transformer  █░░░░░░░░░░░░░░░░░░░░░░░░   54ms  18.6k files/s ⚡

Summary:
native transformer is 29.3x faster than Babel
native transformer is 2.7x faster than SWC
```

## License

The project is licensed under the [MIT](https://github.com/lingui/swc-plugin/blob/main/LICENSE) license.
