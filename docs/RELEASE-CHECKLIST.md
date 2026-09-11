# Native release qualification

A green test suite qualifies the logic, not the installed application. Work through this checklist against a real install on every platform you intend to hand to someone else, and record what you actually observed — an item is not complete because the code that implements it exists.

## Build and dependency gate

Run `cargo fmt --all`, then `cargo test --locked --all-targets` and `cargo clippy --locked --all-targets` on a development machine, and build in release mode. Capture the exact compiler version, the committed `Cargo.lock`, dependency licenses, source revision and resulting binary hashes. If a dependency changed, review and commit the updated lockfile before treating the build as a baseline.

Resolve compilation errors and test failures before packaging, and review the Clippy findings — CI reports them without failing the build, so they do not gate a release on their own.

## Installed application

Record an observed outcome, artifact hash, operating system, architecture and display backend for every target being claimed. At minimum qualify the Windows installer the coworkers will use. The matrix recipes also target Linux and macOS, but they do not establish support by existing in the repository.

| Workflow | Required observation |
|---|---|
| Installation | Per-user install, launch from shortcut, uninstall, preserved project files; record signing status and any warnings |
| Offline operation | Installed executable starts, edits and saves without contacting a service; no prompt or design sidecar is required |
| Blank start | Empty user project and empty contract catalog, not the application's self-design |
| Self-design | File → Open application design exposes seven root components and navigable nested boundaries; Save As keeps the edited copy |
| Basic authoring | Add, rename and delete components; create and remove nested internals; edit purposes and port names |
| Wire gesture | Drag output→input and input→output; release on empty space cancels; click-click route remains available |
| Explicit contracts | New edge never silently chooses a type; choose existing or define a structured type; Cancel creates neither edge nor type |
| Existing wires | Change endpoints/type/label, inspect impact, require shared-binding consent, preserve independent channels |
| Structured forms | Strings, integers, numbers, booleans, enums, arrays, nested objects and field descriptions; invalid structures rejected |
| Boundary behavior | Ports appear on the owning frame; labels track parent connections; no outside component becomes a fake internal node |
| History | Each confirmed semantic edit is one undoable action; invalid changes preserve both current state and redo |
| Three AI scopes | Export/import component, level and subtree; preserve outside data, reject wrong targets/stale base/changed context/shared definitions |
| Prompt | Copy into a fresh chat from inside the executable; prompt contains exact exchange rules and no user project content |
| Project files | Open/Save/Save As, save-close-reopen equality, dirty indicator, Save/Discard/Cancel on New/Open/Close |
| File failure | Unwritable location, damaged existing file, external edit, missing expected file, failed recovery: clear error and no invented success |
| Recovery | Interruption with unsaved work, restore-as-copy, visibly preserve invalid recovery bytes, no foreign-session cleanup |
| Display and input | Target DPI/scaling, resize, long text, keyboard-only non-canvas alternatives, native clipboard and file dialogs |
| Scale | Representative real projects; inspect navigation, frame responsiveness, memory and loading; no arbitrary “large project” certification |

A passing headless pointer-input test establishes only the state transition exercised by that test. It does not establish correct native event delivery, GPU output, modal rendering, native file dialogs, accessibility or clipboard permissions.

## Distribution gate

The Windows NSIS recipe is unsigned. The macOS bundle/DMG recipe is unnotarized. Signing, notarization and workplace installation policy remain release concerns. Do not instruct coworkers to bypass their employer's controls. No telemetry, updater or public publication is configured.

The GitHub Actions workflow builds downloadable **workflow artifacts**, not published Releases. Publication, hosting, license review and installer signing are deliberate owner actions.

## Evidence record

For an actual release, record the source revision and Cargo.lock hash, compiler, OS/backend, test output, binary/installer hashes, manual workflow observations, failures, limitations and disposition. Preserve negative results. A test run does not automatically prove the usefulness or semantic quality of a user's design.
