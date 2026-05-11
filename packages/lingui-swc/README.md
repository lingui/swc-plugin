# `lingui-swc`

LinguiJS utils based on SWC Platform

## Low level methods

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

