# System Designer data model — version 1

This document specifies the format implemented by `src/model` and `src/exchange`, and is the specification those modules are written against. The application stores one semantic project, plus optional visual coordinates. It is a design model, not a workflow runner or semantic-approval system.

## Full project

```json
{
  "format": "system-designer-project",
  "version": 1,
  "id": "project.example",
  "name": "Example",
  "purpose": "Describe the outcome and constraints here.",
  "root": "root",
  "contracts": [],
  "systems": [{"id": "root", "nodes": [], "edges": []}]
}
```

Required fields are shown. Optional `layout` is an object mapping system IDs to objects mapping local node IDs to `{ "x": number, "y": number }`. Coordinates must be finite and nonnegative. Omitted positions receive deterministic auto-layout; positions are not engineering truth. Layout may not reference an absent system or nonlocal node.

A system has exactly `id`, `nodes`, and `edges`. A node has `id`, `name`, `purpose`, `kind`, `ports`, and optional `child` (a system ID). Omit `child` for a leaf; null is invalid. Every non-root system has one owning node. Every node belongs to one containing system. All systems must be reachable from the single root, without containment cycles. There is no fixed node-count or containment-depth rule.

`kind` is one of `component`, `work`, `decision`, `authority`, `controller`, `record`, `success`, `failure`. These fixed visual hints are not runtime behavior or proof of acceptance/completion. New UI nodes default to `component`.

Structural IDs—project, system, node, port, edge—are nonempty strings and globally unique across those categories. Names must be nonempty; purposes may be empty during drafting. Contract IDs use a separate `(id, version)` identity space.

## Ports and connections

```json
{
  "id": "sensor.samples",
  "name": "Samples",
  "direction": "out",
  "contract": {"id": "Sample", "version": 1}
}
```

Port direction is `in` or `out` on its owning component. The required `contract` field is either one exact reference or **null for an unconnected draft port**. A missing field is invalid. A connected null port is invalid, even if the other endpoint is also null.

```json
{
  "id": "edge.samples",
  "from": {"node": "sensor", "port": "sensor.samples"},
  "to": {"node": "processor", "port": "processor.samples"},
  "label": "On sample received"
}
```

An edge has exactly `id`, `from`, `to`, and optional string `label`. Endpoint objects have exactly `node` and `port`. Both endpoints must resolve in the edge's system. A component output produces and an input consumes. Endpoint contract IDs **and versions** must match exactly. There is deliberately no `edge.contract` property: the common endpoint type determines the carried type. Different payloads use distinct ports rather than a union of unrelated contracts on one port.

`node: null` names the current system's boundary, not an external component. It is valid only in a non-root system; `port` references the owning node's actual port. Inside the child, an owner input is a source and an owner output is a target. The public boundary is derived, never independently stored. The renderer derives external neighbor labels from actual parent edges. Deep cross-level edges are invalid; local cycles, including feedback and boundary pass-through, are valid.

## Structured contract definitions

```json
{
  "id": "Sample",
  "version": 1,
  "name": "Sample",
  "purpose": "One reading with its source and value.",
  "definition": {
    "type": "object",
    "fields": [
      {"name": "source", "required": true, "schema": {"type": "string"}},
      {"name": "value", "required": true, "schema": {"type": "number"}}
    ]
  }
}
```

A definition has exactly `id`, positive safe-integer `version`, nonempty `name`, string `purpose`, and a structured `definition`. Every referenced `(id, version)` must exist exactly once in the catalog.

The shape language is intentionally smaller than JSON Schema:

| Shape | Additional required members |
|---|---|
| `string`, `integer`, `number`, `boolean` | None |
| `enum` | `values`: nonempty array of unique nonempty strings |
| `array` | `items`: one nested shape |
| `object` | `fields`: array of field records, which may be empty |

A field is `{name, required, schema}` with optional `description`. `name` is unique/nonempty within its object; `required` is an explicit boolean; `schema` is a nested shape; description is a string. Objects, arrays, and enums may nest. Unsupported schema keywords are rejected instead of ignored. Purpose/description prose can express additional intended constraints, but structural validation does not enforce their meaning or validate runtime payload instances.

## Explicit connection transactions

`edit::connection_impact(project, system_id, from, to, chosen_type, editing_edge_id)` resolves local direction-compatible endpoints, builds adjacency among physical ports using existing edges (excluding the edited edge), and finds the connected port bindings that must take the chosen type. Mirrored boundary references reuse the same physical port ID. This propagates type equality, not cross-level graph connectivity.

The result lists changed ports (with their previous types) and other affected edges. Any change to an already-assigned port, or any other affected edge, requires consent. First assignment of isolated null ports does not require an additional checkbox beyond the explicit Create connection action.

`edit::connect` constructs a clone, optionally appends a newly defined contract, assigns the exact type consistently, inserts/replaces the edge, and validates the complete project. It returns a candidate, never mutates the original. New-contract creation cannot overwrite an existing catalog entry. Failure or cancellation commits nothing. The UI publishes once, so a new contract and edge are one undoable action.

## Scoped exchange envelope

Three packet forms use `format: "system-designer-scope"`, `version: 1`. Every packet includes `scope`, `projectId`, `systemId`, `base`, `contracts`, and `context`. Every field not documented for that form is rejected.

`base` is the hex SHA-256 of canonical UTF-8 JSON of the export's source content **before the proposal modifies it**. Canonical objects have sorted keys; arrays retain order. For level/subtree content this covers `boundary`, `systems`, `contracts`, and `context`. For component content it covers `component`, `contracts`, and `context`. The source project/system/node identity is also checked separately.

The token is an optimistic concurrency check, not authentication or authority. AI and users must copy it unchanged, not recalculate it to force stale edits through. Current relevant state is recomputed at validation and again before UI Apply.

### Current level only

```text
{
 format, version, scope: "level", projectId, systemId, base,
 boundary, systems: [one selected system], contracts, context
}
```

The notation in this section describes shapes, not literal importable JSON. The application exports complete valid examples.

The selected system is editable. Existing `(node ID, child system ID)` ownership pairs must remain exactly the same. You may add/remove leaf nodes and edit local fields/wiring, but cannot add/remove/reparent hidden internals. All hidden systems are retained in the local project and the full merged model is validated, including their boundary wires. A new local port is allowed if structurally valid; removing/retyping a port that preserved hidden wiring relies on is rejected.

`boundary` is null for the root; otherwise it contains `{ownerId, name, purpose, ports}` from the owning node. It is **read-only**, since that node is outside the editable system. `context` is also read-only:

```text
{
 project: {name, purpose},
 ancestors: [{system, node, name, purpose}],
 externalConnections: [derived boundary-link records],
 preservedChildren: [{ownerId, systemId}]
}
```

An external boundary-link record contains `{port, direction, links}`. A link contains `{edge, node, name, port, portName, label, contract}` describing the actual parent edge and remote endpoint. Null remote node names a parent boundary.

Only selected-level port and owner-boundary contract definitions are exported, not the catalog's unused definitions or types used solely in deeper systems. Hidden child internal changes do not invalidate a level packet unless they change a relevant shared type or make the merged proposal structurally invalid.

### Full subtree

The same envelope uses `scope: "subtree"`; `systems` contains the selected system and **all** its descendants. `context.preservedChildren` is empty. Public boundary and context remain read-only.

The subtree can be redesigned, including adding/removing child systems, provided ownership, identities, membership, endpoints and types validate. Missing descendants are deleted. No outside system may be included or have its ID reused. The exported owner interface cannot be rewritten; change that node at its parent level instead. Existing unrelated branches are preserved.

### Selected component only

```text
{
 format, version, scope: "component", projectId, systemId, nodeId, base,
 component: one node, contracts, context
}
```

There is no `systems` or `boundary` field. `systemId` is the node's containing level, which must be open for import. `nodeId` and `component.id` must stay unchanged. `component.child`, when present, must also be preserved.

Editable content is the node record. Siblings, local edges, hidden systems, and layout are not replaced. The full merged project rejects changes that would invalidate those preserved connections. It is valid to rename the node or its human-facing port names, refine purpose, or add unused ports. Rewriting a connected port's ID or type requires a broader appropriate scope.

Read-only context is:

```text
{
 project: {name, purpose},
 ancestors: [{system, node, name, purpose}],
 connections: [actual local edges incident on this node],
 neighbors: [{node, name, port: complete remote port record}],
 preservedChild: child-system ID or null
}
```

Neighbor node bodies and internal designs are omitted. The packet includes definitions for the node's and contextual neighbor ports. A neighbor interface/name change makes it stale; an independent sibling purpose change or compatible hidden-child edit does not.

### Publication and shared contracts

Scoped imports build a candidate while preserving the current project outside the scope. Existing catalog identities may be repeated only with exactly the same canonical definition. New identities/versions can be appended. No existing shared definition is overwritten or removed by scoped import. Referenced definitions must accompany the packet.

Surviving node positions are retained; positions for deleted nodes/systems are removed. Missing positions use auto-layout. Imports validate the **whole resulting project**, not just the packet. A candidate is published only after parsing, concurrency/context checks, ownership/membership checks, catalog merge, and structural validation succeed. The UI has separate Validate and Apply actions and revalidates at Apply.

## Native implementation and limits

`Store` owns validated immutable snapshots. Publication takes ownership of the validated candidate; rendering and history use immutable shared snapshots. Undo/redo retains up to 50 session changes. The native document layer writes actual project files and separate per-instance recovery snapshots. Undo history is not a persistent audit trail, and no multi-user merge protocol is implemented.

The parser accepts the `system-designer-project` version-1 format and no other format identifier; there is no migration path from another tool's format. Positive versions must be integer-encoded JSON numbers, not exponent/floating encodings; authentic version-1 exporter output uses that representation.

Unknown properties are rejected rather than treated as extension metadata. Incomplete draft intent may have an empty purpose, but required identities and names remain nonempty. A leaf omits `child`; explicit null is invalid. Conversely, a draft port must explicitly contain `contract: null`. Endpoints must explicitly contain `node: null` when they refer to a boundary.

Canonical scope hashes use one fixed convention: UTF-16-sorted object keys, unchanged array order, and JSON primitive encoding. Scoped content contains safe-integer versions, but no layout floats. The golden fixtures in `tests/fixtures/` preserve real exports and their expected replacement outputs, including Unicode, and `tests/exchange.rs` asserts against them.

Component containment is normalized and traversed iteratively without an arbitrary tree depth limit. Deeply nested contract shapes remain subject to serde_json's defensive parsing recursion limit, so unlimited nesting of every possible JSON structure is not claimed.

A structurally valid design is not proof of useful decomposition, complete requirements, correct contracts, implementation safety, or human acceptance. The approximate eight-component guideline is advisory and does not reject larger valid systems.

### Guided behavior editing and explicit clearing

A `system-designer-behavior-scope` version-1 packet requires the `flow` key.
Omitting it is an error. `flow: null` explicitly requests clearing only the named
owner, including that owner's flow layout; this is previewed as a destructive
change in the app. A flow object with empty `steps` and `transitions` is an
incomplete, saveable specification, distinct from absence. Full merged-project
validation rejects clearing a child with referenced outcomes. Clearing never
downgrades a version-2 project.

Information contract refinement computes a closure over exact public-port IDs,
interface edges, and explicitly bound DataLinks (including child boundaries and
all call occurrences). A new definition and all assignments are one validated
candidate. Equal names or equal old contract references do not imply a binding.
Unassigned links without a binding remain independent. Material step meaning or
incident information edits invalidate human information-use review. Missing child
outcome handlers and duplicate unguarded handlers are draft issues; dangling
outcome references remain errors.

Extraction reserves the source flow's complete identity namespace before adding
synthetic child markers. Existing source identities are retained. Its stale-plan
stamp conservatively includes both saved layouts; viewport and selection are
session state and do not enter that stamp. Behavior exchange hashes continue to
exclude saved layouts. The first context policy still includes the full contract
catalog, so unrelated catalog edits conservatively stale behavior packets.


The ordinary Handoff dialog now exposes both exact scope formats. Validate
constructs the merged project without publication; Apply reconstructs it against
the then-current Project. Explicit behavior clearing is shown in its preview and
requires a clear checkbox. `designer-check PROJECT [PACKET]` dispatches strictly
by `format` and validates only. Unknown formats and whole projects used as scope
packets are errors. Context-policy versions and canonical hashes are unchanged.
Owner/layer navigation, cameras, selection, lights and dialogs are session state;
they do not appear in persisted JSON. Document reopen uses Fit on the persisted
destination geometry. `flow_layout` and `layout` both survive all file operations.
