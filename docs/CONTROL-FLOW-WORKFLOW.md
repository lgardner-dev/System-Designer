# Guided Control Flow workflow

The ordinary `system-designer` executable edits Control Flow and Interfaces in one
Project through one Store. No AI connection or JSON editing is needed to author a
flow, extract a responsibility or refine its information boundary. This feature
lives in `src/ui/`, not a standalone example, viewer or execution engine.

## Walkthrough

1. Open the normal executable. A blank document offers **Start a flow** and
   **Open Interfaces**. Starting explicitly promotes the document to version 2;
   the compatibility notice and starter are one undoable action. Opening a legacy
   interface project keeps its original layer and version. Switching tabs is a
   visual operation and does not dirty the project.
2. Use **+ Step**, **Edit step**, and **Connect steps** to describe local actions,
   questions, merges and outcomes. Drag from an `out` handle to an `in` handle, or
   in reverse, to open the same transition form. Edit descriptive alternatives
   and exact child outcome references there. Double-click edits a step; drag its
   body to move it. Ctrl/Shift-click selects multiple members. Delete previews
   incident transitions and information links; Escape cancels a gesture or draft.
3. Add **Information requirements** in the inspector. Pick the producer and
   consumer occurrence or scope boundary, optional exact public ports, optional
   matching existing wire, and an unassigned/existing/new contract. Use the
   structured contract editor for new definitions. **Review complete edit** shows
   existing bindings before **Apply information requirement** publishes once.
4. Choose **Extract selected steps** or invoke **Suggest a responsibility**.
   The dialog's checklist remains an alternative to canvas multiselection.
   Review the work, give the shared component a canonical name and one purpose,
   then **Preview boundary**. Inspect entry, each outgoing alternative, public
   requirements and unreviewed information uses. **Create component** checks the
   preview against the current document and publishes once. Cancel creates
   nothing. Scrolling the modal exposes its lower sections on smaller screens.
5. The selected parent call names the local action and displays the canonical
   shared component name below it. **Inspect interfaces / Show in Interfaces**
   highlights that exact child in the same parent scope. **Enter component**
   opens its behavior. A leaf can own a flow without an internal interface
   system. Its Interfaces page displays its actual public boundary.
6. Use **Refine contract** on a public port, or edit the information requirement,
   to assign a draft contract across its exact bindings. Review lists ports,
   existing interface wires and parent/child DataLinks. A new definition and all
   assignments are one validated candidate. **Show uses in Control Flow** lists
   every matching call occurrence for deliberate selection; it never chooses an
   arbitrary call. Renaming the shared component preserves occurrence labels.
7. Record **Primitive / stopping criteria** if further decomposition has no useful
   purpose. The guide remains revisitable; it does not lock phases. Save, Save As,
   Open, backups and recovery retain both layers and layouts. Undo/redo spans
   semantic edits and deliberate moves. Camera, selection, Focus and Lights are
   transient session state, not semantic edits or execution state.
8. In **AI handoff**, copy the embedded initialization prompt, then export
   **Interfaces** (component, level or subtree) or **Control Flow — current
   component**. Validate returned content without publication, inspect the
   preview, and Apply to revalidate and publish once. Explicit `flow: null`
   clearing is identified and requires the visible confirmation checkbox. Removing
   all contents of an existing flow also requires confirmation when an empty Flow
   object remains. Creating a new empty draft does not. The compact change counts
   expand into exact additions/removals and field edits; newly introduced draft
   issues are shown separately and remain saveable. An omitted `flow` is an error.
   Full-project JSON uses the separate Open lifecycle.

## Ownership and geometry

Behavior owners are `@root` or a component ID. `behavior::system` explicitly maps
an owner to its optional internal interface system. `ui::scope` keeps cameras,
layer, selection and Back locations by semantic owner and layer. Invalid saved
locations/selections are pruned after changes; document loads reset them. First
entry fits destination geometry. Native files store each layer's layout; cameras
are session-only and restart with Fit.

Interface levels retain their Detail, Overview, Focus, arrangement, exact-wire
editing and tracing controls. The flow canvas shares `diagram::routes::Path` with them:
one sampled cubic path drives drawing, picking, static arrowheads and lights.
Decisions use diamond perimeter intersections; terminators use ellipse perimeter
intersections. Call cards have the conventional double side lines. Alternative
merges are explicitly labeled, with no synchronization semantics. Direction is
independent of attachment side. Lights persist across navigation in egui session
state; only visible active paths animate, with Focus taking precedence. These
lights are direction illustrations, not execution, timing or concurrent traffic.

Transitions sharing an unordered endpoint pair (including opposite directions)
receive separate smooth lanes ordered by transition ID. Endpoint IDs fix the lane
orientation; names and JSON array order do not choose routes. Repeated self-loops
use separate perimeter attachments and increasing curved reach. Each final sampled
path drives labels, picking, static arrows and lights. Adding/removing a sibling
can redistribute that group's lanes. This is bounded local separation, not global
obstacle routing or a guarantee against all label collisions in dense diagrams.

## Actual extraction algorithm

`behavior::extract::inspect` accepts 1–512 selected local Action, Decision or Merge
steps. It requires one distinct member reached from outside, every member
reachable internally from that entry, a declared outside continuation, labeled
alternatives for decisions, and exactly one successor for ordinary selected work.
Several entering edges to the same entry member are allowed. Internal loops and
returns through the parent are represented explicitly. Existing Call nodes,
entry/outcome markers, multi-entry regions, disconnected members, open ends and
incident exact interface-edge associations are rejected with correction text.

On-demand suggestions walk from every possible start toward every possible stop,
collect eligible reachable local steps, validate the resulting region, deduplicate
by sorted member IDs, sort by size then IDs, and retain at most 24 candidates.
Suggestions are limited to 80 flow steps. This bounded enumeration is neither
canonical nor linear time. Failure to suggest a region is not evidence that no
useful responsibility exists; the manual checklist remains available.

Preview reserves the complete source flow namespace before generating child
entry/outcome markers and transitions. It moves selected work and internal edges
into one child behavior, replaces the region by one parent Call, preserves every
crossing alternative with a separate exact child outcome, and projects each
crossing declared DataLink onto a public port and parent/child declarations. It
does not manufacture an interface wire from a control arrow. Existing meaningful
IDs remain stable within their respective scopes. Historical provenance records
the source member IDs and purpose. Selection and camera never alter the preview
stamp; saved-layout changes conservatively stale it. Apply revalidates the full
candidate and Store publishes once.

Extraction first overlays sparse saved positions on the complete original default
layout. It freezes unselected parent positions and puts the Call at the selected
region entry's position. Moved work keeps its relative geometry; synthetic child
entry/outcomes occupy reserved rows outside the work's measured bounds. The core
and native canvas share small renderer-independent footprint measurements. These
rules avoid introducing marker/card overlaps in initially nonoverlapping scenes;
they do not automatically repair overlaps in previously saved projects.

Regression helpers expand supported extractions and compare original steps,
control alternatives and declared data endpoints modulo the synthetic boundary
markers and Call. They cover sequence, loops, merges, multiple outcomes and
nested extraction. This is a structural round trip, not arbitrary program
equivalence or execution safety.

## Contract reconciliation, drafts and failure cases

`edit::binding_impact` computes a fixed point over exact port identities,
interface edges and bound DataLink endpoints. It neither compares channel names
nor groups equal old contracts. `refine_port`, `connect` and
`behavior::information_candidate` reconcile that closure, include any new
contract definition in the same candidate, then validate the merged project.
Impact previews distinguish both layers. Cancellation leaks no definitions or
history entries. Unassigning a connected port remains invalid because interface
wires require a contract; remove/reconcile the exact wire first.

Draft guidance includes unlabeled alternatives, absent continuations, unhandled
child outcomes, duplicate unguarded returns, unassigned contracts and incomplete
human information review. Drafts remain saveable. Dangling outcomes, invalid IDs,
wrong ownership/direction, type mismatches and broken exact associations are hard
failures. Referenced definition/port/wire/outcome deletion is blocked with its
relevant scopes and uses where bounded reconciliation is unavailable. Removing a
Call occurrence preserves the component and other occurrences.

Meaning or incident information changes invalidate the affected local step's
human review flag. No declared requirements means **No information requirements
declared yet**. These declarations do not infer aliasing, side effects, conditional
availability, runtime transfer order or complete dependencies. A graph metric
cannot certify those properties. More than eight local action/decision steps
prompts review without forced grouping or a save limit.

The displayed complexity normalizes `k` outcomes to one synthetic exit and uses
`E + k + 1 - N` for the resulting connected graph. It is withheld when entry
reachability or paths to outcomes are incomplete. Even when defined, it describes
graph structure rather than feasible executions, complete return coverage,
correctness, test adequacy or a component quota.

## Transfer policy and compatibility

Project version 2 adds `behavior` and `flow_layout`. Existing v1 records and golden
scope/hash conventions remain unchanged. Behavior scope context policy stays at
packet version 1. It includes exact local interface context, immediate-child
signatures/outcomes, caller references and the complete contract catalog. It also
repeats child node structure. Some v2 interface packets include sibling behavior
interiors as preserved context. This release therefore does **not** claim minimal
context. An unrelated contract-catalog change can stale a behavior packet;
unrelated sibling flow changes outside its context do not. A narrower policy
needs explicit protocol versioning and matching stale/context tests.

The CLI dispatches exactly `system-designer-scope` and
`system-designer-behavior-scope`; unknown formats are errors. It validates merged
candidates without writing any file or running behavior. The embedded prompt
contains both schemas, readonly boundaries, explicit clear semantics, human
responsibility review and these transfer limits.

Handoff's semantic diff is ordered by owner and object identity. Its counts cover
changed records and scope metadata; exact details show step meaning/target/review,
control endpoints/conditions/outcomes, information endpoints and exact port,
contract and wire associations, primitive criteria and extraction provenance.
New draft issues compare stable categories and identities, so renaming an existing
problem does not present it as newly introduced. Null removal deletes the scope;
removing all contents to an empty object retains a draft scope. Both destructive
operations need the displayed confirmation. Apply reconstructs the candidate,
rechecks the reviewed changes and validates through the same Store publication
path. No exchange schema, version, canonical hash or context policy changed.

## Prior art and deliberate limits

- Vanhatalo, Völzer and Koehler, [The refined process structure tree (2009)](https://research.ibm.com/publications/the-refined-process-structure-tree)
  motivates hierarchical control regions. Their RPST provides a unique, modular
  decomposition and linear-time construction. We implement bounded candidate
  enumeration and human choice; we do not adopt those guarantees.
- LLVM [RegionBase](https://llvm.org/doxygen/classllvm_1_1RegionBase.html)
  motivates explicit entry/exit boundaries and nested regions. Our selected
  members may have several separately named outgoing outcomes, and we do not
  classify every candidate as a canonical SESE region or compute its dominance
  guarantees.
- LLVM [CodeExtractor](https://llvm.org/doxygen/classllvm_1_1CodeExtractor.html)
  motivates replacing selected work with a call and identifying boundary
  inputs/outputs. Our inputs and outputs are human declarations, not inferred
  LLVM values. We implement no IR execution-safety analysis, code generation,
  allocation transformation or executable extraction.
- Parnas, [On the criteria to be used in decomposing systems into modules (1972)](https://doi.org/10.1145/361598.361623)
  ([inspected text](https://www.cs.lafayette.edu/~gexia/cs301/resources/parnas.html))
  explains why processing-order decomposition alone can give poor modularity.
  We require a named purpose and human responsibility review. This is our design
  choice, not a quotation, semantic proof or automatic detection of responsibility.
- NIST, [Structured Testing, SP 500-235](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication500-235.pdf)
  informs interpretation of cyclomatic graph structure and the limits of test
  coverage. We show a bounded normalized graph metric, not feasible execution
  counts, a correctness proof, a testing engine or an enforced module quota.

No runtime engine, predicate language, state machine, fork/join, concurrency,
general scripting, code generation or automatic architecture regeneration is
implemented. Native operating-system file dialogs, accessibility and platforms
must be qualified separately from headless egui tests. Actual executed evidence
and remaining qualification limits are recorded in [the delivery record](CONTROL-FLOW-DELIVERY.md).


## Shared editing controls

Use global **Project settings…** from any layer/scope. **Save details** updates
name/purpose as one undoable document edit; **Save / Ctrl+S** writes the file.
Dialog Cancel and operation-specific actions stay at top right as content scrolls.
Ctrl/Cmd+Enter requests the primary action; Enter in multiline text adds a line.
Escape dismisses a popup first, then cancels its form or captured gesture.

Both canvases allow signed movement, middle/background pan and cursor-centred
zoom. Fit is explicit. Focus loss cancels unpublished movement. Arrow keys nudge
1 world unit (Shift: 10); **Move / position…** exposes exact X/Y fields. A negative
move promotes the document to project v3 in the same undoable edit and shows a
compatibility notice. Existing v1/v2 files stay unchanged when merely viewed.

Hollow input and filled output handles are the actual connection anchors, on any
side of the approved node shape. Drag or click output→input or input→output;
forms remain available. Control transitions still carry conditions/exact outcomes;
interface connections still require explicit exact contracts. Lights illustrate
direction only. Their menu and display options preserve selection and history.
