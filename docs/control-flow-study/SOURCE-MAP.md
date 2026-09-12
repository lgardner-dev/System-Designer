# Source map

Original source objects and conditions are retained. The only proposed semantic grouping is `S2.experiment`. Neither Method adoption nor restoration of an earlier governing baseline is implied.

## Exact sources

- `atlas_file`: `foundry-method-model.json`
- `atlas_sha256`: `19f35b033ab09e1c4f62b95418d702d804118ab6792f62b7dce116ada6f383ed`
- `atlas_model_version`: `0.1.0-proposal.1`
- `source_S2_sha256`: `2081deecdf5b7e17eaebdb9d2e93da87d4da65474012a29c9c4d0636c00ae337`
- `catalog_file`: `foundry-method-editor-project.json`
- `catalog_file_sha256`: `f072f418d3b596ad3e41f2ec6190b12cb91da42631a872328894b66b984469be`
- `retained_catalog_sha256`: `4ac9facae050f2485173e4b4720c783310a84faf0f98620543de516c28ca0443`

## Source steps

| Original step | Location in candidate |
|---|---|
| `S2.frame` — Frame question | S2 |
| `S2.prior` — Inspect prior art | S2 |
| `S2.enough` — Evidence enough? | S2 |
| `S2.hyp` — Compete hypotheses | S2 / Produce discriminating evidence |
| `S2.plan` — Seal experiment | S2 / Produce discriminating evidence |
| `S2.run` — Execute & observe | S2 / Produce discriminating evidence |
| `S2.interpret` — Bound interpretation | S2 |
| `S2.decide` — Dispose conclusion | S2 |
| `S2.exit` — Return bounded answer | S2 |
| `S2.stop` — Stop honestly | S2 |

## Source transitions

| Source | Condition (exact) | Candidate locations | Typed contracts |
|---|---|---|---|
| `S2.e1` | Unconditional continuation | behavior.S2:S2.e1 | C06 |
| `S2.e2` | Unconditional continuation | behavior.S2:S2.e2 | C07 |
| `S2.e3` | sufficient | behavior.S2:S2.e3 | C07, C11 |
| `S2.e4` | insufficient | behavior.S2:S2.e4, behavior.experiment:S2.e4.inside | C07 |
| `S2.e5` | Unconditional continuation | behavior.experiment:S2.e5 | C08 |
| `S2.e6` | Unconditional continuation | behavior.experiment:S2.e6 | C09 |
| `S2.e7` | Unconditional continuation | behavior.S2:S2.e7, behavior.experiment:S2.e7.inside | C10 |
| `S2.e8` | Unconditional continuation | behavior.S2:S2.e8 | C02 |
| `S2.e9` | resolved / bounded | behavior.S2:S2.e9 | C03 |
| `S2.e10` | inconclusive; renewed bound | behavior.S2:S2.e10 | C03, C04 |
| `S2.e11` | defer / reject / exhausted | behavior.S2:S2.e11 | C03 |

The `.inside` segments at S2.e4 and S2.e7 project the call boundary. They do not add a new condition or activity. The parent owns the condition on entry. Flattening the call removes these boundary markers and recovers the exact original endpoints and conditions.

Retained catalog entries are copied from the migrated version-1 editor project, not redefined from Atlas prose. Their source caveats remain intact.
