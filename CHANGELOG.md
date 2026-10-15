# Changelog

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
