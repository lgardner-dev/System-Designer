# Control Flow delivery record

The subsequent [F1–F3 corrective pass](CONTROL-FLOW-REVIEW-FOLLOWUP.md) records
extraction layout, distinct-route and handoff-preview fixes with updated evidence.

Implemented in the ordinary native `system-designer` executable on
`work/add-logic-diagramming`, through the existing Designer, Project and Store.
This is the main-app feature, not a standalone example, viewer or webview.

## Checkpoints

The workspace was clean at reviewed baseline
`cd6bdbefbd493e6bb7157a2cc9492d674d283805`.

| Commit | Result |
|---|---|
| `8a00167f5938ba0a27afc01ae2edd4bee96fb353` | Tested R1–R4 repairs, extraction expansion, exact contract reconciliation and review invalidation |
| `e35d7d1641f27f5936f6da5fa675441c7a4ead02` | Native guided authoring, owner/layer navigation, extraction, information forms and UI-state tests |
| `7d812c8350684be438854451166648b9f7a52d82` | Lifecycle and handoff qualification, embedded prompt, self-design, rationale and native modal repair |
| `dfa7918ae1acb3bcb9916f3739521d4ea8fef8ae` | Bounded handoff scrolling and leaf-selection correction found during native smoke |

The evidence checkpoint containing this record adds screenshots, sample, logs and
one additional structural expansion regression. Find its exact SHA with
`git log -1 --format=%H -- docs/CONTROL-FLOW-DELIVERY.md`. Its production source is
identical to `dfa7918`. No merge, push, release publication, dependency or
CI-permission change was performed.

## Delivered behavior and source ownership

- Root and leaf flow authoring with explicit version promotion, editable steps and
  alternatives, drag/form connections, purpose and primitive criteria. Interfaces
  retains its structural canvas and editing controls.
- On-demand bounded suggestions and selected-region extraction: choose work, name
  its responsibility, review outcomes/information, then publish once. Cancellation
  leaves the project untouched; one Undo reverses extraction.
- Repeated calls share one component definition. Cross-view selection, call-site
  choice and component entry are separate actions. Leaves need no fake system.
- Editable information requirements, exact optional port/wire associations,
  structured contracts and one atomic impact review across both layers.
- Save/reopen, undo/redo, backup/recovery serialization, deletion blockers and
  ordinary Handoff support both layers. The CLI validates exact packet formats
  without writing.

Core ownership is in `src/behavior/{edit,extract,exchange,validate,model}.rs` and
`src/edit/connection.rs`. Native authoring is in `src/ui/flow/`, owner/layer state
in `src/ui/scope.rs`, shared path geometry in `src/ui/canvas/geometry.rs`, and the
existing dialog shell, inspector and handoff. The initialization prompt remains
embedded from `assets/initialization.txt`.

[MODEL.md](../MODEL.md) documents formats, clearing, reconciliation, review rules
and conservative stamps. [The workflow guide](CONTROL-FLOW-WORKFLOW.md) documents
the algorithm, walkthrough, failures and five inspected prior-art sources with
their actual influence and limitations. README and Help provide guidance;
[ARCHITECTURE.md](ARCHITECTURE.md) maps source ownership.

The application's [native v2 self-design](../design/system-designer.project.json)
retains 7 immediate root components, 8 systems, 38 components overall and 15
contracts. Behavior scopes describe implemented responsibilities under the same
tree, including repeated use of the Canvas component. Normal **File → Open
application design** was exercised in the release app.

## Core review findings and verification

R1–R4 were reproduced before changing their implementations: the initial behavior
integration run reported **22 passed, 4 failed**. The named regressions were
`r1_missing_flow_is_not_clear`, `r2_reserve_crossing_transition_identity`,
`r3_connection_refines_extracted_information_atomically`, and
`r4_missing_and_duplicate_returns_are_draft_issues`. All now pass. No finding was
disproved. The initial failures were observed during implementation; checked-in
logs below record final successful runs.

| Command | Executed result |
|---|---|
| `cargo fmt --all -- --check` | Exit 0; no output |
| `cargo test --locked --no-default-features` | Exit 0; **124 passed**, 0 failed ([log](evidence/control-flow/headless.log)) |
| `cargo test --locked --all-targets` | Exit 0; **195 passed**, 0 failed ([log](evidence/control-flow/all-targets.log)) |
| `cargo clippy --locked --all-targets` | Exit 0 with warnings, detailed below ([log](evidence/control-flow/clippy.log)) |
| `cargo build --locked --release --bin system-designer` | Exit 0; 54.65 s ([log](evidence/control-flow/release.log)) |
| `cargo run --locked --no-default-features --bin designer-check -- design/system-designer.project.json` | Exit 0; `Valid project: 8 systems, 38 components, 15 contracts` ([log](evidence/control-flow/self-design.log)) |

The 195 total is 71 UI/library, 36 behavior, 51 core, 24 exchange, 4 self-design and
9 storage tests. The 124 headless tests are a subset, not extra unique coverage.
Final test logs include the evidence checkpoint's parent-reentry regression.
Release build and native reopen used `dfa7918`; that test addition changes no
production source.

Clippy reported 25 library warnings; library tests reported 26 including those 25
duplicates and one `chunks_exact` suggestion. Behavior tests reported 37
`unwrap_used` warnings. Library warnings concern assignment spacing and collapsible
conditionals. This is a successful command with warnings, not a warning-free run.

Executed regressions cover:

- Missing/null/empty flow, referenced-child clear rejection, merged validation and
  unchanged history after rejected candidates.
- Expansion comparisons for sequences, decisions, alternative merges, internal
  loops, distinct outcomes, nested extraction, parent reentry and incoming,
  outgoing/internal data. Crossing IDs are reserved; multi-entry and existing-call
  selections are rejected. Comparisons establish structure, not execution.
- Exact contract closure, unrelated-channel preservation, review invalidation,
  stale previews, cancellation and one-undo publication.
- Legacy v1 preservation; v2 save/reopen/recovery of both layers/layouts; exact
  deletion blockers and occurrence deletion retaining its component.
- Both packet formats, wrong target/format, read-only context, stale bases, layout
  exclusion, hidden-layer preservation and CLI validation without writes.
- Raw egui events for both drag directions, multiselection, preview/cancel/apply,
  move/Escape, edit/delete/undo, cross-view selection, enter/back, information review,
  explicit clear confirmation and full-workspace dialog sizing/scrolling.

## Native smoke and screenshots

Environment: Zorin OS 18.1 (Ubuntu noble base), Linux x86_64, Rust
`1.98.1 (48a229cea 2026-09-01)`, Cargo `1.98.1 (797e8a9bc 2026-08-05)`.
The ordinary app ran on isolated Xvfb X11 at 1600×1050 with software rendering.
`xdotool` supplied actual pointer/keyboard events, `xclip` accessed the X11
clipboard, and ffmpeg captured actual pixels. Tools were downloaded as Debian
packages and extracted under `/tmp/system-designer-native-tools`; none were
installed system-wide. No automation-only application UI was added.

The initial document was programmatically written as empty v1 solely to give
ordinary **Save** an existing filename without an OS chooser. Starting the flow,
editing the Validate-and-normalize action, declaring input/output needs,
selecting/extracting **Import normalization**, defining the required `source_name`
string field of **Imported record**, reviewing/applying its exact bindings,
navigating both views, recording primitive criteria, adding a second call of the
same component and inspecting both occurrences were native UI actions. Ctrl+S
saved the result.

Handoff export was copied from the native app. A simulated returned proposal was
prepared programmatically by changing only parent `call.1`'s occurrence name to
**Normalize the first record**. It was pasted, validated and applied through native
Handoff controls, then saved. Comparing saved JSON before/after confirmed that
interface systems, contracts, child behavior and both layouts were identical.
Only the intended root occurrence name changed.

The saved sample was reopened by the normal release executable's filename argument
and fitted using the UI. **File → Open application design** then loaded the
embedded project normally. Exiting that unsaved copy displayed the ordinary unsaved
prompt; **Discard and continue** closed it.

All images are unmodified native captures. The first nine used
`target/debug/system-designer` at source `7d812c8`; the last three used
`target/release/system-designer` at `dfa7918`. Some earlier leaf images show the
false removed-selection status subsequently corrected in `dfa7918`.

| Screenshot | Actual native state or transition |
|---|---|
| [Extraction preview](evidence/control-flow/extraction-preview.png) | Authored work selected, named responsibility and declared boundaries previewed |
| [Parent Control Flow](evidence/control-flow/parent-control-flow.png) | Created component selected as a Call |
| [Parent Interfaces](evidence/control-flow/parent-interfaces.png) | Cross-view action highlighted the same shared component |
| [Contract impact](evidence/control-flow/contract-impact.png) | New structured contract reviewed against its exact port and parent/child data links |
| [Child flow](evidence/control-flow/child-flow.png) | Entered extracted component behavior |
| [Primitive scope](evidence/control-flow/primitive-scope.png) | Stopping explanation entered through the inspector |
| [Leaf Interfaces](evidence/control-flow/leaf-interfaces.png) | Same leaf's real public ports, with no internal system |
| [Repeated call sites](evidence/control-flow/repeated-call-sites.png) | Both occurrences listed for selection; each was inspected |
| [Scoped rename](evidence/control-flow/scoped-rename.png) | Returned packet validated in ordinary Handoff |
| [Reopened project](evidence/control-flow/reopened-project.png) | Native-authored sample loaded by release filename argument and fitted |
| [Application design](evidence/control-flow/application-design.png) | Embedded model opened with File → Open application design |
| [Unsaved prompt](evidence/control-flow/unsaved-prompt.png) | File → Exit on that unsaved built-in copy |

Release binary SHA-256:
`d4a718f2cc2487ad00cdf0baa71ce25aef2ea62ac3c389e96e2c035a64685704`.

## Reproduce in the ordinary application

From the repository root:

```sh
cargo run --locked --release --bin system-designer -- docs/evidence/control-flow/validate-normalize.project.json
```

The [saved sample](evidence/control-flow/validate-normalize.project.json) has one
shared component, two parent call occurrences, the child flow, one refined input
contract and a deliberately unassigned output contract. Select a Call, **Show in
Interfaces**, **Show uses in Control Flow**, then **Enter component** to inspect
identity sharing and the leaf boundary. For fresh authoring use **File → New**,
**Start a flow**, then the [walkthrough](CONTROL-FLOW-WORKFLOW.md).

Validate the returned rename against its original document, not the already-renamed
sample:

```sh
cargo run --locked --no-default-features --bin designer-check -- docs/evidence/control-flow/before-scoped-rename.project.json docs/evidence/control-flow/returned.behavior.json
```

Executed successfully: `Scope replacement validates. No files changed.` Validating
the final sample itself reported `Valid project: 1 systems, 1 components, 1
contracts`. The native export is [4,847 bytes of pretty JSON](evidence/control-flow/exported.behavior.json);
the [returned proposal](evidence/control-flow/returned.behavior.json) is 4,854 bytes.
The separate policy-regression child packet measured 1,458 compact JSON bytes;
this fixture-specific measurement is not a general transfer-size guarantee.

## Remaining limits and deferred work

R5's minimal-context narrowing is deliberately deferred. Behavior packet version
1 still includes the full contract catalog and repeated immediate-child structure;
some v2 interface packets retain sibling behavior interiors as preserved context.
Shrinking those boundaries safely requires explicit protocol versioning and stale
context tests. This delivery preserves the policy, discloses it in the
UI/prompt/model and tests its conservative behavior: unrelated catalog changes can
stale a packet, while unrelated sibling flow changes outside its context do not.
No minimal-context delivery or new hash convention is claimed.

Extraction is bounded to local Action/Decision/Merge regions: 512 selected members;
suggestions at most 80 flow steps and 24 candidates. Existing Calls and incident
exact wire associations must be refined/reconciled separately. Purpose and
information review remain human judgments. No runtime, concurrency, predicate
interpreter, RPST guarantee, inferred dependency completeness or automatic
responsibility detection is provided. Saved-layout edits conservatively stale
extraction previews; camera movement does not.

Native smoke qualifies the Linux X11 path described above. Windows, macOS,
Wayland, hardware rendering, installers, accessibility, OS Open/Save As dialogs
and the native recovery chooser remain untested. Save/reopen and clipboard handoff
were exercised natively; backup/recovery and file-error behavior have document/
storage test coverage. Native form buttons were used for editing; double-click
event delivery was not established by this smoke. Remaining platform and shell
interactions belong in the existing [release checklist](RELEASE-CHECKLIST.md).
