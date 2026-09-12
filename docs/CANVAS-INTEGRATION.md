# Unified canvas integration and verification

Implements the approved View / Focus / Arrange specification on baseline `267db9364311327ca0b7e663f854b6569ef6b18f`, retaining four-sided ports and directional lights. The three experiments are reconciled into one canvas, one explicit selection model and one document session.

Implementation commit: **`e0df71ca9a2d140b71a77d325793d4760cba493e`**. Later commits only correct and extend this verification document; they do not change executable source or embedded resources. PR #6 is the clean integration candidate. The earlier `work/unified-canvas` workbench/transport branch must not be merged.

## Implemented scope

Counted directional Overview summaries, exact member inspection/editing, one-hop Focus, physical-port trace navigation, viewport-aware Back trace, connection-aware arrangement in both orientations, footprint-aware Grid, frozen gesture attachments and Focus-over-Lights precedence are integrated. The initialization prompt and self-design remain embedded and are updated.

Version-1 semantic records, canonical scoped exchange, Store publication, dependencies and packaging workflows are unchanged. No format migration, new dependency, webview, runtime network service or execution engine was introduced. Circular nodes, manual side pinning, executable scenarios, semantic relationship categories, a full obstacle router and persistent custom views remain deferred.

## Observed automated results

The Linux integration workbench ran in [run 34658785850](https://github.com/lgardner-dev/System-Designer/actions/runs/34658785850), with artifact `unified-canvas-workbench`.

| Check | Result |
|---|---|
| Rust formatting | Passed |
| `cargo test --locked --all-targets` | **142 passed; 0 failed** |
| Test groups counted from downloaded log | 54 library, 51 core, 24 exchange, 4 self-design, 9 storage; zero in both binary test targets |
| `cargo clippy --locked --all-targets` | Completed successfully with warnings; not warning-free |
| Editor and headless validator release builds | Passed |
| Embedded design validation | Passed: 8 systems, 38 components, 15 contracts |

The earlier preliminary count of 179 was incorrect. The corrected count above comes from executed test-result lines, not source declarations.

The ordinary cross-platform workflow for implementation commit `e0df71c` is [run 34660781675](https://github.com/lgardner-dev/System-Designer/actions/runs/34660781675). **All three platform jobs ultimately passed.**

| Platform | Observed result |
|---|---|
| Ubuntu 24.04 | Formatting, tests, Clippy command, release binaries, embedded design validation, installer packaging, and installer install/uninstall round trip passed. Artifact `SystemDesigner-ubuntu-24.04`, ID `10287118456`. |
| macOS | Formatting, tests, Clippy command, release binaries, embedded design validation, application/DMG packaging passed. Artifact `SystemDesigner-macos-latest`, ID `10287327604`. Unsigned and unnotarized; no native macOS UI test observed. |
| Windows | First attempt passed code verification but failed while installing NSIS. The targeted retry passed NSIS installation, installer packaging and artifact upload as well as the preceding checks. Artifact `SystemDesigner-windows-latest`, ID `10286824941`. Unsigned installer; no installed Windows UI test observed. |

Clippy success means command completion, not elimination of all warnings. Packaging success does not establish the entire installed interaction experience. Artifacts are bound to the implementation SHA above, not merely to a branch name.

## Actual Linux graphical smoke

The ordinary-CI Linux installer was installed into an isolated local prefix. Its actual executable ran under Xvfb/Openbox with software rendering and separate test home/data directories. Actual screenshots were inspected for Detail, Overview, connection-aware arrangement, Incoming focus, and an exact connection's contract dialog. The dialog was cancelled without applying a semantic change.

Two environment limitations prevent treating this as full native acceptance:

1. The X11 window initially remained unmapped in the virtual display. It was manually mapped before the app rendered and the inspected interactions were exercised. Unattended launch behavior is not established by this test.
2. After arrangement dirtied the project, recovery reported `Input/output error (os error 5)`. Independent `os.fsync` probes also failed with error 5 in both the isolated data directory and `/tmp`. This supports an environment limitation, but native durable save/recovery is not qualified here. Errors remain visible in the captures and were not suppressed.

This smoke did not establish every hierarchy-trace action, Windows/macOS interaction, physical-GPU behavior, accessibility, or usability on the user's actual dense Method project. Those claims are separate from headless interaction tests.

## Rejected or limited workbench evidence

The earlier workbench runner could not launch the app because it lacked `libxkbcommon-x11.so.0`. Its shell step did not propagate the child-process failure and captured only the window-manager desktop. That green step and image are rejected as application evidence. The later local graphical check above used the ordinary CI installer in a different runtime environment.

The workbench's final source push failed on a runner-local Git configuration permission error. Source was published separately as a clean integration commit; no transport scaffolding or workbench workflow is included in PR #6. The model's descriptive purpose text was refreshed before ordinary CI, which validated the exact published design.

## Combined regression coverage

Tests exercise all 24 View × Focus × Lights combinations and all three scope packet types, preserving semantic content/base hashes and avoiding dirty publication from visual state. Coverage includes partial summary counts; exact versus aggregate identity; summary deletion guards; exact contract dialog/cancellation; one-hop rather than causal propagation; effective boundary identity; full Back trace restoration; stale targets; document resets; light suppression; and shared geometry.

Arrangement tests include deterministic cyclic/acyclic placement, both orientations, measured card nonoverlap on the embedded design, full-scope preservation and one-step undo/redo. Existing model, exchange, storage and four-sided pointer tests remain intact.

## Release boundary

Inspect the installed candidate with the actual dense Method JSON before adopting the combined behavior. A screenshot cannot reconstruct that authoritative project. Preserve comparison branches until review. No main merge, signing, notarization or public release was performed. This records implemented behavior, passing automated checks and limited native observations—not complete product or usability qualification.
