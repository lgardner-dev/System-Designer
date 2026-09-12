# System Designer

A native desktop editor for recursive system designs: components that contain components, typed ports, explicit local connections, and a bounded way to hand a slice of the design to an AI assistant and bring the result back.

The application is written in Rust with egui/eframe. There is no webview, HTML, JavaScript, CSS, backend service, or AI API integration. The editor reads and writes local JSON files and does not contact a network service.

```sh
cargo run --locked --release --bin system-designer
```

## What it is for

Describe a system as a tree of components. Each component has a purpose and input/output ports. Each connected port carries exactly one contract: a named, versioned, structured type. Components at the same level are connected by edges. Enter a component to design its internals; its own ports appear as the boundary of that internal level.

The result is a design you can navigate one level at a time, with mechanically checked interface consistency. This is not a workflow runner, execution engine or semantic-approval system. Valid structure does not prove a good decomposition.

* [Data model and exchange format](MODEL.md): exact project and scoped JSON formats.
* [Architecture](docs/ARCHITECTURE.md): responsibility boundaries and source map.
* [Contributing](CONTRIBUTING.md): setup and development workflow.
* [Unified canvas UX](docs/UNIFIED-CANVAS-UX.md): accepted interaction specification.
* [Integration and verification notes](docs/CANVAS-INTEGRATION.md): checks and limitations.

## Install

Installers are not committed. Build one or obtain the artifact produced by the repository's native-build workflow for your platform. Use artifacts from the exact revision you intend to test.

**Windows:** `packaging/windows/build.ps1` tests and packages an unsigned per-user NSIS installer. Building needs Rust, the Visual C++ build tools and NSIS. The repository retains its static CRT configuration. Uninstalling preserves project files and recovery data.

**Linux:** `packaging/linux/build.sh` creates a per-user self-extracting installer and desktop entry. This is not an AppImage and does not bundle system graphics libraries. The host must provide the required X11/Wayland/OpenGL runtime libraries; compilation alone does not establish that an arbitrary target host can launch it.

**macOS:** the native workflow builds an unsigned, unnotarized application bundle and DMG. Signing and notarization are separate release actions.

See [the release checklist](docs/RELEASE-CHECKLIST.md) before distributing installers. An artifact from successful CI is not a claim that the installed native interaction experience has been fully qualified.

## Unified canvas

One set of controls composes without changing the underlying design:

| Control | Behavior |
|---|---|
| **View: Detail / Overview** | Exact ports and connections, or compact cards and counted relationships. |
| **Focus: Off / Selection / Incoming / Outgoing** | Quiet unrelated connections; Incoming/Outgoing apply to a whole component. |
| **Arrange: Left to right / Top to bottom / Grid** | Explicit, current-level, undoable positioning action. It is not an automatic mode or execution-order declaration. |
| **Lights: All visible / Selection / Off** | Illustrate direction on eligible displayed connections. Muted focus context never animates. |
| **Fit** | Reframe the viewport without changing saved positions. |

The default is Detail with Focus Off and Lights All visible. Existing saved positions are retained. View and Focus changes do not automatically move components or fit the camera. Arrange considers the complete local graph and measures Detail footprints even when requested in Overview. Subsequent manual movement remains available.

Inputs and outputs can attach to every side of a card. Hollow ports receive and filled ports produce at the current level; direction does not come from the side. A shared port has one anchor per displayed diagram. Port targets remain stable during connection gestures.

### Overview and exact editing

Overview groups only connections between the same ordered pair of distinct local components. Opposite directions remain separate; boundary connections and self-loops stay individual. Every summary retains the exact IDs of its members. It is not a bus, shared contract, or replacement semantic edge.

Click a summary to inspect all members with their full endpoints, contract versions and labels. **Show exact wire** reveals one member in Detail; **Edit this wire** establishes that exact context before opening the normal contract editor. A multi-edge summary cannot be deleted or retyped without choosing an actual member. Partial focus reports both focused and total membership, such as “1 of 4 in focus”. Selecting an exact wire never silently selects its whole group.

### Focus and tracing

Select a component, exact connection or port, then choose Focus. Right-click a port or choose it from the inspector to inspect its exact endpoint. **Source**, **Destination**, **Enter at this port**, **Follow in parent**, and **Back trace** navigate declared relationships and identical owner-port identities. Back restores navigation state, not project history.

A trace does not infer that every input affects every output. Opaque components stop automatic continuation. Keyboard-accessible exact-connection rows provide an alternative to selecting intersecting wires with the pointer.

### Direction lights

Lights follow the actual displayed route, with arrowheads along its destination tangent. The soft white/light-blue pulse is a direction preview, not live execution, traffic quantity, throughput, latency or proof that a branch runs. There is at most one pulse per displayed summary. Off remains off through navigation. Automatic operating-system reduced-motion detection is not implemented; the explicit Off control is always available.

## Authoring

Decompose only when an independently meaningful responsibility, interface or ownership boundary earns another level. Around eight immediate components is a readability heuristic, never a validity cap. Do not invent boundaries merely to hit that number.

Drag between ports in either direction, click both, or use **Connect ports**. No new wire is published until the choose-or-define-contract dialog is confirmed. Cancelling or releasing on empty canvas changes no semantic data. Direct pointer wiring uses Detail; the form is available from either view and reveals Detail first.

Connected ports sharing a channel require the exact same contract ID and version. Retyping can affect fan-out and mirrored child-boundary connections. The dialog lists those effects and requires explicit consent where applicable. Give independently evolving channels separate ports instead of widening one shared contract.

Contracts support string, integer, number, boolean, enum, array and object fields. Unsupported keywords are rejected. Additional intended constraints may be described in prose, but the validator does not enforce that prose or validate runtime payload instances.

## Manual AI collaboration

**AI handoff → Initialize chat** contains the complete embedded design approach and JSON instructions, without project data. Copy it into a fresh chat and export the smallest appropriate scope:

| Scope | Editable | Preserved |
|---|---|---|
| Component | Selected node | Siblings, local edges and deeper internals |
| Level | Immediate nodes and connections | Hidden child ownership and interiors |
| Subtree | Selected level and all descendants | Owner boundary, ancestors and unrelated branches |

Return the complete scope packet to its original level. **Validate candidate** does not publish anything; **Apply validated changes** reconstructs and revalidates before one undoable publication. Changed read-only context, stale bases, missing definitions, illegal cross-level connections, outside identity collisions and silent shared-type rewrites are rejected.

**View, Focus and Lights never narrow an export.** Muted edges remain in their requested semantic scope. Summaries and highlights are not serialized as project objects. Layout-only changes leave scoped canonical bases unchanged. The base is a concurrency token, not authority; do not recompute it to force a stale edit through.

The app sends nothing to an AI service. Copying workplace information into another service is your separate deliberate action and remains subject to your organization's policies.

## Files and recovery

Ctrl/Cmd+S writes the real project file. The previous valid version is retained as `<name>.bak`; replacement uses a synchronized same-directory temporary file. Fingerprints detect ordinary external edits, not race-free multi-process locking. Do not edit one project concurrently in several app instances.

Per-session recovery covers published edits, not unconfirmed form drafts or in-progress gestures. Recovery opens an unsaved copy instead of replacing the original project file. Unknown or damaged recovery bytes are preserved. Maintain independent backups.

View, Focus, trace navigation and Lights do not mark the project dirty. Deliberate movement and arrangement modify the existing saved layout and are undoable. Undo/redo retains 50 session steps, not a persistent audit trail.

## Explore the application's own design

**File → Open application design** opens the embedded `design/system-designer.project.json` as an unsaved copy. It contains seven top-level components, eight systems, 38 components and 15 contract definitions. Canvas has eight immediate responsibilities; every level remains within the advisory eight-component budget. Component purposes identify implementation locations.

This is an optional example, not a seed: new projects start blank with an empty catalog. Both the self-design and initialization prompt are compiled into the executable, with no required resource sidecars.

## Limits

Arrangement is a deterministic heuristic, not a minimum-crossing guarantee or complete obstacle router. Circular cards, manual port-side pinning, executable scenarios, semantic edge taxonomies, persistent custom views and causal inference remain deferred.

Containment has no fixed node-count or depth limit. Deeply nested field schemas remain subject to serde_json's separate parsing guard. There is no multi-user merge protocol, auto-updater, telemetry or AI client. Large-project performance and full native accessibility remain unqualified; consult the integration notes for what was actually executed.

## License

MIT — see [LICENSE](LICENSE). Dependency licensing and security review belong in the release process; no audit is claimed here.
