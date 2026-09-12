# Unified canvas integration and verification

Implements the approved View / Focus / Arrange specification on baseline `267db9364311327ca0b7e663f854b6569ef6b18f`, retaining four-sided ports and directional lights. Algorithms and interactions from the arrangement, overview and focus/trace experiments are reconciled into one canvas, one explicit selection model and one document session rather than stacked experimental frontends.

The implementation commit is `e0df71ca9a2d140b71a77d325793d4760cba493e`. This subsequent documentation correction does not change executable source or the embedded resources. PR #6 is the clean integration candidate; the earlier `work/unified-canvas` transport/workbench branch must not be merged.

## Scope

Exact member inspection/editing, counted directional summaries, one-hop focus, physical-port trace navigation, viewport-aware Back trace, connection-aware arrangement in both orientations, footprint-aware Grid, frozen gesture attachments and Focus-over-Lights precedence are integrated. The embedded prompt and self-design are updated. Existing `MODEL.md`, semantic records, canonical scoped exchange, Store publication, dependencies and packaging workflows are unchanged.

No format migration, new dependency, webview, runtime network service or workflow executor is introduced. Circular nodes, manual side pinning, executable scenarios, semantic relationship categories, a complete obstacle router and persistent view templates remain deferred.

## Observed automated verification

The Linux integration workbench ran in [run 34658785850](https://github.com/lgardner-dev/System-Designer/actions/runs/34658785850), evidence artifact `unified-canvas-workbench`.

| Check | Observed result |
|---|---|
| Rust formatting | Passed |
| `cargo test --locked --all-targets` | **142 passed; 0 failed** |
| Test groups | 54 library, 51 core, 24 exchange, 4 self-design, 9 storage; both binary test targets contain zero tests |
| `cargo clippy --locked --all-targets` | Completed successfully, with warnings; not warning-free |
| Native editor and headless validator release builds | Passed |
| Embedded design validation | Passed: 8 systems, 38 components, 15 contracts |

The earlier preliminary count of 179 was incorrect. The table above is counted from the downloaded test log, not inferred from source declarations.

The ordinary native workflow for implementation commit `e0df71c` is [run 34660781675](https://github.com/lgardner-dev/System-Designer/actions/runs/34660781675).

| Platform | Observed checkpoint |
|---|---|
| Ubuntu 24.04 | Formatting, tests, Clippy command, release binaries, embedded design, installer packaging, and installer install/uninstall round trip passed. Linux artifact produced. |
| macOS | Formatting, tests, Clippy command, release binaries, embedded design and application/DMG packaging passed. macOS artifact produced. No native macOS UI interaction was observed. |
| Windows | First attempt passed formatting, tests, Clippy command, release compilation and embedded design validation. Installing NSIS failed, so no Windows installer artifact was produced in that attempt. A retry was started; at this documentation checkpoint it had passed tests and was rebuilding the release binaries. The workflow is authoritative for later status. |

Clippy success means the command completed; existing and reported warnings are not represented as a warning-free lint qualification. The platform jobs are not substitutes for installed-app interaction tests.

## Actual Linux graphical smoke

The Linux installer from the ordinary `e0df71c` CI artifact was installed into an isolated prefix. Its native executable was run under Xvfb/Openbox with software rendering, in a separate test home/data directory. Actual screenshots were captured and inspected for Detail, Overview, connection-aware arrangement, Incoming focus, and an exact connection's contract dialog. The modal was cancelled without applying a semantic edit.

Two environmental limitations prevent treating this as a complete native acceptance test:

1. The X11 application window initially remained unmapped in the virtual display. It was manually mapped before the app rendered and the inspected interactions were exercised. This does not establish unattended launch behavior in this environment.
2. After an arrangement made the document dirty, recovery reported `Input/output error (os error 5)`. Independent `os.fsync` probes also failed with error 5 in both the isolated data directory and `/tmp`. This supports an environment limitation, but durable native saving/recovery is not qualified here. The recovery error is visible in the captures and was not suppressed.

The native smoke did not establish every hierarchy-trace action, Windows/macOS GUI behavior, accessibility, physical-GPU behavior, or usability on the user's actual dense Method design. Those remain distinct from the automated interaction tests.

## Rejected or limited evidence

The earlier workbench runner could not launch the app because it lacked `libxkbcommon-x11.so.0`. Its shell step failed to propagate the child-process failure and captured only the window-manager desktop. That green step and image are rejected as application evidence. The later local graphical check above used the ordinary CI installer and a different available runtime environment.

The workbench's final source push failed on a runner-local Git configuration permission error. Source was published separately as an ordinary clean integration commit; no transport scaffolding or workbench workflow is part of PR #6. The embedded model's descriptive purpose text was refreshed before ordinary CI; ordinary CI validates that exact published design.

## Combined regression coverage

Tests exercise all 24 View × Focus × Lights combinations and all three scope packet types, preserving semantic content/base hashes and avoiding dirty publication from view state. They also cover partial summary counts, exact-edge identity across view changes, rejection of summary deletion, exact contract dialog/cancellation, one-hop rather than causal propagation, effective boundary identity, full Back trace restoration, stale target handling, default/document resets, animation suppression and shared geometry.

Arrangement tests include deterministic cyclic and acyclic placement, both orientations, measured nonoverlap on the embedded design, full-scope preservation, and one-step undo/redo. Existing model, scope, storage and four-sided pointer regressions remain intact.

## Release boundary

Use CI results for the exact intended revision, not checks that tested only the earlier main tree. An eventual Windows installer must come from successful Windows packaging of this implementation; an older installer is not a substitute.

Inspect the installed editor with the actual dense Method JSON before adopting the combined behavior. A screenshot does not provide enough information to reconstruct that authoritative project. Keep comparison branches until the integrated native behavior has been reviewed. No main merge, signing, notarization or public release was performed by this integration. This document records a qualified implementation candidate and limited observed interactions, not full product or usability qualification.
