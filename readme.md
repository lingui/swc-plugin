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

- [ ] Simple `t` macro:
  - [x] ``t`Some string` ``
  - [x] ``t`Some ${variable} string` ``
  - [x] ``t`Some ${expression} string` ``
  - [x] ``t(i18n)`Some string` `` - custom i18n instance
  - [x] check name of tag === 't' before transformation
  - [ ] `t({ message descriptor })` call with message descriptor object
- [ ] `defineMessage`
- [ ] ICU calls (plural, select, selectOrdinal)
  - [x] ``plural(count, {one: '# item', few: '# items'})`` - simple strings
  - [x] ``plural(count, {one: `${variable} # item`, few: '# items'})`` - tpls with placeholders
  - [x] ``plural(expression(), {one: `${variable} # item`, few: '# items'})`` - expression as parameter
  - [ ] dedup values object literal when the same variable appears few time, eq avoid `{name, name, count}` 
  - [ ] NON-ESSENTIAL nested icu as described here https://lingui.js.org/ref/macro.html#plural
- [ ] Passing macro as `t()` arguments eq: `t({message: plural(...)})`
- [ ] Support JSX transformation
  - [ ] `<Trans>`
    - [x] Simple cases `<Trans>Hello World</Trans>` -> `<Trans message="Hello World" />`
    - [x] Variables interpolation  `<Trans>Hello {name} and {getName()}</Trans>` -> `<Trans variables={name, 1: getName()} msg="Hello {name} and {1}"/>`
    - [x] Recursive Components interpolation `<Trans>Hello <strong>World!</strong></Trans>`
    - [ ] Stripping non-essential props in production
    - [ ] NON-ESSENTIAL Support edge cases `<Trans>{'Hello World'}</Trans>` and ``<Trans>{`How much is ${expression}? ${count}`}</Trans>``
    - [ ] Whitespaces management: stripping, but keeping "forced"
      - [x] Simple cases 
      - --- It's turned to be quite complicated, i implemented basics, but i'm afraid there might be differences in spaces which can cause translations mismatch. 
      - [ ] Port more test cases related to whitespaces from babel version
  - [ ] ICU: `<Plural>` `<SelectOrdinal>` `<Select>`
- [ ] Support narrowing transformation to only function exported from `@lingui/macro` 
- [ ] Automatic adding  `import { i18n } from @lingui/core`
- [ ] Different behavior for macros for Production build as stated [here](https://lingui.js.org/ref/macro.html#plural:~:text=In%20production%20build%2C%20the%20whole%20macro%20is%20replaced) 
- [ ] Splitting into modules and crates, clean up code
- [ ] Error handling: how to properly behave to do if user passed something not expected [HANDLER](https://rustdoc.swc.rs/swc_common/errors/struct.Handler.html)
- [ ] Investigate patterns how to effectively change something in the root of the tree based on the leafs using visitor / folder
- [ ] Building binary and publishing

### Useful links:
- AST Playground https://play.swc.rs/
- SWC Plugin Docs https://swc.rs/docs/plugin/ecmascript/getting-started