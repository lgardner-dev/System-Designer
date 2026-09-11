# A — Focus and exact tracing

Branch: `experiment/focus-trace`

Common baseline: `267db9364311327ca0b7e663f854b6569ef6b18f` (merged four-sided ports and direction lights). This is an independent comparison candidate, not stacked on either of the other two experiments.

This is the strongest candidate for day-to-day reasoning about one responsibility at a time. It borrows the interaction pattern of Simulink's source/destination trace, not its execution or signal-dependency semantics.

## Try it

Focus selection starts enabled. Select a component to emphasize its direct incoming/outgoing wires and their endpoints. Select Both, Incoming or Outgoing to narrow a whole-component view. Other cards retain names and positions; context wires are muted and their labels/lights suppressed.

The inspector's **Trace exact connections** section lists the actual source, destination, versioned contract and label. Choose Source or Destination to focus that exact port. Right-clicking a canvas port is the shortcut. A node's port picker provides the non-drag alternative.

**Enter at this port** follows the same physical port onto its child-system boundary. **Follow in parent** goes back through the owner port. **Back trace** returns through the explicit trace history. A leaf does not acquire an invented input-to-output continuation. Select another output deliberately when the design does not declare the relationship.

**Clear focus** or disabling Focus selection restores the complete drawing. Connection gestures temporarily show all targets, and still require the existing explicit contract-selection dialog.

## Boundaries

Focus is presentation only. It does not remove connections, change ports, rearrange components, invalidate scope packets or narrow the contents of any AI export. Counts disclose muted context. The feature is one-step structural tracing, not causal analysis, arbitrary transitive reachability, scheduling, or live execution. It does not solve wire crossings globally.

The implementation is `src/ui/canvas/focus.rs`, called from the canvas and inspector. Existing Store, validation, exchange and file operations are unchanged.

## Prior-art reference

https://www.mathworks.com/help/simulink/ug/displaying-signal-sources-and-destinations.html

This is an independent native Rust implementation inspired by the documented interaction, not copied Simulink code.

## Comparison and verification

Use separate copies of the same project for A/B/C. All use the existing version-1 JSON and embedded prompt. Close one preview before opening another on the same file. Do not merge these alternatives cumulatively merely to test them.

For each, try finding a contract's producer, understanding a return path, inspecting an exact child boundary, editing one connection, and exporting the level. Compare correctness and comfort before appearance alone.

Unit and headless input tests are included and must pass alongside the unchanged core/exchange/storage tests. The existing CI runs Windows, Linux and macOS builds; it additionally includes a portable Windows executable in its preview artifact so comparisons do not require overwriting the installed application. Check the PR for exact executed commit/run evidence. A passing build is not a full native interaction, accessibility, performance, or usability qualification.
