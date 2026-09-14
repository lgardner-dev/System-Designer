# Coherent editing contract

Assignment baseline and starting local/remote HEAD: `731bfef294c2c624def47d41a169b231713ffcbc`.
The working tree was clean and fetching the requested branch found no newer commits.
The owner's reported usability issues and zero diagnostics are release blockers.
This document records implementation decisions; acceptance remains pending until
qualification and the owner's walkthrough. Work is in the ordinary application.

## Interaction contract

Project settings is one global document command, independent of selection, scope
and layer. Opening, cancelling and saving its form preserve that view state.
Save details publishes metadata once; Ctrl+S outside dialogs writes the file.
Authoring dialogs use a title at left and operation-specific actions at top right,
with a separately scrolling body. Actions are intents consumed after current-frame
form input and validation. Escape dismisses an owned popup before the form; multiline
Enter inserts text. Cancelled/invalid drafts publish nothing.

Both canvases use signed world coordinates, cursor-centred zoom and explicit Fit.
Middle/background drag pans. A drag retains its grab offset and pointer capture
outside the viewport. Escape or window focus loss cancels an unpublished gesture;
view pan is retained. No new canvas gesture may start through another widget.
Keyboard nudge and explicit position editing use the same validated move command.
Edit and Enter remain explicit: double-click retains the existing local meaning
(flow step edits; an interface component enters its interior when present).

One logical endpoint resolves to one visible anchor. Painting, hit testing,
preview, final route, arrow, label and light consume that anchor and route.
Placement scores all neighbours, is deterministic, and freezes during gestures.
Input/output role is independent of geometric side. Control handles are derived
scope/step/role references; interface handles retain their exact existing ports.
Parallel, reciprocal and loop edges remain independently selectable. Overview
remains an explicitly aggregated, noneditable summary of exact interface edges.
Lights explain direction, preserve selection and never indicate execution.

Document controls are global, creation/connect controls belong to the current
layer, object controls belong to its inspector. Guidance is revisitable and does
not certify responsibility or erase draft issues. Information and control remain
different semantic layers over one authoritative Project/component tree.

## Compatibility decision

Use project version 3 for signed layout only. Versions 1/2 keep their existing
nonnegative finite validation and golden exports. Version 3 accepts finite
coordinates in [-1,000,000, +1,000,000] world units on both axes (f64 model;
f32 ephemeral scene; target <=0.5 logical-pixel endpoint agreement at 0.2–2.5 zoom).
Promotion occurs inside the first candidate needing negative coordinates, with
one Store publication and a nonmodal compatibility notice. Undo restores the old
version and coordinates. Opening a legacy project does not upgrade it. An
interface-only promotion creates no behavior. Existing @root reservation stays
explicit; no automatic renaming. A promotion that cannot validate fails intact.
No setter may demote v3. Only known versions 1/2/3 are accepted. Scoped packet
versions, layout exclusion, hashes and readonly context policy remain unchanged;
replacement preserves unrelated signed layout and document version. File save
retains the ordinary prior-valid-file backup. No lossy downgrade is offered.

## Responsibility map

Before: workspace -> mutable Designer orchestration; inspector -> modal layout
and all forms; flow/drawing -> flow projection + camera + gesture + outlines +
routes + edge styling; canvas -> interface projection + camera + gesture + edge
styling; canvas/geometry -> interface layout/ports AND common path mathematics.

Target: workspace -> document commands/scope orchestration; dialogs -> fixed
shell/action intents with form-owned validation; diagram -> immutable viewport,
shape/anchor/route, gesture mechanics and presentation; flow and canvas -> typed
semantic adapters; edit/Store -> complete-candidate validation/publication.
Shared mechanics take small data records, never Designer, Store or filesystem.
The ephemeral scene is not another semantic model. No framework replacement.
Real differences remain contracts versus conditions/outcomes, occurrence versus
component identity, approved shapes, and Overview aggregation.

Research rationale and precise baseline audit are retained in
[evidence/ui-coherence/assignment-audit.md](evidence/ui-coherence/assignment-audit.md).
The owner's task, not the cited heuristics, determines usable acceptance.

## Diagnostics and qualification

Recorded baseline toolchain: rustc 1.98.1 (48a229cea 2026-09-01).
Baseline Clippy: 25 library findings + 1 additional library-test finding + 37
behavior-test findings (63 distinct; repeated build reports are not added).
The original complete log remains in evidence/ui-coherence/baseline-clippy.log.
Configured rustc, Clippy (including unwrap_used), and rustdoc warnings must fail
qualification. No suppression, filtered success logs or toolchain downgrade.
Preserve Windows static CRT and preexisting encoded flags when composing CI.
Final evidence must distinguish headless tests, native synthetic input, OS tasks,
platform build/package checks and pending human acceptance. Do not merge/release.

## Implemented boundaries and first checkpoint

`ui/diagram/viewport` now owns both adapters' Fit, inverse transform and zoom;
`anchors` owns four-side scoring and true rectangle/diamond/ellipse intersections;
`routes` owns sampled cubic routes and stable lanes/loops; `paint` owns connector,
static arrow, caption, handle and light presentation; `interaction` owns press
ownership, focus cancellation and grab-offset displacement. The adapters retain
semantic eligibility and exact-identity selection. `flow/scene` supplies derived
step/role anchors; `canvas/geometry` retains component/boundary projection only.
Both freeze presentation during capture, and both route from displayed handles.
`edit/layout` publishes version-aware position candidates for drag, nudge and
position forms. No mechanics module takes Designer, Store or contract operations.

`dialogs` owns the fixed action shell and dispatch; form modules own candidates
and review gates. Actions are explicit typed intents consumed after input. Handoff
keeps removal confirmation in the header and review details ahead of raw JSON.
Project settings uses the same command from toolbar and inspector. Native pickers
remain native. Existing flow Cancel labels became the shared Cancel label; the
F2 static-arrow test now checks the shared stroked arrow's two wings instead of
the former flow-only filled triangle. No regression obligation was removed.

The mechanics/workflow checkpoint passes 226 all-target tests (86 library/UI,
140 integration/headless-domain); this is one run, not cumulative repeat counts.

## Warning repairs and durable gate

Qualified compiler/rustdoc/Clippy are pinned to Rust 1.98.1, the same version as
initial inspection. Repairs retain semantics: collapse nested guards without
changing evaluation order, space logical-negation assignments unambiguously,
remove an identical color branch, use fixed-size pixel chunks, and replace test
unwraps with named fixture/extraction invariants. No lint suppressions existed
in first-party source/tests and none were added. No dependencies were changed.

`tools/verify.py` composes `.cargo/config.toml` target flags with any existing
encoded or ordinary flags, appends `-D warnings`, and executes the full required
matrix with complete stdout/stderr logs. CI runs it on Linux, Windows and macOS,
archives logs even on failure, and retains installer packaging. The Rust minimum
remains 1.88; pinned qualification upgrades must be deliberate. The first explicit
1.98.1 activation emitted rustup's auto-install deprecation notice; installation
completed, and subsequent qualified cargo runs use the already installed version.
