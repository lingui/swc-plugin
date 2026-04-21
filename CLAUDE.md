# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SWC plugin for [LinguiJS](https://lingui.dev) — a Rust-based compile-time macro that transforms `@lingui/macro` and `@lingui/react/macro` calls into optimized i18n runtime code. Compiles to WebAssembly (wasm32-wasip1) and runs inside SWC/Next.js build pipelines.

## Build & Test Commands

```bash
# Build WASM (primary target)
cargo build-wasi --release

# Run all tests
cargo test

# Run a single test by name
cargo test js_choices_may_contain_expressions

# Run tests matching a prefix
cargo test jsx_

# Update snapshots interactively (requires: cargo install cargo-insta)
cargo insta test --review

# Bulk-accept all snapshot changes
INSTA_UPDATE=always cargo test

# Format / lint
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings

# E2E tests (requires WASM build + Node v22 + yarn)
cargo build-wasi --release && yarn test:e2e
```

## Architecture

The plugin follows SWC's AST visitor pattern using the `Fold` trait for recursive descent transformation.

**Core transformation pipeline:**
1. `lib.rs` — Entry point (`#[plugin_transform]`). Parses config, creates `LinguiMacroFolder` which implements `Fold`.
2. `macro_utils.rs` — `MacroCtx` tracks imports from `@lingui/macro` and `@lingui/react/macro`, maps symbol names to local identifiers.
3. `js_macro_folder.rs` — Transforms JS macro calls (`t()`, `defineMessage()`, `msg()`) into `MsgToken` streams.
4. `jsx_visitor.rs` — `TransJSXVisitor` transforms JSX elements (`<Trans>`, `<Plural>`, `<Select>`) into `MsgToken` streams.
5. `builder.rs` — `MessageBuilder` converts token streams into ICU MessageFormat strings, extracts values/components, and builds message descriptors.
6. `options.rs` — `LinguiOptions` / `DescriptorFields` config. Controls which descriptor fields survive in output ("auto", "all", "id-only", "message").
7. `generate_id.rs` — Deterministic SHA2-based message ID generation.

**Key types:** `MsgToken` enum (String, Expression, TagOpening, TagClosing, IcuChoice) is the intermediate representation between parsing and message building.

## Testing

Tests use [insta](https://insta.rs) snapshot testing. Test macros are defined in `tests/common/mod.rs`:
- `to!(test_name, "input code")` — transform with default options
- `to!(test_name, options, "input code")` — transform with custom options
- `to_panic!(test_name, options, "input code")` — expect compilation error (error message captured in snapshot)

Snapshots live in `tests/snapshots/` and contain input + `↓ ↓ ↓ ↓ ↓ ↓` separator + output (or error text for `to_panic!` tests).

To update snapshots use `INSTA_UPDATE=always cargo test` command or `cargo insta test --review` to review them interactively

## Toolchain

- Rust 1.85 pinned in `rust-toolchain.toml`
- WASM target: `wasm32-wasip1` (aliased as `cargo build-wasi` in `.cargo/config.toml`)
- Node v22 (`.nvmrc`), Yarn v4 with `nodeLinker: node-modules`
- SWC core v50.2.3 (`swc_core` crate)
