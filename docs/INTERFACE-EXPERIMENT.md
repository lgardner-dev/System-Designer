# B — Compact overview

Branch: `experiment/compact-overview`

Common baseline: `267db9364311327ca0b7e663f854b6569ef6b18f` (merged four-sided ports and direction lights). This is an independent comparison candidate, not stacked on either of the other two experiments.

This is the strongest candidate for explaining the entire current level before inspecting individual interfaces. It borrows reversible graph folding from yFiles and progressive disclosure from Node-RED.

## Try it

Select **Overview** on the canvas toolbar. (Detail is initially selected to preserve the normal authoring interaction.) Cards become compact, titles wrap, and connection-count badges replace long wire labels. The node origins stay fixed; Fit is explicit.

Only connections between the same **ordered pair** of local components are summarized together. Opposite directions stay separate. Boundary ports and self-loops remain individual. Every badge retains all member edge identities; it does not manufacture a shared contract.

Click a count badge or summary wire. The inspector lists every member with exact endpoint names, contract ID/version, and branch label. **Show exact wire** switches to Detail and selects that specific edge. **Edit this wire** opens the normal contract/endpoint editor for that specific member. **Expand into Detail** restores all detailed wiring; it does not just expand one pair.

A multiple-edge summary is not itself a semantic edge and is not a destructive target: Delete cannot arbitrarily remove a member. Components can still be moved or opened. Pointer wiring uses Detail; the existing Connect ports form remains available in either view.

## Boundaries

All original ports, contracts, edges and hidden child systems remain in Project. AI exports are the full requested component/level/subtree, never merely the visible summary. Switching views changes neither history nor scope hashes. Direction lights on a summary show direction, not the traffic or timing of its individual members.

This is not a bus representation, contract union, arbitrary edge bundler, or new semantic grouping. Global crossings and obstacle avoidance remain unchanged. Wide separation between original node positions remains until the user moves them or requests layout.

The implementation is `src/ui/canvas/overview.rs`, called from the canvas and inspector. Exact Detail geometry and validation remain the existing ones.

## Prior-art references

https://docs.yfiles.com/yfiles-html/dguide/folding/
https://nodered.org/docs/user-guide/editor/sidebar/information

This is an independent native Rust view projection, not a yFiles or Node-RED dependency.

## Comparison and verification

Use separate copies of the same project for A/B/C. All use the existing version-1 JSON and embedded prompt. Close one preview before opening another on the same file. Do not merge these alternatives cumulatively merely to test them.

For each, try finding a contract's producer, understanding a return path, inspecting an exact child boundary, editing one connection, and exporting the level. Compare correctness and comfort before appearance alone.

Unit and headless input tests are included and must pass alongside the unchanged core/exchange/storage tests. The existing CI runs Windows, Linux and macOS builds; it additionally includes a portable Windows executable in its preview artifact so comparisons do not require overwriting the installed application. Check the PR for exact executed commit/run evidence. A passing build is not a full native interaction, accessibility, performance, or usability qualification.
