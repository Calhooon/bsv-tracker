# bsv-tracker

**0.4.1**, a transaction evidence state machine on **bsv-rs ^0.6.0**. The tag
`v0.4.1` is the release of record, and crates.io carries the same version,
0.4.1; the repository's Releases page has a Release for each tag.

The tracker owns re-asking: per transaction it keeps a word, the evidence
behind it, the hints received and the next re-ask. Checked headers and proofs
establish mining and conflicts; broadcaster words only supply hints. A reorg
re-asks affected transactions. Hosts provide headers, proofs, hints and time,
so wallet, overlay and application hosts can use the same transition logic
without giving this crate a network client or a periodic chain scan.

## Install

```toml
[dependencies]
bsv-tracker = "0.4.1"
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
state. Hosts only serve current inclusion evidence from a non-suspect
`Word::Mined` and calculate current depth from their current tip.

An unavailable header or failed check keeps the word and schedules a re-ask.
Reorg evidence marks affected inclusion stale. Broadcaster rejection and
mempool conflict do not by themselves write `Rejected` or `Conflicted`.
`NodeVerdict` is a separate host-supplied authority boundary; a host must not
relabel a broadcaster's refusal as a local node verdict. The host also supplies
the fact that a proved competitor spends one of the tracked inputs.

The host supplies `Headers`, `ProofFetcher`, `HintSource` and `Clock`, executes
scheduled re-asks and reports decoding and check errors. The tracker performs
pure transitions and indexed event handling. It does not fetch proofs, select
the active header chain or run a script evaluator on the host's behalf.

## Broadcaster hints

`Verdict::from_arc_body` reads an ARC body as metadata and never consults the
HTTP status. The `txStatus` word is trimmed and upper-cased before the
vocabulary check; `extraInfo` is text kept on `Verdict::reason` and never
moves the status. A word outside the vocabulary is
`VerdictError::UnknownStatus`. Every hint, `HintStatus::Mined` included, writes no chain
word.

## Events and records

`docs/SPEC.md` records the specification relationship and
`docs/CHAIN-EVENTS.md` defines the event envelope. `ChainEvent` carries the
six kinds of envelope version 1 (tip, fork, reorg, invalidated, frozen, tip
age) and the two kinds of version 2 (`split`, `splitResolved`); each kind
selects its version. A split marks the mined words at or above its fork height
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

`.github/workflows/ci.yml` runs `test`, `fmt`, `clippy`, `vectors` and
`wasm32`, the five checks `main` requires, on every PR and main push, with
`msrv`, `machete` (non-blocking), `secrets` and `actionlint` beside them. The
`vectors` job runs the eight chain-event envelope vectors and the ARC status
vector by name and fails unless each one executes. Dependabot moves the
stack's crates daily at 18:00Z under `.github/dependabot.yml`. `RELEASING.md`
defines tag-based crates.io trusted publishing; `CHANGELOG.md` records
changes. This is a published library, with host adoption controlled
separately.

Licensed under MIT OR Apache-2.0, as declared in `Cargo.toml`.
