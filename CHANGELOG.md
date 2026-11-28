# Changelog

## [Unreleased]

### Hosts: what a host of 0.9.0-beta.11 changes to take the tenth word (for #56)

**A host of 0.9.0-beta.11 changes each exhaustive match on `Word`, and may start feeding three acts.** (1) `Word` has a tenth word, `Word::Handed`, and is still not `#[non_exhaustive]`, on purpose: every exhaustive match on `Word` outside the crate stops compiling (E0004) until it names the word, because each place that names words must decide what `handed` means for it. (2) A host that feeds no `HostAction::HandOver` never sees `Word::Handed`, and every word it sees is the word it saw at 0.9.0-beta.11: no other act, hint or evidence writes it (the Lean's `handedNamed_run` and the arms of `step` in `lean/Tracker.lean` at bsv-stack-lean `08ce467d`). Once a host feeds the act, a rule that reads a word by its text or through a wildcard arm reads `handed` as whatever that arm says, in silence; name each before feeding the act. (3) A re-ask of the new kind `Reask::Reannounce` asks the host to post the bytes again to each instance of `State::handed_to()`; it is never a proof request, so a host that fetches a proof for every due re-ask whatever its kind fetches the wrong thing for it. `Reask::LABELS` gains `"reannounce"` at its end. (4) `State::to_compact` writes version 2 of the compact form. Version 1 bytes a host stored still restore, as the same state with no instance; an older crate refuses version 2 bytes with `CompactError::UnknownVersion(2)` and restores nothing.

```rust
// 0.9.0-beta.11: nine arms, which no longer compile (E0004).
fn label(word: &Word) -> &'static str {
    match word {
        Word::Unknown => "unknown",
        Word::Built => "built",
        Word::Announced => "announced",
        Word::Seen => "seen",
        Word::Mined(_) => "mined",
        Word::Stale => "stale",
        Word::Rejected { .. } => "rejected",
        Word::Conflicted { .. } => "conflicted",
        Word::Abandoned => "abandoned",
    }
}
// This line: the tenth arm, and the host's decision for it.
fn label(word: &Word) -> &'static str {
    match word {
        Word::Unknown => "unknown",
        Word::Built => "built",
        Word::Handed => "handed",
        Word::Announced => "announced",
        Word::Seen => "seen",
        Word::Mined(_) => "mined",
        Word::Stale => "stale",
        Word::Rejected { .. } => "rejected",
        Word::Conflicted { .. } => "conflicted",
        Word::Abandoned => "abandoned",
    }
}
```

```rust
// 0.9.0-beta.11: a rule that compares a word as a string. Once the host
// feeds HandOver, a "handed" row falls outside this list in silence: it is
// never loaded and never re-announced.
const PENDING: [&str; 5] = ["unknown", "built", "announced", "seen", "stale"];
// This line: the rule names the word, and its reader decides what it means.
const PENDING: [&str; 6] = ["unknown", "built", "handed", "announced", "seen", "stale"];
```

```rust
// 0.9.0-beta.11: a stored compact row, version 1, restored.
let state = State::from_compact(&row)?; // row[0] == 1
// This line: the same row restores to the same state, with no instance;
// what the host writes back is version 2.
let state = State::from_compact(&row)?; // row[0] == 1, still read
assert!(state.handed_to().is_empty());
let row = state.to_compact(); // row[0] == 2
```

```rust
// This line: a host that hands the bytes over says so before the post
// leaves, and says when an instance answered and keeps nothing.
state.step(&params, &headers, Input::Host(HostAction::HandOver { instance: "arcade".into(), at: now }))?;
state.step(&params, &headers, Input::Host(HostAction::Discharge { instance: "arcade".into() }))?;
```

### Added: the tenth word `handed`, its acts and its re-ask (for #56)

- `Word::Handed`, mirrored from `Tracker.Word.handed` (`lean/Tracker.lean` at bsv-stack-lean `08ce467d`): the bytes left the host's hands for one or more named instances and none has answered. It claims nothing of the chain (`Word::chain` answers `None`). The doc of `Word` carries a `compile_fail,E0004` doctest of a nine-arm match.
- `State::handed_to() -> &[HandedTo]`, the Lean's `State.handedTo`: the instances handed the bytes and not discharged, each with the time of its hand-over, in the order of each instance's earliest hand-over. `HandedTo { instance, at }` is `#[non_exhaustive]`, built by `HandedTo::new`, re-exported at the crate root, and stored by its serde form (`{"instance":"arcade","at":1700000000}`).
- Three host acts (`HostAction` is `#[non_exhaustive]`, so they add without a break): `HostAction::HandOver { instance, at }` (`onHandOver`: `unknown`, `built` and `handed` read `handed`, the instance added or its time renewed in its place, a pending `reannounce` cleared; an `abandoned` state left whole; any other word keeps its word and only the pending `reannounce` is cleared), `HostAction::Discharge { instance }` (`onDischarge`: from `handed` the instance leaves the list, and the word returns to `built` when it is empty; every other word untouched) and `HostAction::EndHold` (`onEndHold`: `unknown`, `built` and `handed` read `abandoned`, the list emptied, a pending `reannounce` cleared; every other word untouched; a received payment's act, which the crate cannot tell from one of the host's own).
- `Reask::Reannounce`, label `reannounce`, compact re-ask tag `7` (`Tracker.Reask.reannounce`): a host tick on a `handed` word whose oldest hand-over plus the age threshold is at most the tick's time (`handedDue`, checked addition); never on `built`, and the age re-ask keeps its arm before it.
- The list in every other arm, as the Lean writes it: a hint raises `handed` to `announced` (an acceptance or a mempool presence) or `seen` (a sighting), and empties the list whenever it moves the word (`onHint`); no hint lowers `handed`. `abandon` stays refused on `handed`. A proof that writes `mined` empties the list (`onProof`); a node's `rejected` empties it and its `conflicted` keeps it (`verdictHanded`); a competitor's proof keeps it.
- `bsv_tracker::doubts`: the guard's rule arm names `Word::Handed`, as the Lean's `spendable` does: `handed` is spendable under a rule that allows it and under no other, and reads no header. Nothing else in the module moves.
- Tests: nine examples of the arms in `tests/transitions.rs`, one per row of the stack's `docs/p0/tracker-handed-proof-2026-11-27.md` section 3, and the stored form of `HandedTo`. `tests/labels.rs` names the new label. `src/tracker.rs` is untouched.

### Changed: the compact form's version 2 (for #56)

- `COMPACT_VERSION` is 2. Version 2 is version 1's layout with word tag `9` (handed), re-ask tag `7` (reannounce) and three fields after the re-ask: the instances (a varint count, each a string and a varint time), the record of a conflict's competitors and the word held when it began. The last two are held empty in this release (a count of `0` and a `0`), and the reader refuses any other value there. Version 1 still reads, as the same state with the three fields empty.
- Refused as `CompactError::Malformed`: word tag `9` and re-ask tag `7` under version 1, re-ask tag `8`, a `handed` word with no instance, an instance named twice, instances beside any word but `handed` and `conflicted` (a conflict that began from `handed` keeps its list), and a count of competitors or a prior word that is not `0`. `CompactError` gains no variant.
- `tests/compact_golden.rs`: the ten goldens re-recorded in a commit of their own, each moved only by its opening byte, `01` to `02`, and three bytes `000000` at its end; `version_1_bytes_still_restore` restores each of the ten version 1 strings, kept verbatim, to the state built today, and `version_2_is_version_1_with_three_empty_fields` pins the relation. `tests/compact.rs`: `version_2_refusals` pins each refusal and the bytes of a `handed` state; two tests that read version 1's layout move by their inputs and keep their answers (version 3 is the unread version; word tag `10` is the unknown tag; the hints count is six bytes from the end).

### Added: the transaction's measured length on the tracked state (GHSA-jjhc-7hvh-c3hm, BRC-237)

A transaction of exactly 64 bytes is never taken as proven by a merkle path (BRC-237's condition for an anchor; bsv-rs enforces it in its own verifier, its public advisory GHSA-jjhc-7hvh-c3hm). This crate's `Proof` holds a txid and the crate compares the root itself, so the crate measures the transaction. This entry is the measure on the tracked state and in the stored bytes; `Proof::new` is untouched by it. Every item is ahead of the Lean: bsv-stack-lean `lean/Tracker.lean` at `9a727a64` has no such field yet (its name there will be `State.txLen`); the rule's words are its `docs/DECISIONS-2026-10-19.md` 32.13.

- `AMBIGUOUS_TX_LEN: u64 = 64`, re-exported at the crate root: the one length the rule names. A host and a test read the constant, never the literal.
- `State::tx_len() -> Option<u64>`: the transaction's measured length in bytes, `None` where the host never handed the bytes. It is written only from bytes the crate hashed to the state's own txid, and once written it does not change. A length is a `u64`, never a `usize`, so the stored form does not depend on the host's word size.
- `State::measured(tx_bytes: &[u8]) -> State`: birth from the transaction's serialized bytes. The txid is their double SHA-256, reversed, as 64 lowercase hex digits; the word is `Unknown`. The bytes are hashed and counted, never decoded, and the call never fails: the host compares `State::txid()` with the txid it expects. `State::new(txid)` is unchanged: a state born from a txid is unmeasured.
- `State::measure(&mut self, tx_bytes: &[u8]) -> Result<(), CheckError>`: the bytes handed after birth. Bytes of another transaction are `CheckError::TxidMismatch`, the state left whole; the same length again is `Ok` and moves nothing; another length is `CheckError::LengthMismatch { measured, offered }`, the state left whole; an unmeasured state gains the length, and off 64 bytes nothing else moves, whatever the word.
- `Tracker::track_bytes(&mut self, tx_bytes: &[u8]) -> Result<&State, CheckError>`: a txid not tracked is born measured and no index moves; a tracked txid gains its measure as by `State::measure`, through the indexes.
- **A state measured at 64 takes no proof.** Its proof door answers every `Evidence::Proof` for its txid with `CheckError::AmbiguousLength` and leaves the state whole: no word, no record and no re-ask is written (a re-ask is an order to fetch, and no fetch helps). Such a state is tracked like any other and never reads `mined`; a host reads `state.tx_len() == Some(AMBIGUOUS_TX_LEN)` and asks no proof for it.
- **The fault road of `State::measure`.** When the measure taken is 64 bytes and the state held a proof (a state that took its proof unmeasured, or a row restored from version 1 bytes), the call unsays the proof AND returns the fault: the length is written, the stored record is dropped, a `Mined` or `Stale` word reads `Unknown` with the suspect mark and the re-ask cleared, any other word keeps its word and its re-ask, and the answer is `Err(CheckError::AmbiguousLength)`. `Tracker::track_bytes` returns the same fault with the indexes moved, and `Tracker::get` reads the state. This is a fault, never a transition a host may lean on: the Lean's machine has no such step.
- `CheckError::AmbiguousLength` and `CheckError::LengthMismatch { measured: u64, offered: u64 }`.
- Tests: `tests/measure.rs`, five tables. `births` (a birth from a txid and from bytes of 0, 63, 64, 65 and 191 bytes; the hash and its byte order against the txid of the chain's block 0 coinbase; `Tracker::track_bytes` on a new txid, on a tracked one, and repeated); `measure_after_birth` (bytes of another transaction, 64 bytes and 191 bytes on each of the ten words, and a record beside a word no proof writes); `a_state_measured_at_64_refuses_every_proof` (on five words, three proofs each, under headers that carry the root and under headers that cannot answer, and through `Tracker::apply` with no index moved); the fault road through the tracker; and a `mined` row of version 1 restored. `tx_length_field` in `tests/compact.rs`, and one unit test in `src/state.rs` of the second measure's refusal.

### Changed: `CheckError` is `#[non_exhaustive]`, and version 2 of the compact form gains the length (GHSA-jjhc-7hvh-c3hm, BRC-237)

- `CheckError` is `#[non_exhaustive]`: an exhaustive match on it outside the crate stops compiling (E0004) until it carries a wildcard arm. An error is never a word: a fault a host does not name still means "not `mined`". Its doc carries a `compile_fail,E0004` doctest of the five-arm match of 0.9.0-beta.11.
- Version 2 of the compact form gains one field, its last: `0` (the host never handed the bytes), or `1` and a varint, the measured length (`State::tx_len`). `COMPACT_VERSION` stays 2: no published release wrote version 2, so its layout takes the field. A length of zero is `01 00`, a measure and never absence. A version 1 row still restores, unmeasured; a `mined` row of version 1 restores `mined` as before, and nothing more is asked of it.
- Refused: a length of 64 beside a stored proof, as `CompactError::Evidence(CheckError::AmbiguousLength)`, which the encoder never writes; a flag that is not `0` or `1` (`Malformed("flag")`); a varint not in its shortest form; and bytes that end before the field (`Truncated`). A length of 64 with no stored proof is read: it is a state measured at 64. `CompactError` gains no variant.
- `State`'s equality compares the measured length: a measured state and an unmeasured one are not equal.
- `tests/compact_golden.rs`: the ten version 2 goldens re-recorded in a commit of their own, each gaining the byte `00` at its end and nothing else; the helper's string and the renamed test `version_2_is_version_1_with_four_empty_fields`; the ten version 1 strings byte-identical. `tests/compact.rs`: the four constants that count from the tail.

### Changed: a `Proof` is built on a length, and every door reads it (GHSA-jjhc-7hvh-c3hm, BRC-237)

A transaction of exactly 64 bytes is never taken as proven by a merkle path (BRC-237's condition for an anchor; bsv-rs enforces it in its own verifier, its public advisory GHSA-jjhc-7hvh-c3hm). This entry is the two constructors and the doors: the crate is told the length where a proof is built, and each door that compares a root reads it. Every item is ahead of the Lean (bsv-stack-lean `lean/Tracker.lean` at `9a727a64`; the rule's words are its `docs/DECISIONS-2026-10-19.md` 32.13).

- **A break: `Proof::new(tx_bytes: &[u8], path: MerklePath) -> Result<Proof, CheckError>`** takes the transaction's serialized bytes in place of its txid. It hashes them to bind the txid (the double SHA-256, reversed, as 64 lowercase hex digits; the bytes are counted and never decoded) and yields no proof for exactly 64 bytes (`CheckError::AmbiguousLength`, before anything is hashed, whatever the path). Every old call stops compiling (E0308: a `String`, a `&String` and a `&str` are none of them a `&[u8]`); none changes meaning in silence.
- `Proof::with_stated_len(txid, stated_len: u64, path) -> Result<Proof, CheckError>`: the road of a host with no bytes at the place where it builds the proof. The length is the host's word; the crate cannot check it, and a host with no length has no proof. A stated length of 64 is `CheckError::AmbiguousLength`, read before the txid and the path; no other number is refused.
- **The proof door reads a length.** A state measured at another length than the proof's (built from bytes or stated) refuses it, `CheckError::LengthMismatch { measured, offered }`, the word and the record kept, `Reask::ProofFailed` written. A measured state takes a proof of its own length by either road, and a proof taken out of a record. An unmeasured state is measured by a proof built from bytes (before the header check, whatever it answers), takes a stated one on the host's word (`tx_len` stays `None`), and refuses a record's proof, `CheckError::Unmeasured`.
- **The competitor door** takes a proof built from the competitor's own bytes and refuses any other, `CheckError::Unmeasured`, the word kept, `Reask::ProofFailed` written. The competitor is judged by its own length, never by the tracked transaction's measure: a state measured at 64 still takes a competitor's proof, and a `conflicted` word a competitor's proof wrote holds no proof of the tracked transaction, so a measure of 64 handed after it is no fault.
- **A record's proof carries no length.** The record a door builds keeps the proof with none, and a restore reads it back so: the state's measure, kept beside it, is the length. `Headers::check` (called by a host, with no state) and `doubts::may_enter` take a proof built by either constructor and refuse a record's proof, `CheckError::Unmeasured`. A transition never calls a host's override of `Headers::check`.
- **A break: `Tracker::restore(&mut self, state: State) -> Result<Option<State>, CheckError>`.** A measure the tracker holds is carried across the swap to a restored row that carries no length. When the held measure is 64 and the row holds a proof, the proof is unsaid AND the fault is returned, as `State::measure`'s fault road: the row is held with its record dropped and a `Mined` or `Stale` word read as `Unknown`, out of the suspect set, and the call answers `Err(CheckError::AmbiguousLength)`. A row that carries another measure than the held one is `Err(CheckError::LengthMismatch { measured, offered })`, nothing moved. `Ok` holds the replaced state as before.
- `CheckError::Unmeasured`. `CheckError::AmbiguousLength` is also answered by both constructors and by `Tracker::restore`'s fault road; `CheckError::LengthMismatch` by the proof door and `Tracker::restore`.
- `README.md`'s example builds its coin with `State::measured(&raw_tx)` and its proof with `Proof::new(&raw_tx, path)`. `ProofFetcher::fetch`'s doc names both constructors.
- Tests: `tests/proof_length.rs`, eight tables and the witness. `constructors_by_length` (63, 64 and 65 bytes by each road, a malformed path beside each); `bytes_that_do_not_hash_to_the_txid`; `the_proof_door_pairings` (seven pairings of the state's measure and the proof's length, the measure written whatever the header check answers, a mined word kept by a refusal, and a state measured at 64 against three proofs); `a_measured_state_against_a_stated_length_proof`; `the_competitor_door`; `the_competitor_is_judged_by_its_own_length`; `the_doors_with_no_state`; `restore_carries_the_held_measure` (six rows); and `brc237_t64_is_never_proven`, on BRC-237's own transaction of 64 bytes, `T64`, with its path and the header at its height, held in `tests/vectors/brc237/t64.json`. One unit test in `src/state.rs` for the proof from bytes of another length, which no public constructor reaches. The vectors job runs the witness by name. Every existing test keeps its name and its answer: the helper `proof(..)` of `tests/common/mod.rs` states a length of 191, and eight competitor proofs are built from bytes. No golden string moves.

## [0.9.0-beta.11] - 2026-11-28

### Hosts: what a host of 0.9.0-beta.10 changes to take this line (for #51)

**A host of the published 0.9.0-beta.10 changes one thing, and may start calling one module.** (1) `Params` is built by `Params::new(age_threshold, spend_depth)` and is `#[non_exhaustive]`. In 0.9.0-beta.10 it was a struct of one public field, `age_threshold`, with no constructor, no `Default` and no `#[non_exhaustive]`, built by literal, so every new field broke every host: the literal is refused once more (E0639), and a later field, which a later minor adds here with a default, breaks none. (2) Its three new fields, each public to read and to assign, and their values when the host says nothing: `spend_depth: u64`, which has no default and is the constructor's second argument (the Lean's `Params.spendDepth` has none, and one host has one depth for an act; a host that reads no guard of `doubts` writes 1, the floor; the crate applies no floor, and a depth of 0 passes a `mined` word whose root the headers carry); `bound: Option<u32>`, the feed's named number, absent as `None`, which is no check and the host of today, row for row (it arrived with the keyed splits); and `versions: Vec<u64>`, the row versions the host's decoder reads, absent as `vec![1, 2]`, `Params::VERSIONS` (it arrived with the spend depth, by the table of the stack's cut 1g); `with_spend_depth`, `with_bound` and `with_versions` set one. (3) The new module `bsv_tracker::doubts`, nothing of it re-exported at the crate root: a host may start folding each feed row with `doubts::step` (`step_checked` under a number) before the tracker's own step, which takes the event by value; storing the doubts as their serde text, and reporting a text it cannot read as a fault, never a cold start; and reading `eff_depth` for the depth it may trust at a height, `spendable_eff` for the spend guard on it and `may_enter` for the builder's test. A host whose feed names a number sets `bound` from the service's contract, never from a snapshot or a row (a check against a number the checked party sends checks nothing), and treats `Checked::Out` as the feed out of its contract: it keeps the doubts it was given and reads no row; its depths are `eff_c`'s and its guard `spendable_c`'s, which pass no `mined` word under a spend depth of 1 or more (a word with no height passes as today, the row `outZero`); at each page read it makes one ask for a snapshot and nothing more, one start a page read at most and never a start that causes a start; it takes a snapshot through `accepts` or `start_from` alone, at its cursor or past it, and does not ask at its cursor alone, where an honest snapshot holds the count that tripped the check (the row `stuck_at_cursor`); a refused snapshot (`Refused::Overfull`, one that itself holds more open keys than `bound`, among them) builds no doubts, and the host holds every act, as it is still out, and asks again at its next page read, no faster than its page poll. The snapshot's wire is the header service's and the crate reads none: a host reads it into a `Served`. The page loop is the host's. `docs/DOUBTS.md` is the page of every item, beside the Lean name it mirrors at bsv-stack-lean `08ce467d`. (4) Nothing a host already calls has changed its answer: outside the new module only `Params` moves in `src/`, and the tracker's reducer reads none of its three new fields; `State`, `Tracker`, `ChainEvent`, `decode_envelope`, every serde form and the compact bytes are as at 0.9.0-beta.10 (`tests/compact_golden.rs` untouched). bsv-wallet-cli builds `Params` by literal in `src/tracker_host.rs` (two sites); it pins `=0.9.0-beta.10` and does not move until its next pin.

```rust
// 0.9.0-beta.10
let params = Params { age_threshold: 60 };
// this line: the same host, its spend depth said once
let params = Params::new(60, 1);
// a host whose feed names a number
let params = Params::new(60, 1).with_bound(Some(4));
```

### Added: the page of the doubts (for #51)

- `docs/DOUBTS.md`: what the doubts are for a host; every public item of `bsv_tracker::doubts`, and the fields of `Params` it reads, beside the Lean name it mirrors at bsv-stack-lean `08ce467d`, with one line of what it answers; the stored form and what it refuses; the printed line field by field; the feed's named number, the out host, and the snapshot's five conditions with the refusal each gives; what is not mirrored and where it belongs; the test of record and its variables. `docs/SPEC.md` points to it. `README.md` gains a section, "The doubts", whose example runs as a doc test: a host builds its `Params` with `Params::new`, folds a `split` row and its `splitResolved` row, and asks `spendable_eff` for a coin held by the split and then passed four blocks clear of the rival it left. Two doc comments are corrected: `Refused` has two values for the tip's condition, and `Params::spend_depth` is read by `spendable_c` as well. No line of code moves.

### Added: the doubts' theorems as properties (for #51)

- `tests/doubts_properties.rs`, 31 randomized statements over `bsv_tracker::doubts`, each named for the theorem of bsv-stack-lean at `08ce467d` it mirrors (`lean/Headers/Doubts.lean`; `lean/Headers/Host.lean` sections 6 and 12), 256 cases each from the explicit ChaCha seed with no failure persistence, as `tests/properties.rs`: the six issue #51 names (`safe_either_lag`, `reorg_holds_until_headers`, `run_safe`, `run_exact`, `state_bound`, `lesser_cost_bound`), with `run_safe_from`, `no_feed_is_today`, `stale_feed_only_unseen`, `safety`, `coldFrom_safe`, `least_enough` and `least_fold_coversK`; the keyed split's laws of cut 1f (`split_row_holds`, `split_new_key`, `split_open_key`, `split_adds_rival`, `splitResolved_mem`, `splitResolved_unknown`, `splitResolved_reads_no_height`, `splitResolved_keeps_other`, `split_holds_until_resolved`, `splits_never_shrink`, `run_perm`); and cut 1g's bound, out host, snapshot and page loop (`trips_iff`, `trips_none`, `out_passes_nothing`, `effC_le`, `accepts_iff`, `starts_le_reads`). `registerAncestor_is_onProof` pins that the Lean's `registerAncestor` is the tracker's own proof step on a new state, no new item. `docs/VERIFICATION.md` lists each test beside its theorem. No source line changes.

### Fixed: a split's key in mixed case through `Doubts::new` and the stored form (for #51, #57)

- A split's key is lower-cased where it is read from the wire, in `raw` and in `Snapshot::new` (with the snapshot, above): the decoder accepts a hash in either case and a key is compared as text, so a `split` row and its `splitResolved` row name one key in whatever case each is sent. `Doubts::new` and the stored form's reader did not lower-case it: a key that came in through either of those two in upper case stayed so, and a `splitResolved` row for it, read in lower case, never closed it. Both now lower-case each key, so a key is held in lower case whichever way it comes in, and the stored form refuses two spellings of one key as one key named twice. `Doubts::new` is no longer a `const fn` (no release carries the module). Test: `a_mixed_case_key_is_held_in_lower_case` in `tests/doubts_rows.rs`, which fails before the change.

### Added: the direction rule and its two checks

- The owner's ruling of 2026-11-18, as bsv-wallet-cli holds it: every change is born on `beta`, and `main` takes a `git cherry-pick -x` of a `beta` commit in six classes alone (`RELEASING.md`, "The direction rule"). `.github/workflows/lines.yml` runs `main-in-beta` on every pull request into `beta` and every push to `beta` (each commit of `main` after the merge base `86f193b8` is on `beta` by sha, patch or cherry-pick line, or is a hop commit, as the dependency bot's `Cargo.lock` moves are), `beta-born` on every pull request into `main`, and `lines-selftest`, the 26 planted cases of `.github/ci/test_lines.py`, on every event. No source line, test or public item of the crate changes.

### Added: the effective depth, the guard, the builder's test, the out host and the snapshot (for #51)

- In `bsv_tracker::doubts`, mirrors of bsv-stack-lean at `7e616ab3` under the Lean's names: `lead_tip(headers, ds)` and `eff_depth(headers, ds, x)` (`lean/Headers/Doubts.lean` section 2, `leadTip` and `effDepth`: nothing under an open split at or below `x`, else the lesser of the depth under the host's headers and the lead under the lesser of the feed's tip and the headers' tip over every held entry forking at or below `x`), `spendable_eff(params, headers, ds, rule, state)` (section 4, `spendableEff`) and `may_enter(headers, ds, proof)` (section 5, `mayEnter`: the path's root active at its height and that height's effective depth at least 2), with `AppRule` (`Tracker.AppRule`: a name and `allows: fn(&Word) -> bool`; `#[non_exhaustive]`, `AppRule::new`) and the private mirrors `depth`, `still_confirmed`, `root_checks` and `rule_allows` of `lean/Tracker.lean`. `spendable_eff` reads the spend depth from `Params::spend_depth`, as the Lean's reads `P.spendDepth`, and takes no depth argument: the host sets one depth when it builds its parameters, and the crate applies no floor, as the Lean's guard applies none (Hosts, above).
- The Lean's headers are total and the crate's can fail: a headers fault is an `Err(CheckError::Headers(_))` in each function that reads a header, never a pass. A depth is read before a header, so an open split at or below the height answers `Ok(0)` (and `may_enter` and `spendable_eff` `Ok(false)`) before any header is read, and a word that claims nothing of the chain reads no header; a height the headers carry no header for is `false`, never a pass and never a fault.
- The out host (`lean/Headers/Host.lean` section 12): `eff_c(headers, checked, x)` (nothing at any height while `Checked::Out`, no header read; else `eff_depth`) and `spendable_c(params, headers, checked, rule, state)` (the Lean's `effC` and `spendableC`).
- The snapshot: `Snapshot` (`rival`, `splits`, `tip`; section 6: `Snapshot::new`, `empty`, `least`, `doubts`, and section 12's `fold`, the service's fold), `Served` (the snapshot, its cursor, the row version of the decoder that folded it, the cursor through which every row was readable to that decoder, its tip's hash; `Served::new`), and `accepts(params, headers, cursor, served)` with `start_from`, which gives the doubts and the cursor of a taken snapshot. A snapshot is taken only at the host's cursor or past it; of a version `Params::versions` names; readable through its own cursor; holding at most `Params::bound` open splits; and with its tip, by height and by hash, the host's own verified header. Each refusal is a value of its own, `Refused::Behind`, `Version`, `Unreadable`, `Overfull`, `TipAbsent` and `TipOtherHash`, read in the Lean's order, with `Refused::Headers` for a fault; the four on the snapshot's own parts read no header. The tip's hash is compared in either case. After a start every later row's version is the decoder's to check, as before: the snapshot's version covers the rows to its cursor only. `Snapshot`, `Served` and `Refused` are `#[non_exhaustive]`.
- Tests: `tests/doubts_rows.rs` `rows_of_record_asks`, every ask of the rows of record (`eff_depth` under headers carrying no header, and `lead_tip`), 184 counted; `stored_form_refuses` gains the key named twice. `tests/doubts_guard.rs`, 32 tests: appendix B's rows of the plan by the Lean's names (`oneBehind`, `feedAhead` three ways, `Builder.lean`'s `tieDs` and `rivalTwo`, `gapNoHeader`, `spendableEff_today`, `split_holds_coin` with `level_rival_holds_coin`, the guard's two lower arms, the headers fault, `emptyFork`, `emptySplit`, `lowestSplitOnly`, `oneKeyOfTwo`, `settledCut`, `Snapshot.doubts`), and the rows of record of cut 1g's table by name (`absent_today`, `over_then_back`, `extra_key_trips`, `out_zero`, `stuck_at_cursor`, `start_trip_loop`, `overfull_forever`, `covers_not_kept`, `resolve_unknown`, `least_commutes` (its three), `skipped_split`, `after_start`, `other_branch`, `hash_not_content`) over a local mirror of the Lean's `readC`, and every refusal by its value. `rows_of_record.json` gains three cases of the crate's own, worked from `raw` at `7e616ab3` and checked by the run: `rekey_all_order` (the direction of `rekeyAll`'s `foldr`: a front-to-back fold fails it), `reorg_rekey_visible` (a `run` case whose `reorg` rekeys a kept rival: reverting the rekey fails it) and `key_case` (a key in upper case closed in lower case: reading it as sent fails it); 80 cases, 184 asks, sha256 `1287ff6fa1699e673718a27a8e08846991a465f8084ca96af8535f1609c8e2c5`. The CI job `vectors` runs `rows_of_record_states` and `rows_of_record_asks` by name and counts two passes.

### Changed: the doubts' open splits keyed by the fork point (for #51, the stack's cut 1f)

- An open split is now a pair, its key and its fork height, and nothing more, under `lean/Headers/Doubts.lean` of bsv-stack-lean at `656c502a`, section 7d (`isOpen`, `openSplit`, `closeSplit`; the stack's `docs/DECISIONS-2026-10-19.md` 30.7). The key is the two branches' common ancestor's hash: the row's `key` with its `split:` prefix taken off. `Doubts::new` takes and `Doubts::splits` returns `(Hash, Height)` pairs, the newest key at the front.
- `raw`'s `split` arm: a key not open goes to the front with the row's fork height; a key already open keeps its place and takes the lower of its stored height and the row's, so a repeated `split` row adds nothing. The row's lower witness tip is added as a rival either way. `raw`'s `splitResolved` arm closes the pair of its key and no other, and reads no height; a key not open changes nothing. Two keys open at one height no longer close together.
- `Display` prints each open split as `key:height`: `tip=60 splits=[kb:50 ka:50] rivals=[50:54] summary=-`; an empty list prints `splits=[]` as before.
- The stored form holds each open split as `["<key>", <fork height>]`: `"splits":[["k95",95]]`. No release carries the form of fork heights alone (`"splits":[95]`), so there is no reader of it to keep: this reader refuses it, as it refuses a pair without its height, with a third part, or with a key that is not text.
- `tests/vectors/doubts/rows_of_record.json` is at schema `bsv-tracker-doubts-rows/2`: a `split` row is `["split", k, fh, h1, h2]`, a `splitResolved` row `["resolved", k, fh]`, and a start's open split `[k, fh]`. The five cases holding a split are re-worked from the Lean's own rows at `656c502a` (`split_only`, `split_holds`, `two_splits`, `two_splits_one_resolved`, `split_above_tip`, under the Lean's keys `k0`, `k95`, `k5`, `k8`, `k20`); every other case is byte for byte as it was.

- Tests: `tests/vectors/doubts/split_cases.txt`, the stack's reducer's own run over twelve synthetic rows (open, lower, keep the lower, a second key at the same height, a resolve of a key never opened, a resolve that leaves the other, a split above the tip, the same split row twice), its opening line naming where it comes from, and `split_cases_oracle` in `tests/doubts_rows.rs`, which replays the rows through `step` from a cold start and compares each state line byte for byte (all twelve agree). `rows_of_record.json` gains 18 cases from the Lean's rows at `656c502a`: cut 1f's (`two_keys`, `closes_both`, `closes_both_then_kb`, the three cases of a key already open, `keep_stored`, two resolves of a key never opened, the three of a split above the tip, the two orders of K5 and the hold past the settled line of K6) and cut 1g's for the keys (`resolve_unknown`, `covers_not_kept_fold`, `covers_not_kept_kept`). The file's count and sha256 after every change of this line are in the depth's entry above (80 cases).

### Added: the feed's named number on `Params`, checked after every row (for #51, the stack's cut 1g)

- `Params::bound: Option<u32>`, the feed's named number: the most split keys the header service's contract says it can have open at once, as the host was told it. Its absent value, and where a host takes it from, are in Hosts, above. It mirrors `StartParams.bound` of `lean/Headers/Host.lean` section 12 at `656c502a`, the field of the table of `docs/p0/reorg-proof-cut2-2026-11-18.md`, "Cut 1g".
- `doubts::over(params, ds)` (the Lean's `over`: more open splits than the number; never with none), `doubts::step_checked(bounds, params, event, ds)` and `doubts::run_checked(bounds, params, ds, rows)` (the Lean's `trips` beside `run`), answering `doubts::Checked`: `Within(doubts)` or `Out(doubts)`, with `doubts`, `into_doubts` and `is_out`. The count is made after every row, not at a page's end: one row over the number answers `Out`, even when a later row brings the count back. The check drops no split and merges no key: the doubts in either answer are `step`'s or `run`'s, every row applied. The start itself is not counted (`trips` of no rows is false). What an out host does next is the host's (Hosts, above).
- Tests: six cases of `rows_of_record.json` under the table's rows of record (`absent_today`, `over_then_back` under 1 and under 2, `extra_key_trips` for the started host and the kept one, and the trip of `stuck_at_cursor`), each naming its `bound` and whether it is `out`; every other `run` case is checked under no number and is within, its doubts `run`'s; and `bound_counts_after_rows` (more than the number is over and as many is within, the start is not counted, a row that opens no key still answers `Out` while the count stands over, and the same split row twice never trips it).

### Added: the doubts' state and reducer (for #51)

- `bsv_tracker::doubts`, a new public module beside the tracker's reducer: a host's doubts, folded from the header service's feed one row at a time. It mirrors `lean/Headers/Doubts.lean` of bsv-stack-lean at `7213ef4a` under the Lean's names. Nothing is re-exported at the crate root, and no existing item, serde form or compact byte moves: `State`, `Tracker`, `ChainEvent`, the decoder and `tests/compact_golden.rs` are untouched (`Params` is the one existing item that moves: Hosts, above), and a host changes nothing until it calls the module.
- `Rival` (`fork`, `tip`; `Rival::new`, `covers`, `merge`), `Doubts` (the kept rivals, the open splits (keyed above), the feed's tip and the summary of what the prune dropped; `Doubts::new`, `Doubts::cold`, the readers `rivals`, `splits`, `tip`, `gone`, and `held`) and `Bounds` (`settled`; `Bounds::new`, `Bounds::B400`, the feed's settled line). `Rival` and `Bounds` are `#[non_exhaustive]`; `Doubts` keeps its fields private.
- `raw(event, ds)` (the table of rows, no prune), `prune(bounds, ds)`, `step(bounds, event, ds)` (`raw`, then `prune`) and `run(bounds, ds, rows)` (`step` folded from the left). They take the decoded `ChainEvent` a host already holds, take and return values, and read a row's heights and a split row's key alone: no header, clock, root or chain work. The split arms are the keyed ones above.
- The stored form of `Doubts`, its serde text, with the fields in the Lean's order and under the Lean's names: `{"rivals":[{"fork":101,"tip":101}],"splits":[],"tip":103,"gone":null}`. All four fields are required on read, `gone` included (`null` or a rival: a text without it is refused, never read as "nothing dropped"), and a field this reader does not know is refused. `Bounds` has no serde. The compact format stays one transaction's and stays at version 1.
- `Display` for `Doubts`: `tip=103 splits=[] rivals=[101:101] summary=-`, the lists in their order, the summary as `fork:tip` or `-`.
- Tests: `tests/vectors/doubts/rows_of_record.json` (53 cases taken from the `#guard` rows of `lean/Headers/Doubts.lean` section 8 and `lean/Headers/Host.lean` section 11 at `7213ef4a`; no journal row) and `tests/doubts_rows.rs`: `rows_of_record_states` (every case's printed state after its last row, 53 counted), `stored_form_round_trips` and `stored_form_refuses`. The file's `asks` are for the depth functions, read by `rows_of_record_asks` (the depth's entry above). The CI job `vectors` runs `rows_of_record_states` by name and counts its pass.

### Added: the doubts' test of record (for #51)

- `tests/journal_states.rs`, `journals_reproduce_the_states_after_every_row`: the header service's two journals, folded from `Doubts::cold(0)` under `Bounds::new(400)` with the crate's own `decode_envelope` and `doubts::step`, reproduce the stack's journal driver's state after every row (bsv-stack-lean at `7213ef4a`, `tools/journal/JournalRun.lean`): each states line equal by the ruled comparison (the cursor, the tip, the splits, the rivals as a set, the summary) and byte for byte. The journals are private to the estate and read in place from `BSV_TRACKER_JOURNALS_DIR` and `BSV_TRACKER_JOURNAL_STATES_DIR`; nothing of them is committed, and a failure names a line's number and the fields that differ, never a value. With the variable unset (CI's run) the test prints a `SKIPPED` line and passes. The run of record: production 6,254 rows, 6,254 states lines equal and byte-equal; the beta 39,686 rows, 11 unreadable deliveries skipped as the driver skips them, 39,675 states lines equal and byte-equal (`docs/VERIFICATION.md`).
- `a_wrong_line_is_named_by_its_fields`: the comparison's witness on a hand-built `Doubts` and hand-written lines, run everywhere. No source line or public item changes.

## [0.9.0-beta.10] - 2026-11-18

The pin to bsv-rs 0.9.0-beta.10, bsv-rs's security prerelease, so that the toolbox's and the CLI's beta lines can follow onto it through this crate. No behaviour moves and no public item changes: every test of the base keeps its name and its answer, the compact bytes are unchanged (`tests/compact_golden.rs`), and a host changes nothing at this prerelease.

### Changed: the pin to bsv-rs 0.9.0-beta.10

- `bsv-rs = "=0.9.0-beta.10"` (the beta pin the toolbox and the CLI follow onto). No source line of this crate changes for it and no answer of this crate moves: it names `bsv_rs::transaction::MerklePath` and `MerklePathLeaf` alone (it builds a path, computes its root and reads its height), and bsv-rs's `src/transaction/merkle_path.rs` is byte for byte the same in 0.9.0-beta.9 and 0.9.0-beta.10; between the two packages bsv-rs's `src` moves only in its key-value store (`kvstore/global.rs`, `kvstore/interpreter.rs`), a module behind bsv-rs's `kvstore` feature, which this crate does not turn on (it asks for `transaction`, and for `wasm` under its own `wasm` feature). bsv-rs 0.9.0-beta.10 states no breaking change since 0.9.0-beta.9. Read at the pin as `ci.yml` runs it: `cargo test --locked` (87 passed, 0 failed, each test of the base by the same name and the same answer, `tests/compact_golden.rs` unchanged), the documentation, rustfmt, Clippy with `-D warnings`, the two vector steps, the wasm32 build and the check at 1.88; and, outside `ci.yml`, `cargo deny` 0.20.2 (advisories, bans, licenses and sources ok).

### Security

- bsv-rs 0.9.0-beta.10 carries the fix of GHSA-j4w5-4g84-q3f4 (Medium, CVSS 3.1 5.3), which names every prerelease through 0.9.0-beta.9. In the advisory's words: "`GlobalKVStore::query`, `get_by_controller` and `get_by_tags` send the caller's selector ... to the overlay lookup resolver and return the resolver's answer to the caller without re-checking the selector on the client side", and the fix: "`query_overlay`, the single path every read door and the existing-token lookup share, now drops a decoded entry that does not satisfy the query's key, controller (compared case-insensitively), protocol (App-namespace-aware) or tags (all/any mode) before it is returned". This crate does not reach those reads: through bsv-rs it builds a merkle path and reduces it to a root, and it calls no key-value store (`src` and `tests` name no `kvstore`, `GlobalKVStore` or `query_overlay`; their five `bsv_rs::` lines name `MerklePath` and `MerklePathLeaf` alone), and the `kvstore` module is not compiled in its build. The pin matters to this crate's consumers: the toolbox and the CLI pin bsv-rs exactly and a build holds one bsv-rs, so they reach the patched prerelease only with a tracker that pins it.

## [0.9.0-beta.9] - 2026-11-17

Room for the three rows of the rulings of 2026-11-17 (bsv-stack-lean `docs/RULINGS-2026-10-12.md`, 16:1xZ; `docs/DECISIONS-2026-10-19.md` 28.8.1 (c) with its three additions, and the keeper's reading of 17:1xZ for `VerdictError`): finality (#45), the status code (#46) and `conflicted` keeping its proof (#47) land after the 0.9.0 cut as minors, each behind its Lean, and none of them is built here. Five public types turn `#[non_exhaustive]`, so a host's exhaustive `match` on `HostAction`, `Reask`, `HintStatus` or `VerdictError`, and a `Verdict` built by literal, stop compiling: a break on this line, by the semver note of 0.3.0. No behaviour moves: every test of the base keeps its name and its answer, and the compact bytes are unchanged, pinned by `tests/compact_golden.rs`, which landed on the base before any other change.

### Changed

- `#[non_exhaustive]` on the enums `HostAction`, `Reask`, `HintStatus` and `VerdictError` and on the struct `Verdict`: the type alone, no variant marked, `Verdict`'s four fields still public to read and to assign. A `compile_fail` doctest on each pins the break (E0004 for a match outside the crate, E0639 for the literal).

### Added

- `Verdict::new(status)`: a verdict holding the word as given (not trimmed, not checked), with no height, no reason and no competitors; the fields are assigned after it, and `Verdict::from_arc_body` stays the checked reader.
- `HintStatus::label` and `Reask::label`: each kind's stable word, the payload not in it (`accepted`, `seen`, `mined`, `stale_block`, `rejected`, `double_spend`, `orphan_mempool`, `unknown`, `mempool_present`; `reorg`, `fork`, `hint`, `proof_failed`, `recheck`, `age`, `spend`). `HintStatus::LABELS` and `Reask::LABELS`: every word in declaration order, which is the compact form's tag order. A word never changes and is never reused; a later minor only appends.
- The serde form of `HintStatus`, the crate's stored form of a status: internally tagged by its word under `status`, the payload beside it as `height`, `reason`, `competitors` (left out when empty) and `courier` (`MempoolPresent`'s `source`), so `{"status":"mined","height":900}`. A word this release does not know, and a kind whose payload is missing, are refused, never read as `Unknown`. `Reask` has the words alone; `Hint`, `Verdict` and `HostAction` have no serde.
- Tests: `tests/compact_golden.rs` (the compact bytes of ten states that hold every word, a checked proof, every status with and without a verdict and every re-ask), `tests/labels.rs` (the words, both lists, every status's text round-tripped, the refusals, and the host's shape below), and two unit tests that `LABELS[i]` is the word of the kind the compact writer tags `i`.

### Changed: the pin to bsv-rs 0.9.0-beta.9

- `bsv-rs = "=0.9.0-beta.9"` (the beta pin the toolbox and the CLI follow onto). No source line of this crate changes for it and no answer of this crate moves: it names `bsv_rs::transaction::MerklePath` and `MerklePathLeaf` alone (it builds a path, computes its root and reads its height), and bsv-rs's `src/transaction/merkle_path.rs` is byte for byte the same in 0.9.0-beta.8 and 0.9.0-beta.9; between the two packages bsv-rs's `src` moves only in its BEEF linker (`beef.rs`, `beef/link.rs`, the new `beef/link_limits.rs`) and a notes file. bsv-rs 0.9.0-beta.9 states no breaking change since 0.9.0-beta.8. Read at the pin as `ci.yml` runs it: `cargo test --locked` (87 passed, 0 failed, each test of the base by the same name and the same answer, `tests/compact_golden.rs` unchanged), the documentation, rustfmt, Clippy with `-D warnings`, the two vector steps, the wasm32 build and the check at 1.88; and, outside `ci.yml`, `cargo deny` 0.20.2 (advisories, bans, licenses and sources ok).

### Security

- bsv-rs 0.9.0-beta.9 carries the fix of GHSA-33fm-jcm5-4m3m (High, CVSS 3.1 7.5), which names every prerelease through 0.9.0-beta.8. In the advisory's words: "the #749 link bound (GHSA-gvmh-f5gf-5vjj) meters a BEEF's serialized wire bytes but not the heap each cloned stub re-materializes - the proof a stub carries and the struct backings of its inputs and outputs - so an untrusted BEEF within the lookup resolver's size bound can force a memory-exhaustion denial of service past the 128 MB Worker isolate; the fix keeps that wire budget and adds a second, heap budget beside it". Its door is linking a BEEF: "`Transaction::from_beef` runs this link, and the overlay lookup resolver and its sibling doors reach `from_beef` on an untrusted answer", and bsv-rs's changelog adds `Beef::find_transaction_for_signing`. This crate does not reach that door: through bsv-rs it builds a merkle path and reduces it to a root, and it links no BEEF (`src` and `tests` name no `Beef`, `from_beef` or `find_transaction_for_signing`; their five `bsv_rs::` lines name `MerklePath` and `MerklePathLeaf` alone). The one default of 0.9.0-beta.9 that refuses earlier, the heap bound on linking (`BeefLinkLimits::max_link_heap_bytes`), sits on that door, so it moves no answer here. The pin matters to this crate's consumers: the toolbox and the CLI pin bsv-rs exactly and a build holds one bsv-rs, so they reach the patched prerelease only with a tracker that pins it.

### Hosts

- A match on `HostAction`, `Reask`, `HintStatus` or `VerdictError` gains a wildcard arm that keeps and surfaces what the host does not name: a re-ask by `Reask::label`, an error as the fault it is. Never an arm that maps to `HintStatus::Unknown` or drops the value.
- A `Verdict` built by literal becomes `Verdict::new(status)` with its other fields assigned after it.
- A host that stores hints stores the serde form of `HintStatus` and reads it back by it, never by a hand map, and surfaces a status the reader refuses.
- bsv-wallet-cli's stored rows read as they are: a struct with the status flattened between `source` and `observed` reads every text its `StoredHint::of` writes (beta at 108f80e) and writes it back identical (`tests/labels.rs`). Its pin of the next prerelease replaces its two hand maps with these words.

## [0.9.0-beta.8] - 2026-11-17

### Changed: the pin to bsv-rs 0.9.0-beta.8

- `bsv-rs = "=0.9.0-beta.8"` (the beta pin of the toolbox and the CLI). No source line of this crate changes: it names `bsv_rs::transaction::MerklePath` and `MerklePathLeaf` alone (it builds a path, computes its root and reads its height), and none of bsv-rs 0.9.0-beta.8's ten breaking changes reaches a call here. The one that names `MerklePath`, its writers `to_binary`, `to_hex` and `to_writer` returning `Result` (bsv-rs #817), meets no call: this crate writes a path through its own compact form. Read at the pin as `ci.yml` runs it: `cargo test --locked` (64 passed, 0 failed), the documentation, Clippy with `-D warnings`, the two vector steps and the wasm32 build; and, outside `ci.yml`, `cargo deny` 0.20.2 (advisories, bans, licenses and sources ok).
- Two answers of this crate move with the pin. bsv-rs #817 also changes the merkle walk behind `MerklePath::new_unchecked` and `compute_root`, which `Proof::new` and `Proof::root` call: a path whose walk pairs a hash holding a character that is not hex is now refused at the constructor, `CheckError::InvalidProof` (bsv-rs's `Invalid hex string`). Through 0.9.0-beta.7 bsv-rs dropped that side of the pair, so `Proof::new` answered `Ok` and the header check answered `RootMismatch` (or `Unavailable`, or `Headers`), which sets `Reask::ProofFailed`. `State::from_compact` builds its proof through the same constructor, so a stored path of that shape is refused there in the same class (read, not run). A sibling that is hex of another length is paired as the reference concatenates it: of even length as at 0.9.0-beta.7; of odd length with one leading `0`, where 0.9.0-beta.7 dropped that side, so `Proof::root` answers another root for it than it did. Either root is one the well-formed proof of the txid does not have. No word moves: such a path reached neither `Mined` nor `Conflicted` before, and reaches neither now. `tests/transitions.rs` (`malformed_sdk_paths_are_rejected_before_root_reduction`) pins the refusal and the other-length pairing.

### Security

- bsv-rs 0.9.0-beta.8 is the prerelease of the 0.9.0 line that carries the stable line's advisory fixes (the sixteen advisories its release note lists); the advisories name every earlier prerelease. This crate calls none of the doors they name: through bsv-rs it reads no transaction, BEEF, signature, certificate or Extended Format bytes. The pin matters to its consumers: the toolbox and the CLI pin bsv-rs exactly and a build holds one bsv-rs, so they reach the patched prerelease only with a tracker that pins it.

### Hosts

- A host that writes a proof's path with bsv-rs's own writers (`proof.path().to_hex()`, `proof.path().to_binary()`) adds `?` at bsv-rs 0.9.0-beta.8 (bsv-rs #817).
- A host that builds a path by hand (the public fields, a provider's text hashes) and hands it to `Proof::new`: a sibling that is not hex is refused there with `CheckError::InvalidProof`, before any step, so this crate sets no `Reask::ProofFailed` for it; the host treats the refusal as it treats any malformed proof. Through 0.9.0-beta.7 the same path was built and failed at the header check.
- This crate's types, words and compact bytes do not move.

## [0.9.0-beta.7] - 2026-11-04

### Changed: the pin to bsv-rs 0.9.0-beta.7

- `bsv-rs = "=0.9.0-beta.7"` (the beta pin of the toolbox and the CLI); no code change: this crate uses the `transaction` feature alone and calls no wallet method, so beta.5's wallet, auth and wire changes do not reach it; the additive verify methods and the typed not-judged class leave this crate's call as beta.3's (confirmed by cargo check at the pin).

## [0.9.0-beta.6] - 2026-11-01

### Changed: the pin to bsv-rs 0.9.0-beta.6

- `bsv-rs = "=0.9.0-beta.6"` (the beta pin of the toolbox and the CLI); no code change: this crate uses the `transaction` feature alone and calls no wallet method, so beta.5's wallet, auth and wire changes do not reach it; the additive verify methods and the typed not-judged class leave this crate's call as beta.3's (confirmed by cargo check at the pin).

## [0.9.0-beta.5] - 2026-10-31

### Changed: the pin to bsv-rs 0.9.0-beta.5

- `bsv-rs = "=0.9.0-beta.5"` (the beta pin of the toolbox and the CLI); no code change: this crate uses the `transaction` feature alone and calls no wallet method, so beta.4's wallet, auth and wire changes do not reach it; the additive verify methods and the typed not-judged class leave this crate's call as beta.3's (confirmed by cargo check at the pin).

## [0.9.0-beta.4] - 2026-10-30

### Changed: the pin to bsv-rs 0.9.0-beta.4

- `bsv-rs = "=0.9.0-beta.4"` (the beta pin of the toolbox and the CLI); no code change: this crate uses the `transaction` feature alone and calls no wallet method, so beta.4's wallet, auth and wire changes do not reach it; the additive verify methods and the typed not-judged class leave this crate's call as beta.3's (confirmed by cargo check at the pin).

## [0.9.0-beta.3] - 2026-10-28

### Changed: the pin to bsv-rs 0.9.0-beta.3

- `bsv-rs = "=0.9.0-beta.3"` (the beta pin of the toolbox and the CLI); no code change: this crate uses the `transaction` feature alone and calls no wallet method, so beta.3's wallet-client, validator and default-script-mode changes do not reach it; `Transaction::verify`'s signature is beta.2's.

## [0.9.0-beta.2] - 2026-10-28

### Changed: the pin to bsv-rs 0.9.0-beta.2

- `bsv-rs = "=0.9.0-beta.2"` (the beta pin of the toolbox and the CLI); no code change: this crate uses the `transaction` feature alone and calls no wallet method, so #329 (the originator as `Option`) and #331 (the verifier's memory budget) do not reach it.

## [0.9.0-beta.1] - 2026-10-26

A prerelease cut from `beta` on bsv-rs 0.9.0-beta.1, the pin of bsv-wallet-toolbox-rs 0.14.0-beta.1 and bsv-wallet-cli's beta line, so a consumer pinning both takes one bsv-rs and this crate's `MerklePath` is theirs. `Cargo.toml` declares `bsv-rs = "=0.9.0-beta.1"` exactly (a caret requirement admits no prerelease), the same features; the committed lock carries one bsv-rs, 0.9.0-beta.1. Never merged into `main` as a prerelease.

- No code change: the crate builds, tests, lints and builds for wasm32 and at the MSRV against bsv-rs 0.9.0-beta.1 as it stood. bsv-rs's `Kind` and `Reason` turn `#[non_exhaustive]` (bsv-rs `CHANGELOG.md:310`, `:330` at `v0.9.0-beta.1`); this crate imports neither, nor the BEEF reader nor a certificate type: its only bsv-rs items are `MerklePath`, `MerklePathLeaf`, `MerklePath::new_unchecked` and `compute_root` (with the two structs' public fields, read and built by literal, `src/compact.rs:243-258`, `:450-459`, and bsv-rs's error text carried in `CheckError::InvalidProof`, `src/evidence.rs:78-80`) (`src/evidence.rs:3`, `:77-80`, `:104-106`; `src/compact.rs:34`, `:450`).
- `.github/workflows/ci.yml`: the gate runs on pull requests into, and pushes to, `main` and `beta` (it ran on `main` alone, so a pull request into `beta` ran no checks).
- `.github/workflows/release.yml`: a tag whose version holds a `-` (a prerelease) must be on `beta`, any other on `main`, as bsv-rs's `release.yml` at `v0.9.0-beta.1` checks its line ("The tagged commit is on its line").

### Notes: cross-SDK parity

The even-offset duplicate is observed here: bsv-rs 0.9.0-beta.1's `MerklePath::new_unchecked` refuses a duplicate leaf (flag `01`) at an even offset, `Duplicate at an even offset: N, at height: N` (bsv-rs `CHANGELOG.md:1433` at `v0.9.0-beta.1`; `src/transaction/merkle_path.rs:204`, `src/transaction/beef/in_memory.rs:131-141`), and `compute_root` computes no node from a left duplicate (`merkle_path.rs:704-710`). Through bsv-rs 0.8.3 (the lock this pin replaces) `Proof::new` accepted such a path: a duplicate at 0 beside the txid at 1 reduces to sha256d(h || h); a computed node over a left duplicate hashed an empty left hash (bsv-rs `CHANGELOG.md:1416-1419` at `v0.9.0-beta.1`). So `Proof::new` (`src/evidence.rs:77-80`) returns `CheckError::InvalidProof` for such a path, and `State::from_compact` returns `CompactError::Evidence` for a stored proof holding one (`src/compact.rs:138`); bsv-rs records no writer and no corpus BUMP of that shape, and TS pairs the working hash with itself where the duplicate is its sibling, giving up only where it computes a node from a left duplicate (`ts-stack@edf6e03` `MerklePath.ts:60`, `:628-631`). The BEEF reader's order tolerance and `DuplicateOffset` are not observed: this crate reads no BEEF and no BUMP bytes (its compact form builds the `MerklePath` struct itself, `src/compact.rs:432-459`), and matches no bsv-rs kind. The refusal is bsv-rs 0.9.0-beta.1's alone: bsv-rs's `beta` reverses it, Unreleased (#167's re-read, bsv-rs `CHANGELOG.md:159` on `beta` at `3428530`), so at the next bsv-rs prerelease the level-0 shape reduces again and only a computed node over a left duplicate is refused (`Missing hash at height N, offset N`); this crate's beta.2 re-reads the note there.

## [0.8.0] - 2026-10-21

bsv-rs 0.8.0 (Calhooon/bsv-rs #71; the wave of Calhooon/bsv-wallet-cli #44). `Cargo.toml` declares `bsv-rs = "0.8.0"` by caret, the same features; the committed lock carries one bsv-rs, 0.8.0. The crate's public types carry bsv-rs's, so a consumer moves bsv-rs and this crate together: the minor, by the semver note of 0.3.0. bsv-rs 0.8.0 makes its `Error` `#[non_exhaustive]` and adds `Error::WalletBusy`; this crate matches no bsv-rs error (a path that does not reduce is `CheckError::InvalidProof` with the error's text), so no verdict, no word and no re-ask moves.

## [0.7.0] - 2026-10-20

The reorg rule by the root and the presence hint, rulings (5) and (2) of 2026-10-20 19:4xZ (bsv-stack-lean `docs/RULINGS-2026-10-12.md`; `docs/DECISIONS-2026-10-19.md` sections 8, 8a, 8b). No public type changes; what `Tracker::on_chain` and `State::step` return for the same input does, so a minor.

- The reorg rule by the root: a `mined` word is `stale` when, and only when, its stored proof's root is not the active header's root at its height in the host's headers. A reorg at or below the word's height and an invalidation of its checked header both run the crate's own recheck, a header read and never a fetch. The same root, the word stands and its marks clear; a moved root, `Word::Stale` with `Reask::Reorg { fork_height }` (the reorg's fork height, or the word's height for an invalidation); no header, the word stands suspect with `Reask::Recheck` and the next tip rechecks it. The selection is unchanged (at or above the fork height for a reorg, a fork and a split; the block's hash for an invalidation): it is the bound of the recheck, never the stale mark. Until 0.7.0 a reorg wrote `stale` by the height alone and an invalidation by the hash alone. Forks, splits and resolved splits are unchanged.
- The presence hint: `HintStatus::MempoolPresent` over `built` moves the word to `announced`, as `Accepted` does (a node we did not broadcast through holds the bytes, so someone announced it; the chain has no author). Over `announced` it leaves `announced`: the same word either way. Until 0.7.0 it left `built` with the hint's re-ask.
- Tests: `tests/reorg_by_root.rs` (seven); the property `reorg_reasks_at_or_above` restated as `stale_iff_header_inactive`; the tests whose expectation was the height rule now move the root before the event, each with a comment naming the ruling.

Migration for hosts: after a chain event, `Reask::Reorg` means "the root moved, fetch a proof". A host that re-asked every row at or above the fork now re-asks only the rows whose root moved, a few a pass, the pacing the host's; a host's own drain may drop its recheck step (stored root against the active header), which the crate now runs. A host that read a reorg's `stale` without supplying the new branch's headers now sees the word stand; a host whose headers lack the height sees `Reask::Recheck` and a suspect word, never served. A host that matched `built` after a presence hint now sees `announced`.

## [0.6.0] - 2026-10-18

bsv-rs 0.7.0 (the update loop's PR #14, green on the gate, taken as the base; the wave of 2026-10-18). The crate's public types carry bsv-rs's, so a consumer moves bsv-rs and this crate together: the minor, by the semver note of 0.3.0. No behaviour of the tracker changes; bsv-rs's default signing digest is not on any path of this crate.

## [0.5.0] - 2026-10-18

The compact State form and the courier's mempool presence (bsv-stack-lean #92, #93, #98). The public `HintStatus` enum widens by one variant, so a host's exhaustive `match` on it stops compiling: the minor, by the semver note of 0.3.0. Every other change is additive.

- `State::to_compact() -> Vec<u8>` and `State::from_compact(&[u8]) -> Result<State, CompactError>`: a version byte (`COMPACT_VERSION`, 1), the txid, the word, the evidence, the suspect mark, the hints, the last hint time and the re-ask, the layout in `src/compact.rs`. The decoder takes only the bytes the encoder writes and refuses a stored proof that does not hold together (`CompactError::Evidence`); it consults no headers.
- The restore rule: a restored `mined` word is still suspect until its proof is rechecked against the host's headers (`CheckedProof` has no deserializer). `from_compact` restores the word on the stored record, marks the state suspect and sets `Reask::Recheck` when no other re-ask was pending; every other field returns exactly, and a restored state is a fixed point of the round trip.
- `Tracker::restore(State) -> Option<State>`: holds a restored state with its indexes, so a restored mined word joins the suspect set and the next tip rechecks it.
- `HintStatus::MempoolPresent { source }`: a courier's mempool presence, not a broadcaster word. It writes no chain word, never promotes `built`, agrees with `announced` and `seen`, restarts the age clock (the age re-ask falls due `age_threshold` after it) and schedules the hint's re-ask against any other word. `Verdict::into_hint` never produces it.
- `tests/compact.rs`: the round trip over 256 reproducible random states, every truncation, forged records, and restore then recheck (agreeing headers clear the mark; moved headers make the word stale). `tests/transitions.rs`: the presence rows. The property traces of `tests/properties.rs` include the presence hint.

## [0.4.1] - 2026-10-18

The Arcade vocabulary against LOW's nine bodies (bsv-stack-lean #169; bsv-overlay-cloudflare `crates/overlay-cloudflare/src/door_words.rs`, test `tracker_vocabulary_differs_from_the_door_exactly_here`). Every broadcaster word is still a hint, never a verdict, and a hint writes no chain word. No change to the public types, only to what they hold.

- The status word is trimmed before the vocabulary check: `" SEEN_ON_NETWORK "` is `Seen`, where it used to be `UnknownStatus`. `Verdict::status` now holds the trimmed, upper-cased word (`rejected` reads as `REJECTED`).
- `PENDING_RETRY` and `STUMP_PROCESSING`, a broadcaster still working on it, are `Accepted`; they used to be `Unknown`.
- Only the status word moves the status. `extraInfo` is text and stays on `Verdict::reason`: an `ORPHAN` there no longer makes `OrphanMempool` (only `SEEN_IN_ORPHAN_MEMPOOL` does), and a `reorg_unmined` there no longer makes `StaleBlock` (only the stale-block words do). The P0-2 vector's `200_seen_on_network_extra_info_orphan` is now the tracker's `seen` hint.
- Unchanged, deliberately: a mined word with no height is `Unknown`; named competitors behind an accepting word are `DoubleSpend`; `INVALID` is `Rejected`, `UTXO_SPENT` `DoubleSpend`, `CONFIRMED` with a height `Mined`.
- `tests/arcade_bodies.rs`: the nine bodies verbatim, one test each.

## [0.4.0] - 2026-10-17

Decode the header service's feed at envelope version 2 (Calhooon/bsv-tracker #11; chaintracks-cloudflare v0.4.1). The public `ChainEvent` enum widens by two variants, so a host's exhaustive `match` stops compiling: the minor, by the semver note of 0.3.0.

- `ChainEvent::Split { key, fork_height, tips: [SplitTip; 2] }` and `ChainEvent::SplitResolved { key, fork_height }`, `v: 2` on the wire; `SplitTip { hash, height, chain_work, rungs }` exported; `ChainEvent::version()`. Shape and relationship checks as at chaintracks-cloudflare@v0.4.1 `src/events.rs:157-161,210-232`.
- The version gate: `v: 1` carries the six original kinds, `v: 2` the two split kinds; `v: 2` with a version-1 kind, or `v: 1` with a split kind, is `UnknownKind`; any other `v` is `UnknownVersion`. A `v: 2` envelope used to be `UnknownVersion(2)` whatever its kind. The six version-1 kinds decode as before.
- The state rule: a split marks the mined words at or above `forkHeight` suspect with `Reask::Fork { height: forkHeight }` and writes no word; `splitResolved` rechecks the suspect words at or above `forkHeight` against the host's headers. A split never writes a word; a resolved split moves a word only through a checked proof.
- The `vectors` job runs eight envelope vectors (the two split kinds added).

## Unreleased

The lint bar (bsv-stack-lean #134, #135, #137, #139, #140, #145); no behavior change.

- `[lints]` table: clippy pedantic as warnings (blocking under the gate's `-D warnings`), `unwrap_used`, `expect_used` and `panic` denied, `unsafe_code` forbidden, `missing_docs` denied. Every public field and variant is documented and every fallible item names its errors.
- `#[must_use]` on 27 accessors and constructors: a caller that drops one of their values now gets a warning.
- `rust-version = "1.88"` declared (bsv-rs 0.6.0 needs `slice::as_chunks`).
- Gate jobs: `msrv` (check at 1.88), `machete` (unused dependencies, non-blocking), `secrets` (gitleaks 8.28.0) and `actionlint` (1.7.12), the two tools built from source tarballs pinned by sha256.

## [0.3.0] - 2026-10-15

- bsv-rs 0.6.0 by caret: the one home of the chain primitives (bsv-rs types sit in the public API, so 0.3.0). The stack's template `deny.toml` adopted byte for byte.

## [0.2.3] - 2026-10-12

- Declare `bsv-rs = "0.5.0"` by caret in the published libraries. The update loop can deliver compatible SDK patches to consumers without another library release; deployables keep their exact pins and committed locks.

## [0.2.2] - 2026-10-12

- Pin bsv-rs exactly to `=0.5.0` and re-lock the committed Cargo.lock; tracker types, transitions, wire formats and dependency features stay unchanged. Hosts passing SDK MerklePath values must use bsv-rs 0.5.
- bsv-rs 0.5.0 adds mining checks and explicit missing-context declines to its streaming BEEF readers; the tracker uses MerklePath and does not run those readers.

## [0.2.1] - 2026-10-12

bsv-rs 0.4.5 (the lone leaf at offset 0 only; the LOCAL work bound inside one input's script); no behavior change under the defaults. The L2 re-pin of bsv-stack-lean item 2.

## Unreleased

- bsv-rs 0.4.5: the lone leaf at offset 0 only; the LOCAL work bound; no behavior change under the defaults. Pin exactly to `=0.4.5` and update the lock.

## 0.2.0 (2026-10-09)

- The dependency moves from bsv-rs 0.3.35 to bsv-rs 0.4.1, the same features (`transaction`; `wasm` under ours). Breaking for hosts: `bsv_rs::transaction::MerklePath` crosses this crate's public API (`Proof::new` takes one, `Proof::path` returns one), so a host on bsv-rs 0.3 that passes its own path no longer compiles (`E0308`, "multiple different versions of crate `bsv_rs`") until it builds the path with bsv-rs 0.4.
- No tracker type, word, transition or wire format changes; the suite is the same 38 tests, green before and after.
- The tracker binds `MerklePath` and runs no BEEF reader, so 0.4.1's refusal of a transaction with no input (`Kind::NoInputs`) does not reach it; bsv-rs's `merkle_path.rs` and `chain_tracker.rs` are byte-identical between 0.3.35 and 0.4.1, and a block's one-leaf coinbase path (`MerklePath::from_coinbase_txid`) is accepted as before.
- The README's sentence on what bsv-rs exports is re-read at 0.4.1, including the reader's `Headers` and `Verdict`, which share names with this crate's.

## 0.1.0 (2026-10-09)

- Implement the tracker charter's nine words and Lean transition machine.
- Require an immutable `CheckedProof` from `Headers::check` for mined words.
- Run its default checker against the current snapshot even when the host overrides `check` with a cached capability.
- Bind SDK BRC-74 paths to the proven txid and retain the checked header and depth.
- Record broadcaster hints without writing chain words, including the ARC vector and Arcade reorg vocabulary.
- Route reorgs by affected mined heights, invalidations by checked header hash and tips by suspect txid.
- Return re-ask values for host execution, with fail-closed header errors and explicit envelope faults.
- Reconcile `ChainEvent` serde with rust-chaintracks' version 1 envelope: full headers and work, tip time/header, two competing tips, deactivated headers/new tip, object outpoints and tip-age tip; reject unknown kinds with typed host faults while retaining the checked-proof boundary and height routing.
- Clear the completed trigger on a matching recheck, following the landed Lean machine at f95520f.
- Expose the four host traits, with no platform clock, network or storage calls.
- Replay seven tracker witnesses and four named randomized Lean properties locally; compile the wasm32 target.

The crate has no deployed predecessor. Three host integrations remain separate lanes. Successful rechecks clear their completed triggers as in the landed Lean machine.
