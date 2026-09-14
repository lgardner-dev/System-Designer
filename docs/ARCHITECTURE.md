# Application design and source map

System Designer is one Rust crate producing a native editor and a headless validator. The authoritative explorable architecture is `design/system-designer.project.json`; the application embeds those bytes and opens them as an unsaved example. New projects remain blank.

## Responsibility tree

| Root component | Owns | Main source locations |
|---|---|---|
| Workspace | Native composition, document lifecycle, navigation and unsaved-change decisions | `src/main.rs`, `src/ui/mod.rs`, `src/ui/workspace.rs` |
| Canvas | Local projection, View/Focus/Arrange controls, drawing, picking, motion and gestures | `src/ui/canvas.rs`, `src/ui/canvas/` |
| Authoring forms | Component and contract drafts, exact connection consent | `src/ui/inspector.rs`, `src/ui/contracts.rs` |
| Edits and history | Validated candidates, shared-port consistency, authoritative Store and undo/redo | `src/edit/` |
| Design model | Typed records, structural validation and derived hierarchy queries | `src/model/` |
| AI handoff | Embedded prompt, bounded exports and safe replacements | `src/ui/handoff.rs`, `src/exchange/`, `assets/initialization.txt` |
| Local files | Validated reads, guarded replacement, backup and recovery | `src/storage.rs` |

The model contains eight systems and 38 components. The root has seven components; Canvas has eight immediate responsibilities, and no level exceeds eight. This is a design guideline, not an enforced project limit. These boundaries do not imply separate crates, services or a runtime message bus.

## Unified Canvas decomposition

| Responsibility | Implementation |
|---|---|
| Geometry projection | `canvas/geometry.rs`: measured Detail cards, one anchor per physical port, derived boundaries, common Path geometry |
| Viewport and navigation | `canvas.rs`, `canvas/focus.rs::Viewport`: pan/zoom, fit, remembered level cameras |
| Drawing and direction lights | `canvas.rs`, `canvas/motion.rs`: exact or summary routes, static direction, illustrative pulses |
| Exact hit testing | `canvas.rs`, `geometry::Path::distance`: select the displayed route or exact endpoint |
| Gestures and controls | `canvas.rs`: frozen attachment choices, explicit contract dialogs, validated completed edits |
| Overview projection | `canvas/overview.rs`: counted ordered-pair summaries retaining real edge IDs |
| Focus and exact tracing | `canvas/focus.rs`: exact one-hop membership, physical boundary navigation and Back trace |
| Connection-aware arrangement | `canvas/arrangement.rs`: temporary SCCs/layers, bounded Detail-size measurement, current-level position candidates |

The responsibilities share focused modules where appropriate; they are not eight separate dispatch services. No new crate, generic graph framework, persistent view schema, plugin mechanism or execution scheduler was added.

## One project, one publication path

`Store` owns the current validated Project. Rendering reads immutable shared snapshots; only candidate construction creates a mutable prospective Project. All semantic edits and deliberate layout changes use validated publication. A failed candidate cannot alter the current project or its undo/redo history.

`canvas::Session` owns document-scoped View, Focus, remembered viewports and trace locations. `Selection` explicitly identifies a node, exact edge, port, boundary port or ordered component-pair summary. A summary recomputes membership from the current Project and has no independently editable representative edge. Project generation changes cancel stale gestures; invalid selections are cleared and reported by identity.

Overview consumes full local geometry. Focus is calculated from real edges before aggregation, preserving total versus focused counts. Arrangement consumes the full graph, not a focus subset, and reserves footprints adequate for Detail. Substantial relayout is explicit, local and undoable. Mode changes do not move saved origins or automatically move the camera.

Drawing, pointer picking, arrowheads and particles consume the same final displayed Path. The animation is only a direction illustration. Effective direction of a projected child boundary still comes from its owner port, not its screen side. One physical port is not duplicated to accommodate fan-out.

## Interface meaning

The self-design edges describe responsibility-level inputs and returns, not an exact ABI or message transport. Where the small schema cannot express a borrowed or recursively typed Rust value, the diagram names its serialized projection. Runtime implementation uses direct Rust calls and borrows. `CanvasEmphasis` documents a transient presentation result; it is not another graph of engineering truth.

Parent-local edges are preserved. Child boundaries derive from the owning node, and external labels derive from actual parent edges. Trace navigation follows that identity without adding a cross-level semantic connection. A reached component input does not imply that all outputs depend on it.

## Scoped exchange and embedded guidance

Component, level and subtree packets retain their version-1 shapes and canonicalization. Context and source-base checks precede whole-project validation. Existing shared definitions cannot be silently overwritten. Apply reconstructs and validates again rather than trusting a cached candidate.

View, Focus, summary objects, lights and trace history do not enter project or replacement JSON. Muted data is not omitted from a requested export. Saved layout remains separate from semantic scope hashes. The embedded initialization prompt explains these boundaries and requires the human's explicit data transfer; the application has no AI network client.

## Local continuity

Project saves retain the previous valid version and use a synced temporary file in the same directory. Expected fingerprints detect ordinary external edits but are not a filesystem compare-and-swap or multi-writer lock. A failed durability operation need not mean that no disk effect occurred.

Recovery uses isolated per-session files and restores an unsaved copy. It covers published edits, not temporary pointer gestures or unconfirmed modal drafts. User data is preserved on uninstall. Independent backups remain necessary.

## Evidence and limits

See [CANVAS-INTEGRATION.md](CANVAS-INTEGRATION.md) for executed checks and runtime limitations. Structural tests do not prove good design, full native accessibility or a measured readability improvement. Arrangement is a heuristic, not a complete obstacle router. There is no approval engine, multi-user merge system, universal workflow execution, live activity feed, semantic edge taxonomy or persistent custom-view format.

## Guided control flow integration

The same seven responsibilities now also describe behavior in the native v2
self-design. Root calls and Interfaces reference the same existing Nodes; two
Canvas occurrences demonstrate shared identity. No new architecture modules were
invented. `ui/scope.rs` maps arbitrary behavior owners to optional interface
systems. `ui/flow/` renders the guided forms and control canvas; its drawing uses
the existing sampled Path and motion functions. `behavior/edit.rs` and
`edit/connection.rs` own validated edits and exact contract reconciliation;
`behavior/extract.rs` owns bounded extraction. Both formats enter the existing
Handoff and Store lifecycle. See CONTROL-FLOW-WORKFLOW.md for the algorithm,
conservative context policy and remaining limitations.

`behavior/layout.rs` shares small renderer-independent flow footprints and
effective-position resolution with extraction and the native canvas.
`behavior/changes.rs` compares validated candidates by identity for Handoff review;
draft diagnostics retain stable identity keys for distinguishing new issues from
renamed text. Neither helper adds a publication path or an exchange protocol.


## Coherent production editing (v3 layout extension)

The current responsibility map and dependency changes are specified in
[UI-COHERENCE.md](UI-COHERENCE.md). `ui/diagram` owns immutable camera, anchor,
route, painting/motion and input ownership mechanics used by both `flow/scene`
and `canvas/geometry`. Model/edit/Store remain headless. `ui/dialogs` owns the
fixed action shell and typed intents; forms retain validation and review gates.
`edit/layout` is the common deliberate-move candidate boundary, including atomic
v3 promotion. Project settings is a workspace command. Historical source paths
above describe the earlier interface-only arrangement; shared Path and motion
now live in diagram, and show_dialog lives in dialogs/show.rs.

Remaining coupling outside this assignment: recursive schema widgets and contract
impact review remain in contracts.rs; tree rendering and native lifecycle remain
on Designer; large text-based handoff presentation remains in handoff.rs. These
are concrete local responsibilities, not a new service registry or GUI layer.
