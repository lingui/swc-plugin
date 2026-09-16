# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Monorepo for [LinguiJS](https://lingui.dev) Rust/SWC-based tooling. Contains two main tools:

1. **AST Macro Transform** (`crates/lingui_macro`) — SWC plugin that transforms `@lingui/macro` and `@lingui/react/macro` calls into optimized i18n runtime code. Compiles to WebAssembly (wasm32-wasip1) and runs inside SWC/Next.js build pipelines.
2. **Message Extractor** (`crates/lingui_extractor`) — AST visitor (built on SWC) that extracts translatable messages from source files.

Published npm packages:
- `packages/lingui-macro` (`@lingui/swc-plugin`) — ships the compiled WASM binary for the macro transform.
- `packages/lingui-swc` (`lingui-swc`) — NAPI-RS native Node.js binding for the extractor.

## Repository Structure

```
├── crates/
│   ├── lingui_macro/       # SWC plugin (wasm32-wasip1 target)
│   └── lingui_extractor/   # Message extractor library (depends on lingui_macro)
├── packages/
│   ├── lingui-macro/       # npm package wrapping the WASM binary
│   └── lingui-swc/         # NAPI-RS Node.js binding for the extractor
├── Cargo.toml              # Workspace root (members: crates/*, packages/lingui-swc)
└── package.json            # Yarn workspaces root (packages/*)
```

## Build & Test Commands

```bash
# === Rust (whole workspace) ===
cargo test                    # Run all Rust tests
cargo test -p lingui_macro    # Tests for the macro crate only
cargo test -p lingui_extractor # Tests for the extractor crate only
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings

# Run a single test by name
cargo test js_choices_may_contain_expressions

# Run tests matching a prefix
cargo test jsx_

# Update insta snapshots (lingui_macro) interactively
cargo insta test --review

# Bulk-accept all insta snapshot changes (lingui_macro)
INSTA_UPDATE=always cargo test -p lingui_macro

# Update extractor snapshots
UPDATE=1 cargo test -p lingui_extractor

# === WASM build (lingui_macro plugin) ===
cargo build-wasi --release    # alias defined in .cargo/config.toml

# === NAPI-RS build (lingui-swc) ===
cd packages/lingui-swc && yarn build

# === E2E tests (lingui-macro npm package) ===
cd packages/lingui-macro && yarn test:e2e

# === lingui-swc tests ===
cd packages/lingui-swc && yarn test
```

## Architecture

### crates/lingui_macro

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

### crates/lingui_extractor

AST visitor that walks source files and collects message descriptors (id, message, context) using the same parsing logic from `lingui_macro`. Exposes its API via `napi-derive` for consumption by the NAPI binding.

### packages/lingui-swc

NAPI-RS native Node.js addon that wraps `lingui_extractor`. Provides a JS-callable interface for extracting messages from source files. Built with `@napi-rs/cli`.

## Testing

### Rust (lingui_macro)

Tests use [insta](https://insta.rs) snapshot testing. Test macros are defined in `crates/lingui_macro/tests/common/mod.rs`:
- `to!(test_name, "input code")` — transform with default options
- `to!(test_name, options, "input code")` — transform with custom options
- `to_panic!(test_name, options, "input code")` — expect compilation error (error message captured in snapshot)

Snapshots live in `crates/lingui_macro/tests/snapshots/` and contain input + `↓ ↓ ↓ ↓ ↓ ↓` separator + output (or error text for `to_panic!` tests).

To update snapshots use `INSTA_UPDATE=always cargo test` or `cargo insta test --review` to review interactively.

### Rust (lingui_extractor)

Uses a custom snapshot mechanism (not insta). Fixtures live in `crates/lingui_extractor/tests/fixtures/`, snapshots in `crates/lingui_extractor/tests/__snapshots__/` (JSON files). To update snapshots: `UPDATE=1 cargo test -p lingui_extractor`.

### JS (packages/lingui-swc)

Uses Vitest. Tests live in `packages/lingui-swc/__test__/`.

## Reference

- [SWC Transform Testing](.agents/SWC_TRANSFORM_TESTING.md) — how to parse, transform, and emit code using SWC's Rust API outside the plugin host (thread-locals, error handling, pass ordering, codegen).

## Toolchain

- Rust nightly (pinned in `___rust-toolchain.toml`)
- WASM target: `wasm32-wasip1` (aliased as `cargo build-wasi` in `.cargo/config.toml`)
- SWC core v56 (`swc_core` workspace dependency)
- Node v22, Yarn v4 with workspaces
- NAPI-RS v3 for native Node.js bindings
