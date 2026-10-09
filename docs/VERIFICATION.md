# A6 #30 verification record

[SRC] Specification: bsv-stack-lean@24f22f808b1fb44ce9bf128ec89ac5c8839d9d60, `docs/charters/tracker.md`, `maps/tracker.json`, `lean/Tracker.lean`, `docs/APP-CONTRACT.md` and the seven named scenarios. SDK and ARC vocabulary: bsv-rs@7bc623c, `tests/vectors/arc_tx_status_verdicts.json`; the actual dependency is crates.io `bsv-rs =0.3.35` with only `transaction`, and `wasm` under this crate's `wasm` feature.

[X] New repository, `main`, no base commit, no remote. This is the lane's explicit exception to the shared worktree rule. No host integration or deployment is authorized by this lane.

## Witness before the transitions

[X] The seven tests compile and fail because the transition function is missing. All encounter `Unknown` instead of `Built` or `Mined`; the complete output below records every failure. Fixtures use synthetic two-leaf SDK paths and in-memory active header snapshots, no node or production data.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test --test scenarios
    Updating crates.io index
     Locking 114 packages to latest compatible versions
      Adding generic-array v0.14.7 (available: v0.14.9)
   Compiling libc v0.2.190
   Compiling cfg-if v1.0.5
   Compiling zeroize v1.9.1
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling subtle v2.6.1
   Compiling generic-array v0.14.7
   Compiling const-oid v0.9.6
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.26
   Compiling quote v1.0.47
   Compiling getrandom v0.2.17
   Compiling rand_core v0.6.4
   Compiling block-buffer v0.10.4
   Compiling base64ct v1.8.3
   Compiling cpufeatures v0.2.17
   Compiling crypto-common v0.1.7
   Compiling zerocopy v0.8.62
   Compiling digest v0.10.7
   Compiling pem-rfc7468 v0.7.0
   Compiling syn v3.0.6
   Compiling der v0.7.10
   Compiling ff v0.13.1
   Compiling block-padding v0.3.3
   Compiling base16ct v0.2.0
   Compiling autocfg v1.5.1
   Compiling num-traits v0.2.19
   Compiling spki v0.7.3
   Compiling pkcs8 v0.10.2
   Compiling sec1 v0.7.3
   Compiling inout v0.1.4
   Compiling group v0.13.0
   Compiling hmac v0.12.1
   Compiling crypto-bigint v0.5.5
   Compiling getrandom v0.3.4
   Compiling cipher v0.4.4
   Compiling elliptic-curve v0.13.8
   Compiling serde_core v1.0.229
   Compiling memchr v2.8.3
   Compiling futures-core v0.3.34
   Compiling futures-sink v0.3.34
   Compiling futures-channel v0.3.34
   Compiling futures-macro v0.3.34
   Compiling ppv-lite86 v0.2.21
   Compiling rfc6979 v0.4.0
   Compiling signature v2.2.0
   Compiling universal-hash v0.5.1
   Compiling pin-project-lite v0.2.17
   Compiling futures-task v0.3.34
   Compiling slab v0.4.12
   Compiling zmij v1.0.23
   Compiling opaque-debug v0.3.1
   Compiling futures-io v0.3.34
   Compiling ecdsa v0.16.9
   Compiling polyval v0.6.2
   Compiling futures-util v0.3.34
   Compiling rand_core v0.9.5
   Compiling syn v2.0.119
   Compiling sha2 v0.10.9
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling thiserror v1.0.69
   Compiling ghash v0.5.1
   Compiling num-integer v0.1.47
   Compiling thiserror-impl v1.0.69
   Compiling rand_chacha v0.3.1
   Compiling serde_derive v1.0.229
   Compiling futures-executor v0.3.34
   Compiling primeorder v0.13.6
   Compiling ctr v0.9.2
   Compiling aes v0.8.4
   Compiling aead v0.5.2
   Compiling itoa v1.0.18
   Compiling once_cell v1.21.4
   Compiling k256 v0.13.4
   Compiling aes-gcm v0.10.3
   Compiling p256 v0.13.2
   Compiling futures v0.3.34
   Compiling rand v0.8.8
   Compiling num-bigint v0.4.8
   Compiling async-trait v0.1.92
   Compiling cbc v0.1.2
   Compiling pbkdf2 v0.12.2
   Compiling ripemd v0.1.3
   Compiling sha1 v0.10.7
   Compiling bs58 v0.5.1
   Compiling urlencoding v2.1.3
   Compiling base64 v0.22.1
   Compiling lazy_static v1.5.1
   Compiling hex v0.4.3
   Compiling rand_xorshift v0.4.0
   Compiling rand v0.9.5
   Compiling rand_chacha v0.9.0
   Compiling bsv-rs v0.3.35
   Compiling regex-syntax v0.8.11
   Compiling bitflags v2.13.2
   Compiling unarray v0.1.4
   Compiling proptest v1.11.0
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.15s
     Running tests/scenarios.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/scenarios-a029baa32b53a4d2)

running 7 tests
test contract_3_tracker_word_only_chain_word ... FAILED
test contract_2_pending_wallet_call_tracker_projection ... FAILED
test reorg_announced_nobody_hears_go_server_ts_client_shape ... FAILED
test mined_orphaned_remined_broadcaster_latch ... FAILED
test contract_1_versioned_envelope_fails_loud ... FAILED
test production_proofs_orphaned_healed_without_a_hand ... FAILED
test contract_4_served_proof_tracker_projection ... FAILED

failures:

---- contract_3_tracker_word_only_chain_word stdout ----

thread 'contract_3_tracker_word_only_chain_word' (42999417) panicked at tests/scenarios.rs:98:55:
assertion `left == right` failed
  left: Unknown
 right: Built
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- contract_2_pending_wallet_call_tracker_projection stdout ----

thread 'contract_2_pending_wallet_call_tracker_projection' (42999416) panicked at tests/scenarios.rs:86:56:
assertion `left == right` failed
  left: Unknown
 right: Built

---- reorg_announced_nobody_hears_go_server_ts_client_shape stdout ----

thread 'reorg_announced_nobody_hears_go_server_ts_client_shape' (42999421) panicked at tests/common/mod.rs:43:49:
assertion `left == right` failed
  left: Unknown
 right: Built

---- mined_orphaned_remined_broadcaster_latch stdout ----

thread 'mined_orphaned_remined_broadcaster_latch' (42999419) panicked at tests/common/mod.rs:43:49:
assertion `left == right` failed
  left: Unknown
 right: Built

---- contract_1_versioned_envelope_fails_loud stdout ----

thread 'contract_1_versioned_envelope_fails_loud' (42999415) panicked at tests/common/mod.rs:36:5:
assertion `left == right` failed: word: Unknown
  left: None
 right: Some(100)

---- production_proofs_orphaned_healed_without_a_hand stdout ----

thread 'production_proofs_orphaned_healed_without_a_hand' (42999420) panicked at tests/common/mod.rs:36:5:
assertion `left == right` failed: word: Unknown
  left: None
 right: Some(100)

---- contract_4_served_proof_tracker_projection stdout ----

thread 'contract_4_served_proof_tracker_projection' (42999418) panicked at tests/common/mod.rs:36:5:
assertion `left == right` failed: word: Unknown
  left: None
 right: Some(100)


failures:
    contract_1_versioned_envelope_fails_loud
    contract_2_pending_wallet_call_tracker_projection
    contract_3_tracker_word_only_chain_word
    contract_4_served_proof_tracker_projection
    mined_orphaned_remined_broadcaster_latch
    production_proofs_orphaned_healed_without_a_hand
    reorg_announced_nobody_hears_go_server_ts_client_shape

test result: FAILED. 0 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test scenarios`

exit: 101
```

[X] Witness formatting: `CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo fmt --all`, exit 0, empty output.

[D] The successful `Tracker.recheck` in Lean preserves `reask` (`lean/Tracker.lean:318-328`). The charter and scenario table describe clearing it. The implementation will follow the executable definition and document this discrepancy for the captain.

## Completion, 2026-10-09

[X] Witness commit: `f75c0d3084c8a2ce3f2411239ce60f3148a87830 test: record seven tracker witnesses before the transition machine`. Implementation commit: `4cc6999b249449033622b4496e84b95d042a2c89 feat: derive tracker words from checked proofs and target reasks`. New repository has no parent/base before the witness commit. Work is on the lane-authorized `main`, with no remote. The boundary correction and final runs are recorded below and in the subsequent local commit.

[X] The seven witness tests are now green. The complete final suite is 27 passed, 0 failed, 0 ignored: four property tests, seven scenario tests, eight branch/scheduler/boundary tests, two vocabulary tests and six documentation tests. Of the documentation tests, three reject unchecked construction/deserialization and three compile the host sketches. The ARC fixture has 20 cases and is byte-identical to its pin. Each named property runs 256 deterministic cases. Expected fixture roots and property observations call the SDK directly rather than echoing the tracker's root wrapper.

[X] Required gates: native suite, clippy with warnings denied, fmt check, wasm32 feature build and documentation with RUSTDOCFLAGS=-Dwarnings all exit 0. Environment:

```text
rustc 1.95.0 (59807616e 2026-04-14)
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
```

[X] Dependency feature inspection:

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo tree -e features -i bsv-rs
bsv-rs v0.3.35
├── bsv-rs feature "primitives"
│   └── bsv-rs feature "script"
│       └── bsv-rs feature "transaction"
│           └── bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
│               └── bsv-tracker feature "default" (command-line)
├── bsv-rs feature "script" (*)
└── bsv-rs feature "transaction" (*)
```

[X] A local audit compared the README paragraph with charter section 0 at the stack pin, compared the ARC vector's bytes with bsv-rs@7bc623c, checked Cargo.lock's registry source/version, inspected the normal dependency tree for reqwest/tokio, checked the authored files for an em dash, and ran git diff --check. All passed. The earlier wording search returned no matches (exit 1). No CI, server, node, wallet, deployment, network transport or private-program material was used. The sole external issue read was `gh api repos/Calgooon/bsv-stack-lean/issues/30`; it succeeded and left the issue open.

[D] Unverified: host integrations, real feed delivery and no-feed polling, #32's final wire schema, cache invalidation/TTL/ETags/count attestations, wallet approval deadlines/cancellation, header freshness/validity/active-chain selection, trusted-node production policy, browser execution and JavaScript bindings, and formal Rust-to-Lean refinement. Local tracker projections do not upgrade the corpus's host-replay labels. `Headers` uses the charter's header projection; the SDK does not export a full transaction-module header type.

[D] Next: the captain reconciles #32's fields (including tip time/full headers/tipAge tip) and the successful-recheck trigger sentence; then the app layer, overlay and wallet integration lanes run the three adapters. Proposed signatures: `fn app_badge(state: &State) -> BadgeWithVerifyPath`; `fn overlay_settlement(state: &State) -> SettlementDecision`; `fn wallet_spend_guard<H: Headers>(state: &mut State, params: &Params, headers: &H) -> Result<bool, CheckError>`. Owner ruling: approve the crate name `bsv-tracker` and proposed home `Calhooon/bsv-tracker`. Rejection without a trusted node stays a hint under the current charter; the independent-evidence question remains open.

[D] Semver 0.1.0 is a new unpublished API. Adoption requires the four host traits and explicit input routing. Stored paths are rechecked on named reads/events, bounded by the host's tracked transactions; the crate deserializes no trusted state. Rollback is reverting host adoption, outside this lane. No existing deployment or stored-state migration is changed.

## Validation commands and tails

### Witness formatting

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo fmt --all
(empty output)
```

### Seven witness tests after transitions

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test --test scenarios
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.74s
     Running tests/scenarios.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/scenarios-a029baa32b53a4d2)

running 7 tests
test contract_2_pending_wallet_call_tracker_projection ... ok
test contract_3_tracker_word_only_chain_word ... ok
test contract_1_versioned_envelope_fails_loud ... ok
test contract_4_served_proof_tracker_projection ... ok
test reorg_announced_nobody_hears_go_server_ts_client_shape ... ok
test mined_orphaned_remined_broadcaster_latch ... ok
test production_proofs_orphaned_healed_without_a_hand ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Property/branch compile error, repaired by renaming the local reorg result

[X] Command and tail, exit 101.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test --test properties --test transitions
   ::: tests/common/mod.rs:67:1
    |
 67 | pub fn reorg(height: u32) -> ChainEvent {
    | --------------------------------------- this function of the same name is available here, but it's shadowed by the local binding

error[E0618]: expected function, found `Vec<ChainUpdate>`
   --> tests/transitions.rs:105:38
    |
100 |     let reorg = tracker.on_chain(&h, reorg(3)); assert_eq!(reorg.len(), 2); assert_eq!(h.lookups.get(), 2);
    |         ----- `reorg` has type `Vec<ChainUpdate>`
...
105 |     assert!(tracker.on_chain(&inner, reorg(3)).is_empty());
    |                                      ^^^^^---
    |                                      |
    |                                      call expression requires function
    |
   ::: tests/common/mod.rs:67:1
    |
 67 | pub fn reorg(height: u32) -> ChainEvent {
    | --------------------------------------- this function of the same name is available here, but it's shadowed by the local binding

For more information about this error, try `rustc --explain E0618`.
error: could not compile `bsv-tracker` (test "transitions") due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
```

### Four properties and seven transition witnesses

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test --test properties --test transitions
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.54s
     Running tests/properties.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/properties-de7cb4f8a30b8928)

running 4 tests
test reorg_reasks_at_or_above ... ok
test hint_changes_no_chain_word ... ok
test mined_implies_checked_proof ... ok
test same_evidence_same_word ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.14s

     Running tests/transitions.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/transitions-e7130c28c0d21cbd)

running 7 tests
test malformed_sdk_paths_are_rejected_before_root_reduction ... ok
test age_abandon_and_agreeing_hints_follow_the_lean_table ... ok
test envelopes_round_trip_and_reject_every_unknown_shape ... ok
test fork_tip_recheck_and_invalidation_retain_the_evidence_record ... ok
test proof_checks_fail_closed_and_bind_the_tracked_txid ... ok
test node_and_competitor_evidence_outrank_hints_but_reask_a_mined_word ... ok
test collection_routes_only_affected_heights_headers_and_suspects ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### ARC vector and Arcade vocabulary

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test --test verdicts
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.26s
     Running tests/verdicts.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/verdicts-1dd76d88a07b161d)

running 2 tests
test arcade_twelve_words_and_reorg_markers_remain_hints ... ok
test arc_p0_2_twenty_bodies_are_only_hints ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Formatting before full suite

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo fmt --all
(empty output)
```

### Full suite before lint repair

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test
test collection_routes_only_affected_heights_headers_and_suspects ... ok
test node_and_competitor_evidence_outrank_hints_but_reask_a_mined_word ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/verdicts.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/verdicts-1dd76d88a07b161d)

running 2 tests
test arcade_twelve_words_and_reorg_markers_remain_hints ... ok
test arc_p0_2_twenty_bodies_are_only_hints ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests bsv_tracker

running 6 tests
test src/evidence.rs - evidence::CheckedProof (line 101) - compile fail ... ok
test src/state.rs - state::Mined (line 8) - compile fail ... ok
test src/evidence.rs - evidence::CheckedProof (line 110) - compile fail ... ok
test src/lib.rs - (line 61) ... ok
test src/lib.rs - (line 30) ... ok
test src/lib.rs - (line 46) ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
```

### Clippy collapsible-match finding, repaired by combining the age guard

[X] Command and tail, exit 101.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo clippy --all-targets -- -D warnings
    Checking regex-syntax v0.8.11
    Checking bitflags v2.13.2
    Checking unarray v0.1.4
    Checking bsv-rs v0.3.35
    Checking proptest v1.11.0
    Checking bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
error: this `if` can be collapsed into the outer `match`
   --> src/state.rs:199:17
    |
199 | /                 if self
200 | |                     .last_hint_at
201 | |                     .and_then(|t| t.checked_add(params.age_threshold))
202 | |                     .is_some_and(|deadline| deadline <= now)
203 | |                 {
204 | |                     self.reask = Some(Reask::Age);
205 | |                 }
    | |_________________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.95.0/index.html#collapsible_match
    = note: `-D clippy::collapsible-match` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::collapsible_match)]`

error: could not compile `bsv-tracker` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

### Formatting after lint repair

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo fmt --all
(empty output)
```

### Clippy after lint repair

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo clippy --all-targets -- -D warnings
    Checking bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.71s
```

### Formatting check before wasm

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo fmt --all -- --check
(empty output)
```

### Wasm build before README labels were escaped

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo build --target wasm32-unknown-unknown --features wasm
   Compiling signature v2.2.0
   Compiling rfc6979 v0.4.0
   Compiling universal-hash v0.5.1
   Compiling sha2 v0.10.9
   Compiling polyval v0.6.2
   Compiling aes v0.8.4
   Compiling ghash v0.5.1
   Compiling ctr v0.9.2
   Compiling aead v0.5.2
   Compiling rand_chacha v0.3.1
   Compiling cbc v0.1.2
   Compiling aes-gcm v0.10.3
   Compiling rand v0.8.8
   Compiling pbkdf2 v0.12.2
   Compiling ripemd v0.1.3
   Compiling elliptic-curve v0.13.8
   Compiling ecdsa v0.16.9
   Compiling primeorder v0.13.6
   Compiling sha1 v0.10.7
   Compiling k256 v0.13.4
   Compiling p256 v0.13.2
   Compiling bsv-rs v0.3.35
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 19.45s
```

### Rustdoc house-label link errors, repaired by escaping the labels

[X] Command and tail, exit 101.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps
  --> src/../README.md:84:2
   |
84 | [SRC] A successful `Tracker.recheck` preserves its existing `reask` at `lean/Tracker.lean:328`, including `Spend`; the scenario tabl...
   |  ^^^ no item named `SRC` in scope
   |
   = help: to escape `[` and `]` characters, add '\' before them like `\[` or `\]`

error: unresolved link to `D`
  --> src/../README.md:84:389
   |
84 | ...reconcile that table sentence before integration [D]. Heights use the SDK's `u32` and times use `u64`, a finite projection of Lea...
   |                                                      ^ no item named `D` in scope
   |
   = help: to escape `[` and `]` characters, add '\' before them like `\[` or `\]`

error: unresolved link to `D`
  --> src/../README.md:98:321
   |
98 | ...at host's previous adapter, a host-lane rollback [D]. No CI is configured.
   |                                                      ^ no item named `D` in scope
   |
   = help: to escape `[` and `]` characters, add '\' before them like `\[` or `\]`

error: could not document `bsv-tracker`
```

### Full suite after lint repair

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test
test node_and_competitor_evidence_outrank_hints_but_reask_a_mined_word ... ok
test collection_routes_only_affected_heights_headers_and_suspects ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/verdicts.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/verdicts-1dd76d88a07b161d)

running 2 tests
test arcade_twelve_words_and_reorg_markers_remain_hints ... ok
test arc_p0_2_twenty_bodies_are_only_hints ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests bsv_tracker

running 6 tests
test src/state.rs - state::Mined (line 8) - compile fail ... ok
test src/evidence.rs - evidence::CheckedProof (line 101) - compile fail ... ok
test src/evidence.rs - evidence::CheckedProof (line 110) - compile fail ... ok
test src/lib.rs - (line 46) ... ok
test src/lib.rs - (line 61) ... ok
test src/lib.rs - (line 30) ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s
```

### Rustdoc after label repair

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps
 Documenting bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.71s
   Generated /Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/doc/bsv_tracker/index.html
```

### Formatting of independent SDK-root fixtures

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo fmt --all
(empty output)
```

### Final full native suite

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.18s
     Running unittests src/lib.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/bsv_tracker-25e0a87c7ed6b7bd)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/properties.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/properties-de7cb4f8a30b8928)

running 4 tests
test reorg_reasks_at_or_above ... ok
test hint_changes_no_chain_word ... ok
test same_evidence_same_word ... ok
test mined_implies_checked_proof ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.93s

     Running tests/scenarios.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/scenarios-a029baa32b53a4d2)

running 7 tests
test contract_1_versioned_envelope_fails_loud ... ok
test contract_2_pending_wallet_call_tracker_projection ... ok
test contract_3_tracker_word_only_chain_word ... ok
test contract_4_served_proof_tracker_projection ... ok
test mined_orphaned_remined_broadcaster_latch ... ok
test production_proofs_orphaned_healed_without_a_hand ... ok
test reorg_announced_nobody_hears_go_server_ts_client_shape ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/transitions.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/transitions-e7130c28c0d21cbd)

running 7 tests
test malformed_sdk_paths_are_rejected_before_root_reduction ... ok
test age_abandon_and_agreeing_hints_follow_the_lean_table ... ok
test envelopes_round_trip_and_reject_every_unknown_shape ... ok
test fork_tip_recheck_and_invalidation_retain_the_evidence_record ... ok
test proof_checks_fail_closed_and_bind_the_tracked_txid ... ok
test node_and_competitor_evidence_outrank_hints_but_reask_a_mined_word ... ok
test collection_routes_only_affected_heights_headers_and_suspects ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/verdicts.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/verdicts-1dd76d88a07b161d)

running 2 tests
test arcade_twelve_words_and_reorg_markers_remain_hints ... ok
test arc_p0_2_twenty_bodies_are_only_hints ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests bsv_tracker

running 6 tests
test src/state.rs - state::Mined (line 8) - compile fail ... ok
test src/evidence.rs - evidence::CheckedProof (line 101) - compile fail ... ok
test src/evidence.rs - evidence::CheckedProof (line 110) - compile fail ... ok
test src/lib.rs - (line 61) ... ok
test src/lib.rs - (line 46) ... ok
test src/lib.rs - (line 30) ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
```

### Final clippy

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo clippy --all-targets -- -D warnings
    Checking bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.60s
```

### Final formatting check

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo fmt --all -- --check
(empty output)
```

### Final wasm build

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo build --target wasm32-unknown-unknown --features wasm
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.66s
```

### Rustdoc after evidence wording was scoped

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps
 Documenting bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.96s
   Generated /Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/doc/bsv_tracker/index.html
```

## Snapshot override boundary and final gates, 2026-10-09

[X] A boundary review found that a host could override `Headers::check` with a cached capability. With current headers unavailable, the pre-fix transition wrongly returned `Ok(())` and wrote a mined word. The added witness failed for that reason. Own and competitor transitions now invoke the default `Headers::check` through an internal wrapper forwarding current snapshot lookups; that method remains the only capability constructor. The witness now passes, preserving `Unknown`, returning `CheckError::Headers` and scheduling `ProofFailed(10)` for both kinds of proof.

### Boundary witness before the fix

[X] Command and tail, exit 101.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test --test transitions cached_host_check_cannot_bypass_fail_closed_snapshot_lookup
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.04s
     Running tests/transitions.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/transitions-e7130c28c0d21cbd)

running 1 test
test cached_host_check_cannot_bypass_fail_closed_snapshot_lookup ... FAILED

failures:

---- cached_host_check_cannot_bypass_fail_closed_snapshot_lookup stdout ----

thread 'cached_host_check_cannot_bypass_fail_closed_snapshot_lookup' (43561431) panicked at tests/transitions.rs:287:5:
Ok(())
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    cached_host_check_cannot_bypass_fail_closed_snapshot_lookup

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test transitions`
```

### Boundary witness after the fix

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test --test transitions cached_host_check_cannot_bypass_fail_closed_snapshot_lookup
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.07s
     Running tests/transitions.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/transitions-e7130c28c0d21cbd)

running 1 test
test cached_host_check_cannot_bypass_fail_closed_snapshot_lookup ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
```

### Formatting after the boundary fix

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo fmt --all
(empty output)
```

### Final native suite after the boundary fix

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.69s
     Running unittests src/lib.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/bsv_tracker-25e0a87c7ed6b7bd)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/properties.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/properties-de7cb4f8a30b8928)

running 4 tests
test reorg_reasks_at_or_above ... ok
test hint_changes_no_chain_word ... ok
test same_evidence_same_word ... ok
test mined_implies_checked_proof ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.92s

     Running tests/scenarios.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/scenarios-a029baa32b53a4d2)

running 7 tests
test contract_2_pending_wallet_call_tracker_projection ... ok
test contract_3_tracker_word_only_chain_word ... ok
test contract_1_versioned_envelope_fails_loud ... ok
test reorg_announced_nobody_hears_go_server_ts_client_shape ... ok
test contract_4_served_proof_tracker_projection ... ok
test mined_orphaned_remined_broadcaster_latch ... ok
test production_proofs_orphaned_healed_without_a_hand ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/transitions.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/transitions-e7130c28c0d21cbd)

running 8 tests
test age_abandon_and_agreeing_hints_follow_the_lean_table ... ok
test malformed_sdk_paths_are_rejected_before_root_reduction ... ok
test fork_tip_recheck_and_invalidation_retain_the_evidence_record ... ok
test envelopes_round_trip_and_reject_every_unknown_shape ... ok
test cached_host_check_cannot_bypass_fail_closed_snapshot_lookup ... ok
test proof_checks_fail_closed_and_bind_the_tracked_txid ... ok
test node_and_competitor_evidence_outrank_hints_but_reask_a_mined_word ... ok
test collection_routes_only_affected_heights_headers_and_suspects ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/verdicts.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/verdicts-1dd76d88a07b161d)

running 2 tests
test arcade_twelve_words_and_reorg_markers_remain_hints ... ok
test arc_p0_2_twenty_bodies_are_only_hints ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests bsv_tracker

running 6 tests
test src/state.rs - state::Mined (line 8) - compile fail ... ok
test src/evidence.rs - evidence::CheckedProof (line 101) - compile fail ... ok
test src/evidence.rs - evidence::CheckedProof (line 110) - compile fail ... ok
test src/lib.rs - (line 63) ... ok
test src/lib.rs - (line 32) ... ok
test src/lib.rs - (line 48) ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.76s
```

### Final clippy after the boundary fix

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo clippy --all-targets -- -D warnings
    Checking bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.58s
```

### Final formatting check after the boundary fix

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo fmt --all -- --check
(empty output)
```

### Final wasm build after the boundary fix

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo build --target wasm32-unknown-unknown --features wasm
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.62s
```

### Final rustdoc after the boundary fix

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps
 Documenting bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.65s
   Generated /Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/doc/bsv_tracker/index.html
```

[X] Final total: 27 passed, 0 failed, 0 ignored. This is four properties, seven scenarios, eight branch/scheduler/boundary tests, two vocabulary tests and six documentation tests. The previous 26-test runs above are the recorded implementation history before the additional boundary witness.
