# Application design and source map

Status: implemented as a **source candidate**. No Rust compilation or native GUI execution has been observed in the authoring environment. The JSON describes this source, not a certified release.

The authoritative explorable design is `design/system-designer.project.json`, in the existing System Designer project format. It can be opened now in the previous editor. The Rust app includes the same bytes with `include_str!` and offers **File → Open application design**, as an unsaved copy. A new user project remains empty.

## Seven immediate responsibilities

| Component | Owns | Does not own | Source |
|---|---|---|---|
| Workspace | Window composition, navigation, selected level, document lifecycle, unsaved-change decisions | Structural validity or a second semantic project | `src/main.rs`, `src/ui/mod.rs`, `src/ui/workspace.rs` |
| Canvas | Local projection, drawing, hit testing, pan/zoom, temporary gestures | Direct publication or automatic contract selection | `src/ui/canvas.rs` |
| Authoring forms | Context inspection, component and contract drafts, explicit connection consent | A weaker validation path | `src/ui/inspector.rs`, `src/ui/contracts.rs` |
| Edits and history | Complete mutation candidates, shared-port type consistency, authoritative Store, undo/redo | File effects or semantic design approval | `src/edit/` |
| Design model | Typed records, structural validation, derived hierarchy and boundary queries | Rendering, file storage, workflow execution | `src/model/` |
| AI handoff | Embedded prompt, bounded exports, context and base checks, scoped replacements | AI connectivity, autonomous publication, execution of supplied code | `src/ui/handoff.rs`, `src/exchange/`, `assets/initialization.txt` |
| Local files | Validated reads, guarded replacement, previous-version backup, isolated recovery | Cloud sync, multi-writer merging, semantic authority | `src/storage.rs` |

These boundaries represent independent responsibilities, not seven crates or services. A single Cargo package keeps build and dependency machinery small. The UI is one Rust module with focused submodules; the model, edits, exchange, and files can be built without the desktop feature. Standard eframe and rfd supply windowing/rendering and file dialogs. No custom platform abstraction, plugin engine, database, or backend was added.

The JSON contains eight systems: the root and seven child decompositions. It contains 35 component nodes, with seven immediate root nodes and three to five in each child. The application **does not reject a user system containing more than eight nodes**. Eight is a reasoning heuristic, not proof that a decomposition is sound.

## What the edges mean

The self-design diagrams show responsibility-level inputs, outputs, and return paths. They are **not** a runtime message bus and are not an exact Rust ABI description. Several diagram contracts use an explicit JSON projection for a typed `Project`, `Contract`, or geometry value because the editor's small shape language has neither Rust references nor recursive type aliases. The actual application uses direct typed calls and borrows.

For example, `ViewContext` includes a serialized projection of the project, selected system, selection, and generation. Actual native code reads the typed `Project` held by `Store`. `FileRequest` includes the document for writes, rather than leaving the storage component to obtain hidden global state. `FileResult` preserves fingerprint and failure information. A `ContractDraft` is deliberately distinct from a whole-project candidate.

Parent-local edges are used throughout. Ports on child frames derive from the owning node. No separate public boundary is stored. The JSON describes the semantic responsibility decomposition; it is not intended as a complete call graph of every standard-library operation or borrowed value.

## Publication and draft state

`Store` owns the current validated project. UI frames obtain a cheap immutable `Arc<Project>` snapshot rather than cloning the entire project for every panel; only edit candidates are mutable copies. These snapshots do not introduce another publisher. Forms and wire gestures keep drafts outside it. An operation builds a complete candidate, validates it, and calls `Store::publish`. Failed validation cannot mutate current state, undo, or redo. A successful connection transaction can add a type, assign both endpoints, propagate required shared bindings, and create the edge as one undoable change.

The connection impact calculation follows equality between physical ports. A child-boundary endpoint uses the same owner port ID, so type consistency is propagated without permitting cross-level graph edges. Existing assigned ports or other affected edges require explicit confirmation. Separate contracts that should evolve independently need separate ports.

UI selection, viewport pan/zoom, temporary wire previews, dialog drafts and file paths are not persisted as engineering truth. Optional layout is stored separately from the design records and is omitted from AI scope hashes.

## Bounded AI exchange

Component scope replaces one node. Level scope replaces the current local system but preserves hidden child ownership and internals. Subtree scope can change the complete selected internal subtree, but not its externally owned public boundary or unrelated branches.

The original source content is hashed using the version-1 canonicalization convention. The base is an optimistic concurrency token, **not authentication**. Read-only context is checked independently. Shared catalog entries cannot be silently rewritten by a scoped import. The complete merged project is validated before publication, including preserved hidden systems. The UI deliberately validates again on Apply, rather than trusting a cached candidate.

The initialization prompt is compiled into the executable. Opening it or copying it does not expose project data; the user separately exports and transfers a selected scope. There is no network client or AI provider configuration in this implementation.

## File continuity

Normal saves write actual project files. The previous validated file becomes a `.bak` copy. The replacement uses a temporary file in the same directory, syncs its contents, replaces the target, and on Unix attempts a directory sync. Expected fingerprints detect ordinary external modifications. This is **not** a filesystem compare-and-swap, a distributed lock, or a guarantee against every power-failure/filesystem condition.

Recovery covers published project edits; unconfirmed dialog drafts and temporary gestures are not persisted. Each application instance uses a separate recovery filename. Recovery restores as an unsaved copy instead of overwriting the original project. Foreign and damaged recovery files are not automatically deleted. User data is not removed by the Windows uninstaller.

## Deliberate limits

No semantic acceptance engine, multi-user collaboration, generic rule system, execution scheduler, auto-updater or AI API is included. Component containment is iterative and has no fixed depth rule. Nested field schemas still encounter serde_json's defensive parsing recursion limit; that is a distinct technical limit, not a component-tree limit. Full native accessibility, performance and platform behavior need execution evidence. See `RELEASE-CHECKLIST.md`.
