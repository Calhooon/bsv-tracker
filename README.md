# bsv-tracker

**0.9.0-beta.11**, a transaction evidence state machine: a prerelease of the
`beta` line, on **bsv-rs =0.9.0-beta.10**. The stable line is `main`, whose
release 0.8.0 is on bsv-rs 0.8.0. crates.io carries both versions; the
repository's Releases page has a Release for each tag.

The tracker owns re-asking: per transaction it keeps a word, the evidence
behind it, the hints received and the next re-ask. Checked headers and proofs
establish mining and conflicts; broadcaster words only supply hints. A reorg
re-asks only the transactions whose proof's root left the active headers. Hosts provide headers, proofs, hints and time,
so wallet, overlay and application hosts can use the same transition logic
without giving this crate a network client or a periodic chain scan.

## Install

```toml
[dependencies]
bsv-tracker = "=0.9.0-beta.11"
```

The crate needs Rust 1.88 or newer (`rust-version` in `Cargo.toml`). Enable
the `wasm` feature for a wasm host. `Cargo.toml` disables the SDK's default
features and enables transaction types; it adds no HTTP client or async
runtime.

## Evidence and host obligations

`Headers::check` constructs checked proof evidence by binding a merkle path
to a transaction and comparing its root with the active header snapshot.
`CheckedProof` has no other constructor and no deserializer, so stored proofs
must be checked again; deserializing bytes does not create a trusted mined
state. `State::to_compact` and `State::from_compact` (0.5.0) store and restore
one state as versioned bytes for a host that keeps no memory between requests;
a restored `Word::Mined` comes back suspect with `Reask::Recheck` due, and a
`Tracker` that holds it through `Tracker::restore` rechecks it on the next tip. Hosts only serve current inclusion evidence from a non-suspect
`Word::Mined` and calculate current depth from their current tip.

An unavailable header or failed check keeps the word and schedules a re-ask.
Broadcaster rejection and
mempool conflict do not by themselves write `Rejected` or `Conflicted`.
`NodeVerdict` is a separate host-supplied authority boundary; a host must not
relabel a broadcaster's refusal as a local node verdict. The host also supplies
the fact that a proved competitor spends one of the tracked inputs.

The host supplies `Headers`, `ProofFetcher`, `HintSource` and `Clock`, executes
scheduled re-asks and reports decoding and check errors. After a chain event a
host fetches a proof only for a `Reask::Reorg`, the words whose root moved, a
few a pass; the pacing is the host's, and the recheck that decides which words
those are is the crate's. The tracker performs
pure transitions and indexed event handling. It does not fetch proofs, select
the active header chain or run a script evaluator on the host's behalf.

## Broadcaster hints

`Verdict::from_arc_body` reads an ARC body as metadata and never consults the
HTTP status. The `txStatus` word is trimmed and upper-cased before the
vocabulary check; `extraInfo` is text kept on `Verdict::reason` and never
moves the status. A word outside the vocabulary is
`VerdictError::UnknownStatus`. Every hint, `HintStatus::Mined` included, writes no chain
word. `HintStatus::MempoolPresent { source }` (0.5.0) records a courier's
mempool presence, a node the host did not broadcast through holding the bytes.
Since 0.7.0 it moves `built` to `announced`, as an acceptance does: someone
announced the transaction, and the same hint gives the same word whether or
not this host did. It writes no chain word.

## Kinds a later release adds

`HostAction`, `Reask`, `HintStatus`, `VerdictError` and `Verdict` are
`#[non_exhaustive]`: a later minor may add a kind, an error or a field, so a
host's match on one of the four enums carries a wildcard arm, and a host
builds a `Verdict` with `Verdict::new` and assigns its fields after it. That
arm keeps and surfaces what the host does not name. A re-ask of a kind the
host does not name is kept and surfaced by `Reask::label`, never dropped by a
wildcard arm. A hint status is stored by its serde form
(`{"status":"mined","height":900}`) and read back by it, never mapped by hand,
and a status the reader refuses is surfaced, never read as
`HintStatus::Unknown`. Each kind's word (`label`, and `LABELS` for every word
in declaration order, the compact form's tag order) never changes and is
never reused; a later minor only appends.

## The doubts

`bsv_tracker::doubts` holds what a host's header feed has shown it and not
yet settled: the rival branches of its forks and reorgs, its open splits and
its tip. A host folds each feed row into them with `doubts::step`, beside the
tracker's own step, and reads the depth it may trust while they stand with
`doubts::eff_depth`; `doubts::spendable_eff` is the spend guard on that depth.
The fold reads no header, clock or network, and nothing a host already calls
changes its answer. `docs/DOUBTS.md` is the page: every public item beside the
Lean name it mirrors, the stored form, the printed line, the feed's named
number and the snapshot.

```rust
use bsv_rs::transaction::{MerklePath, MerklePathLeaf};
use bsv_tracker::doubts::{self, Bounds, Doubts};
use bsv_tracker::{decode_envelope, Evidence, Header, HeaderError, Headers, Height};
use bsv_tracker::{Input, Params, Proof, State};

// The host's own verified headers: the block at 100, the tip at 105.
struct Verified(Header);
impl Headers for Verified {
    fn header_at(&self, height: Height) -> Result<Option<Header>, HeaderError> {
        Ok((height == 100).then(|| self.0.clone()))
    }
    fn tip_height(&self) -> Result<Height, HeaderError> {
        Ok(105)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A coin of a transaction mined at 100, its proof checked against them.
    let txid = "aa".repeat(32);
    let leaves = vec![MerklePathLeaf::new_txid(0, txid.clone()), MerklePathLeaf::new(1, "bb".repeat(32))];
    let proof = Proof::new(&txid, MerklePath::new(100, vec![leaves])?)?;
    let headers = Verified(Header { hash: "cc".repeat(32), merkle_root: proof.root()? });
    let params = Params::new(3600, 3); // the age threshold in seconds, the spend depth
    let mut coin = State::new(&txid);
    coin.step(&params, &headers, Input::Evidence(Evidence::Proof(proof)))?;

    // The feed splits at 100: two branches past it, each with its witness tip at 101.
    let key = format!("split:{}", "dd".repeat(32));
    let tip = |hash: &str, rung: &str| {
        format!(r#"{{"hash":"{}","height":101,"chainWork":"{}","rungs":["{rung}"]}}"#, hash.repeat(32), "0e".repeat(32))
    };
    let split = format!(
        r#"{{"v":2,"kind":"split","key":"{key}","forkHeight":100,"tips":[{},{}]}}"#,
        tip("e1", "a"),
        tip("e2", "b")
    );
    let resolved = format!(r#"{{"v":2,"kind":"splitResolved","key":"{key}","forkHeight":100}}"#);

    let mut ds = Doubts::cold(105);
    ds = doubts::step(&Bounds::B400, &decode_envelope(split.as_bytes())?, &ds);
    // An open split at or below the coin's height: nothing there is deep.
    assert_eq!(doubts::eff_depth(&headers, &ds, 100)?, 0);
    assert!(!doubts::spendable_eff(&params, &headers, &ds, None, &coin)?);

    ds = doubts::step(&Bounds::B400, &decode_envelope(resolved.as_bytes())?, &ds);
    // Resolved: the rival branch's tip at 101 stays a doubt, four blocks behind.
    assert_eq!(ds.to_string(), "tip=105 splits=[] rivals=[100:101] summary=-");
    assert_eq!(doubts::eff_depth(&headers, &ds, 100)?, 4);
    assert!(doubts::spendable_eff(&params, &headers, &ds, None, &coin)?);
    Ok(())
}
```

## Events and records

`docs/SPEC.md` records the specification relationship and
`docs/CHAIN-EVENTS.md` defines the event envelope. `ChainEvent` carries the
six kinds of envelope version 1 (tip, fork, reorg, invalidated, frozen, tip
age) and the two kinds of version 2 (`split`, `splitResolved`); each kind
selects its version. The reorg rule is by the root (0.7.0): a `mined` word is
`stale` when, and only when, its stored proof's root is not the active
header's root at its height in the host's headers. A reorg selects the mined
words at or above its fork height and an invalidation the words checked
against its block; each selected word is rechecked against the host's headers,
a header read and never a fetch. The same root, the word stands and its marks
clear; a moved root, `Word::Stale` with `Reask::Reorg`; no header, the word
stands suspect with `Reask::Recheck`. A split marks the mined words at or above its fork height
suspect and writes no word; a resolved split rechecks them against the host's
headers. `decode_envelope` returns `UnknownVersion`, `UnknownKind` or
`UnknownShape` for an envelope it does not know, never a silent tracker input.
`docs/VERIFICATION.md` records scoped witnesses; `tests` contains chain-event
and ARC vectors, the Arcade bodies, and transition, split and property tests.
Host integration and live transport behavior require their own evidence.

## Build and gate

```sh
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc --locked --no-deps
cargo build --locked --target wasm32-unknown-unknown --features wasm
```

`.github/workflows/ci.yml` runs `test`, `fmt`, `clippy`, `vectors`, `wasm32`
and `locks`, the six checks each line requires, on every PR and push to `main`
and `beta`, with `msrv`, `machete` (non-blocking), `secrets` and `actionlint`
beside them; `.github/workflows/lines.yml` runs the direction rule's check of
each line, `main-in-beta` on `beta` and `beta-born` on `main` (every change is
born on `beta`, `RELEASING.md`). The
`vectors` job runs the eight chain-event envelope vectors and the ARC status
vector by name and fails unless each one executes. Dependabot moves the
stack's crates daily at 18:00Z under `.github/dependabot.yml`. `RELEASING.md`
defines tag-based crates.io trusted publishing; `CHANGELOG.md` records
changes. This is a published library, with host adoption controlled
separately.

Licensed under MIT OR Apache-2.0, as declared in `Cargo.toml`.
