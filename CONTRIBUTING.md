# Developing System Designer

## Set up

Install Rust stable; `rust-toolchain.toml` pins the channel and pulls in rustfmt
and clippy. The crate needs Rust 1.88 or newer (edition 2024).

* **Windows** also needs the Visual C++ build tools. `.cargo/config.toml` links
  the CRT statically, so the resulting binary runs without the Visual C++
  Redistributable — do not remove that setting.
* **Debian/Ubuntu** needs the X11/Wayland/GL development libraries installed by
  the Linux job in `.github/workflows/ci.yml`.

Desktop Windows builds additionally use the Windows SDK resource compiler
(installed with the C++ tools) through the build-only `embed-resource` crate.
Missing resource tooling fails the desktop build instead of silently shipping
an unbranded executable. Headless builds do not invoke the resource compiler.
No Node, bundler, image conversion tool, or runtime service is required.

```sh
cargo test --locked --all-targets
cargo run --locked --release --bin system-designer
```

An optional first argument opens a project on launch:

```sh
cargo run --locked --release --bin system-designer -- design/system-designer.project.json
```

`Cargo.lock` is committed. Build with `--locked` so you are testing the same
dependency versions as CI, and commit the lockfile whenever you change a
dependency.

## The core builds without a desktop

Everything except the UI compiles with `--no-default-features`, which drops
`eframe` and `rfd`. This is the fast path for working on the model, edits,
exchange or storage, and it is how the headless validator runs:

```sh
cargo test --no-default-features
cargo run --no-default-features --bin designer-check -- design/system-designer.project.json
cargo run --no-default-features --bin designer-check -- project.json proposed.scope.json
```

With one argument `designer-check` validates a project file; with two it
validates a returned scope packet against its project, which is the same check
the UI performs behind **Validate candidate**.

## Source layout

```
src/
  model/       typed records, structural validation, derived queries
  edit/        candidate edits, connection consistency, Store, undo/redo
  exchange/    canonical hashing, scoped export, safe scoped replacement
  storage.rs   project files, backups, recovery snapshots
  ui/          workspace, canvas, inspector, schema/contract forms, AI handoff
  main.rs      native executable          check.rs  headless validator
assets/        embedded initialization prompt and approved branding/icon family
design/        the application's own design, embedded and importable
tests/         integration tests plus JSON golden fixtures
packaging/     Windows NSIS and Linux self-extracting installer recipes
```

[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) maps those directories onto the
seven responsibilities in the application's own design, and explains the
invariants each one owns. Read it before moving a responsibility across a module
boundary.

## Invariants worth knowing before you change things

* **`Store` is the only publisher.** An operation builds a complete candidate,
  validates it, and publishes it. Failed validation must not mutate current
  state, undo, or redo. Never mutate a project in place behind the Store's back.
* **One semantic edit is one undoable action.** A connection that defines a
  contract, assigns both endpoints, propagates shared bindings and inserts the
  edge publishes once.
* **The public boundary is derived, never stored.** Child frames take their ports
  from the owning node; external neighbour labels come from actual parent edges.
* **Validation rejects unknown properties** rather than ignoring them, so adding
  a field to the format means updating the parser, the validator, `MODEL.md` and
  the fixtures together.
* **Canonical hashing is part of the format.** `src/exchange/canonical.rs`
  reproduces the version-1 convention — UTF-16-sorted keys, unchanged array
  order. Changing it invalidates every existing scope packet.
* **`unsafe` is forbidden** crate-wide, and `clippy::unwrap_used` warns.

## Tests

The suite is 94 tests and runs in well under a second; there is no reason to
skip it.

| File | Covers |
|---|---|
| `tests/core.rs` | Parsing and round trips, required/null field strictness, identity and ownership rules, locality, direction, exact contract versions, schema forms, derived boundaries, safe deletion, history, a 2,048-level containment chain, and that more than eight nodes is valid |
| `tests/exchange.rs` | Export bytes and hashes, Unicode, replacement and retyping outputs, stale base and ancestor state, read-only context and boundaries, preserved hidden internals, new versus rewritten shared types, wrong targets, missing definitions, layout exclusion, atomic connection creation |
| `tests/storage.rs` | Save/open, prior-version backups, external modification, missing and damaged files, invalid candidates, recovery round trips, I/O failures |
| `tests/self_design.rs` | The embedded application design, all three scopes, blank defaults, the embedded initialization prompt |
| `src/ui/canvas.rs` | Cubic-curve hit testing and raw egui pointer events for forward, reverse and cancelled drags |

`tests/fixtures/` holds golden JSON — real exports and their expected
replacement outputs, including Unicode cases. Treat them as a specification: if a
change makes a fixture fail, establish that the new bytes are correct before
updating it.

A passing headless pointer test establishes the state transition it exercises
and nothing more. It says nothing about native event delivery, GPU output, modal
rendering, file dialogs, clipboard permissions or accessibility — those need the
running application, which is what
[docs/RELEASE-CHECKLIST.md](docs/RELEASE-CHECKLIST.md) is for.

## Before you open a change

```sh
cargo fmt --all
cargo test --locked --all-targets
cargo clippy --locked --all-targets
```

CI runs the same three on Ubuntu, Windows and macOS, then builds the release
binaries, validates the embedded design with `designer-check`, and packages the
platform installers as workflow artifacts. Formatting and the tests gate the
build; Clippy is reported but does not fail it, and the crate currently carries
some existing lint warnings — avoid adding more.

If your change touches the on-disk or exchange format, update [MODEL.md](MODEL.md)
in the same commit — it is the specification `src/model` and `src/exchange` are
written against, not a description written after the fact. Changes to a
responsibility boundary should also be reflected in
`design/system-designer.project.json`, which the app can open on itself.

Distribution — signing, notarization and publication — is a deliberate owner
action, covered by [docs/RELEASE-CHECKLIST.md](docs/RELEASE-CHECKLIST.md). The
CI workflow produces artifacts, not public releases.
