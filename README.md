# Interface comparison: B — Compact overview

See [experiment instructions and limits](docs/INTERFACE-EXPERIMENT.md). This branch starts from the merged four-sided-port baseline and contains only this strategy.

# System Designer

A native desktop editor for recursive system designs: components that contain
components, typed ports, explicit local connections, and a bounded way to hand a
slice of the design to an AI assistant and bring the result back.

The application is written in Rust with egui/eframe. There is no webview, HTML,
JavaScript, CSS, backend service, or AI API integration — it is a single native
executable that reads and writes local JSON files and never contacts a network
service.

```sh
cargo run --release --bin system-designer
```

## What it is for

You describe a system as a tree of components. Each component has a purpose and
a set of input/output ports. Each port carries exactly one contract — a named,
versioned, structured type from the project's catalog. Components at the same
level are wired to each other by edges; a component with internals can be
entered, and its own ports appear as the boundary of the level inside it.

The result is a design you can navigate one level of detail at a time, where
every interface between two responsibilities is written down and checked for
consistency. It is a design model, not a workflow runner, an execution engine,
or a semantic-approval system: a structurally valid project is not proof that
the decomposition is good.

* [Data model and exchange format](MODEL.md) — the exact JSON the app reads and writes.
* [Architecture](docs/ARCHITECTURE.md) — how the application is built, and the source map.
* [Contributing](CONTRIBUTING.md) — development setup, tests, and the change workflow.

## Install

Installers are not committed to the repository. Build one, or download the
artifact that `.github/workflows/ci.yml` produces for your platform.

**Windows.** `.\packaging\windows\build.ps1` runs the tests, then packages
`dist\system-designer-setup.exe` — an unsigned per-user install that needs no
administrator rights and no Visual C++ Redistributable. Uninstalling preserves
your project files and recovery data. Building it needs Rust and NSIS.

**Linux.** `packaging/linux/build.sh` produces a self-extracting
`dist/system-designer-setup.sh` that installs a per-user binary and desktop
entry. It is not an AppImage and does not bundle system graphics libraries.

**macOS.** CI builds an unsigned, unnotarized `.app` bundle and DMG. Sign and
notarize it before distributing.

People running the app need none of the build tooling — only the installed
binary. Signing and notarization are release decisions; see
[docs/RELEASE-CHECKLIST.md](docs/RELEASE-CHECKLIST.md).

## Using it effectively

### Decompose only when a boundary earns it

Add a component, write its purpose, and give it ports. Decompose it only when an
independently meaningful responsibility or interface justifies another level —
not because a component feels large. Double-click a component with internals to
enter it; breadcrumbs and the tree take you back out.

Around eight immediate components per level is a readability heuristic, and the
self-design follows it. The validator does not enforce it and will not reject a
larger coherent system. Never invent a boundary just to hit the number.

### Every wire is a deliberate typing decision

Drag between two ports in either direction — or click both, or use **Connect
ports** — and the contract dialog opens. No wire exists until you confirm it.
You either pick an existing contract or define a new structured one; the app
never silently chooses a type for you. Releasing a drag on empty space cancels.

Connected ports that share a channel must hold one exact contract ID *and*
version. Retyping a wire can therefore propagate to fan-out edges and to
mirrored child-boundary ports. The dialog shows the full impact list before you
commit, and any change to an already-assigned port needs explicit consent. Read
that list — the contract belongs to the ports, not to the single edge you happen
to be editing.

If two channels should be able to evolve independently, give them separate ports
rather than widening one contract to cover both.

### Contracts are small on purpose

The shape language is `string`, `integer`, `number`, `boolean`, `enum`, `array`,
and `object` with named fields — deliberately smaller than JSON Schema, so that
interfaces stay readable. Unsupported keywords are rejected rather than silently
ignored. Express anything further in the purpose and field descriptions; the
validator checks structure, not meaning, and never validates runtime payloads.

Reuse an existing contract only when the meaning genuinely matches. A new
version is cheap; a wrong shared type is not.

### Work with AI on the smallest scope that fits

The app contains everything the exchange needs, and performs none of it for you.
**AI handoff → Initialize chat** gives you the complete initialization prompt: it
explains the design method and the exact JSON format, and contains no project
data. Copy it into a fresh chat, then export the smallest scope that covers the
change:

| Scope | Editable | Preserved |
|---|---|---|
| Component | The selected node | Siblings, local edges, deeper internals |
| Level | Immediate nodes and their connections | Hidden child ownership and internals |
| Subtree | The selected level and all descendants | Owner boundary, ancestors, unrelated branches |

Return to the same level, load the complete returned packet, choose **Validate
candidate**, review what it reports, then **Apply validated changes**. Nothing is
published until Apply, and the app revalidates at that moment.

Imports are rejected — with a reason — for changed read-only context, a stale
base token, missing contract definitions, cross-level edges, identity collisions
with outside components, and any silent rewrite of an existing shared type. The
base token is an optimistic concurrency check, not authentication: it exists to
catch a design that moved on while the assistant was working, so neither you nor
the assistant should ever recalculate it to force a stale edit through.

Copying a scope into another service is always your own deliberate action. The
executable does not send it anywhere.

### Files and recovery

Ctrl/Cmd+S writes the real project file. The previous valid version is kept as
`<name>.bak`, and the replacement goes through a synchronized temporary file in
the same directory. Fingerprints detect ordinary external edits, so a changed or
damaged file is never silently overwritten — but they are not an interprocess
lock, so do not edit one project in several app instances at once.

Recovery snapshots are written per session while the project is dirty, and cover
published edits, not half-finished dialog drafts or in-progress gestures.
Recovery always opens as an unsaved copy and never overwrites the original path;
unknown or damaged recovery files are preserved rather than cleaned up. Keep
independent backups of work you care about.

## Explore the application's own design

`design/system-designer.project.json` is an ordinary System Designer project
describing this application, and **File → Open application design** opens it as
an unsaved copy. It has seven top-level components across eight systems in all,
35 component nodes and 14 contracts, with component purposes pointing at the
source paths that implement them.

It is an example, not a default: new projects start blank, with an empty
contract catalog. See [the architecture notes](docs/ARCHITECTURE.md) for what its
edges do and do not claim to mean.

## Known limits

Containment has no fixed depth or node-count limit; deeply nested *field schemas*
are still bounded by serde_json's parsing recursion guard, which is a separate
concern from the flat normalized component tree. Structural validation is not
semantic validation, a runtime payload validator, or a formal acceptance system.
There is no multi-user merge protocol, no auto-updater, no telemetry, and no AI
API client. Undo/redo keeps 50 session steps and is not a persistent audit trail.
Large-project performance and full accessibility have not been qualified.

## License

MIT — see [LICENSE](LICENSE). A dependency license and security review belongs in
your own release process; none is claimed here.
