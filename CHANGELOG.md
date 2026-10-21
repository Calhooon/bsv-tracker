# Changelog

## [Unreleased]

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
