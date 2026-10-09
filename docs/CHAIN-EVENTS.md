# Chain event assumptions for #32

[SRC] The Lean kinds and payload are `bsv-stack-lean@24f22f808b1fb44ce9bf128ec89ac5c8839d9d60 lean/Tracker.lean:128`, the charter's section 3, R1 section 2a, and application contract rule 1. [D] The wire projection in this crate is the flat version 1 shape below. The captain reconciles the final emitter's fields; no #32 emitter has been exercised by this lane.

| kind | required fields beside `v` and `kind` | example |
|---|---|---|
| `tip` | `height: u32`, `hash: string` | `{"v":1,"kind":"tip","height":100,"hash":"block hash"}` |
| `fork` | `height: u32`, `competingTips: string[]`, `depth: u32` | `{"v":1,"kind":"fork","height":100,"competingTips":["A","B"],"depth":1}` |
| `reorg` | `forkHeight: u32`, `depth: u32`, `deactivated: string[]`, `newTip: string` | `{"v":1,"kind":"reorg","forkHeight":100,"depth":1,"deactivated":["A"],"newTip":"B2"}` |
| `invalidated` | `blockHash: string` | `{"v":1,"kind":"invalidated","blockHash":"A"}` |
| `frozen` | `outpoint: string` | `{"v":1,"kind":"frozen","outpoint":"txid:0"}` |
| `tipAge` | `seconds: u64` | `{"v":1,"kind":"tipAge","seconds":600}` |

`v` is an unsigned integer, currently exactly 1. `kind` uses the casing in the table. Hashes are the host's canonical display-order strings, compared exactly as in Lean. The host supplies header validity, active-chain selection and the frozen-output parameter set; this crate does not establish them. `forkHeight` means the lowest replaced height, inclusive, matching the charter's at-or-above rule.

`deactivated`, `competingTips` and `newTip` are opaque block hashes here. The #32 brief also names full headers, a tip time on `tip`, and a tip on `tipAge`; this projection assumes none of those extra fields. They are not silently ignored. It also does not assume a cursor, event id, network id, `oldTip`, `commonAncestor`, delivery timestamp, or a compatibility field. These are explicit reconciliation questions, not inferred schema guarantees.

`decode_envelope` checks the version before the known-version body, irrespective of key order. Unsupported versions return `UnknownVersion`, malformed known-version bodies return `UnknownShape`. The typed `ChainEnvelope` deserializer gates the same version and shape; its fields are private. Hosts report either fault on their status, metric or listener error channel in the same tick, and deliver no event on a fault. A host with no feed polls the heights of the proofs it holds and supplies `Evidence::Recheck` for those txids, never a chain scan. Delivery and this fallback are host work [D].

[X] `tests/scenarios.rs` exercises the wire-shape witness and contract rule 1; `tests/transitions.rs` round-trips every assumed kind and rejects unknown versions, kinds and extra fields. These are pure decode and tracker runs, not a transport replay.
