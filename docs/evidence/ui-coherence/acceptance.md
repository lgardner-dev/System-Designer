# Acceptance — coherent production UI

All criteria below apply to the ordinary `system-designer` executable, not a study viewer. The owner's walkthrough is paused and the merge gate is open. Existing tests are necessary regression coverage, not substitutes for these tasks.

Use scratch files. Do not modify or reconstruct the user's actual Foundry process without its source. A fixture titled “Foundry Method — Process Design” with the starter flow and two unconnected placeholder components may reproduce the supplied screen, but label it as a UI fixture, not an accepted Method design.

## A. Project purpose and modal actions

**A1. Layer/scope independence.** At root and at a nested/leaf component, in both Control Flow and Interfaces, with no selection and with selected elements, invoke the same global Project settings command. It must target the document name/purpose, not the selected component. It must not navigate or switch layers. Open/Cancel preserves owner, camera, selection, layouts, dirty state, history, and generation.

**A2. Save correctness.** Change the purpose, including the last character immediately before clicking Save. One save publishes one metadata edit; Undo/Redo restores exact values. Failed validation retains draft text. Save/reopen persists the result through the normal document workflow. Changes in project intent can legitimately stale scope packets whose context includes that intent; do not hide that currentness effect.

**A3. Fixed header.** Fill the purpose with enough text to require scrolling. At body top, middle, and bottom, Cancel and Save details remain top-right, visible and usable. Test 1280x720, approximately the user's 1600x960 frame, and 1920x1080, including enlarged UI/OS scale. Bound modal dimensions by available logical space; do not shrink its text to solve overflow. Other core authoring/review dialogs use the shared shell and retain operation-specific gates.

**A4. Focus and cancellation.** Tab/Shift-Tab visits usable controls; focus never disappears behind a modal. Multiline Enter adds a newline. Escape first dismisses the owned popup/gesture where applicable, then cancels the appropriate uncommitted form; it never also deletes/changes background state. Ctrl+S outside the modal retains its document meaning. Save does not lose same-frame text input.

## B. Free movement and compatibility

**B1. The original wall.** In both layers press Fit, then drag a node up and left beyond the fitted world origin while remaining inside the visible canvas. Move near each usable canvas corner and into negative world X, Y, and both. Pointer-to-grab-point offset stays continuous. No clamp, teleport, canvas-wide rebase, or automatic fit masks the operation.

**B2. Transform parity.** Repeat at zoom 0.2, 1.0 and 2.5 and several positive/negative pan offsets. Round-trip world->screen->world within a documented tolerance. Repeat after scope/layer navigation and viewport resizing. Pan/zoom/selection are not document edits. Deliberate move is exactly one edit; cancelled move is none.

**B3. Nested boundary correctness.** Drag children above/left of zero in a non-root interface system. Its frame encloses its real children with usable padding; inherited ports still refer to the actual owner ports and retain effective direction. No invented root ports or cross-level wires. Include empty scopes and sparse saved positions.

**B4. Persistence.** Save, close, reopen and recover signed positions in both layers. Run extraction at negative coordinates and retain F1's nonoverlap/relative-position obligations. Verify ordinary unchanged v1/v2 records remain valid and unchanged on viewing. New coordinate-policy promotion is atomic and undoable, with no behavior invented for an interface-only project. Edits on new-version projects cannot demote them through legacy setters.

**B5. Transfer.** Behavior-only and interface-only replacements preserve unrelated signed layouts, types, identities, and project version. Existing golden packet hashes remain unchanged under their original conventions. Missing payloads, stale bases and altered readonly context remain rejected. Unknown versions and nonfinite coordinates remain invalid. No silent downgrade.

**B6. Capture.** Start dragging on a legitimate node, cross outside the viewport, release/cancel, and return. No stuck gesture. Starting a press on a popup/menu/toolbar/modal must not start a canvas gesture even if its screen coordinates overlap the canvas. Test both editors with raw full-workspace input, not just directly calling command methods.

## C. Exact ports and common connectors

**C1. Endpoint agreement.** For every exact rendered edge, assert that its first/last route point equals its resolved visible source/target handle position, in both world and screen coordinates, within a small stated floating-point tolerance (target <=0.5 logical pixel at supported zoom). Check the actual drawn/hit-tested handle, not merely “some point on the outline.” Cover rectangle, diamond, terminator and subprocess shapes, and boundary frames.

**C2. All sides and roles.** Exercise input and output placement on top, right, bottom and left. A source remains a source regardless of its side. Drag out->in and in->out and use click-to-connect/form alternatives. All reach the same semantic operation. Invalid pair/drop cancels or explains the issue without mutation. New interface wires require an explicit contract choice; control wires open condition/outcome editing, not a fake contract.

**C3. Stability.** Shared/fan-out ports resolve to one visible anchor for all incident wires. During node/connection drags their side/order does not jump. Relabeling/reordering arrays cannot change semantic endpoints. After publication every connection remains attached to its displayed handles. Any deterministic placement update is presentation only.

**C4. Shared path service.** Two/three parallel transitions, reciprocal transitions, distinct ports on the same component pair, and repeated self-loops remain separately selectable with readable labels. All lanes start/end at their own handles. F2 must continue passing. Test shape and layout combinations, small and large distances, pan/zoom and negative coordinates.

**C5. Visual parity.** Matched neutral scenes in both layers use the same connector stroke, hover/selected/dim styling, arrow treatment, label background/placement policy and bloom. Different labels/shape semantics are expected; arbitrary renderer drift is not. Interface Overview remains visibly aggregated and does not pretend a summary is an exact editable wire.

**C6. One geometry truth.** The same final route drives painting, nearest-edge selection, arrowheads, captions and lights. Test it through shared functions and through both adapters. Connection-preview geometry uses the same anchor resolution and does not snap to unrelated outline points on completion.

**C7. Lights and events.** With one exact edge selected, change Lights among Off/Selected/All and close the dropdown. Preserve selection, scope, camera and generation. Static arrows remain with lights Off. With lights enabled, verify source->destination travel on the selected displayed route. Off/inactive/minimized/modal-obscured states must not schedule unnecessary continuous animation. Do not infer execution from the light.

## D. Guided task and code boundaries

**D1. Context.** Global controls concern the document; layer-local controls concern the current editor; selected-object controls concern that object. A flow user should not accidentally create an unrelated interface component through an indistinguishable primary action. Essential alternatives remain available, not removed.

**D2. Identity.** Extract a responsibility, inspect it in Interfaces, follow its uses back to Control Flow, and enter its interior. Action-first labels and canonical component names expose the relationship. Repeated calls keep independent occurrences. A local decision is not automatically a new component.

**D3. Honest drafts.** Guidance identifies a useful next action, not a mandatory bureaucracy or a fake completeness percentage. Information review is not automatically accepted from a text field. Keep unassigned requirements visible and incomplete drafts saveable. Preserve F3's exact changes, newly introduced issues, stale revalidation and destructive-content confirmation.

**D4. Accessibility/usability.** Test readable UI scale separate from canvas zoom, practical hit targets, keyboard/form alternatives for drag operations, and visible focus. Add a pointer-accessible move alternative or position control as well as keyboard nudging. Show why an action is unavailable. Do not require a tooltip, raw JSON edit or assistant explanation for ordinary editing.

**D5. Responsibility boundaries.** Show the before/after module dependency map and meaningful changes. Common viewport/routing/interaction/rendering must not depend on mutable Designer, Store, contract acceptance, or disk I/O. Layer adapters use the common mechanics. Domain/model/edit/exchange still compile without the desktop feature. Small shared immutable data/intent records are not a second project model. No unused interface methods or speculative framework.

## E. Quality and evidence

**E1. Warning gate.** All commands in CODEX-BRIEF pass on their applicable matrix with zero emitted first-party compiler/configured-Clippy/rustdoc warnings. Inventory and resolve any emitted dependency/build-tool warnings. No new suppression, filtered logs, ignored error codes, feature deletion, or toolchain downgrade to hide debt. Preserve Windows target/CRT flags and locked dependencies. Demonstrate the gate fails injected sample warnings in a temporary checkout, then remove them.

**E2. Regression accounting.** Report actual test counts and platform results, not sums of repeated runs. Existing 215 all-target tests and 136 headless tests are baseline evidence, not mandatory final counts; refactoring may reorganize them, but all previous obligations must remain covered. Explain removed/replaced tests individually. Tests of outline geometry alone are insufficient for handle agreement.

**E3. Native evidence.** Capture the ordinary app using an exact source revision and binary hash. At least one supported native environment must exercise the full flow below, with its display server, GPU/software renderer, scale, and any mapping/focus accommodation disclosed. Where both Wayland and X11 are available, exercise both rather than implying one qualifies the other. Build/package checks on Windows/macOS do not establish GUI usability on those platforms; record any gap.

**E4. Human acceptance.** Final status can be “implementation and automated/native checks passed; owner's walkthrough pending.” Do not claim the user accepted a workflow from a passing unit test. No merge/release until the blocking task cases and owner's review are satisfied.

## Native acceptance journey

Run the ordinary app from the final branch with a scratch project. From Control Flow, edit the project purpose with a long text and save using the visible top-right action. Draw/rename the starter steps. Fit, then move nodes beyond both world axes and save. Connect using the visible handles in forward and reverse pointer directions; inspect the exact final anchors. Use a same-endpoint accepted/rejected fixture and select each path with lights Off. Change Lights and verify selection survives.

Declare input/output requirements, extract one meaningful responsibility, inspect it in Interfaces, refine its contract, and return to the same call occurrence. Enter its behavior, move within a signed-coordinate child scope, and use Back. Inspect a scoped proposal that removes one return; verify disclosure, then cancel. Reopen the settings from that child without losing context. Save/reopen both layers. Perform one Undo/Redo for the most recent real edit.

No step may require switching to Interfaces to edit document purpose, scrolling to find the modal's main actions, pressing Fit to get around an origin barrier, manually reconnecting a finished edge to its displayed dot, or remembering a different connector interaction for the second layer.
