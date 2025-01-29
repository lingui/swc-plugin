## Install rust

You can follow instructions at ['Install Rust' page from the official rust website](https://www.rust-lang.org/tools/install)

## Add wasm target to rust

```bash
rustup target add wasm32-wasip1
```

## Running tests
```bash
# run all test suite
cargo test

# run individual test
cargo test js_choices_may_contain_expressions

# you may specify only prefix of test name to target more cases
cargo test jsx_
```

## Building for production

```bash
# (alias for `cargo build --target wasm32-wasip1`)
cargo build-wasi --release
```
Then wasm binary would be on the path: `./target/wasm32-wasip1/release/lingui_macro_plugin.wasm`

You can check it in your own project or in the `examples/nextjs-13` example in this repo by specifying full path to the WASM binary:

```ts
/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    swcPlugins: [
      ['/Users/tim/projects/lingui-macro-plugin/target/wasm32-wasip1/release/lingui_macro_plugin.wasm', {}],
    ],
  },
};

module.exports = nextConfig;
```

## Rust Version

It's important to build a plugin with the same Rust version used to build SWC itself.

This project uses `rust-toolchain` file in the root of project to define rust version.

To update Rust, put new version into `rust-toolchain` and call `rustup update` command