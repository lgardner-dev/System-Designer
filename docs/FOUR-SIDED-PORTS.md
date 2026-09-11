# Four-sided ports and direction lights

This iteration changes presentation, not the System Designer project format.

## Port placement

Cards remain rectangular. Each connected port can appear on the top, right,
bottom or left. A deterministic local heuristic scores sides against the actual
neighbouring components; it does not infer a process order. Ports on the same
side are ordered toward their neighbours. Moving a component updates its port
placement. Unconnected draft ports fall back to input-left/output-right until
there is a connection to orient them.

A physical port with several connections still has one anchor. It is not split
into multiple independently editable ports. Child frames project the same owner
ports, with their effective inside-facing directions. Source/destination labels
still come from actual parent connections.

Inputs are hollow circles and outputs are filled circles at the current level.
The static arrowhead follows the actual destination tangent rather than always
pointing right. Side is never used as a substitute for semantic direction.

Top/bottom labels have their own horizontal slots. Crowded names can be shortened
on the canvas; hover a port or use the inspector for complete text. Cards can grow
to provide port space. Existing saved component positions are retained. This is
not a full collision-free auto-layout or obstacle router: dense diagrams can
still need manual positioning, and unrelated cards can still obstruct a wire.

## Direction lights

The canvas offers **Off**, **Selected**, and **All**. All is the initial setting;
Off is retained when navigating within the application session. Selected animates
the chosen wire or the actual incident wires of the selected component/boundary.

A white core, light-blue glow and short fading trail move from the edge's declared
source toward its destination. Position follows sampled arc length rather than
Bezier parameter time, so bends do not arbitrarily accelerate the light. A brief
gap separates arrival from the next pass. Static arrows remain when lights are off.

This is a **direction preview, not live execution**. Speed and spacing do not
encode timing, traffic, throughput, authority, feasibility or progress. The glow
uses translucent drawing layers, not an added bloom postprocessing pipeline.
Animation repaint requests stop when the canvas is disabled or the window is
unfocused, when Off is selected, and when no eligible wire is visible. No new
rendering dependencies are introduced.

## Model and exchange

Port direction, contract identity/version, edge endpoints, child ownership,
project schema and scoped-exchange hashing rules are unchanged. Side choices,
wire geometry and animation settings are not new JSON fields. Rendering does not
publish edits or create undo records. Wire creation and editing still require the
explicit contract dialog and the existing full-project validation.

The embedded application design maps geometry, drawing, hit testing and motion to
their source locations. The embedded initialization prompt explains why visual
side and moving lights must not be interpreted as additional semantic fields.

## Verification

`src/ui/canvas/tests.rs` exercises all four sides with inputs and outputs,
forward/reverse pointer gestures, click connection, child boundaries, fan-out,
self-loops, path hit testing, animation direction, gaps, Off, and preservation of
project/exchange state. The existing core, exchange, storage and self-design tests
remain applicable and are not weakened.

Run the ordinary repository checks:

```sh
cargo fmt --all --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets
cargo run --locked --release --bin system-designer
```

Native visual review should additionally check a dense real project, drag a card
around its neighbours, connect from each side, inspect labels at different zoom
levels, and compare All/Selected/Off. Passing headless tests does not establish
native GPU rendering quality, hardware input delivery, or a measured readability
improvement. Circles, edge aggregation, scenario walkthroughs and graph-aware
node auto-layout are deliberately left for later iterations.
