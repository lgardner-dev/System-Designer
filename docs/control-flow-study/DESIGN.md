# Design and adversarial disposition

## Resulting candidate

**One component tree; two relation types; separate linked views.** Control-flow steps represent occurrences of actions or judgments. A referenced component remains the authoritative owner of interfaces. Source step definitions and transition conditions are retained. A new component named `S2.experiment` owns the contiguous region S2.hyp → S2.plan → S2.run.

The boundary earns consideration because it produces the observations from an explicitly specified discrimination effort. It owns the ordering obligations between serious alternatives, a preregistered plan, and actual acquisition. It does not own interpretation or accountable disposition. It is a proposed grouping, not proven superior simply because it fits eight boxes.

At S2 there are eight components, nine control transitions, and eleven exact typed exchanges. The child contains three components, five CFG vertices including two boundary markers, four control transitions and four typed exchanges. The boundary markers are not fake architecture components; they project the owning node's ports. Flattening the child reconstructs ten source steps and eleven source transitions. Splitting a cross-boundary exchange into two local segments adds interface edges but no new independent control alternatives.

The predecessor/successor labels are preserved from the source. A control edge means a declared sequential continuation under its recorded condition. It is neither a ready-queue dependency nor a grant, clock, thread, actual message receipt, or observed execution. These distinctions matter if this procedural vocabulary later meets Foundry's concurrent eligibility model.

## Local complexity convention

For each finite sequential fixture, require one declared entry, declared terminal outcomes, every node reachable from entry, and a structural route from every node to some outcome. Add one virtual sink and one analysis-only edge from each outcome to that sink. Calculate `E' - N' + 2`. The virtual sink is not persisted as a design component.

The original S2 has E=11, N=10, two outcomes: M=(11+2)-(10+1)+2=4.
The decomposed S2 parent has E=9, N=8, two outcomes: M=(9+2)-(8+1)+2=4.
The experiment child has E=4, N=5, one outcome: M=(4+1)-(5+1)+2=1.

The number is computed on modeled control transitions, not interface channels. Pure visual hiding cannot change it. A missing/reachable-outcome failure makes this metric unavailable instead of quietly dropping nodes. That does not make every nonterminating reactive system invalid; such a system is outside this metric's stated applicability.

M is not the number of possible runs or a test count proving correctness. Prose such as “defer / reject / exhausted” is a single grouped source alternative in this graph; expanding the distinctions in a future implementation can change complexity. Likewise, short-circuit expressions and concurrency require explicit representation/counting policies. No cyclomatic threshold was invented as a normative Method acceptance rule.

Metric reference: NIST SP 500-235, *Structured Testing: A Testing Methodology Using the Cyclomatic Complexity Metric*, https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication500-235.pdf . Used only for the counting convention, not to replace or expand source Method semantics.

## Adversarial findings

| Failure to attack | Smallest retained defense | Disposition |
|---|---|---|
| Compress S2 so hypotheses or preregistration vanish | Keep all three inner source steps and source IDs; show a descriptive subprocess subtitle | Repaired in candidate; compare flat view remains available |
| Pretend a lower box count reduces decisions | Show 4 → 4 for parent complexity and 1 for child; report source counts separately | Repaired |
| Merge C07 and C11 into a fake common payload | One control arrow maps to two unchanged, separately inspectable typed exchanges | Repaired |
| Turn inconclusive work into an unlimited retry | Preserve “inconclusive; renewed bound” and both C03/C04 associations | Repaired at source-preservation level, not proof a renewed grant exists |
| Treat a sufficiency diamond as a deterministic computation | Explicit semantic-assessment/authority roles; no predicate evaluator | Repaired |
| Add invented failure recovery paths to make the source appear complete | Retain source graph and separately disclose that work-node failure/timeout routing is not fully explicit | Open source-design limitation |
| Infer S2's callers or patch unresolved migrated ports | Keep caller context in source declarations; no made-up root/external wires | Repaired |
| Duplicate a child public interface | Owner port IDs are reused by the child boundary and verified | Repaired |
| Wire a parent directly into a hidden grandchild | Current example resolves all endpoints in the local system using production validation | Repaired for candidate |
| An interface-only save erases behavior | Separate explicit export formats; read-only study cannot be applied as a production replacement | Safe research boundary; production integration remains to design |
| A scenario shows one successful story but hides alternatives | Render the complete selected source graph including bypass, loop and stop | Repaired |
| A read-only prototype looks like shipped authoring | Persistent “Read-only worked candidate” label and explicit non-goals; production remains unchanged | Repaired |
| A call returns to the wrong occurrence | This single-call fixture validates its exact binding; multiple call sites are not claimed implemented | Deferred to production schema test cases |
| A renamed component drifts from step metadata | Source names/purposes checked exactly; no editing in the candidate | Safe study boundary, not a final identity design |

## Next production design decisions—not silently settled by this fixture

A future model should support behavior on root and on leaves without creating empty child systems. It needs one authoritative declaration of externally relevant behavior entries/outcomes, references to action occurrences and their results, explicit honest draft states, and a distinction between structural component identity and a repeated call site. The proof fixture's `system` association is not enough for every leaf or repeated-use case and must not be promoted unchanged merely because it compiles.

The candidate format stores source comparison data and authored positions for study reproducibility. A production behavior schema should not require a reference Atlas object or source IDs. It should separate semantic data from positions, version old projects without inventing behavior, retain all existing interface functionality, and make behavior-only/interface-only replacement preserve the other layer while validating cross-references. The current initialization prompt and project schema are not overwritten here.

Start with entries, actions, choices, transitions, outcomes, and calls. Add concurrency only with explicit fork/join meanings and test cases. We have not chosen a scripting language, a semantic approval system, a workflow runtime, or a generic extensibility framework.

## Human comparison tasks

Find the prior-art early exit; identify who disposes a conclusion; explain why a retry is bounded; inspect both contracts on the sufficient transition; enter the experimental sequence and establish plan-before-execution; follow the child boundary; locate truthful stop; compare the flat and decomposed explanations.

Acceptance of this UX candidate would be evidence to proceed with the generic schema/editor design, not qualification of the Method or an authorization to silently migrate the live project.
