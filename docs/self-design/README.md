# System Designer: inspect its own interfaces and control flow

This is the application's self-design, not the Foundry Method fixture. It preserves the existing seven top-level responsibilities and all existing contract definitions. It adds source-mapped behavior for the root and **every component**, and exposes narrower validator/publication responsibilities until their primitive rules can be inspected.

## Open it

```sh
cargo run --locked --release --example self-design-atlas
cargo run --locked --release --example self-design-atlas -- --check
```

The native executable embeds the design, initialization prompt and approved icon. No server, webview, sidecar data file or AI service is required. Double-click a call/component or select it in the tree to enter its scope. **Back** restores the prior scope, layer and selection. The two layers have independent session cameras. The direction lights offer Off, Selection and All; they remain a direction preview, never a simulation.

The production application's **File > Open application design** opens the updated **interface projection** only. Its existing generic editor has not gained arbitrary control-flow authoring in this change. The dedicated native self-design atlas displays both layers at all scopes. This is deliberate: no unreviewed production file-format migration or behavior-scoped replacement implementation is smuggled into a design update.

## One source, two layers

`design/system-designer.atlas.json` is the authored paired model. Its `project` field is a valid `system-designer-project` version-1 document. Behavior scopes refer to that project's component identities; they do not copy the component tree. `design/system-designer.project.json` is the generated compatibility projection, tested for exact semantic equality with `atlas.project`. When changing interfaces, update the authored atlas and regenerate the projection together. The production scope hash convention is unchanged.

Each behavior scope is bound to either `@root` or an existing component ID. A component does not need structural children to have a behavior graph. Local actions and questions carry their input, output and primitive rule. A call targets an immediate child only. Transition `exchanges` references only explicitly associated exact interface edge IDs at this level; an empty list does not assert there is no information use, and it does not manufacture a channel from sequencing.

The atlas is **not** a production project or a scope replacement packet. Exporting its interface projection cannot carry behavior. Its read-only viewer cannot apply returned AI edits. The embedded instructions say so explicitly.

## Recommended drill-down

**System Designer > Design model > Structural validation > Local edge validity > Compare exact contracts**

At each scope, switch **Control Flow / Interfaces**. At the final leaf, inspect the decision:

```text
source.contract.is_some() && source.contract == destination.contract
```

The comparison includes both ID and version. The leaf Interfaces view shows its actual public ports, not a fabricated internal system. No children are invented to make the last screenshot look busier.

A second useful route is **System Designer > Edits and history > Publication and history > Install new snapshot**. Its primitive operations are assigning the admitted snapshot, clearing redo, and incrementing the generation. Publication is not durable file saving.

## Coverage and limits

The authored design has 51 components, 11 structural systems and 52 behavior scopes including the root. Every scope has both a native rendered view and an explicit source mapping. Structural levels have at most eight immediate components; the authored behavior diagrams also have at most eight work/question steps, excluding entry/outcome markers. This is a fixture design choice, not a general validator restriction.

Source mappings describe implementation evidence at baseline `26c6e999cfc303dfc4828d6682ef79a0941f2fa3`. Behavior is a design abstraction, not automatically recovered machine instructions, complete path coverage, a code-generation specification or proof of equivalence. The additional tree decomposition maps responsibilities already inside existing functions; it does not create new runtime modules. Pending generic CFG work is recorded in `planned` and displayed as **not implemented**.

The local cyclomatic metric normalizes declared outcomes through a virtual common exit. It is neither feasible execution count nor test sufficiency. Visual collapsing does not change it. Runtime concurrency, memory allocation failure, OS internals, graphics driver behavior and abstract human decisions are not silently replaced with invented deterministic logic.

## Capture every scope

```sh
cargo run --locked --release --example self-design-atlas -- --capture screenshots
```

Capture visits all 52 scopes in both layers, writes 104 actual frame images and a manifest identifying every scope/layer. This demonstrates rendered output, not manual interaction or usability qualification. `--scope model.validation.edges.contracts` opens a specific component directly.

Screenshots and platform executables are workflow artifacts, not a public release. Windows/macOS signing and physical-desktop verification remain separate release obligations.
