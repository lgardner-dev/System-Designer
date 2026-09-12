# System Designer — Unified Canvas UX

**Status:** Accepted for implementation by the repository owner. Verification is reported separately; this specification is not execution evidence.  
**Revision:** 1 — 2026-09-11.  
**Purpose:** Combine the existing four-sided ports and direction lights with connection-aware arrangement, compact overview, and exact focus/tracing without introducing competing project models.

## 0. Source baseline and scope

Repository: `lgardner-dev/System-Designer`.

| Source | Exact revision inspected | Observed status |
|---|---|---|
| Main | `267db9364311327ca0b7e663f854b6569ef6b18f` | Includes the merge of PR #1, four-sided ports and directional light previews. |
| Experiment A, PR #3 | `5f7699cf77973239169c793941a88a4adf9fd092` | Open, unmerged: focus and exact connection tracing. |
| Experiment B, PR #4 | `bdda19e37809afc2a6b6d40b06b2b1890b21e764` | Open, unmerged: compact overview with exact member inspection. |
| Experiment C, PR #5 | `09efbcc6fc0366f7aaf9b4f10abb4a606c7ba80c` | Open, unmerged: connection-aware arrangement. |

The branch statuses above were read during preparation. They are not a prediction of later repository state. Source basis: the corresponding PR metadata; `docs/INTERFACE-EXPERIMENT.md` on each experiment; `src/ui/canvas/motion.rs` and `MODEL.md` on the inspected main revision.

Those experiments are independent candidates. Their documented individual verification does not establish the behavior of a combined application. No source changes, branch writes, merges, or execution tests were performed while preparing this specification.

The scope is one integrated native Rust canvas experience. Existing project semantics, contracts, recursion, validated publication, and scoped exchange remain intact. There is no new workflow engine, semantic edge taxonomy, AI integration, or generic visualization framework.

## 1. Governing interaction rule

**Arrange changes positions. View changes displayed detail. Focus changes emphasis. Lights illustrate direction. None changes what the system means.**

The component tree continues to describe containment. Local edges describe actual declared connections. Layout direction is not execution order, a prerequisite, or a declaration of where a process begins and ends.

There is one authoritative `Project`, one publication path through `Store`, and one selected object at a time. A counted connection summary is a presentation object referencing real edge IDs, not another editable semantic edge.

The rough eight-component guideline remains a reasoning aid, never a validity limit. Do not decompose a responsibility merely to reduce its visible wires.

## 2. Workspace and controls

Keep the design tree, canvas, and existing inspector. Put the current-system title and breadcrumb above the canvas. Use one compact toolbar, wrapping or moving secondary actions into labeled overflow on narrow windows.

| Control | Choices | Meaning |
|---|---|---|
| View | Detail / Overview | Exact ports and wires versus compact component relationships. |
| Focus | Off / Selection / Incoming / Outgoing | Emphasis around the selected object. Incoming/Outgoing apply to a whole component. |
| Arrange | By connections: Left to right / Top to bottom; Grid | Explicit, current-level positioning commands. |
| Lights | All visible / Selection / Off | Direction illustration on the eligible visible connections. |
| Fit | Action | Changes the viewport, not saved component positions. |

Do not add a persistent “Manual versus Automatic layout” mode. Arrangement runs when requested; subsequent manual movement is always allowed.

Initial defaults: **Detail**, **Focus Off**, saved positions preserved, and **Lights All visible**, preserving main's current All behavior. A user's light choice survives navigation and View/Focus changes throughout the session. No new settings subsystem is needed for this increment. Where a reduced-motion preference is available, use Off as the initial motion choice and allow deliberate user control; do not claim detection on unsupported platforms.

When Focus is enabled, show the anchor and scope plainly, for example “Focus: Processor · Incoming”. Show exact current-level counts distinguishing emphasized and muted connections. Overview additionally distinguishes summary-stroke count from actual edge count. Numbers in UI examples are illustrative, never hard-coded.

Keep “Direction preview — not live execution” available whenever lights are enabled. A compact legend identifies hollow receiving ports and filled producing ports; color is not the sole direction cue.

Changing View or Focus must not automatically fit or pan. Explicitly opening a new level may use its remembered viewport or fit it on first entry.

## 3. Preserve four-sided ports

Inputs and outputs may appear on any of the four sides. Their semantic direction remains the recorded `in`/`out`; position is presentation only.

Retain rounded rectangular cards for this increment. Circular cards remain a possible later experiment, not a prerequisite. Four-sided placement already removes the left-input/right-output restriction while retaining space for readable names and contract labels.

A physical port has one anchor in each displayed diagram, even when several edges use it. Do not duplicate one port around a node to satisfy each incident edge. Shared-port contract consistency remains enforced by the existing edit path.

Automatic side selection considers actual neighboring endpoints, label space, and deterministic tie-breaking. It must not use a Focus subset as though other connections no longer existed. Boundary anchors remain projections of the owning node's ports. Their displayed side may differ between parent and child views without changing identity; effective direction inside the boundary remains correctly interpreted.

Port-side placement is derived UI geometry, not a new semantic field. Do not add hand-maintained copies of child interfaces or require AI exports to contain port coordinates.

**Gesture stability:** freeze anchor-side and ordering decisions during a wire gesture. Component dragging may move routes continuously, but side/order changes settle after the gesture rather than making targets jump while the user is connecting. A project edit that invalidates an active gesture cancels it explicitly.

Give each label a nonoverlapping local slot. Full endpoint names, contract ID/version, and external context remain available in the inspector as well as on hover. A small or truncated canvas label must not be the only way to inspect a port.

## 4. Detail and Overview

### Detail

Show exact local components, ports, boundary anchors, and edges. Preserve drag-to-connect in either direction, the click-based alternative, and the explicit choose-or-define-contract dialog. Editing a connection continues to use its exact semantic edge ID and the existing shared-binding impact confirmation.

Detail does not imply displaying every long label at equal prominence while Focus is active. Muted context keeps a recognizable outline; the focused connections retain readable labels and direction cues.

### Overview

Show compact cards with readable names, a child-system indicator, and port/connection counts. Omit the full interior port listing, not the information itself.

Group edges only when they share the same ordered pair of non-boundary, distinct components in the current system. A→B and B→A remain separate. Self-loops and projected boundary connections remain individual. A summary contains stable member edge IDs and a count; it does not invent a common contract, event, bus, or execution meaning.

Switching views keeps the same saved node origins and viewport. Compact cards may expose more whitespace; that is preferable to silently rearranging the design. Overview routing may attach to compact card geometry, but no semantic endpoint changes.

Clicking a summary opens its member list in the existing inspector. Each member shows source component/port, destination component/port, exact contract ID/version, and full edge label. Ordering is deterministic.

“Show exact wire” switches to Detail and selects that member. “Edit this wire” also establishes exact Detail context before opening the normal editor. A single-edge relationship can select its real edge directly. Multi-edge summary Delete/retype is disabled with an explanation to choose a member. No arbitrary representative edge and no implicit bulk operation are permitted.

Direct pointer wiring happens in Detail. The existing Connect ports form remains available in either view. Entering a connection dialog from Overview reveals Detail first; cancellation changes no semantic data and does not automatically switch the user back to Overview.

### Overview combined with Focus

Compute focused membership on real edges before drawing summaries. Every summary retains its total membership. When only part of a summary is emphasized, disclose both counts: for example “1 of 4 in focus”. Never relabel four connections as one simply because three are muted.

An exact edge selection remains that exact selection across a View switch. Its containing overview summary may be highlighted, but the inspector and emphasis counts must preserve the distinction between that member and the whole group.

Do not partially expand a selected card in Overview. Use the inspector or explicit Detail action; otherwise selection would unexpectedly change card geometry and wire attachment.

## 5. Focus and exact tracing

Focus changes emphasis, not membership in `Project`.

| Anchor | Selection focus means |
|---|---|
| Component | Its actual direct incoming and outgoing edges and endpoints. Incoming/Outgoing can narrow this set. |
| Exact port | The actual edges incident on that port. |
| Exact edge | That edge and its endpoints only. |
| Overview summary | All exact members of that selected summary and their endpoints. |

Do not automatically expand to neighbors-of-neighbors or to all outputs of a component reached through one input.

Incoming/Outgoing choices are enabled only for whole-component anchors. Selecting an exact port, edge, or summary while a directional component filter is active visibly changes Focus to Selection. Do not leave a toolbar reading “Incoming” while silently applying different semantics.

A user may select and inspect without enabling Focus. Clear focus preserves selection and restores ordinary emphasis. Clearing selection or deleting its target disables Focus rather than dimming the entire diagram. Ordinary component-tree navigation clears the old anchor; explicitly following a trace transfers it as described below.

Keep unrelated component names and positions recognizable. Muted wires lose distracting labels and animation, not their existence. During a connection gesture, temporarily reveal direction-compatible targets at the current level, including boundary ports. Restore the previous focus policy after the gesture/dialog ends. This never bypasses explicit contract selection or full candidate validation.

### Stepwise tracing

Use the existing inspector for full exact-connection rows and Source/Destination navigation. Selecting an endpoint can highlight it without assuming an internal continuation through its component.

“Enter at this port” opens the owning component's child system at the identical projected port. “Follow in parent” selects the corresponding owner port in the parent. No cross-level edge is invented. A leaf or unresolved interior stops the trace with a plain explanation: no declared continuation is represented. Choosing a different output is a deliberate selection, not a proven continuation.

Back trace restores a session navigation record containing system, selection, View, Focus, and viewport. It does not undo the project. Restore only still-valid IDs after edits; report and clear an invalid trace target rather than silently following a similarly named object.

Keyboard-accessible connection rows and controls provide the non-pointer route. Hover and glowing particles are never required to identify an endpoint.

## 6. Connection-aware arrangement

Keep connection-aware arrangement as a command under Arrange, alongside Grid. Relabel the old Auto-layout as Grid for clarity rather than retaining two competing top-level arrangement buttons.

Both orientations use the complete authoritative current-level graph, regardless of View, Focus, or Lights. Summaries may be useful for presentation, but they must not discard edge multiplicity when the arrangement algorithm uses that information.

Arrange measures a stable footprint sufficient for Detail and Overview. In particular, arranging while Overview is active must not pack small cards so tightly that switching to Detail creates overlap. Account for port-side-dependent dimensions and required boundary/loop space during bounded measurement.

Feedback grouping stays temporary layout data. Never reverse/delete real edges, create semantic components for layout clusters, add Start/End nodes, or interpret layers as prerequisite order. The experiment's ring policy for cyclic groups remains a heuristic; it is not a minimum-crossing or full obstacle-routing guarantee.

A successful command changes only the current level's saved positions, as one validated undoable action, then fits the result. It leaves model records, other levels' layout, hidden internals, and AI scope hashes untouched. Undo/Redo restores positions; Fit remains available if the current viewport no longer frames the restored arrangement. Do not add a second persistent history merely to restore viewport changes.

If bounded measurement/layout cannot produce a valid candidate, explain the failure and preserve the prior positions. Never publish a partially calculated arrangement.

Normal file opening preserves supplied positions. Missing positions continue to receive deterministic fallback placement; do not silently run a potentially disruptive full rearrangement when opening an existing design.

## 7. Direction lights and geometry precedence

Keep the white/light-blue moving point with a short soft trail, existing static arrowheads, and the explicit Off control. This is illustration of source→destination, not live data, execution, throughput, branch selection, or timing evidence.

Selection animates only relationships belonging to the selected object. All visible considers all drawn relationships, but **Focus suppression takes precedence**: muted context never animates. A tooltip states this so “All visible” is not mistaken for an override of Focus. Off overrides everything.

In Overview, draw at most one illustrative pulse on an eligible summary stroke. It represents the summary's common direction, not one packet per member. Do not synchronize a network of summaries into an apparent scenario. A partially focused summary exposes its member counts while using the same directional illustration rule.

Painting, pointer picking, arrowheads, and particles use the same final displayed route. Arrowheads follow the destination tangent, including right-side inputs, top/bottom ports, and self-loops. Lights advance by displayed path distance, not raw curve parameter; their visual speed has no system-timing meaning.

Use a modest bloom that does not conceal labels or junctions. Keep static direction readable when animation is disabled. Request animation repaints only while at least one eligible on-screen route requires them; stop animation-driven repainting for inactive/minimized windows where platform state is available. Ordinary input and document handling continue normally.

## 8. State, export, and safety

| Information | Owner / lifetime | Project/history effects |
|---|---|---|
| Components, ports, contracts, exact edges | Authoritative Project / Store | Existing validated semantic edit path. |
| Saved node positions | Existing `Project.layout` | Saved with project; deliberate changes undoable. |
| View, Focus, selection, trace stack, viewport | UI session, appropriately scoped to document/level | No semantic publication and no dirty flag. |
| Light preference | Application session, not project content | No dirty flag; navigation does not reset it. |
| Summaries, port anchors, routes, layout clusters | Derived UI data | Rebuild from real IDs and current geometry. |

This increment does not require a project-format version change. Do not add arbitrary fields to strict version-1 JSON. Update the embedded initialization prompt to explain that visual summaries and highlights do not change the exported design or establish control flow.

**AI exchange exports the requested semantic scope, not the visible drawing.** Existing component/level/subtree semantics stay exactly as documented. Overview, Focus, and Lights must not remove any content the selected scope currently includes. Layout-only changes do not change the scoped canonical base. Existing read-only context, shared-definition, identity, and stale-base checks remain enforced, with revalidation at Apply.

After a genuine semantic edit or imported replacement, recompute summaries and focus membership. Preserve a still-valid selection by ID; clear missing targets visibly. Do not cache a stale summary list and apply destructive operations to it.

Escape cancels the topmost gesture/dialog first; it must not accidentally clear focus, dismiss a draft, and undo work simultaneously. Delete acts only on an exact allowed semantic selection. Crossing/overlapping wires always remain individually selectable through the inspector list even when precise canvas picking is ambiguous.

## 9. Native component integration

Keep these responsibilities local to Canvas rather than adding root-level subsystems or a generic view framework:

| Local responsibility | Implementation location / intended ownership |
|---|---|
| Canvas composition | `src/ui/canvas.rs`: frame ordering, controls, selection dispatch. |
| Geometry | `canvas/geometry.rs`: measured cards, anchors, final paths, shared hit geometry. |
| Arrangement | `canvas/arrangement.rs`: temporary graph and position candidate. |
| Overview | `canvas/overview.rs`: reversible summaries and exact member mapping. |
| Focus and trace | `canvas/focus.rs`: exact membership and navigation context. |
| Motion | `canvas/motion.rs`: eligible-route direction illustration. |

Gesture handling can remain with canvas composition unless its existing responsibilities justify another small module. Tests are supporting evidence, not additional semantic components. No new services, crates, plugin system, or layout dependency are required merely to integrate the existing prototypes.

When implementation changes these boundaries, update the embedded application's existing Canvas decomposition and source mapping. Do not maintain a second competing self-design. Keep the initialization prompt and self-design compiled into the executable; new user projects remain blank.

## 10. Adversarial acceptance cases

These are planned integration checks, not executed results.

| Attack / scenario | Required observable result |
|---|---|
| Overview + Focus on one member of a four-edge summary | Total remains four, focused count is one, and all four are inspectable. |
| Delete/retype a multi-edge summary | No semantic mutation; user must choose an exact member. |
| Export a level in Detail and in Overview with unrelated connections muted | Requested semantic content and canonical base are identical. |
| Arrange an identical level with Focus Off and with a component focused | Same deterministic candidate given the same inputs/orientation; no omitted nodes or edges. |
| Arrange in Overview, then switch to Detail | Saved origins stay fixed; arranged Detail cards do not overlap because compact-only bounds were used. |
| Connect to input/output ports on every side | Directions stay correct, targets stay stable during the gesture, contract dialog remains explicit. |
| Move a shared fan-out port's component | One physical anchor per diagram; no duplicate ports or altered bindings. |
| Inspect a self-loop or backward edge with Lights Off | Static direction remains distinguishable; no semantic reversal. |
| Follow a parent port into a child and back | Same physical port identity, appropriate local direction, no fabricated cross-level edge. |
| Reach an opaque leaf during a trace | No automatic input-to-output continuation is claimed. |
| Enable All visible lights while Focus is active | Only non-muted eligible routes animate; Off still disables every pulse. |
| Toggle View/Focus, navigate, or cancel connection editing | No accidental publication or dirty flag; Off motion choice remains Off. |
| Delete a selected member or apply a scoped replacement | Membership and selection are refreshed by ID; stale UI cannot mutate the wrong edge. |
| Undo/Redo arrangement | Current-level positions restore; graph semantics and other levels remain unchanged. |
| Layout sizing failure, empty system, one node, dense cyclic region | Bounded completion/failure, no partial layout or invented process order. |
| Keyboard-only inspection at reduced zoom | Full labels and exact members are accessible without hover or animation. |

Retain existing core, storage, and exchange regression tests. Exercise all meaningful View × Focus × Lights combinations, plus pointer/modal transitions. Individual experiment CI is not combined-product evidence.

Use the exact dense Method project from the user's screenshot as a usability fixture when its JSON is available; the screenshot alone is not reconstructable authoritative input. Do not fabricate hidden ports or edges from pixels. Also use repository fixtures and the embedded self-design.

Native verification should record actual environment and commit, inspect readable labels and selection at ordinary desktop sizes, and exercise real dragging, mode changes, member editing/cancellation, nested tracing, save/reopen, and Undo/Redo. Compare task correctness and tracing effort with the baseline; do not claim a measured readability improvement from compilation alone. Captures must come from the actual application, not illustrative mockups labeled as execution evidence.

## 11. Integration sequence and deliberate exclusions

The user's requested first increment is already merged. Preserve it rather than reimplementing it.

Prepare a separate integration branch from the verified current main, reconciling the experimental changes instead of blindly merging three overlapping canvas implementations. Reuse the useful algorithms and tests. Give the combined branch one set of controls, one selection model, and one final geometry path. Preserve the original comparison branches until the combined build is reviewed.

Integrate in three reviewable checkpoints: arrangement with stable footprints; overview with exact-member editing; focus/tracing with explicit summary/light precedence. Run the combined behavior checks before any main merge. No public release or merge is implied by this specification.

Change prototype A's initial Focus-enabled behavior to Off for the unified default. Keep B's initial Detail behavior. Consolidate C's orientation buttons and the old grid into one Arrange menu. Preserve main's light controls, but reconcile them with Focus and Overview as specified above.

Defer circular cards, arbitrary manual port-side pinning, semantic edge categories, user-authored execution scenarios, global causal inference, universal start/end markers, a full obstacle router, and persistent custom view templates. They are not needed to combine these three capabilities safely.

**Acceptance principle: the combined canvas must make the existing system easier to inspect without hiding, inventing, or silently changing its meaning.**
