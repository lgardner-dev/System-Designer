# Verification record — System Designer native source candidate

## Disposition

**Source-only handoff. Not a compiled, tested or installed native application.**

This environment had no `cargo`, `rustc`, or rustfmt. Toolchain and package acquisition attempts failed because the required download hosts were not reachable. The actual command `cargo test --all-targets` exited with status 127 (`cargo: command not found`); see `rust-attempt.txt`. It did not reach dependency resolution, compilation, or test discovery.

No native executable, Windows installer, Linux binary, or macOS bundle was built. No native GUI screenshot, installed-app smoke test, signing, notarization, dependency audit or cross-platform qualification is claimed. No GitHub workflow was run or repository modified.

## Executed evidence

| Check | Outcome | What it establishes |
|---|---|---|
| Existing-editor validation of self-design | PASS | The new JSON is accepted by the previously delivered System Designer model implementation |
| Every self-design component/level/subtree round trip | 51 / 51 PASS | The self-design can be exported and replaced through the existing editor's scope protocol |
| Existing-editor and golden-fixture checks, combined | 66 / 66 PASS | Includes self-design counts, scope round trips and reference fixture authenticity |
| Source/package checks | See `static-checks.json` | TOML/YAML/JSON syntax, shell syntax, local links, embedded resources, module/source paths, lexical delimiter balance; not Rust grammar or typing |
| Rust test functions authored | 94 | Named test functions present in the source; includes parameterized loops over more than one fixture |
| Rust test functions executed | **0** | No Rust test or native interaction passed in this environment |
| Native build / installer / installed behavior | **NOT RUN / NOT BUILT** | Must be established on a build machine before release |

`reference-checks.json` records the exact 66 existing-editor checks, the reference `core.js` hash and the self-design hash. `static-checks.json` records each source/package check and the authored-test inventory. `SOURCE-SHA256.txt` binds the source and data files shipped in this package. The source manifest is not a digital signature or an attestation of correctness.

The existing reference implementation was taken from the previously delivered `system-designer-package.zip`. It was used only as a development oracle and is **not bundled** in this Rust source package. The Rust project itself has no HTML, JavaScript or CSS source files. JSON golden fixtures preserve actual reference exports and expected replacement/retyping outputs; they do not substitute for running the Rust assertions against those fixtures.

## Authored Rust tests

`tests/core.rs` covers parsing and round trips, strict required/null fields, identities, ownership, locality, direction, exact contract versions, schema forms, projected boundaries, derived external labels, safe deletion, history and a normalized 2,048-level containment chain. It also expressly tests that more than eight nodes is valid.

`tests/exchange.rs` covers reference exports and hashes, Unicode, reference replacement/retyping outputs, stale local/ancestor state, read-only context/boundaries, preserved hidden internals, component-only replacement, new versus rewritten shared types, wrong targets, missing definitions, layout exclusion and atomic connection creation.

`tests/storage.rs` covers local save/open, prior-version backups, external changes, missing expected files, damaged files, invalid candidates, recovery round trips and I/O errors.

`tests/self_design.rs` covers the embedded application design, all three scopes, blank defaults, and the embedded initialization protocol. Canvas unit tests include cubic-curve hit testing and raw egui pointer-event scenarios for forward/reverse drag and cancellation. These tests **have not been compiled or run**.

## Review and known technical limits

The implementation was inspected against pinned eframe/egui 0.33.3 native `App::update`, modal and painter APIs and rfd 0.15.4. That documentation review cannot establish that the complete crate compiles. The source received a whitespace-only readability pass preserving lexer tokens; actual rustfmt has not run. The build scripts run rustfmt before tests.

No genuine Cargo.lock could be generated, so none is fabricated. The first build must resolve dependencies, preserve the resulting lock, and run the tests. Until a reviewed lock is adopted, dependency versions are not an established release baseline.

The native parser deliberately targets the current `system-designer-project` version-1 format and genuine current exporter output. Earlier product-specific identifiers need a deliberate export through the current editor first. Float/exponent-encoded version numbers are not accepted as integer versions. Deep field-schema JSON retains serde_json's parsing guard; normalized component-tree depth is a separate concern.

Project-file fingerprint checks detect ordinary external edits but do not eliminate every multi-process race. Same-directory replacement and syncing are not a universal power-failure guarantee. Unconfirmed form drafts and temporary gestures are outside recovery snapshots. Graphics, native clipboard, native file dialogs, accessibility, display scaling and performance remain unqualified.

## Next evidence gate

Run the supplied Rust tests on a development machine, resolve any compiler or behavior failures, and build the actual executable. Then execute `RELEASE-CHECKLIST.md` against the installed app on the intended coworker platform. The provided CI and installer files are recipes for obtaining that evidence, not evidence that it already exists.
