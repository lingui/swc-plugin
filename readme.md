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

### Useful links:
- AST Playground https://play.swc.rs/
- SWC Plugin Focs https://swc.rs/docs/plugin/ecmascript/getting-started