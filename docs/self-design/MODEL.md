# Self-design atlas format (review-only v1)

Envelope: `format = system-designer-self-design-atlas`, `version = 1`, `baseline`, `purpose`, `project`, `scopes`, `source_hashes`, `limits`.

The project is the existing strict version-1 interface model. The companion scopes use existing component IDs, with `@root` denoting the project root. Exactly one scope per component and root is required. A leaf has `primitive_stop`; a decomposed component delegates calls to its immediate children. No fabricated leaf child system is stored.

Each scope has `owner`, `title`, `notes`, source references (`path`, `symbol`), `primitive_stop`, `planned`, `steps`, and `transitions`. Each step has a scope-local stable `id`, short `label`, `kind` (`entry`, `outcome`, `action`, `decision`, `call`, `io`), exact descriptive `rule`, `inputs`, `outputs`, optional `target`, optional `source`, and `status` (`mapped`, `proposed`). Only a call has a target. Sources are indexed in `source_hashes`; build-time tests bind LF-normalized UTF-8 source text and symbol names (Windows CRLF checkout conversion is normalized, but no other content is ignored). Mapped is source-grounded design intent, not formal equivalence proof.

Transitions have `id`, `from`, `to`, `label`, and `exchanges`. They are local; references resolve. One entry, at least one declared outcome, labeled alternatives from each decision, entry reachability and a structural path to an outcome are checked. Cycles are allowed; reachability does not prove runtime termination. Exact interface exchange references must belong to the corresponding owner-local structural system. No transitive dataflow is guessed.

The viewer uses native eframe and the production geometry module directly for its cubic curves and exact interface scene. Flow shapes and call references are another projection, not an execution engine. Cameras, selections, coordinates calculated for the viewer, lights and screenshots are session/presentation state; they do not modify this authored model.

The compatibility projection is derived as `atlas.project` and must equal `design/system-designer.project.json` under the production typed parser. This is not an undocumented production extension. The production parser must reject the atlas envelope, and production replacement keeps its unchanged protocol. A future integrated two-layer editor needs a deliberate versioned migration and complete cross-layer replacement rules.

The viewer uses a bounded clearance heuristic over the existing cubic curve sampler; it may choose another shape attachment or outward handle to keep a bypass from crossing an unrelated card. Exact interface port positions and directions remain unchanged. This is not a global minimum-crossing or arbitrary-graph routing guarantee.
