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

## Source-mapped self-design atlas

The authored paired design is now `design/system-designer.atlas.json`; the existing embedded `design/system-designer.project.json` is its checked compatibility projection. The seven root responsibilities and their existing public identities remain. Structural validation is refined into catalog/identity, containment, local-edge, layout and result responsibilities. Local-edge checking is refined into endpoint resolution, direction, exact contract comparison and diagnostic accumulation. Publication/history is refined into admission, unchanged detection, prior-state retention and installation. These are design responsibility boundaries mapped to current function bodies, not additional deployed modules.

Run `cargo run --locked --release --example self-design-atlas` to inspect both facets at every component, including leaves. The native atlas explicitly records primitive stopping rules and pending CFG-authoring/migration work. The generic production application remains the existing interface editor; this change updates the self-design and adds a read-only review surface, not an implicit version-2 release. See `docs/self-design/README.md` and `MODEL.md`.
