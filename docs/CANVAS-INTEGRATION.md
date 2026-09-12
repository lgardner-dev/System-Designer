# Unified canvas integration and verification

Implements the approved View / Focus / Arrange specification on baseline `267db9364311327ca0b7e663f854b6569ef6b18f`, retaining four-sided ports and directional lights. Algorithms and interactions from the arrangement, overview and focus/trace experiments are reconciled into one canvas, one explicit selection model and one document session rather than stacked experimental frontends.

## Scope

Exact member inspection/editing, counted directional summaries, one-hop focus, physical-port trace navigation, viewport-aware Back trace, connection-aware arrangement in both orientations, footprint-aware Grid, frozen gesture attachments and Focus-over-Lights precedence are integrated. The embedded prompt and self-design are updated. Existing `MODEL.md`, semantic records, canonical scoped exchange, Store publication, dependencies and packaging workflows are unchanged.

No format migration, new dependency, webview, runtime network service or workflow executor is introduced. Circular nodes, manual side pinning, executable scenarios, semantic relationship categories, a complete obstacle router and persistent view templates remain deferred.

## Observed integration workbench results

The integration source was formatted and executed by the Linux job in [run 34658785850](https://github.com/lgardner-dev/System-Designer/actions/runs/34658785850), with evidence artifact `unified-canvas-workbench`.

| Check | Observed result |
|---|---|
| Rust formatting | Passed |
| `cargo test --locked --all-targets` | 179 passed; 0 failed |
| Test groups | 91 library, 34 core, 32 exchange, 7 self-design, 15 storage |
| `cargo clippy --locked --all-targets` | Completed successfully, with warnings; not warning-free |
| Native editor and headless validator release builds | Passed |
| Embedded design validation | Passed: 8 systems, 38 components, 15 contracts |
| Actual Linux window launch | Failed: the runner lacked `libxkbcommon-x11.so.0` |
| Native screenshot inspection | The attempted capture showed only the window-manager desktop, not the application; rejected as application evidence |

The native launch shell step did not correctly propagate its child-process failure. Its green workflow step is therefore NOT accepted as a passed native smoke test. The log and capture were inspected and the limitation is recorded here. Compilation and headless egui interaction tests are distinct from actual native event delivery and rendering.

The workbench's final source push also failed on a runner-local Git configuration permission error. Tested source is published separately as an ordinary integration commit; no transport scaffolding is part of that commit. The embedded model's purpose text was subsequently refreshed to remove obsolete bootstrap status claims; its component/port/edge content remains the tested design.

## Combined regression coverage

Tests include all 24 View × Focus × Lights combinations and all three scope packet types, preserving semantic content/base hashes and no dirty publication from view state. They also cover partial summary counts, exact-edge identity across view changes, rejection of summary deletion, normal exact contract dialog/cancel behavior, one-hop rather than causal propagation, effective boundary identity, full Back trace restoration, stale target handling, default/document resets, animation suppression and shared geometry.

Arrangement tests include deterministic cyclic and acyclic placement, both orientations, measured nonoverlap on the embedded design, full-scope preservation, and one-step undo/redo. Existing model, scope, storage and four-sided pointer regressions remain intact.

## Release boundary

Use the ordinary native CI results attached to the final integration commit for Windows, Linux and macOS compilation/tests/packaging, not runs that tested only the earlier main tree. Per-platform results are reported on the pull request once observed.

No installed Windows/macOS interaction test, completed Linux graphical smoke test, accessibility audit or measured dense-diagram usability study is claimed here. Check the installed editor with the user's actual dense Method JSON, because a screenshot is not enough to reconstruct its authoritative data. Keep the comparison branches until the integrated native behavior has been reviewed. No signing, notarization or public release was performed by this integration.
