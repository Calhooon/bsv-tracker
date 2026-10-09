# bsv-tracker 0.1.0

Every incident in the matrix is one incident: a word about the chain was written once by one machine, believed by the next, and never re-asked when the chain moved (`docs/STEP-BACK-2026-10.md` section 0). The tracker is the component that owns the act of re-asking. Per transaction it holds one word, the evidence behind the word, the hints it has heard, and the next re-ask. Only headers and proofs change a word. Every broadcaster word is a hint that schedules a re-ask and changes nothing by itself. A reorg re-asks the transactions whose proof sits at or above the fork point and no other. It is a library, one Rust crate with a wasm32 build, hosted by the wallet, the overlay engine and the app layer, so that the three hold the same word from the same code.

\[SRC\] The paragraph above is section 0 of the tracker charter at `bsv-stack-lean@24f22f808b1fb44ce9bf128ec89ac5c8839d9d60 docs/charters/tracker.md:9`. The executable transition contract is `lean/Tracker.lean:238-397` at that pin. This repository is the local crate implementation of lane A6 #30. Proposed GitHub home \[D\]: `Calhooon/bsv-tracker`, subject to the owner's naming ruling. There is no remote or publication from this lane.

## Evidence and words

`State` holds `Unknown`, `Built`, `Announced`, `Seen`, `Mined`, `Stale`, `Rejected`, `Conflicted` or `Abandoned`. A `Word::Mined` contains a `Mined` value with private fields, backed by an immutable `CheckedProof`. Only `Headers::check` constructs that capability. Its proof is a real `bsv_rs::transaction::MerklePath` bound to the tracked txid; its computed root must equal the active header's root at that height. The record keeps the path, header hash and root, height, and depth at the checked snapshot. It cannot be deserialized into a trusted state. Stored paths return as raw `Proof` inputs and are checked again, one named transaction at a time. \[X\] The local runs are in [docs/VERIFICATION.md](docs/VERIFICATION.md).

A failed proof check returns `CheckError`, keeps the word, and schedules a re-ask. This includes header faults, missing headers, root mismatches and wrong txids. A recheck that finds a different root writes `Stale`. Reorgs keep the old evidence as a historical record while the word becomes `Stale`. Hosts serve a current verify path only from `Word::Mined` and check the suspect mark before acting. `CheckedProof::depth` is the depth of its recorded snapshot, not a live confirmation count. A host computes current depth from its own current tip.

`NodeVerdict` is explicitly the verdict of a node the host trusts, as specified in charter sections 1 and 10 \[SRC\]. It is separate from broadcaster `Verdict`, whose words always become hints. Until the owner rules otherwise, a production host without its own trusted node does not supply `NodeVerdict::Rejected`. A competitor proof is bound to the competitor's txid and checked through the same header boundary. The host supplies the fact that the competitor spends one of our inputs, an assumption of the Lean `Evidence.competitor` input \[D\]. As in Lean, the state's retained evidence is its own inclusion record; the host retains competitor paths if it needs to serve their verification paths.

`Tracker` indexes mined heights, checked header hashes and suspect txids. `on_chain` routes a reorg or fork only to mined words in the affected height range, invalidation only to proofs checked against the named hash, and a tip only to suspect words. Hints, age ticks and spends are applied to one named txid. There is no periodic collection sweep. Agreeing hints retain any already pending re-ask. Hints are recorded newest received before older hints, and keep their original `Verdict` when present.

## The host's four things

The host supplies `Headers`, `ProofFetcher`, `HintSource` and `Clock`. Headers answer from one fresh, verified active-chain snapshot, using `header_at` and `tip_height`. The default `check` method performs the merkle arithmetic and keeps the matching header. Header validity and active-chain selection are the host's obligations, corresponding to `HeadersAreTheActiveChain` \[SRC, `lean/Tracker.lean:1021`\]. The crate uses the charter's header projection, `{ hash, merkle_root }`; bsv-rs 0.3.35 exports `MerklePath` and `ChainTracker`, and no full block-header type from `transaction/mod.rs:104-168` \[SRC, bsv-rs@7bc623c\]. No header validity rule is implemented here.

Proof transitions call the default `Headers::check` through an internal snapshot wrapper. A host override returning an older checked capability cannot bypass current `header_at` and `tip_height` lookups. This applies to our own proof and competitor proofs. \[X\] A regression exercises an override returning cached evidence while the current snapshot is unavailable.

The host executes `Reask` with its proof transport, supplies callback or poll metadata through its hint sources, and turns `Clock::now` into `HostAction::Tick` for the transaction whose age is due. It reads and reports every `CheckError` and `DecodeError`. The tracker calls no proof fetcher, platform clock, storage, network transport or hint source. `serde_json` is used only for pure byte decoding; the SDK dependency disables its default features and enables `transaction`, with `wasm` under our feature. No HTTP or async runtime feature is enabled.

The host feeds proofs from callbacks, the overlay or peers as `Evidence::Proof`, independently of their hint. `Verdict::from_arc_body` checks metadata against the P0-2 vocabulary at `bsv-rs@7bc623c tests/vectors/arc_tx_status_verdicts.json` \[SRC\]. `Verdict::into_hint` recognizes Arcade's twelve words and its `reorg_unmined` and `reorg_reanchor` markers \[SRC, arcade@1ae1208 models/transaction.go:89-126,319-332\]. A named competitor on an accepting or seen word is reduced to a double-spend hint so the re-ask retains its txids \[D\]. A mempool sighting with an orphan marker becomes an orphan hint. Neither writes `Conflicted` or `Rejected`.

## App layer: badge and verify path \[D\]

This sketch reads the word and exposes the evidence only when it can be used. The integration lane must connect it to the badge response and cache invalidation.

```rust
use bsv_tracker::{CheckedProof, State, Word};

fn badge_path(state: &State) -> (&Word, Option<&CheckedProof>) {
    let path = match state.word() {
        Word::Mined(mined) if !state.suspect() => Some(mined.checked()),
        _ => None,
    };
    (state.word(), path)
}
```

## Overlay: settlement \[D\]

The overlay settles only on checked, non-suspect inclusion. It withdraws settlement on `Stale` and discards the orphaned proof from the served BEEF; a new checked proof restores it.

```rust
use bsv_tracker::{CheckedProof, State, Word};

fn settlement(state: &State) -> Option<&CheckedProof> {
    match state.word() {
        Word::Mined(mined) if !state.suspect() => Some(mined.checked()),
        _ => None,
    }
}
```

## Wallet: spend guard \[D\]

The wallet executes the scheduled recheck before using the coin. An unavailable header is an error and prevents the spend. The host also applies its own confirmation-depth and frozen-output rules.

```rust
use bsv_tracker::{CheckError, Evidence, Headers, HostAction, Input, Params, State, Word};

fn spend_guard<H: Headers>(state: &mut State, params: &Params, headers: &H)
    -> Result<bool, CheckError>
{
    state.step(params, headers, Input::Host(HostAction::SpendAttempt))?;
    state.step(params, headers, Input::Evidence(Evidence::Recheck))?;
    Ok(matches!(state.word(), Word::Mined(_)) && !state.suspect())
}
```

These three code sketches are compiled as documentation tests \[X\], but remain integration readings \[D\] until a host lane runs them. Full adapters to write next \[D\]:

- `fn app_badge(state: &State) -> BadgeWithVerifyPath`
- `fn overlay_settlement(state: &State) -> SettlementDecision`
- `fn wallet_spend_guard<H: Headers>(state: &mut State, params: &Params, headers: &H) -> Result<bool, CheckError>`

## Envelope assumptions and replay scope

\[D\] The flat #32 envelope has `v: 1`, `kind`, and exactly the fields in [docs/CHAIN-EVENTS.md](docs/CHAIN-EVENTS.md). Unknown versions, kinds, missing fields and extra fields are errors for the host's status surface. A fault produces no tracker input. The captain must reconcile this projection with #32's final wire schema, including the `time` and `tip` mentioned by that lane. No wire compatibility with its emitter is claimed by the local decode tests.

\[X\] The two scenarios of record, the anonymized two-proof production witness, and the four application-contract scenarios replay the tracker portions with in-memory headers and synthetic proofs. The pending-wallet-call scenario tests only tracker inputs, not prompt cancellation or deadline enforcement. Host cache TTLs, ETags, counts, feed delivery, node behavior and beta integrations remain unverified \[D\]. Four property tests are named `mined_implies_checked_proof`, `reorg_reasks_at_or_above`, `hint_changes_no_chain_word`, and `same_evidence_same_word`. The last compares chain words, evidence and suspect marks; hint-tier words may differ, exactly as in Lean.

\[SRC\] A successful `Tracker.recheck` preserves its existing `reask` at `lean/Tracker.lean:328`, including `Spend`; the scenario table says it clears. This crate follows the executable definition. Hosts treat `reask` as a trigger and deduplicate completed work externally; there is no clearing transition invented here. The captain should reconcile that table sentence before integration \[D\]. Heights use the SDK's `u32` and times use `u64`, a finite projection of Lean's naturals. An overflowing age deadline is never treated as due.

## Build and license

```sh
export CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target
export CARGO_BUILD_JOBS=4
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
cargo build --target wasm32-unknown-unknown --features wasm
RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps
```

This crate is licensed under MIT OR Apache-2.0. Version 0.1.0 is an unpublished new API, with no existing stored-state migration or deployed rollback. Adoption is opt-in by each host; stored proofs must be checked on a named read or event. Reverting adoption restores that host's previous adapter, a host-lane rollback \[D\]. No CI is configured.
