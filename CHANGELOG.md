# Changelog

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
