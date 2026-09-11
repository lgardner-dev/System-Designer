# System Designer — native Rust rewrite

A native desktop editor for recursive components, typed ports, local connections,
and narrowly scoped AI collaboration. The UI is written in Rust using egui/eframe;
there is no webview, HTML, JavaScript, CSS, backend service, or AI API integration.

**Delivery status: source candidate, not a verified native release.** This package
was authored in an environment without Rust/Cargo and without working package
downloads. Rust compilation, Rust tests, the native UI, and installer execution
have NOT been run here. No executable or ready-to-install installer is included.
See [verification](docs/VERIFICATION.md) for the exact evidence and limitations.
Do not distribute a coworker release until the build and installed-app smoke
checks pass on the intended platform.

## Build and run

Install Rust stable on the development machine. Windows also needs the Visual C++
build tools. The application user will not need Rust or these build tools.

```sh
cargo generate-lockfile
cargo fmt --all
cargo test --locked --all-targets
cargo run --locked --release --bin system-designer
```

An optional first argument opens a project:

```sh
cargo run --locked --release --bin system-designer -- design/system-designer.project.json
```

For core-only verification without the desktop dependencies:

```sh
cargo test --no-default-features
cargo run --no-default-features --bin designer-check -- design/system-designer.project.json
cargo run --no-default-features --bin designer-check -- project.json proposed.scope.json
```

On Debian/Ubuntu, the desktop build may need the development libraries listed in
`.github/workflows/ci.yml`. The native renderer uses OpenGL through eframe's Glow
backend, with X11 and Wayland enabled. None of these target configurations has
been qualified in this delivery.

A dependency lockfile is deliberately not fabricated. Generate it during the
first actual build, inspect it, and commit the resulting `Cargo.lock`. The supplied
CI currently resolves dependencies and captures the real lockfile. After adopting
one, replace its resolve step with a locked fetch/build policy.

## What is in the rewrite

* Native tree, breadcrumbs, selected-level canvas, component/edge inspector,
  pan, zoom, fit, deterministic layout, and drag/click connection gestures.
* A real contract dialog for new wires; choose or define a contract explicitly.
  Existing wires have editable endpoints, contract, and label. Shared-port
  changes require visible impact confirmation.
* Primitive, enum, array, and nested object schema forms. Unconnected draft ports
  may be unassigned. Mirrored boundaries remain one physical port identity.
* One authoritative typed project in `Store`, validated candidate publication,
  safe recursive deletion, and 50-step session undo/redo.
* Actual Open/Save/Save As, external-file fingerprint checks, a prior-version
  `.bak`, independent recovery snapshots, and unsaved-work guards.
* Embedded initialization prompt and embedded self-design model, using
  `include_str!`. No sidecar resource is needed at runtime.
* Original version-1 project and three-scope exchange format. Returned scope
  packets have separate Validate and Apply actions, with revalidation at Apply.

These describe the supplied implementation, not verified execution results.

## Explore the application's own design

The file `design/system-designer.project.json` is a normal System Designer project.
It can be opened now in the earlier working editor. The native source also exposes
it through **File → Open application design**, as an unsaved copy.

It contains seven top-level components and eight local systems in total. No level
has more than seven immediate nodes. The architecture is a responsibility model,
not a separately implemented message bus: interface schemas explicitly identify
where they are explanatory projections of typed, borrowed Rust values. Source
paths in component purposes map the tree to this repository. See
[architecture rationale](docs/ARCHITECTURE.md).

New projects still start blank with an empty catalog. The self-design is an
optional example, not a preloaded product-specific default. Around eight local
components is guidance; the validator also permits larger coherent systems.

## Editing

Add a component, define its purpose, and give it input/output ports. Decompose only
when an independently meaningful responsibility or interface justifies another
level. Double-click a component with internals to enter it. Owner ports appear on
the containing boundary; outside neighbors are labels derived from parent edges.

Drag between two ports in either direction to open the contract dialog. Clicking
ports is also supported, and **Connect ports** provides a form-based route. No new
wire exists until confirmation. Double-click a wire or select **Change contract /
endpoints** to edit it.

Connected ports that share a channel must keep one exact contract ID/version.
Retyping can include fan-out and mirrored child-boundary connections. Review the
impact list rather than treating it as an independent edge-only property.

## AI collaboration — the app contains everything

**AI handoff → Initialize chat** provides the complete copyable prompt. It explains
the design approach and exact JSON representation and contains no project data.
Then export the smallest relevant scope:

| Scope | Editable | Preserved |
|---|---|---|
| Component | Selected node | Siblings, edges, deeper internals |
| Level | Immediate nodes and connections | Existing hidden-child ownership and internals |
| Subtree | Selected level and all descendants | Owner boundary, ancestors, unrelated branches |

Return to the same level, load the complete returned packet, select **Validate
candidate**, review it, then **Apply validated changes**. The app rejects changed
read-only context, stale base tokens, missing definitions, cross-level edges,
outside identity collisions, and silent rewrites of existing shared types.

Copying work information into another service remains the user's deliberate
separate action. The executable does not send it anywhere.

## Files, recovery, and limits

Ctrl/Cmd+S saves the actual project file. Saving preserves the previous valid file
as `<name>.bak` and replaces through a synchronized same-directory temporary file.
Changed or damaged existing files are not silently overwritten. Fingerprints
detect ordinary external edits; they are not a race-free interprocess lock.
Do not edit the same project concurrently in several app instances.

Recovery snapshots cover published project edits, not unconfirmed form drafts or temporary gestures. They are isolated per session and written when dirty. Recovery
opens an unsaved copy, never silently overwrites the original path, and preserves
unknown or damaged recovery files. Current-session recovery is removed after
successful save or deliberate discard/clean close. Other recovery copies are
retained. Keep independent backups of important work.

Containment has no fixed depth or node-count limit. Imported deeply nested *field
schemas* are subject to serde_json's JSON nesting guard; that is separate from the
flat normalized component tree. Structural validation is not semantic validation,
a runtime payload validator, a formal acceptance system, or proof of good design.
Large-project performance and accessibility remain unqualified.

## Installers and CI

Windows: with Rust, Visual C++ tools and NSIS available on the build machine:

```powershell
.\packaging\windows\build.ps1
```

The script runs tests before building an unsigned per-user NSIS installer in
`dist/`. It embeds only the compiled application. Uninstall preserves user files
and recovery data. No Windows installer was built or installed in this delivery.

Linux: `./packaging/linux/build.sh` produces a tarball with a per-user install
script and desktop entry after successful tests/build. It is not a universal
AppImage and does not bundle system graphics libraries.

`.github/workflows/ci.yml` defines Windows/Linux/macOS builds and installer or
bundle artifacts. It does not create a repository or publish a public release;
none of its jobs have been executed here. macOS signing/notarization and Windows
signing are not configured. Inspect the real outputs and follow workplace policy
before sharing installers. The [release checklist](docs/RELEASE-CHECKLIST.md)
covers native save/reopen, recovery, drag behavior, scoped exchange and upgrades.

## Source layout

```
src/
  model/       typed records, validation, derived queries
  edit/        candidate edits, connection consistency, Store/history
  exchange/    canonical tokens, scoped export, safe replacement
  storage.rs   native project files and recovery
  ui/          workspace, canvas, inspector, schema/contract forms, AI handoff
  main.rs      native executable
  check.rs     headless validator
assets/        compile-time initialization prompt
design/        importable application design
tests/         Rust tests plus JSON golden fixtures from the previous editor
packaging/     installer recipes (not prebuilt installers)
```

MIT license for the supplied source. Dependency notices and a dependency/security
review belong in the actual release process; no such audit is claimed here.
