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

## Code Quality Checks

Before submitting a pull request, please ensure your code passes all quality checks. The CI system will run these same checks, so running them locally will save you time.

### Formatting
```bash
# this project uses rustfmt to enforce a consistent code style
cargo fmt
```

### Linting
```bash
# we use clippy to catch common mistakes and improve code quality
# all clippy warnings are treated as errors in the CI
cargo clippy --all-targets --all-features -- -D warnings
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

To update Rust, put new version into `rust-toolchain` and call `rustup update` command.

## Code Coverage

This project uses [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) to generate code coverage reports.

### Installing cargo-llvm-cov
```bash
cargo install cargo-llvm-cov
```

### Running coverage locally
```bash
# Generate HTML coverage report for local viewing
cargo llvm-cov --all-features --workspace --html --open
```
