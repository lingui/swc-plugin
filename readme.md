# A SWC Plugin For LinguiJS

## Disclaimer
Project is on very early stage. Check the task list to keep track of progress. 

## Description
This is a Rust versions for [LinguiJS macro](https://lingui.js.org/ref/macro.html)
This plugin attempts to mimic most of behaviour from original plugin.
However, for the first launch only essential subset of syntax and features would be supported.

This will allow to unblock transition to SWC to the most of the users (me as well) and 
than we can continue working on the rest of features set. 

## Contributing
If you want to help, please check issues, i left there question which i could not find answer. 

Also any help related to Rust or SWC plugin architecture would be much appreciated.

If you know Rust and could do a Code Review, please check the code, together we make it better. 

### How to start
After following rust and swc instructions of installation just call:

```bash
cargo test
```

### Tasks:

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
  - [ ] Omit non-essential props on production
- [x] JS ICU calls (plural, select, selectOrdinal)
  - [x] ``plural(count, {one: '# item', few: '# items'})`` - simple strings
  - [x] ``plural(count, {one: `${variable} # item`, few: '# items'})`` - tpls with placeholders
  - [x] ``plural(expression(), {one: `${variable} # item`, few: '# items'})`` - expression as parameter
  - [x] dedup values object literal when the same variable appears few time, eq avoid `{name, name, count}`
  - [x] nesting expressions as described here https://lingui.js.org/ref/macro.html#plural
  - [X] Support `offset:1` and exact matches `=1 {...}`
- [ ] Support JSX transformation
  - [ ] `<Trans>`
    - [x] Simple cases `<Trans>Hello World</Trans>` -> `<Trans message="Hello World" />`
    - [x] Variables interpolation  `<Trans>Hello {name} and {getName()}</Trans>` -> `<Trans variables={name, 1: getName()} msg="Hello {name} and {1}"/>`
    - [x] Recursive Components interpolation `<Trans>Hello <strong>World!</strong></Trans>`
    - [x] Support edge cases `<Trans>{'Hello World'}</Trans>` and ``<Trans>{`How much is ${expression}? ${count}`}</Trans>``
    - [ ] Stripping non-essential props in production
    - [ ] Whitespaces management: stripping, but keeping "forced"
      - [x] Simple cases
      - [ ] escaping forced `\r`
  - [x] ICU: `<Plural>` `<SelectOrdinal>` `<Select>`
    - [x] Support `offset:1` and exact matches `=1 {...}`
- [x] Support narrowing transformation to only function exported from `@lingui/macro` 
- [x] Automatic adding  `import { i18n } from @lingui/core`
- [ ] Unicode escaping, validate how SWC produce values
- [ ] NON-ESSENTIAL Injecting uniq variables, avoiding collision with existing variables
- [ ] NON-ESSENTIAL support renamed macro calls `import {t as macroT} from "@lingui/macro"`
- [ ] Error handling: how to properly behave to do if user passed something not expected [HANDLER](https://rustdoc.swc.rs/swc_common/errors/struct.Handler.html)
- [ ] Building binary and publishing
- [ ] Support `runtimeConfigModule` settings

### Useful links:
- AST Playground https://play.swc.rs/
- SWC Plugin Docs https://swc.rs/docs/plugin/ecmascript/getting-started
- SWC Plugin FormatJs https://github.com/kwonoj/swc-plugin-formatjs
- Collection of plugins https://github.com/swc-project/plugins
- SWC Folder [Docs](https://rustdoc.swc.rs/swc_ecma_visit/fn.fold_jsx_element.html) 