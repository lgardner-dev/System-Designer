# C — Connection-aware layout

Branch: `experiment/connection-layout`

Common baseline: `267db9364311327ca0b7e663f854b6569ef6b18f` (merged four-sided ports and direction lights). This is an independent comparison candidate, not stacked on either of the other two experiments.

This is the strongest candidate for improving the global geometry without concealing any relationship. It borrows layered-layout principles from ELK, adapted to this editor's freely cyclic local graphs and four-sided ports.

## Try it

Select **Left to right** or **Top to bottom**, then **Arrange by connections**. This explicitly rearranges the current level and fits it into view. **Undo** restores the previous positions in one action. The old Auto-layout grid remains available for comparison.

The algorithm finds strongly connected groups on a temporary local drawing graph. Feedback groups are arranged as small rings (two-node feedback is a pair), while connections between groups determine directional layers. Bounded barycenter sweeps reduce some crossings between layers. Port-aware card dimensions are measured again as placement changes and reserved drawing cells grow until the actual cards fit.

A feedback group is not a new component. No edge is reversed or deleted to obtain a cleaner drawing. Boundary ports remain derived from the owner, and no artificial Start or End is invented. The graph may have multiple roots, cycles, disconnected components, or only one node.

## Boundaries

Only current-level visual positions are published, using the ordinary validated Store and undo history. Component purposes, ports, contracts, connections, hidden internals, other levels' layouts, and AI scope hashes remain unchanged. Node positions do not encode execution order or prerequisites.

This is a small native heuristic, **not the ELK implementation**, a complete obstacle-avoiding router, or a minimum-crossing guarantee. Large feedback regions can occupy more area than a carefully arranged manual design. Existing arrow/light geometry remains unchanged. There is a bounded sizing iteration; failure leaves the prior layout intact instead of partially publishing.

The implementation is `src/ui/canvas/arrangement.rs`. No new runtime dependency, schema field, graph engine, or semantic grouping is introduced.

## Prior-art reference

https://eclipse.dev/elk/reference/algorithms/org-eclipse-elk-layered.html

ELK is the reference for directional layers and crossing-reduction considerations; the cycle-ring policy is this experiment's deliberate adaptation, not a claim that ELK uses this exact policy.

## Comparison and verification

Use separate copies of the same project for A/B/C. All use the existing version-1 JSON and embedded prompt. Close one preview before opening another on the same file. Do not merge these alternatives cumulatively merely to test them.

For each, try finding a contract's producer, understanding a return path, inspecting an exact child boundary, editing one connection, and exporting the level. Compare correctness and comfort before appearance alone.

Unit and headless input tests are included and must pass alongside the unchanged core/exchange/storage tests. The existing CI runs Windows, Linux and macOS builds; it additionally includes a portable Windows executable in its preview artifact so comparisons do not require overwriting the installed application. Check the PR for exact executed commit/run evidence. A passing build is not a full native interaction, accessibility, performance, or usability qualification.
