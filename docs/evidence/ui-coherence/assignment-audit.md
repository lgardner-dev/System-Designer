# Research and source audit

## Evidence and limits

Source review is pinned to `731bfef294c2c624def47d41a169b231713ffcbc` on `work/add-logic-diagramming`. The branch reference was read through the GitHub connector. This pass inspected production source and the supplied screenshot; it did not run a newly changed binary or independently rerun Rust tests. Earlier successful CI is not evidence that the usability issues below are absent.

Source links below are immutable commit links. Line numbers are pointers into that revision, not promises about a subsequent refactor.

## Confirmed causes

### U1 — Project metadata is incorrectly discoverable through a particular view

The root/no-selection branch of `src/ui/inspector.rs` offers `Edit project purpose`, while `src/ui/flow/inspect.rs` supplies a different flow inspector. The global toolbar in `src/ui/workspace.rs` does not supply a corresponding project-settings command. Thus editing document intent requires navigating to the interface inspector rather than operating on a document-level command.

- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/src/ui/inspector.rs
- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/src/ui/flow/inspect.rs
- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/src/ui/workspace.rs

Decision: one document command, reachable in the global toolbar from any scope/layer/selection. A root-inspector shortcut may invoke the same command; do not keep a second form or update path. The breadcrumb still navigates; do not silently repurpose it as an edit action.

### U2 — Modal actions are inside the scrolling content

`Designer::show_dialog` in `src/ui/inspector.rs` places the whole variant match inside a vertical ScrollArea. The Project variant renders its fields first and its Save/Cancel row afterward. Moving that row earlier inside the same ScrollArea would still allow it to scroll away. The modal also imposes minimum dimensions that need checking at small logical viewport sizes/high UI scale.

Decision: fixed title/action header; separately scrolling form body. Project purpose gets Cancel and Save details at the top right, as the owner explicitly requested. Other authoring dialogs use the same shell, with operation-specific actions and their existing validation/review gates. Native OS file pickers remain native.

### U3 — The invisible wall is a world-coordinate constraint

Both `src/ui/flow/drawing.rs` and `src/ui/canvas.rs` clamp drag results:

```rust
x: (initial.x + delta.x as f64).max(0.0),
y: (initial.y + delta.y as f64).max(0.0),
```

Both model validators require nonnegative positions. `MODEL.md` explicitly specifies nonnegative version-1 layout coordinates. Consequently removing only the UI clamp makes publication invalid.

A point is mapped to the screen by `area.min + pan + world * zoom`. After Fit, the world origin can be near the middle of the visible canvas. Refusing to cross world zero therefore produces a visible wall inside a perfectly usable canvas. This is an inference directly from the inspected transform and clamp, not a claim of new native reproduction.

The interface `geometry::frame_for` fixes its minimum corner to `(40, 40)` while expanding only right/bottom. Signed positions also require correcting enclosing frame/bounds derivation, not just dragging.

- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/src/ui/flow/drawing.rs
- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/src/ui/canvas.rs
- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/src/model/validation.rs
- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/src/behavior/validate.rs
- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/src/ui/canvas/geometry.rs
- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/MODEL.md

Decision: signed world coordinates, no origin wall; explicit compatibility handling for persisted coordinates. Do not work around the problem by continually moving the camera, rebasing untouched siblings, or telling the user to press Fit.

### U4 — Sharing a path type has not produced a shared connector system

The interface renderer uses `geometry::Scene` with real `Anchor` endpoint objects and four-sided port placement. The flow renderer imports the same `Path` and `motion`, but implements its own route construction, painting, arrowheads, colors, labels, gestures, and zoom/fit logic.

In particular, the flow renderer paints/hit-tests handles at `left_center()` and `right_center()`. Its completed routes instead intersect node outlines toward peer/lane positions. A connection preview follows a handle; the completed connection can attach somewhere else. The user's screenshot visibly exhibits this mismatch.

Interface edges use a stroked arrow near the destination; control edges use a filled triangle at the endpoint. Selection colors, label backing, and repaint gating also differ. These differences do not encode an identified requirement for the two relationship kinds.

Decision: one common anchor -> route -> display/pick/animate pipeline and shared style. Different node shapes and relationship labels remain meaningful; inconsistent mechanics do not.

### U5 — Input ownership is a cross-cutting risk

Both canvas handlers inspect raw pointer presses gated largely by geometric `area.contains(cursor)`. The reviewed flow code does not use the canvas response to establish that an overlapping widget owns the press. A popup drawn over canvas coordinates must not also clear a selection or start a drag underneath. The earlier Lights-popup selection loss is a related reported case; reproduce it before claiming its exact cause is fixed.

Decision: explicit gesture ownership and capture. Prevent new gestures through popups/modals; once captured, keep a legitimate drag until release/cancel. Do not gate all ongoing drags on current hover and accidentally break capture outside the viewport.

### U6 — Warning policy explicitly permits current debt

`Cargo.toml` forbids unsafe code but configures `unwrap_used` as a warning. `CONTRIBUTING.md` says Clippy is reporting-only and existing warnings may remain. The CI command does not promote warnings to errors. The committed Clippy log includes suspicious assignment formatting, collapsible conditionals, and other findings. They are not all compiler warnings; the new gate must address rustc and Clippy separately.

- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/Cargo.toml
- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/CONTRIBUTING.md
- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/.github/workflows/ci.yml
- https://github.com/lgardner-dev/System-Designer/blob/731bfef294c2c624def47d41a169b231713ffcbc/docs/evidence/control-flow-review/clippy.log

Decision: fix current warnings and make recurrence fail qualification. Do not rewrite historical evidence to pretend previous builds were warning-free.

## Research and what we adopt

The following are primary-source heuristics, design-system guidance, library documentation, and software-design guidance. They inform the proposal; none proves our precise layout is optimal. The owner's task walkthrough remains the decisive usability test.

### R1 — Nielsen: consistency, recognition, user control

Source: Jakob Nielsen, Ten Usability Heuristics for User Interface Design.
https://www.nngroup.com/articles/ten-usability-heuristics/

Relevant guidance: use consistent actions and terms, expose useful choices rather than requiring recall, and provide understandable escape/recovery paths.

Our application: users should learn one move/connect/select interaction, see a document-settings command where they work, and cancel an unfinished gesture/form without changing the project. These are product choices informed by the heuristics, not numerical guarantees about cognitive load.

### R2 — Shneiderman: coherent interactions and feedback

Source: Ben Shneiderman, The Eight Golden Rules of Interface Design.
https://www.cs.umd.edu/~ben/goldenrules.html

Relevant guidance: consistent sequences, informative feedback, reversible actions, and reduced memory burden; the author explicitly calls for domain-specific validation.

Our application: compare entire tasks across both layers, not just isolated primitives. One release should be one undoable move; toggling a display setting should preserve what the user is working on.

### R3 — GNOME: action-dialog headers

Source: GNOME Human Interface Guidelines, Dialogs.
https://developer.gnome.org/hig/patterns/feedback/dialogs.html

Relevant guidance: action dialogs have a heading and affirmative/cancel actions in their header; labels should identify the operation, focus should be deliberate, and Escape should cancel where available.

Our application: a pinned top-right Cancel/Save-details cluster with a scrolling body. The exact top-right grouping is the owner's chosen application convention, not a universal research result. Plain Enter in a multiline purpose field must keep inserting text rather than unexpectedly submitting.

### R4 — GNOME and NN/G: progressive disclosure

Sources:
https://developer.gnome.org/hig/principles.html
https://www.nngroup.com/articles/progressive-disclosure/

Relevant guidance: keep frequent actions available, and reveal secondary detail when needed rather than overwhelming the primary task.

Our application: document actions in the shell, layer actions with the layer, selected-object editing in the inspector. Put IDs/provenance behind accessible details, but never hide destructive changes or actionable blocking errors. A dismissible next-action hint is preferable to a mandatory wizard for every edit.

### R5 — W3C: targets, dragging alternatives, visible focus

Sources:
https://www.w3.org/TR/WCAG22/
https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum
https://www.w3.org/TR/wcag2ict-22/

Relevant guidance: provide adequately sized/spaced targets, alternatives to dragging, and perceivable focus. WCAG2ICT explains applying the guidance to non-web software and density-independent measurements.

Our application: screen-sized handle hit targets, keyboard/form alternatives, predictable focus in dialogs, and no canvas-zoom shrinking of toolbar text. A 24-logical-pixel target/spacing baseline is an engineering target informed by this guidance, not a declaration of WCAG conformance. Test the native accessibility behavior separately.

### R6 — React Flow: explicit handles and a shared edge renderer

Sources:
https://reactflow.dev/learn/customization/handles
https://reactflow.dev/learn/customization/custom-edges

Relevant precedent: edges reference specific source/target handles; a reusable base-edge surface paints a supplied route. Custom node content need not reinvent connection mechanics.

Our application: a stable, typed presentation anchor shared by drawing, hit-testing, preview and final routing. Borrow the separation of responsibilities, not React, JavaScript, HTML, CSS, a webview, or this library as a dependency.

### R7 — Robert C. Martin: single responsibility

Source: The Single Responsibility Principle.
https://blog.cleancoder.com/uncle-bob/2014/05/08/SingleReponsibilityPrinciple.html

Relevant guidance: separate responsibilities that change for different reasons and keep related policy together.

Our application: common graphical interaction mechanics depend on a small scene description; semantic adapters retain different domain rules. A geometry utility should not need the whole mutable Designer or know how a contract is accepted. Splitting files while leaving those dependencies intact would not resolve the coupling.

### R8 — Rust: enforce diagnostics, do not relabel them

Sources:
https://doc.rust-lang.org/clippy/continuous_integration/index.html
https://doc.rust-lang.org/rustc/lints/levels.html
https://doc.rust-lang.org/cargo/reference/environment-variables.html

Relevant guidance: Clippy recommends `-D warnings` in CI; `RUSTFLAGS` carries compiler flags and `RUSTDOCFLAGS` carries rustdoc flags. Lint levels are configurable, so suppressed diagnostics are not evidence of a repaired issue. Cargo normally caps dependency lints; a clean first-party run does not lint the entire dependency universe.

Our application: enforce a supported matrix of native, headless, tests, docs, and release builds. Fix emitted first-party warnings. Address emitted dependency/build-tool warnings rather than filtering logs. Preserve existing Windows target flags when arranging the warning gate.

## Architecture conclusion

The confirmed issue is incomplete separation: shared path mathematics and animation are nested under an interface-specific canvas, while behavior draws and interacts independently. There is also model-specific scene construction inside that geometry module. This supports extracting the genuinely shared presentation responsibilities. It does not establish that every module in the repository violates SOLID.

Do not turn the fix into a general plugin framework, a new GUI engine, or a generic semantic graph format. Two typed adapters and small shared functions/records are sufficient unless implementation evidence establishes another need. The core Project/Store, exact contracts, behavior identities, and scope protocol remain the semantic authority.
