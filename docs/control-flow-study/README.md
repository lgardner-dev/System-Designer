# S2 control-flow-first worked candidate

**Review artifact, not an adopted Method revision and not the finished control-flow editor.**

This implements the next experiment proposed in the design discussion: compare the original Atlas's S2 procedure with one recursive decomposition, show the linked typed interfaces separately, and test that simplification does not erase behavior. It is an independent native Rust example. Production files, `src/`, version-1 parsing, scoped replacement, storage, and the normal application are unchanged.

## Run

```sh
cargo run --locked --release --example s2-flow-candidate
```

The executable embeds the original S2 source, the worked candidate, exact retained contract definitions, interface project, and initialization instructions. No HTML, JavaScript, CSS, account, backend, or runtime data files are required.

```sh
cargo test --locked --example s2-flow-candidate
cargo run --locked --release --example s2-flow-candidate -- --check
```

Normal production tests remain available with `cargo test --locked --all-targets`.

## Review route

1. Open **Control Flow**, and follow Frame question → Inspect prior art → Evidence enough? The sufficient branch bypasses experiments. The insufficient branch enters the proposed experiment subprocess.
2. Select the **sufficient** arrow. Its inspector shows C07 and C11. Switch to **Interfaces**: the one control transition is associated with two distinct typed exchanges. Select either channel to inspect its exact endpoints.
3. Enter **Produce discriminating evidence** by double-clicking it or using the tree/inspector. Compete hypotheses, Seal experiment, and Execute & observe remain separate steps; the child cannot make a conclusion or grant its own authority.
4. Inspect the child boundary. Its physical input/output port identities are exactly the owner's ports in the parent. They are not independently maintained copies.
5. Use **Back**, then inspect Dispose conclusion. Resolved/bounded, inconclusive with a renewed bound, and truthful stop remain distinct alternatives.
6. Select **Original Atlas · 10 steps** for an unchanged behavioral comparison. Returning to Interfaces locates the selected work in the candidate's one component tree.

Fit and pan/zoom are presentation only. Lights are optional selected-direction previews, never execution. The list and inspector provide ordinary keyboard-accessible buttons; the canvas additionally supports clicking and double-clicking. This is not a completed accessibility qualification.

## Files and protocol boundary

- `examples/s2-flow-candidate/control-flow.study.json`: experimental, strictly decoded read-only study. It is **not** production version 2 and is not a version-1 scope-replacement packet.
- `examples/s2-flow-candidate/interfaces.project.json`: a complete valid **version-1 System Designer project**. Export it from the study and open it in the existing application for normal interface editing. This file intentionally contains no new control-flow fields.
- `examples/s2-flow-candidate/source-S2.json`: original source object preserved for comparison and fidelity checks.
- `examples/s2-flow-candidate/initialization.txt`: embedded instructions for discussing this candidate with a fresh chat.

**Copy study JSON** copies the entire study, not the visible subset. **Export interfaces** writes only the explicitly named version-1 project. A normal interface export cannot carry the experimental flow; the app makes this distinction visible. The prototype does not accept returned JSON or edit flow topology. Those require the production model/migration and scoped-exchange design described in `DESIGN.md`.

## Source and preservation

The control reference is **the original `foundry-method-model.json` requested in the discussion**, not an assertion that an earlier Method revision supersedes the current design. The retained typed contracts come from the later migrated `foundry-method-editor-project.json`, since the original Atlas did not supply complete executable schema definitions. Every selected contract entry is preserved unchanged, including its existing draft/migration caveats. The differing source roles are explicit in the study's provenance block and `SOURCE-MAP.md`.

No external S1/S3/S4/S6 attachment is guessed. The original S2 entry/eligibility/authority declarations remain inspectable. The old migration's unresolved external attachments are not “fixed” by inventing caller wiring.

## What this demonstrates

- One component tree, with separate control and interface relations.
- One control transition associated with multiple typed exchanges.
- A call to one child behavior with explicit entry/return boundary correspondence.
- Exact preservation of source steps, transitions, labels, conditions and contract associations after flattening the proposed call.
- A visible difference between local step count and local cyclomatic complexity.
- No production model mutation by selection, view changes, tracing, export or animation.

## What it does not demonstrate

A general editable behavior model, arbitrary component/leaf behavior authoring, multiple call sites, extraction/refactoring, fork/join semantics, operation/result binding, executable predicates, scheduling, simulation, code generation, formal deadlock/termination proofs, or a production migration. Its hand-authored fixture layout does not qualify a general CFG layout algorithm. No application release, installer replacement, main-branch merge, or Method adoption is performed.

The selected current repository baseline already contains the corrected owner-selected atom mark. The study reuses the native window icon from that baseline. It does not create a new branding migration.
