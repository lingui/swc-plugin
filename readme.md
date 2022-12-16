# A SWC Plugin For LinguiJS

## Disclaimer
Project is on very early stage, it's even not 0.1 it's just 
a playground where i try to get more familiar with Rust and SWC internals

Help is very welcome, at least in Rust itself.

## Description
Project is scafolded using `swc plugin new --target-type wasm32-wasi`

I've don't use a `Visit` trait. Use A `Fold` instead, because i found it easier to implement

### How to start
After following rust and swc instructions of installation just call 

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
  - [ ] nested icu as described here https://lingui.js.org/ref/macro.html#plural - WONT DO, due to very high complexity and low priority
- [ ] Passing macro as `t()` arguments eq: `t({message: plural(...)})`
- [ ] Support JSX transformation TODO - describe more cases
  - [ ] `<Trans>`
  - [ ] Simple cases and cases with inner JSX elements
  - [ ] ICU: `<Plural>` `<SelectOrdinal>` `<Select>`
- [ ] Support narrowing transformation to only function exported from `@lingui/macro` 
- [ ] Automatic adding  `import { i18n } from @lingui/core`
- [ ] Different behavior for all macros for Production build as stated [here](https://lingui.js.org/ref/macro.html#plural:~:text=In%20production%20build%2C%20the%20whole%20macro%20is%20replaced) 
- [ ] Investigate in testing fixtures and create a comprehensive test suite
- [ ] Splitting into modules and crates, clean up code
- [ ] Error handling: how to properly behave to do if user passed something not expected
- [ ] Investigate patterns how to effectively change something in the root of the tree based on the leafs using visitor / folder
- [ ] Building binary and publishing

### Useful links:
- AST Playground https://play.swc.rs/
- SWC Plugin Docs https://swc.rs/docs/plugin/ecmascript/getting-started