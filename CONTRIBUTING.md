## Install rust

You can follow instructions at ['Install Rust' page from the official rust website](https://www.rust-lang.org/tools/install)

## Add wasm target to rust

```bash
rustup target add wasm32-wasip1
```

## Testing

Tests use [insta](https://insta.rs) snapshot testing. Test macros are defined in `crates/lingui_macro/tests/common/mod.rs`:
- `to!(test_name, "input code")` — transform with default options
- `to!(test_name, options, "input code")` — transform with custom options
- `to_panic!(test_name, options, "input code")` — expect compilation error (error message captured in snapshot)

```bash
# run all test suite
cargo test

# run individual test
cargo test js_choices_may_contain_expressions

# you may specify only prefix of test name to target more cases
cargo test jsx_

# run the whole file from the /tests folder (omit .rs extension)
cargo test --test js_icu

# Update snapshots interactively (requires: cargo install cargo-insta)
cargo insta test --review

# Bulk-accept all snapshot changes
INSTA_UPDATE=always cargo test
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
cargo build -p lingui_macro --target wasm32-wasip1 --release
```
Then wasm binary would be on the path: `./target/wasm32-wasip1/release/lingui_macro.wasm`

You can check it in your own project by specifying full path to the WASM binary:

```ts
/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    swcPlugins: [
      ['/path/to/swc-plugin/target/wasm32-wasip1/release/lingui_macro.wasm', {}],
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

## Releasing

The two npm packages are versioned and published independently. Publishing is triggered by a GitHub Release and authenticated with npm trusted publishing.

1. Open a PR that bumps `version` in the package's `package.json` and merge it:
   - `@lingui/swc-plugin` - `packages/lingui-macro/package.json`
   - `@lingui/native-tools` - `packages/native-tools/package.json`
2. Create a GitHub Release from `main` with a tag in the form `<package>@<version>`:
   - `swc-plugin@6.8.0` runs [`release-swc-plugin.yml`](.github/workflows/release-swc-plugin.yml). Mark the release "Latest" to publish to the `latest` dist-tag, or "Pre-release" to publish to `next`.
   - `native-tools@0.1.1` runs [`release-native-tools.yml`](.github/workflows/release-native-tools.yml). It always publishes to `latest`. Until 1.0, mark these releases as "Pre-release" so the repository's "Latest" release stays on `@lingui/swc-plugin`.

The version that gets published is the one in `package.json`, so make sure the tag matches it. A release with any other tag format publishes nothing.
