# F1–F3 corrective pass

Continued on `work/add-logic-diagramming` from reviewed commit
`f307fe756f303fd093a67b0b9af61284a49a6961`. The working tree was clean and there was
no newer work to reconcile. The supplied fixtures are retained unchanged under
[`tests/fixtures/control-flow-review`](../tests/fixtures/control-flow-review/).

## Checkpoints

| SHA | Change |
|---|---|
| `dab1667ad6a845c19a84c542edc2c1f53a2b39df` | F1 layout, F2 exact curved routes, F3 semantic review, regressions and workflow/model documentation |
| `b410322a6747a249cc67a5fc7a9aa383a08e91fc` | Readable exact references in change details, without Rust wrapper/type names |
| `c688acaf38c432c5a98cab2c042f53790d52617b` | Compare typed data endpoints so a literal ID cannot be confused with a boundary/unassigned display placeholder |

The evidence checkpoint containing this record adds command logs, actual native
captures, saved samples and the manifest. Its SHA is available with
`git log -1 --format=%H -- docs/CONTROL-FLOW-REVIEW-FOLLOWUP.md`.
No merge, push, release publication, dependency or CI-permission change was made.

## Reproduced before fixes

All three findings were confirmed before changing production behavior; none was
disproved. The supplied review screenshots were inspected, and the cases were
reproduced locally as failing regressions:

- F1's parent regression failed with `entry` and `call.1` both at `(80,70)`;
  its child regression failed with `a` at `(420,230)` overlapping `outcome.1`
  at `(420,300)`. Both tests failed, with 0 passing in that initial run
  ([log](evidence/control-flow-review/reproduce-layout.log)).
- The raw F2 canvas test found Accepted and Rejected labels at the identical
  screen position `(750,502)`. The F3 ordinary-dialog test found no removal count
  in the validated summary. That initial filtered UI run had 2 failed and 2
  preexisting passed tests ([log](evidence/control-flow-review/reproductions.log)).

## Repairs and boundaries

**F1:** Extraction overlays sparse saved positions on the complete original
default layout before changing membership. It freezes retained parent positions
and places the Call at the selected entry's occupied position. Child work keeps
its relative geometry; synthetic entry/outcomes receive reserved rows outside its
bounds. `behavior/layout.rs` shares small, renderer-independent footprint
measurements with the native canvas. The change remains one currentness-checked,
validated, undoable Store publication.

**F2:** Transitions are grouped by unordered endpoint IDs, including reciprocal
edges, then ordered by transition ID. Separate tangent-continuous cubic lanes
retain perimeter attachments and actual source/destination direction. Repeated
self-loops have distinct attachments and curved reach. One sampled Path still
drives drawing, picking, static arrows and lights; labels follow their own paths.
Names and JSON array order do not determine route assignment.

**F3:** `behavior/changes.rs` compares validated candidates by owner and identity.
A compact added/removed/edited count expands into exact step, transition,
information and metadata edits. Details include purpose/kind/target, control
endpoints/conditions/outcomes, data endpoints and exact port/contract/wire
associations, review flags, primitive criteria and provenance. Stable draft-issue
keys distinguish new issues from renamed text or a reduced existing review count.
References are displayed as IDs and exact contract versions, not Rust values.

Null-clearing and emptying an existing flow are disclosed separately and require
the visible confirmation checkbox. Creating a new empty draft does not. Valid
incomplete candidates remain saveable. Apply reconstructs and validates the
candidate and verifies the reviewed changes before publishing. Missing payload,
read-only context and stale checks remain intact. There are no protocol, hash or
context-policy changes; the accepted R1–R4 repairs and deferred R5 policy remain.

## Executed checks

Final checks ran on production source `c688aca`, with locked dependencies:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | Exit 0, no output |
| `cargo test --locked --no-default-features` | Exit 0: **136 passed**, 0 failed ([log](evidence/control-flow-review/headless.log)) |
| `cargo test --locked --all-targets` | Exit 0: **215 passed**, 0 failed ([log](evidence/control-flow-review/all-targets.log)) |
| `cargo clippy --locked --all-targets` | Exit 0 with warnings ([log](evidence/control-flow-review/clippy.log)) |
| `cargo build --locked --release --bin system-designer` | Exit 0 ([log](evidence/control-flow-review/release.log)) |
| `cargo run --locked --no-default-features --bin designer-check -- design/system-designer.project.json` | Exit 0: `Valid project: 8 systems, 38 components, 15 contracts` ([log](evidence/control-flow-review/self-design.log)) |

The 215 tests comprise 79 UI/library, 36 original behavior, 8 behavior-diff,
4 extraction-layout, 51 core, 24 exchange, 4 self-design and 9 storage tests.
The 136 headless tests are a subset, not additional unique coverage. Repeated
runs during implementation are not added to these totals.

The three native saved snapshots and both returned packets also passed
`designer-check`; exact commands and outputs are in
[artifact-checks.json](evidence/control-flow-review/artifact-checks.json).

Clippy reports 25 library warnings; library tests report 26 including those 25
duplicates and one `chunks_exact` suggestion. The original behavior tests report
37 `unwrap_used` warnings. Warning counts are unchanged from the reviewed baseline;
the command succeeded but is not warning-free.

Added regressions cover 1/3/8-step extraction with default, manual and sparse
layouts, loops, multiple outcomes, retained positions, deterministic preview,
relative geometry and Undo/Redo; actual native-scene card bounds; two/three sibling
routes, reciprocal routes and repeated loops across all supported shapes; finite
perimeter geometry, stable ordering, exact raw pointer selection and static arrows
with lights Off. Diff tests cover unchanged/reordered packets, guard/outcome-only
edits, retargeted calls without renaming, step/data removals, meaningful fields and
associations, empty versus null, and identity/placeholder separation. Full-workspace
UI tests verify expandable exact changes, new issues, confirmation, valid draft
Apply, cancellation, invalid/stale input and unchanged history/generation.

## Native evidence

The ordinary `target/release/system-designer` ran on Linux X11 with software
rendering in isolated Xvfb display `:94` at 1600×1050. Test XDG data/config/cache/
runtime directories were under `/tmp/system-designer-followup-native`. Xvfb,
xdotool and xclip came from the previously extracted local test tools; no system
packages were installed. Native XTest clicks/keys operated the app, and ffmpeg
captured unmodified full frames. Mapping/focusing the window was performed
explicitly because this isolated display had no window manager.
Initial focus/clipboard attempts needed retries; captures follow successful
native input. [GLX inspection](evidence/control-flow-review/graphics.log) reports
Mesa llvmpipe with acceleration disabled.

For F1, a scratch copy of the supplied before-extraction fixture was loaded by
the normal filename argument. Ctrl-selection of `a`, `b`, `c`, responsibility
naming (**Import record**, **Prepare one record**), Preview, Create, Fit, Save,
Undo/Redo, Back and child entry were native actions. The saved result differs from
the supplied after-extraction fixture **only in `flow_layout`**. Parent `call.1`
is now `(420,70)`, while retained `entry` remains `(80,70)` and `done` remains
`(420,300)`. Child entry is `(420,70)`, moved `a` is `(420,252)`, and its outcome
is `(80,664)`, with no marker/work overlap.

For F3, the supplied returned-delete-transition packet was pasted into Handoff
against that native extraction result. It remains compatible because saved layout
does not enter the packet's semantic base. Validate and expanding the exact changes
were native actions. Apply/Save removed only root `t3`; a saved comparison verified
that all other fields, layers and layouts were identical. Undo restored it. The
saved extraction was then reopened in the ordinary release app.

The separate [empty-flow packet](evidence/control-flow-review/returned-empty.behavior.json)
was prepared programmatically by replacing only the exported flow with an empty
object. Native Validate disclosed five removed records and kept Apply disabled
until confirmation. Checking the box, Apply/Save, then one Undo/Save were exercised.
The shared component and child behavior survived; Undo restored the full saved
extraction and both layouts.

For F2, the supplied same-endpoint fixture was loaded programmatically via the
normal filename argument. Fit, Lights Off and clicking each distinct curve were
native actions. Both **accepted** and **rejected** were independently selected and
shown by exact ID in the inspector. The fixture was not modified or reauthored.

| Actual native capture | Source commit |
|---|---|
| [Before extraction](evidence/control-flow-review/01-before-extraction.png) | `dab1667` |
| [Boundary preview](evidence/control-flow-review/02-extraction-preview.png) | `dab1667` |
| [Corrected parent](evidence/control-flow-review/03-extracted-parent.png) | `dab1667` |
| [One Undo restores original work](evidence/control-flow-review/04-extraction-undo.png) | `dab1667` |
| [Corrected child](evidence/control-flow-review/05-extracted-child.png) | `dab1667` |
| [Removed return and introduced issues](evidence/control-flow-review/06-handoff-removed-return.png) | `c688aca` |
| [Applied, saveable incomplete draft](evidence/control-flow-review/07-applied-incomplete-draft.png) | `dab1667` |
| [Reopened saved extraction](evidence/control-flow-review/08-reopened-extraction.png) | `c688aca` |
| [Empty-flow confirmation](evidence/control-flow-review/09-empty-flow-confirmation.png) | `b410322` |
| [Accepted selected, lights Off](evidence/control-flow-review/10-accepted-return-selected.png) | `b410322` |
| [Rejected selected, lights Off](evidence/control-flow-review/11-rejected-return-selected.png) | `b410322` |

Exact source/binary and artifact hashes are in the
[manifest](evidence/control-flow-review/manifest.json). The native snapshots are
[after extraction](evidence/control-flow-review/after-extraction.project.json),
[after removing the return](evidence/control-flow-review/after-return-removal.project.json)
and [after confirmed emptying](evidence/control-flow-review/after-empty.project.json).
These were saved through the app, not synthesized for capture.

## Reproduce and remaining limitations

Open the corrected sample in the ordinary application:

```sh
cargo run --locked --release --bin system-designer -- docs/evidence/control-flow-review/after-extraction.project.json
```

To repeat F1, copy `tests/fixtures/control-flow-review/before-extraction.project.json`
to a scratch file, open it, select `a`, `b`, `c`, and extract with the name/purpose
above. Inspect the parent and enter the child, pressing Fit where needed. For F2,
open `tests/fixtures/control-flow-review/coincident-returns.project.json`, set
Lights Off, and click each return curve. For F3, open the corrected extraction,
paste `tests/fixtures/control-flow-review/returned-delete-transition.behavior.json`
into **AI handoff → Load changes**, Validate, and expand **Exact behavior changes**.

The same packet can be checked without writing:

```sh
cargo run --locked --no-default-features --bin designer-check -- docs/evidence/control-flow-review/after-extraction.project.json tests/fixtures/control-flow-review/returned-delete-transition.behavior.json
```

Previously saved overlapping layouts are not automatically migrated. The new
extraction rule prevents new marker/card overlaps in initially nonoverlapping
scenes. Lane separation is local to endpoint groups, not global obstacle avoidance;
dense diagrams can still have crossings or label collisions. Adding/removing a
sibling can redistribute that group's lanes. Reciprocal and repeated-loop routes
have pure geometry/raw UI coverage, while native F2 capture covers the supplied
two-return fixture. No runtime or execution-equivalence claim is made.

This pass qualifies the described Linux X11 interactions, not Windows/macOS GUI,
Wayland, physical GPU rendering, accessibility, OS file choosers, installers or
release distribution. Model versions, exact contract semantics and the explicitly
deferred full-catalog context policy remain unchanged.
