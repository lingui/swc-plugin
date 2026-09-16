# `lingui-swc`

LinguiJS utils based on SWC Platform

## Low level extraction methods

- `extractMessagesFromFiles`
- `extractMessages`

Check TypeScript types for more info.

## Lingui Extractor Plugin

This will add rust based extractor implementation to your existing lingui setup. 

:::note
To achieve better performance you need to disable multithreading support on the lingui cli side using `--workers 1`.
:::

```ts
import {createSwcExtractor} from 'lingui-swc'

// lingui.config.ts
defineConfig({
  extractors: [createSwcExtractor()],
})
```

`createSwcExtractor()` accepts extractor options: 

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

In most of the cases you don't need to specify anything, unless you use some non-standard parser features or has a custom 
configuration for macro itself. The macro options automatically inferred from your Lingui Config.

## Transform

A native Lingui macro transformer that can be used as a standalone alternative to a full SWC or Babel setup.

It is a minimal SWC setup with the Lingui macro transform baked into a single native binary. It skips the SWC plugin system overhead and omits all other SWC transforms — only Lingui macros are processed, everything else is emitted as-is.

This is useful when you have a custom build pipeline (e.g. esbuild, Rollup, or a dev server) and only need to transform Lingui macros without pulling in the full SWC or Babel toolchain.

```ts
import { transform } from 'lingui-swc'

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
import { transform, type TransformOptions } from 'lingui-swc'

const options: TransformOptions = {
  // SWC parser config (auto-inferred from filename by default)
  parser: { syntax: 'typescript', tsx: true },
  // Lingui macro options
  macro: {
    runtimeModules: {
      i18n: ['@lingui/core', 'i18n'],
      Trans: ['@lingui/react', 'Trans'],
      useLingui: ['@lingui/react', 'useLingui'],
    },
  },
  // External source map JSON string (inline source maps are used if not provided)
  sourceMap: '...',
}

const result = await transform(code, 'app.tsx', options)
```

Here is a benchmark results for native transformer (lower value - better):

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
