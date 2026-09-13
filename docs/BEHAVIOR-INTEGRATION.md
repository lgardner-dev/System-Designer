# Guided behavior integration — recovery checkpoint

Baseline: `26c6e999cfc303dfc4828d6682ef79a0941f2fa3`.

This branch integrates the approved control-flow-first workflow into the ordinary System Designer executable under `src/`. It is not another read-only example. At this initial checkpoint no implementation or test success is claimed.

## Accepted intent

One project and one component tree; linked editable Control Flow and Interfaces views at root and leaf scopes. Draw behavior first, identify structurally extractable regions, explicitly accept a single-responsibility boundary, mechanically create a child and replace the region with a shared-identity call, then refine its data requirements and exact contracts. Structural extractability does not prove single responsibility. Preserve occurrence identity, labeled outcomes, unresolved requirements and the original interface graph. Never infer data flow from control sequencing.

## Delivery gates

- Versioned behavior data and legacy project handling; no silent loss of either layer.
- Guided generic editing, conventional flowchart shapes, existing curved connectors, direction lights, cross-view navigation and scope-local layout.
- Deterministic bounded extraction with explicit responsibility, boundary preview, stale-base rejection and one undoable publication.
- Save/open, recovery and scoped AI exchange keep both layers; embedded initialization instructions match the implementation.
- Explain what RPST/SESE regions, LLVM extraction and Parnas influenced, and distinguish implemented algorithms from inspiration.
- Tests and screenshots of the normal application, not a separate viewer. No merge/release without review.

## Recovery

The failed run did not leave a confirmed complete working directory or implementation commit. Reconstruct from the current repository; do not label earlier recorded drafts compiled or tested. Commit incremental checkpoints on this branch and record exact verification limits.
