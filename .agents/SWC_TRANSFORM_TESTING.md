# Running SWC transforms outside the plugin host

This document explains how to parse, transform, and emit JavaScript/TypeScript code using SWC's Rust API **without** the SWC plugin host (WASM runtime). This is what powers our test suite, but the techniques apply any time you want to run an SWC `Fold`/`VisitMut` pass in a standalone Rust binary or test harness.

## Required `swc_core` features

```toml
swc_core = { version = "50.2.3", features = [
  "ecma_ast",       # AST types, Program, Pass trait
  "ecma_parser",    # Parser, Lexer, Syntax
  "ecma_visit",     # Fold, VisitMut, FoldWith, fold_pass()
  "ecma_codegen",   # Emitter, to_code_default()
  "ecma_utils",     # pulls in swc_ecma_transforms_base (resolver, hygiene, fixer)
  "common",         # SourceMap, GLOBALS, HANDLER, Mark, Comments
] }
```

The `ecma_utils` feature transitively enables `swc_ecma_transforms_base`, which provides the standard passes: `resolver`, `hygiene`, and `fixer`.

The `ecma_codegen` feature provides `to_code_default()` and the lower-level `Emitter` + `JsWriter` for turning an AST back into a string.

> **You do NOT need `testing_transform`.** That feature pulls in the entire `swc_ecma_transforms_testing` crate (and its transitive deps: `testing`, `swc_error_reporters`, `ansi_term`, `base64`, `tempfile`, Node.js exec support, etc.). Everything it does can be inlined in ~60 lines.

## Thread-local globals

SWC uses two scoped thread-locals that must be set before any transform work:

### `GLOBALS`

```rust
use swc_core::common::{Globals, GLOBALS};

GLOBALS.set(&Globals::new(), || {
    // All SWC work goes here
});
```

`Mark::new()` (used by the resolver and by hygiene) allocates from a global arena stored in `GLOBALS`. If `GLOBALS` is not set, `Mark::new()` will panic.

### `HANDLER`

```rust
use swc_core::common::errors::{Handler, HandlerFlags, HANDLER};

let handler = Handler::with_emitter_and_flags(
    Box::new(my_emitter),
    HandlerFlags {
        can_emit_warnings: true,
        ..Default::default()
    },
);

HANDLER.set(&handler, || {
    // Transforms that emit diagnostics go here
});
```

Plugin code (and some built-in SWC transforms) report errors via `HANDLER.with(|h| h.struct_span_err(span, msg).emit())`. If `HANDLER` is not set, those calls panic with a "HANDLER not set" message.

**`HANDLER` must be nested inside `GLOBALS`** — both must be active simultaneously.

#### Handler behavior flags

| Flag | Default | Effect |
|---|---|---|
| `can_emit_warnings` | `true` | Whether warning-level diagnostics are emitted |
| `treat_err_as_bug` | `false` | If `true`, any error immediately panics via `bug!()` |
| `continue_after_error` | `true` (Cell) | If `true`, errors are collected and execution continues. If `false`, the first error triggers `FatalError.raise()` which calls `panic::resume_unwind` |

For tests, the default flags work well: errors are collected silently, and you check for them after the transform completes.

#### Custom error emitters

The `Handler` accepts a `Box<dyn Emitter + Send>` where `Emitter` is `swc_core::common::errors::Emitter` (not the codegen `Emitter`). This is how you capture error messages programmatically:

```rust
use std::sync::{Arc, Mutex};

struct StringEmitter {
    buffer: Arc<Mutex<String>>,
}

impl swc_core::common::errors::Emitter for StringEmitter {
    fn emit(&mut self, db: &mut swc_core::common::errors::DiagnosticBuilder<'_>) {
        // DiagnosticBuilder derefs to Diagnostic
        // Diagnostic.message is Vec<Message> where Message(pub String, pub Style)
        let msg: String = db.message.iter().map(|m| m.0.as_str()).collect::<Vec<_>>().join("");
        let mut buf = self.buffer.lock().unwrap();
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(&msg);
    }
}
```

The `Arc<Mutex<String>>` gives you a handle to read errors after the transform. This is preferable to `Handler::with_tty_emitter()` when you need to inspect or assert on error output.

> **Caveat:** The `Emitter` trait requires `Send`. `Rc<...>` won't work. Use `Arc<Mutex<...>>` or `Arc<RwLock<...>>`.

## Parsing

```rust
use swc_core::common::{FileName, SourceMap, sync::Lrc};
use swc_core::common::comments::SingleThreadedComments;
use swc_core::ecma::parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

let cm: Lrc<SourceMap> = Default::default();
let comments = SingleThreadedComments::default();

let syntax = Syntax::Typescript(TsSyntax {
    tsx: true,
    ..Default::default()
});

let fm = cm.new_source_file(
    FileName::Real("input.tsx".into()).into(),
    input.to_string(),
);

let lexer = Lexer::new(
    syntax,
    Default::default(),  // EsVersion — latest
    StringInput::from(&*fm),
    Some(&comments),     // attaches comments to the AST
);

let mut parser = Parser::new_from(lexer);
let program: Program = parser.parse_program().expect("parse error");
```

**Key points:**

- `SourceMap` (`Lrc<SourceMap>`) is required. It maps byte positions in the AST back to source locations. Even if you don't need source maps in output, the parser and codegen need it.
- `SingleThreadedComments` stores comments (leading/trailing) associated with AST nodes. Pass it to the parser, your transforms, and the code emitter to preserve `/* ... */` and `// ...` comments through the pipeline.
- `parse_program()` auto-detects Module vs Script. Use `parse_module()` or `parse_script()` if you need a specific mode.
- Parse errors are returned as `Result`, not emitted through `HANDLER`. The parser does **not** require `HANDLER` to be set (though it won't hurt).

### Routing parse errors through HANDLER

The parser returns errors as `Result`, but you can convert them to diagnostics emitted through `HANDLER`. This is useful when you want parse errors to be captured by a custom `Emitter` alongside transform errors, rather than causing a hard panic:

```rust
let mut parser = Parser::new_from(lexer);
let program = match parser.parse_program() {
    Ok(program) => program,
    Err(e) => {
        // Convert the parse error to a diagnostic and emit it through HANDLER
        e.into_diagnostic(&handler).emit();
        for e in parser.take_errors() {
            e.into_diagnostic(&handler).emit();
        }
        return None; // or however you signal failure
    }
};
// Also emit any non-fatal parse errors (warnings, recoverable errors)
for e in parser.take_errors() {
    e.into_diagnostic(&handler).emit();
}
```

This pattern (from SWC's `Tester::with_parser`) is what we use in `tests/common/mod.rs` so that `to_panic!` tests can capture parse errors in snapshots just like transform errors.

## Applying transforms

SWC has two transform APIs: the older `Fold` trait (immutable, returns new nodes) and the newer `VisitMut`/`Pass` trait (mutable in-place). Both are actively used.

### The `Pass` trait

```rust
// Defined in swc_core::ecma::ast
pub trait Pass {
    fn process(&mut self, program: &mut Program);
}
```

`Program` has an `.apply()` convenience method:

```rust
let program = program.apply(my_pass);   // consumes and returns Program
```

### Wrapping `Fold` into `Pass`

If your transform implements `Fold` (like `LinguiMacroFolder`), wrap it:

```rust
use swc_core::ecma::visit::fold_pass;

let program = program.apply(fold_pass(MyFolder::new()));
```

`fold_pass()` returns an opaque type implementing `Pass`.

### Standard passes

All three standard passes return `impl Pass + VisitMut`:

```rust
use swc_core::ecma::transforms::base::{resolver, hygiene, fixer};
use swc_core::common::Mark;

// 1. Resolver — assigns unique Marks to identifiers for scope tracking
//    Requires GLOBALS to be set (Mark::new() allocates from it)
let program = program.apply(resolver(
    Mark::new(),    // unresolved_mark — for free identifiers
    Mark::new(),    // top_level_mark — for top-level declarations
    true,           // typescript mode
));

// 2. Your transform
let program = program.apply(fold_pass(MyTransform::new()));

// 3. Hygiene — renames identifiers to avoid conflicts introduced by transforms
let program = program.apply(hygiene::hygiene());

// 4. Fixer — corrects operator precedence, adds necessary parentheses
let program = program.apply(fixer::fixer(Some(&comments)));
```

### Pass composition

Tuples of `Pass` implement `Pass` (up to 12 elements):

```rust
let program = program.apply((
    resolver(Mark::new(), Mark::new(), true),
    fold_pass(MyTransform::new()),
));
```

### Ordering caveats

- **Resolver must come first** if your transform relies on identifier scoping (e.g., distinguishing local `t` from imported `t`). Without it, all identifiers with the same name look identical.
- **Hygiene must come after your transform.** It cleans up naming conflicts your transform may have introduced (e.g., injecting an `i18n` import that clashes with a user's `i18n` variable).
- **Fixer must come last.** It fixes the AST for correct code emission (adds parens for precedence, etc.). Running transforms after fixer can undo its corrections.
- **Fixer takes `Option<&dyn Comments>`** so it can adjust comment positions when it restructures the AST. Always pass your comments reference.

## Code emission

### Simple: `to_code_default`

```rust
use swc_core::ecma::codegen::to_code_default;

let code: String = to_code_default(cm.clone(), Some(&comments), &program);
```

This is a one-liner that creates an `Emitter` + `JsWriter` internally, emits the program, and returns a `String`. Good enough for tests and most use cases.

### Manual: `Emitter` + `JsWriter`

For source map output or custom writer config:

```rust
use swc_core::ecma::codegen::{Emitter, text_writer::JsWriter};

let mut buf = Vec::new();
let mut src_map = Vec::new(); // (BytePos, LineCol) pairs

{
    let mut emitter = Emitter {
        cfg: Default::default(),
        cm: cm.clone(),
        comments: Some(&comments),
        wr: JsWriter::new(
            cm.clone(),
            "\n",               // line separator
            &mut buf,
            Some(&mut src_map), // None to skip source map
        ),
    };
    emitter.emit_program(&program).unwrap();
}

let output = String::from_utf8(buf).unwrap();
```

> **Naming collision:** `swc_core::ecma::codegen::Emitter` (code emission struct) and `swc_core::common::errors::Emitter` (diagnostic trait) are completely different types that happen to share a name. If you use both, alias one: `use swc_core::common::errors::Emitter as DiagEmitter;`

## Complete example

Putting it all together — a function that parses TSX, applies a `Fold` transform, and returns the output code (or collected error messages):

```rust
use std::sync::{Arc, Mutex};
use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::errors::{Handler, HandlerFlags, HANDLER};
use swc_core::common::{FileName, Globals, Mark, SourceMap, GLOBALS, sync::Lrc};
use swc_core::ecma::codegen::to_code_default;
use swc_core::ecma::parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_core::ecma::transforms::base::{fixer, hygiene, resolver};
use swc_core::ecma::visit::fold_pass;

fn run_transform(input: &str) -> Result<String, String> {
    let error_buffer = Arc::new(Mutex::new(String::new()));
    let cm: Lrc<SourceMap> = Default::default();
    let comments = SingleThreadedComments::default();

    let handler = Handler::with_emitter_and_flags(
        Box::new(StringEmitter { buffer: error_buffer.clone() }),
        HandlerFlags { can_emit_warnings: true, ..Default::default() },
    );

    let syntax = Syntax::Typescript(TsSyntax { tsx: true, ..Default::default() });

    let output = GLOBALS.set(&Globals::new(), || {
        HANDLER.set(&handler, || {
            let fm = cm.new_source_file(
                FileName::Real("input.tsx".into()).into(),
                input.to_string(),
            );
            let lexer = Lexer::new(syntax, Default::default(), StringInput::from(&*fm), Some(&comments));
            let mut parser = Parser::new_from(lexer);
            let program = parser.parse_program().expect("parse failed");

            let program = program
                .apply(resolver(Mark::new(), Mark::new(), true))
                .apply(fold_pass(MyTransform::new()))
                .apply(hygiene::hygiene())
                .apply(fixer::fixer(Some(&comments)));

            to_code_default(cm.clone(), Some(&comments), &program)
        })
    });

    let errors = error_buffer.lock().unwrap().clone();
    if errors.is_empty() { Ok(output) } else { Err(errors) }
}
```

## How `swc_ecma_transforms_testing` does it (and why we don't use it)

The `testing_transform` feature of `swc_core` pulls in `swc_ecma_transforms_testing`, which provides `test!()` and `test_inlined_transform()`. Under the hood:

1. `test!` is a macro that generates a `#[test]` fn calling `test_inlined_transform`.
2. `test_inlined_transform` uses `#[track_caller]` to derive the snapshot file path from the calling file location: `tests/__swc_snapshots__/{test_file}.rs/{test_name}.js`.
3. It delegates to `test_fixture_inner`, which:
   - Creates a `Tester` (wraps `SourceMap`, `Handler`, `Rc<SingleThreadedComments>`)
   - Sets `GLOBALS`, `HANDLER`, and `HELPERS` thread-locals
   - Parses input, applies the transform pass, then `hygiene()` + `fixer()`
   - Emits code via `Emitter` + `JsWriter` (manually, not `to_code_default`)
   - Compares output to a snapshot file using `NormalizedOutput::compare_to_file`
   - Supports `UPDATE=1` env var to overwrite snapshots

This is roughly 200 lines of code, but it pulls in a large dependency tree (the `testing` crate with graphical error reporters, the `swc_ecma_testing` crate with Node.js execution support, etc.). The actual transform logic can be inlined in ~60 lines (see `tests/common/mod.rs` in this repo).

### What `HELPERS` is (and why we skip it)

`swc_ecma_transforms_testing` also sets `HELPERS` (from `swc_ecma_transforms_base::helpers`). This thread-local tracks which runtime helpers a transform needs to inject (e.g., `_class_call_check`, `_inherits` for class transpilation). Our plugin doesn't use runtime helpers, so we don't set it. If your transform calls `helper!()` macros, you'll need:

```rust
use swc_core::ecma::transforms::base::helpers::HELPERS;

HELPERS.set(&Default::default(), || {
    // transform code here
});
```

## SWC error handling model

SWC transforms report errors as **side effects** through the `HANDLER` thread-local, not by returning `Result`. This is a deliberate design choice — the `Fold` and `VisitMut` traits don't have `Result` return types.

The flow:
1. Transform calls `HANDLER.with(|h| h.struct_span_err(span, "message").emit())`
2. The handler's emitter receives the diagnostic
3. Depending on `HandlerFlags`:
   - `continue_after_error: true` (default) — error is recorded, execution continues
   - `continue_after_error: false` — `abort_if_errors()` is called, which triggers `FatalError.raise()` → `panic::resume_unwind(Box::new(FatalErrorMarker))`
   - `treat_err_as_bug: true` — immediate panic via `bug!()`

**Caveat:** `FatalError.raise()` uses `resume_unwind`, not `panic!()`. The panic payload is a `FatalErrorMarker` struct, not a string. If you use `catch_unwind`, you won't get a human-readable message from the panic itself — you need to capture it from the emitter.

This is why our test helper uses a custom `StringEmitter`: it collects error text in a buffer, then checks the buffer after the transform completes. The `to_panic!` test macro uses `Result::expect_err()` on the buffered errors rather than `#[should_panic]`.
