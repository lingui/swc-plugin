# ADR-001: Inline macro options source into lingui-swc

**Status:** Accepted  
**Date:** 2026-05-25

## Context

The `lingui-swc` package ships a native binary (compiled from Rust via NAPI-RS) that includes the macro transformation logic from `lingui_macro`. The TypeScript layer in `lingui-swc` needs to reuse type definitions (`LinguiMacroOptions`) and a mapping function (`mapOptions`) that are defined in the `@lingui/swc-plugin` package — they describe and configure the behavior baked into the native binary.

However, declaring `@lingui/swc-plugin` as a runtime dependency would be misleading: the types and mapping logic must match the compiled native code at the exact same revision, not whatever version of `@lingui/swc-plugin` a user happens to install. Updating the dependency would not update the compiled code, creating a false sense of alignment.

## Decision

Inline the relevant source from `packages/lingui-macro/src-js` into the `lingui-swc` build using a directory symlink (`src-js/macro-src` -> `../../lingui-macro/src-js`). TypeScript compiles both files into `dist/`, producing a self-contained package with no cross-package import paths.

The shared source (`map-options.ts`) contains only types and the pure `mapOptions` function — no heavy runtime dependencies. The `getConfig`-dependent code (`linguiMacroSwcPlugin`) remains exclusively in `@lingui/swc-plugin`.

## Consequences

- **Alignment guaranteed at build time.** The types and mapping logic are compiled from the same commit as the native binary. Upgrading `@lingui/swc-plugin` independently cannot silently desync them.
- **No runtime dependency on `@lingui/swc-plugin`.** `lingui-swc` does not need to declare it as a dependency.
- **Single source of truth.** The symlink means both packages compile from the same `.ts` file — no manual copy-paste drift.
- **Platform caveat.** Git symlinks require no special setup on Linux/macOS. On Windows they need Developer Mode or elevated privileges; CI (Linux) is unaffected.
