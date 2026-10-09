# Mapping to the executable charter

[SRC] All Lean citations on this page refer to `bsv-stack-lean@f95520fcd85e066ec341faffa9dd8f1f8f35a37e lean/Tracker.lean`. The charter is `docs/charters/tracker.md` at the same pin. The Rust projection is an implementation under test, not a new node rule.

| definition | Rust surface | reading |
|---|---|---|
| `State.init` | `State::new` | private state prevents direct writes |
| `hintWord`, `agrees`, `onHint`, lines 252, 259, 270 | `State::on_hint` | three upward hint moves, newest received hint, agreeing hint keeps pending trigger |
| `onHost`, line 295 | `State::on_host` | build and abandon only in the host tier; spend and age produce triggers |
| `onProof`, line 306 | `State::on_proof`, `Headers::check` | reduce a real SDK path, bind the txid, match the active header, keep it |
| `onCompetitor`, line 320 | `State::on_competitor` | the host identifies a competing input spend; a mined word stands and re-asks |
| `recheck`, line 334 | `State::recheck` | matching root clears suspect and the completed trigger, mismatch makes stale, unavailable keeps word |
| `onReorg`, `onFork`, lines 346, 353 | `State::on_chain` | at-or-above comparison only; reorg clears suspect and records a trigger |
| `onInvalidated`, line 361 | `State::on_chain` | match the checked header hash; keep the old evidence |
| `onChain`, line 370 | `State::on_chain`, `Tracker::on_chain` | tip touches suspect only; frozen and tipAge move no word |
| `onVerdict`, line 385 | `State::on_verdict` | mined proof outranks a node's mempool verdict; new competitors precede old ones |
| `step`, `run`, lines 398, 406 | `State::step`, sequential calls | headers are a snapshot per input, with no internal effect loop |
| `reorgReask`, line 705 | `Tracker` mined-height range index | exactly affected txids, bounded by what the host tracks |

[D] Differences of representation: real owned BRC-74 paths replace Lean's reduced `(height, root)`; `Mined` wraps an immutable checked capability and derives its height; the capability and state's record share an allocation. Header and proof failures add a returned `CheckError` after the Lean-compatible re-ask update. Header freshness and tip coherence are host obligations; checking a proof above the host's tip fails closed. The retained depth is a snapshot annotation, not a Lean word field. Rust heights and times are finite. An overflowing age deadline remains not due, matching the natural-number comparison. Raw `Verdict` metadata is retained beside its reduced hint and never read by evidence transitions.

## The four test explainers

[X] `cached_host_check_cannot_bypass_fail_closed_snapshot_lookup` exercises a host returning an old capability from its `check` override while current header lookups fault. Own and competitor proofs fail closed. Transitions wrap the host's snapshot methods and invoke the default `Headers::check`, preserving the single capability constructor and current-snapshot check.

`mined_implies_checked_proof` checks every prefix of randomized evidence, hint and host traces. A mined word's stored path is for its own txid and height, computes the retained header's merkle root, and has positive depth. The only writing transition constructs both the word and retained record from the same checked capability. This corresponds to the invariant at Lean lines 666-670. It does not establish that the host's header snapshot is the node's active chain.

`reorg_reasks_at_or_above` generates registries containing mined and unmined words at random heights. It compares the indexed result set to the exact affected set, asserts stale plus the reorg trigger for every affected word, and full state equality for every other transaction. This exercises the theorem at lines 676-678, `reorg_spares_below`, `reorg_spares_unmined`, `reorgReask_only` and the bound.

`hint_changes_no_chain_word` inserts random hints into random evidence and host traces. It compares chain words, retained evidence and suspect marks before and after each hint, and checks that any changed hint-tier word is one of the three allowed moves. This is the charter's theorem (iii), including the no-new-claim half.

`same_evidence_same_word` feeds two instances identical evidence and header snapshots but independently generated hints and host acts around each evidence input. It compares the chain words, evidence and suspect marks after every prefix. It intentionally does not compare `Announced` with `Seen`, matching `Word.chain` and theorem (iv).

[X] Each property has 256 cases with an explicit deterministic ChaCha seed, failure persistence disabled, and random trace lengths up to 79 inputs. The full local results are in `VERIFICATION.md`. These Rust checks are not a formal Lean refinement proof or an execution of the Lean evaluator; that link remains unverified [D].

## Landed recheck rule

[SRC] The earlier stack pin retained a successful recheck's trigger. At the final pin, `lean/Tracker.lean:340` clears `reask` along with `suspect`, matching the charter and scenario table. [X] The crate follows this rule; `successful_recheck_clears_completed_spend_and_fork_triggers` failed on the old Rust transition, then passed after the correction. Unavailable headers continue to keep the mined word and schedule `Recheck`.

[D] The new #31 spendability predicate and host confirmation-depth policy remain host adapter work under this lane's integration exclusion. The scheduling projection `Params` holds the age threshold; a host supplies its spend depth separately, as the wallet sketch states.
