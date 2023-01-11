# A SWC Plugin For LinguiJS

<div align="center">

A Rust versions of [LinguiJS Macro](https://lingui.js.org/ref/macro.html) [<img src="https://img.shields.io/badge/beta-yellow"/>](https://github.com/lingui/swc-plugin)

[![npm](https://img.shields.io/npm/v/@lingui/swc-plugin?logo=npm&cacheSeconds=1800)](https://www.npmjs.com/package/@lingui/swc-plugin)
[![npm](https://img.shields.io/npm/dt/@lingui/swc-plugin?cacheSeconds=500)](https://www.npmjs.com/package/@lingui/swc-plugin)
[![CI](https://github.com/lingui/swc-plugin/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/lingui/swc-plugin/actions/workflows/ci.yml)
[![GitHub contributors](https://img.shields.io/github/contributors/lingui/swc-plugin?cacheSeconds=1000)](https://github.com/lingui/swc-plugin/graphs/contributors)
[![GitHub](https://img.shields.io/github/license/lingui/swc-plugin)](https://github.com/lingui/swc-plugin/blob/main/LICENSE)

</div>

## Usage

`.swcrc`
https://swc.rs/docs/configuration/swcrc

```json5
{
  "$schema": "https://json.schemastore.org/swcrc",
  "jsc": {
    "experimental": {
      "plugins": ["@lingui/swc-plugin", {
        
        // Optional
        // Unlike the JS version this option must be passed as object only.
        // Docs https://lingui.js.org/ref/conf.html#std-config-runtimeConfigModule
        "runtimeModules": {
          "i18n": ["@lingui/core", "i18n"],
          "trans": ["@lingui/react", "Trans"]
        }
      }]
    }
  }
}
```

### Tasks
- [x] Essential  `t` macro cases:
  - [x] ``t`Some string` ``
  - [x] ``t`Some ${variable} string` ``
  - [x] ``t`Some ${expression} string` ``
  - [x] ``t(i18n)`Some string` `` - custom i18n instance
  - [x] dedup values object literal when the same variable appears few time, eq avoid `{name, name, count}`
- [x] NON Essential `t` macro cases:
    - [x] `t({ message descriptor })` call with message descriptor object
    - [x] Passing other macros as arguments for `t()` eq: `t({message: plural(...)})`
- [x] `defineMessage`
  - [x] Transform
  - [x] Strip non-essential props on production
- [x] JS ICU calls (plural, select, selectOrdinal)
  - [x] ``plural(count, {one: '# item', few: '# items'})`` - simple strings
  - [x] ``plural(count, {one: `${variable} # item`, few: '# items'})`` - tpls with placeholders
  - [x] ``plural(expression(), {one: `${variable} # item`, few: '# items'})`` - expression as parameter
  - [x] dedup values object literal when the same variable appears few time, eq avoid `{name, name, count}`
  - [x] nesting expressions as described here https://lingui.js.org/ref/macro.html#plural
  - [X] Support `offset:1` and exact matches `=1 {...}`
- [x] Support JSX transformation
  - [x] `<Trans>`
    - [x] Simple cases `<Trans>Hello World</Trans>` -> `<Trans message="Hello World" />`
    - [x] Variables interpolation  `<Trans>Hello {name} and {getName()}</Trans>` -> `<Trans variables={name, 1: getName()} msg="Hello {name} and {1}"/>`
    - [x] Recursive Components interpolation `<Trans>Hello <strong>World!</strong></Trans>`
    - [x] Support edge cases `<Trans>{'Hello World'}</Trans>` and ``<Trans>{`How much is ${expression}? ${count}`}</Trans>``
    - [x] Normalizing whitespaces
    - [x] Stripping non-essential props in production
  - [x] ICU: `<Plural>` `<SelectOrdinal>` `<Select>`
    - [x] Support `offset:1` and exact matches `=1 {...}`
- [x] Support narrowing transformation to only function exported from `@lingui/macro` 
- [x] Automatic adding  `import { i18n } from @lingui/core`
- [x] Unicode escaping, validate how SWC produce values
- [x] Support `runtimeConfigModule` settings
- [ ] NON-ESSENTIAL Injecting uniq variables, avoiding collision with existing variables
- [ ] NON-ESSENTIAL support renamed macro calls `import {t as macroT} from "@lingui/macro"`
- [ ] Error handling: how to properly behave to do if user passed something not expected [HANDLER](https://rustdoc.swc.rs/swc_common/errors/struct.Handler.html)
- [ ] Building binary and publishing

## License

The project is licensed under the [MIT](https://github.com/lingui/swc-lugin/blob/main/LICENSE) license.
