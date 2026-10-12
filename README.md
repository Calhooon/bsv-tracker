# bsv-tracker

**0.2.2**, a transaction evidence state machine on **bsv-rs =0.5.0**.
The tracker owns re-asking: per transaction it keeps a word, the evidence
behind it, the hints received and the next re-ask. Checked headers and proofs
establish mining and conflicts; broadcaster words only supply hints. A reorg
re-asks affected transactions. Hosts provide headers, proofs, hints and time,
so wallet, overlay and application hosts can use the same transition logic
without giving this crate a network client or a periodic chain scan.

## Install

```toml
[dependencies]
bsv-tracker = "=0.2.2"
```

Enable the `wasm` feature for a wasm host. [Cargo.toml](Cargo.toml) disables
the SDK's default features and enables transaction types; it adds no HTTP
client or async runtime.

## Evidence and host obligations

`Headers::check` constructs checked proof evidence by binding a merkle path
to a transaction and comparing its root with the active header snapshot.
Stored proofs must be checked again; deserializing bytes does not create a
trusted mined state. Hosts only serve current inclusion evidence from a
non-suspect `Word::Mined` and calculate current depth from their current tip.

An unavailable header or failed check keeps the word and schedules a re-ask.
Reorg evidence marks affected inclusion stale. Broadcaster rejection and
mempool conflict do not by themselves write `Rejected` or `Conflicted`.
`NodeVerdict` is a separate host-supplied authority boundary; a host must not
relabel a broadcaster's refusal as a local node verdict. The host also supplies
the fact that a proved competitor spends one of the tracked inputs.

The host supplies `Headers`, `ProofFetcher`, `HintSource` and `Clock`, executes
scheduled re-asks and reports decoding/check errors. The tracker performs
pure transitions and indexed event handling. It does not fetch proofs, select
the active header chain or run a script evaluator on the host's behalf.

## Events and records

[docs/SPEC.md](docs/SPEC.md) records the specification relationship and
[docs/CHAIN-EVENTS.md](docs/CHAIN-EVENTS.md) defines the event envelope.
Unknown envelope versions, kinds or shapes are errors, not silent tracker
inputs. [docs/VERIFICATION.md](docs/VERIFICATION.md) records scoped witnesses;
[tests](./tests) contains chain-event vectors and transition tests.
Host integration and live transport behavior require their own evidence.

## Build and gate

```sh
cargo test --locked
cargo fmt --all -- --check
cargo build --locked --target wasm32-unknown-unknown --features wasm
```

[.github/workflows/ci.yml](.github/workflows/ci.yml) runs `test`, `fmt`, `clippy`,
`vectors` and `wasm32` on every PR and main push. The vector job requires the
named chain-event and broadcaster-word witnesses to actually execute.
[RELEASING.md](RELEASING.md) defines tag-based crates.io trusted publishing;
[CHANGELOG.md](CHANGELOG.md) records changes. This is a published library,
with host adoption controlled separately.

Licensed under MIT OR Apache-2.0, as declared in [Cargo.toml](Cargo.toml).
