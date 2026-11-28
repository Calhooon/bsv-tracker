# A6 #30 verification record

[SRC] Initial specification: bsv-stack-lean@24f22f808b1fb44ce9bf128ec89ac5c8839d9d60, `docs/charters/tracker.md`, `maps/tracker.json`, `lean/Tracker.lean`, `docs/APP-CONTRACT.md` and the seven named scenarios. Final machine pin: `f95520fcd85e066ec341faffa9dd8f1f8f35a37e`, including the landed #31 successful-recheck correction recorded at the end. SDK and ARC vocabulary: bsv-rs@7bc623c, `tests/vectors/arc_tx_status_verdicts.json`; the actual dependency is crates.io `bsv-rs` at the caret requirement `0.4.1` (the tracked `Cargo.lock` holds 0.4.1; the 0.3 line left at bsv-tracker 0.2.0) with only `transaction`, and `wasm` under this crate's `wasm` feature.

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

[D] At the initial specification pin, successful `Tracker.recheck` preserved `reask` (`lean/Tracker.lean:318-328`), while the charter and scenario table described clearing it. The implementation followed that definition at the time. [SRC] The landed #31 machine resolves this at the final pin, and the final crate follows it; see the final run section.

## Completion, 2026-10-09

[X] Witness commit: `f75c0d3084c8a2ce3f2411239ce60f3148a87830 test: record seven tracker witnesses before the transition machine`. Implementation commit: `4cc6999b249449033622b4496e84b95d042a2c89 feat: derive tracker words from checked proofs and target reasks`. New repository has no parent/base before the witness commit. Work is on the lane-authorized `main`, with no remote. The boundary correction and final runs are recorded below and in the subsequent local commit.

[X] The seven witness tests are now green. The complete final suite is 28 passed, 0 failed, 0 ignored: four property tests, seven scenario tests, nine branch/scheduler/boundary tests, two vocabulary tests and six documentation tests. Of the documentation tests, three reject unchecked construction/deserialization and three compile the host sketches. The ARC fixture has 20 cases and is byte-identical to its pin. Each named property runs 256 deterministic cases. Expected fixture roots and property observations call the SDK directly rather than echoing the tracker's root wrapper.

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

[D] Next: the captain reconciles #32's fields (including tip time/full headers/tipAge tip); then the app layer, overlay and wallet integration lanes run the three adapters. Proposed signatures: `fn app_badge(state: &State) -> BadgeWithVerifyPath`; `fn overlay_settlement(state: &State) -> SettlementDecision`; `fn wallet_spend_guard<H: Headers>(state: &mut State, params: &Params, headers: &H) -> Result<bool, CheckError>`. Owner ruling: approve the crate name `bsv-tracker` and proposed home `Calhooon/bsv-tracker`. The production independent-evidence policy and the new #31 spendability predicate remain host-lane questions.

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

[X] Boundary-fix total: 27 passed, 0 failed, 0 ignored. This is four properties, seven scenarios, eight branch/scheduler/boundary tests, two vocabulary tests and six documentation tests. The previous 26-test runs above are the recorded implementation history before the additional boundary witness.

## Landed #31 recheck correction and final gates, 2026-10-09

[SRC] While the lane ran, the stack advanced from the initial pin to `f95520fcd85e066ec341faffa9dd8f1f8f35a37e`. The only change to the transition's behavior is at `lean/Tracker.lean:340`: a matching recheck clears the completed `reask` as well as `suspect`. Charter sections 1 to 6 and the lane brief are unchanged. The added spendability theorem and its depth/application-rule predicate are outside this lane's host integration scope [D]. The scheduling projection exposes the age threshold; the host supplies its confirmation-depth policy separately.

[X] The new `successful_recheck_clears_completed_spend_and_fork_triggers` witness failed on the old Rust reduction with `Some(Spend)` instead of `None`. The fix clears the trigger on a matching root. The successful spend branch of the scenario of record and the successful suspect-tip branch now expect no pending trigger. An unavailable header still returns a fault and keeps the pending recheck; a stale word still keeps its spend trigger when there is no mined proof to recheck. The retained evidence is unchanged.

### Landed-rule witness before the fix

[X] Command and tail, exit 101.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test --test transitions successful_recheck_clears_completed_spend_and_fork_triggers
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.05s
     Running tests/transitions.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/transitions-e7130c28c0d21cbd)

running 1 test
test successful_recheck_clears_completed_spend_and_fork_triggers ... FAILED

failures:

---- successful_recheck_clears_completed_spend_and_fork_triggers stdout ----

thread 'successful_recheck_clears_completed_spend_and_fork_triggers' (43686409) panicked at tests/transitions.rs:185:5:
assertion `left == right` failed
  left: Some(Spend)
 right: None
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    successful_recheck_clears_completed_spend_and_fork_triggers

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test transitions`
```

### Landed-rule witness after the fix

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test --test transitions successful_recheck_clears_completed_spend_and_fork_triggers
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.19s
     Running tests/transitions.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/transitions-e7130c28c0d21cbd)

running 1 test
test successful_recheck_clears_completed_spend_and_fork_triggers ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s
```

### Formatting after the landed-rule fix

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo fmt --all
(empty output)
```

### Final native suite at the final stack pin

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo test
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.58s
     Running unittests src/lib.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/bsv_tracker-25e0a87c7ed6b7bd)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/properties.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/properties-de7cb4f8a30b8928)

running 4 tests
test reorg_reasks_at_or_above ... ok
test hint_changes_no_chain_word ... ok
test same_evidence_same_word ... ok
test mined_implies_checked_proof ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.94s

     Running tests/scenarios.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/scenarios-a029baa32b53a4d2)

running 7 tests
test contract_2_pending_wallet_call_tracker_projection ... ok
test contract_3_tracker_word_only_chain_word ... ok
test contract_4_served_proof_tracker_projection ... ok
test contract_1_versioned_envelope_fails_loud ... ok
test mined_orphaned_remined_broadcaster_latch ... ok
test reorg_announced_nobody_hears_go_server_ts_client_shape ... ok
test production_proofs_orphaned_healed_without_a_hand ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/transitions.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/debug/deps/transitions-e7130c28c0d21cbd)

running 9 tests
test age_abandon_and_agreeing_hints_follow_the_lean_table ... ok
test successful_recheck_clears_completed_spend_and_fork_triggers ... ok
test fork_tip_recheck_and_invalidation_retain_the_evidence_record ... ok
test envelopes_round_trip_and_reject_every_unknown_shape ... ok
test cached_host_check_cannot_bypass_fail_closed_snapshot_lookup ... ok
test malformed_sdk_paths_are_rejected_before_root_reduction ... ok
test node_and_competitor_evidence_outrank_hints_but_reask_a_mined_word ... ok
test collection_routes_only_affected_heights_headers_and_suspects ... ok
test proof_checks_fail_closed_and_bind_the_tracked_txid ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

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
test src/lib.rs - (line 32) ... ok
test src/lib.rs - (line 48) ... ok
test src/lib.rs - (line 63) ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
```

### Final clippy at the final stack pin

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo clippy --all-targets -- -D warnings
    Checking bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.58s
```

### Final formatting check at the final stack pin

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo fmt --all -- --check
(empty output)
```

### Final wasm build at the final stack pin

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 cargo build --target wasm32-unknown-unknown --features wasm
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.68s
```

### Final rustdoc at the final stack pin

[X] Command and tail, exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-target CARGO_BUILD_JOBS=4 RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps
 Documenting bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/bsv-tracker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.67s
   Generated /Users/johncalhoun/bsv/targets/a6/bsv-tracker-target/doc/bsv_tracker/index.html
```

[X] Total at base `60cc768`: 28 passed, 0 failed, 0 ignored: four properties, seven scenarios, nine transition/scheduler/boundary tests, two vocabulary tests and six documentation tests. All five required gates exit 0. The older 26/27-test runs are history; the envelope reconciliation below supersedes this base count.


## Envelope reconciliation (2026-10-09)

[X] Repository `/Users/johncalhoun/bsv/bsv-tracker`, no remote. Base `60cc7686b1ed748afa591cdc4094ee2f9e38cebd`; branch `a6-30/envelope-reconcile`; worktree `/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope`. This section is committed with the implementation that follows witness commit `dcc70f8` (`test: pin six emitter envelopes to expose tracker schema drift`). Main remains at the base; no push, PR, stash, transport, wallet operation or sub-agent was used.

[SRC] Both reference files were read only through `git -C /Users/johncalhoun/bsv/rust-chaintracks show a62f9ed:<path>`. Schema: `docs/CHAIN-EVENTS.md:13-35`; exact H/outpoint/event widths: `src/events.rs:13-85`; shape checks: `src/events.rs:125-185`; host fault rule: `docs/CHAIN-EVENTS.md:200-206`; six examples: `docs/CHAIN-EVENTS.md:265,269,273,277,281,285`. The examples match the stack's `docs/p0/a3-32-chain-event-envelope.md` copy byte for byte, and the fixture files preserve them with a trailing newline.

[X] The six example witnesses fail before implementation: five have unknown fields or header/outpoint maps where the old projection expects strings; invalidated fails direct ChainEvent serde on the previously unsupported `v`. After implementation, all six parse and round-trip through ChainEvent, ChainEnvelope and decode_envelope. Invalidated/frozen also match example bytes; header-bearing cases compare JSON values because object key order is not a schema rule. Four additional tests cover typed future-kind/version faults, malformed known shapes/legacy fields, inclusive reorg routing with historical evidence, and a tip payload that cannot bypass a failed checked snapshot.

[X] Base suite 28/28; final suite 38/38 (ten envelope tests, four properties at 256 cases each, seven scenarios, nine transition/boundary tests, two vocabulary tests and six doc tests). All final gates exit 0. The transient rustdoc failure was three unescaped [SRC] labels in API documentation, corrected below; no machine behavior changed in that repair.

[X] `src/evidence.rs`, `src/state.rs`, `src/tracker.rs`, Cargo.toml and Cargo.lock are byte-identical to the base. The decoder accepts the pinned shape, including required H fields and exactly two competing headers; it validates relationships and returns UnknownKind/UnknownVersion/UnknownShape instead of an event on a fault. Headers::check still supplies checked inclusion evidence. Reorg routing uses forkHeight, inclusive of the deactivated headers' heights, while retaining the original checked proof record. The 0.1.0 changelog records the correction; complete field changes and caller migration are in docs/CHAIN-EVENTS.md.

[D] This is an unpublished 0.1.0 Rust/accepted-shape API correction; after publication it would require a minor release. No stored-state migration is involved: old proofs still re-enter through targeted checking, bounded by tracked rows. Risk before shipping: hosts must report a returned fault and preserve its unacknowledged cursor; the crate has no transport to enforce that behavior. Live emitter delivery, cursor persistence, host adapters and browser bindings remain unverified. Next: local branch review, then the app badge, overlay settlement and wallet spend adapters. Rollback: return a host to its previous adapter, keeping the old hash-only projection disconnected from this emitter.

### Exact validation commands and tails

[X] Every run below used the worktree and the explicit target directory with four build jobs. Logs live alongside that target directory, named `bsv-tracker-envelope-target-<run>.log`.


### baseline

[X] Exit 0. Total: 28 passed, 0 failed, 0 ignored.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo test --workspace

running 6 tests
test src/state.rs - state::Mined (line 8) - compile fail ... ok
test src/evidence.rs - evidence::CheckedProof (line 110) - compile fail ... ok
test src/evidence.rs - evidence::CheckedProof (line 101) - compile fail ... ok
test src/lib.rs - (line 32) ... ok
test src/lib.rs - (line 63) ... ok
test src/lib.rs - (line 48) ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
```


### red

[X] Exit 101. Total: 0 passed, 6 failed, 0 ignored.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo test --test chain_events
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.50s
     Running tests/chain_events.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target/debug/deps/chain_events-422d103fce1076d1)

running 6 tests
test pinned_fork_round_trip ... FAILED
test pinned_tip_round_trip ... FAILED
test pinned_reorg_round_trip ... FAILED
test pinned_frozen_round_trip ... FAILED
test pinned_tip_age_round_trip ... FAILED
test pinned_invalidated_round_trip ... FAILED

failures:

---- pinned_fork_round_trip stdout ----

thread 'pinned_fork_round_trip' (43951397) panicked at tests/chain_events.rs:8:50:
the pinned envelope must decode: UnknownShape { version: Some(1), detail: "invalid type: map, expected a string" }

---- pinned_tip_round_trip stdout ----

thread 'pinned_tip_round_trip' (43951402) panicked at tests/chain_events.rs:8:50:
the pinned envelope must decode: UnknownShape { version: Some(1), detail: "unknown field `header`, expected `height` or `hash`" }

---- pinned_reorg_round_trip stdout ----

thread 'pinned_reorg_round_trip' (43951400) panicked at tests/chain_events.rs:8:50:
the pinned envelope must decode: UnknownShape { version: Some(1), detail: "unknown field `deactivatedHeaders`, expected one of `forkHeight`, `depth`, `deactivated`, `newTip`" }

---- pinned_frozen_round_trip stdout ----

thread 'pinned_frozen_round_trip' (43951398) panicked at tests/chain_events.rs:8:50:
the pinned envelope must decode: UnknownShape { version: Some(1), detail: "invalid type: map, expected a string" }

---- pinned_tip_age_round_trip stdout ----

thread 'pinned_tip_age_round_trip' (43951401) panicked at tests/chain_events.rs:8:50:
the pinned envelope must decode: UnknownShape { version: Some(1), detail: "unknown field `tip`, expected `seconds`" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- pinned_invalidated_round_trip stdout ----

thread 'pinned_invalidated_round_trip' (43951399) panicked at tests/chain_events.rs:9:57:
ChainEvent serde must gate v:1: Error("unknown field `v`, expected `blockHash`", line: 0, column: 0)


failures:
    pinned_fork_round_trip
    pinned_frozen_round_trip
    pinned_invalidated_round_trip
    pinned_reorg_round_trip
    pinned_tip_age_round_trip
    pinned_tip_round_trip

test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test chain_events`
```


### green-examples

[X] Exit 0. Total: 6 passed, 0 failed, 0 ignored.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo test --test chain_events
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.13s
     Running tests/chain_events.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target/debug/deps/chain_events-422d103fce1076d1)

running 6 tests
test pinned_invalidated_round_trip ... ok
test pinned_frozen_round_trip ... ok
test pinned_tip_age_round_trip ... ok
test pinned_fork_round_trip ... ok
test pinned_reorg_round_trip ... ok
test pinned_tip_round_trip ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```


### fmt-apply

[X] Exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo fmt --all
(empty output)
```


### green

[X] Exit 0. Total: 10 passed, 0 failed, 0 ignored.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo test --test chain_events
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.45s
     Running tests/chain_events.rs (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target/debug/deps/chain_events-422d103fce1076d1)

running 10 tests
test pinned_invalidated_round_trip ... ok
test pinned_frozen_round_trip ... ok
test pinned_tip_age_round_trip ... ok
test pinned_reorg_round_trip ... ok
test future_kind_and_version_are_typed_host_faults ... ok
test pinned_tip_round_trip ... ok
test pinned_fork_round_trip ... ok
test pinned_tip_header_cannot_bypass_the_checked_snapshot ... ok
test pinned_reorg_keeps_the_inclusive_height_routing_and_evidence_record ... ok
test known_shapes_refuse_legacy_fields_and_malformed_headers ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```


### fmt-apply-final

[X] Exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo fmt --all
(empty output)
```


### test

[X] Exit 0. Total: 38 passed, 0 failed, 0 ignored.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo test --workspace

running 6 tests
test src/state.rs - state::Mined (line 8) - compile fail ... ok
test src/evidence.rs - evidence::CheckedProof (line 101) - compile fail ... ok
test src/evidence.rs - evidence::CheckedProof (line 110) - compile fail ... ok
test src/lib.rs - (line 32) ... ok
test src/lib.rs - (line 63) ... ok
test src/lib.rs - (line 48) ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
```


### clippy

[X] Exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo clippy --all-targets -- -D warnings
    Checking proptest v1.11.0
    Checking bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.32s
```


### fmt

[X] Exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo fmt --all -- --check
(empty output)
```


### wasm

[X] Exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo build --target wasm32-unknown-unknown --features wasm
   Compiling bsv-rs v0.3.35
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 19.75s
```


### doc-failed

[X] Exit 101.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps
 Documenting bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope)
error: unresolved link to `SRC`
 --> src/events.rs:7:6
  |
7 | /// [SRC] rust-chaintracks@a62f9ed docs/CHAIN-EVENTS.md:13-23;
  |      ^^^ no item named `SRC` in scope
  |
  = help: to escape `[` and `]` characters, add '\' before them like `\[` or `\]`
  = note: `-D rustdoc::broken-intra-doc-links` implied by `-D warnings`
  = help: to override `-D warnings` add `#[allow(rustdoc::broken_intra_doc_links)]`

error: unresolved link to `SRC`
  --> src/events.rs:25:6
   |
25 | /// [SRC] rust-chaintracks@a62f9ed src/events.rs:43-48.
   |      ^^^ no item named `SRC` in scope
   |
   = help: to escape `[` and `]` characters, add '\' before them like `\[` or `\]`

error: unresolved link to `SRC`
  --> src/events.rs:34:6
   |
34 | /// [SRC] rust-chaintracks@a62f9ed docs/CHAIN-EVENTS.md:24-35.
   |      ^^^ no item named `SRC` in scope
   |
   = help: to escape `[` and `]` characters, add '\' before them like `\[` or `\]`

error: could not document `bsv-tracker`
```


### doc

[X] Exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps
 Documenting bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.53s
   Generated /Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target/doc/bsv_tracker/index.html
```


### final-test

[X] Exit 0. Total: 38 passed, 0 failed, 0 ignored.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo test --workspace

running 6 tests
test src/evidence.rs - evidence::CheckedProof (line 110) - compile fail ... ok
test src/state.rs - state::Mined (line 8) - compile fail ... ok
test src/evidence.rs - evidence::CheckedProof (line 101) - compile fail ... ok
test src/lib.rs - (line 48) ... ok
test src/lib.rs - (line 63) ... ok
test src/lib.rs - (line 32) ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
```


### final-clippy

[X] Exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo clippy --all-targets -- -D warnings
    Checking bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.58s
```


### final-fmt

[X] Exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo fmt --all -- --check
(empty output)
```


### final-wasm

[X] Exit 0.

```text
$ CARGO_TARGET_DIR=/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope-target CARGO_BUILD_JOBS=4 cargo build --target wasm32-unknown-unknown --features wasm
   Compiling bsv-tracker v0.1.0 (/Users/johncalhoun/bsv/targets/a6/bsv-tracker-envelope)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.43s
```

## Envelope version 2: `split` and `splitResolved` (2026-10-17)

[X] Worktree `/Volumes/Lanes/worktrees/bsv-tracker/tracker-v2`, branch `lane/tracker-v2`, base `5f5bd94`; Calhooon/bsv-tracker #11. No push, PR, tag, publish or stash.

[SRC] The emitter was read only through `git -C /Volumes/Lanes/targets/l2/ct-public-v0.4.1 show v0.4.1:<path>`: `src/events.rs:50-57` (`SplitTip`), `95-107` (the two variants), `130-146` (the version gate), `157-161,210-232` (the relationship checks); `docs/SPLITS.md:68-101`; `docs/CHAIN-EVENTS.md:1-8,204-221`; `migrations/0010_rung_tips.sql:63-74,82-87` (the emitted key order). The state rule's source is `bsv-stack-lean docs/RULINGS-2026-10-12.md:323-329`.

| change | witness | result |
|---|---|---|
| `ChainEvent::Split`, `ChainEvent::SplitResolved`, `SplitTip`, `ChainEvent::version()`; serialization carries the kind's version | `pinned_split_round_trip`, `pinned_split_resolved_round_trip` (byte equal) | pass |
| gate: `v: 1` six kinds, `v: 2` two split kinds; other `v` `UnknownVersion`, crossed kinds `UnknownKind` | `future_kind_and_version_are_typed_host_faults`, `version_one_kinds_under_version_two_are_refused_with_valid_bodies` | pass |
| the emitter's split relationship checks | `split_shapes_mirror_the_emitter_relationship_checks` (41 refused shapes, the tip-at-`forkHeight` boundary admitted) | pass |
| split: suspect plus `Reask::Fork { height: forkHeight }` at or above, no word moves | `split_marks_a_mined_word_suspect_and_never_stale`, `split_writes_no_word` (256 ChaCha traces, emptied snapshot) | pass |
| splitResolved: recheck of suspect words at or above through the snapshot | `split_resolved_rechecks_through_the_snapshot` (clear, stale, unavailable, untouched) | pass |
| tracker routing `at_or_above(forkHeight)` for both kinds | `tracker_routes_splits_to_the_mined_range_at_or_above` | pass |
| the six version-1 kinds unchanged | the six `pinned_*_round_trip` and every earlier test, unmodified except three assertions that named `v: 2` as an unknown version, now `v: 3` | pass |

[X] Gate, each exit 0, logs under `/Volumes/Lanes/out/bsv-tracker/tracker-v2/`: `cargo fmt --check`; `cargo +1.99.0 clippy --all-targets -- -D warnings` (and the default stable); `cargo test`, 43 passed (chain_events 14, properties 4, scenarios 7, splits 4, transitions 9, verdicts 2, doc tests 3); `cargo +1.88.0 check`; `cargo build --target wasm32-unknown-unknown --features wasm`; `actionlint .github/workflows/ci.yml`. The `vectors` job's command, run locally: `test result: ok. 8 passed`.

[D] The two split vectors are synthetic in the documented shape: the emitter publishes no literal example and no beta envelope was captured. The decoder refuses a version-1 kind under `v: 2`, which the emitter's own reader admits; the emitter never writes one. The Lean machine has no split input, so the rule is a projection of the ruling, unproved in Lean. Unverified: live delivery from the beta, a host adapter, the stack's corpus scenario for the rule.

## The doubts' test of record: the journals' states after every row (#51, 2026-11-28)

[X] Worktree `bsv-tracker-trk-51-2`, branch `lane/trk-51-2`, base `3e792c1` (beta, with the doubts' state and reducer of #51 landed). The test is `tests/journal_states.rs`; the reducer is `bsv_tracker::doubts`, unchanged by this section.

[SRC] The stack: bsv-stack-lean at `7213ef4a`, `lean/Headers/Doubts.lean` (`raw`, `prune`, `step`, `run`) and the journal driver `tools/journal/JournalRun.lean`, whose line the states files hold: `<cursor> tip=<t> splits=[..] rivals=[..] summary=<fork:tip or ->`. The totals are those of `docs/DECISIONS-2026-10-19.md` 28.22: production 6,254 rows, the beta 39,686 rows, eleven of them unreadable.

The journals and the states files are private to the estate and nothing of them is committed: no row, no states line, no cursor, hash or height, no fixture cut from them. The test reads them in place from `BSV_TRACKER_JOURNALS_DIR` (`production.events.jsonl`, `beta.events.jsonl`) and `BSV_TRACKER_JOURNAL_STATES_DIR` (`production.states.txt`, `beta.states.txt`).

What the test does, for each journal: from `Doubts::cold(0)` under `Bounds::new(400)`, each row (an object of exactly `cursor`, a string of digits, and `event`) is decoded by the crate's own `decode_envelope`, folded by `doubts::step`, and compared with the next states line two ways: the ruled comparison (the cursor, the tip, the splits in order, the rivals as a set with no pair twice, the summary) and the line byte for byte against `format!("{cursor} {ds}")`, which pins the rivals' order. A delivered `unreadable` row is not an envelope; it is counted and skipped, as the driver skips it, and consumes no states line. A decoder refusal, a kind the driver writes no line for, a states line too many or too few, or a total other than the stack's fails the test. A failure names the journal, the line's number and the names of the fields that differ (`cursor`, `tip`, `splits`, `rivals`, `summary`, `line`), never a value. `a_wrong_line_is_named_by_its_fields` pins that naming on a hand-built `Doubts` and hand-written lines, with nothing of a journal.

With `BSV_TRACKER_JOURNALS_DIR` unset (CI's run) the test prints its `SKIPPED` line and passes; with the journals named and `BSV_TRACKER_JOURNAL_STATES_DIR` unset it fails, so a half-named run never reads as green.

[X] The run of record, on the host that mounts both folders (the SDK of rule 10 named for the link):

```text
$ BSV_TRACKER_JOURNALS_DIR=<journals> BSV_TRACKER_JOURNAL_STATES_DIR=<states> \
    cargo test --locked --test journal_states -- --nocapture
running 2 tests
test a_wrong_line_is_named_by_its_fields ... ok
JOURNAL STATES production: rows 6254, unreadable skipped 0, folded 6254, states lines compared 6254, equal 6254 (the rivals as a set), byte-equal 6254
JOURNAL STATES beta: rows 39686, unreadable skipped 11, folded 39675, states lines compared 39675, equal 39675 (the rivals as a set), byte-equal 39675
test journals_reproduce_the_states_after_every_row ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.78s
```

[X] With the variable unset:

```text
$ cargo test --locked --test journal_states
running 2 tests
SKIPPED journals_reproduce_the_states_after_every_row: BSV_TRACKER_JOURNALS_DIR is unset (the header service's journals are private to the estate; nothing of them is committed)
test journals_reproduce_the_states_after_every_row ... ok
test a_wrong_line_is_named_by_its_fields ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

[X] Every row of both journals but the eleven unreadable ones passes the crate's decoder, and every folded row's state equals its states line both ways: the reducer reproduces the state after every row of the header service's two journals from a cold start.

[D] The journals exercise `tip`, `fork`, `reorg` and `tipAge` rows alone, and no `reorg` among them removes more than one header; the `split` and `splitResolved` arms, a reorg of several headers or of none, and a start that is not cold are pinned by `tests/doubts_rows.rs`, not here.

## The doubts' theorems as properties (#51, 2026-11-28)

[X] Worktree `bsv-tracker-trk-51-4`, branch `lane/trk-51-4`, from beta with the doubts' reducer, the keyed splits, the bound, the depth, the guard and the snapshot of #51 landed. The test is `tests/doubts_properties.rs`: 31 tests, each named for the Lean theorem it mirrors, 256 cases each from the explicit ChaCha seed with no failure persistence, as `tests/properties.rs`. No case is a journal row.

[SRC] The stack: bsv-stack-lean at `08ce467d` (`08ce467d2251c14f8a32719deb5f6e904d869bc4`), `lean/Headers/Doubts.lean` and `lean/Headers/Host.lean` (sections 6 and 12), read as they stand there, and `docs/p0/reorg-proof-cut2-2026-11-18.md`, its parts "Cut 1f" and "Cut 1g".

The draw: rows over heights 0 to 600 (half the cases anywhere in that range, half within 40 of a drawn base, where rivals, splits and tips meet), under a settled line of 2, 5 or 400; a `tip` row half the time, a `fork`, a well-formed `reorg` of up to 24 removed headers, now and then the hand-built `reorg` that removes none, a `split` and a `splitResolved`, each split row naming one of four keys, two of which are one key in two cases. A start is any value `Doubts::new` holds: no invariant is asked of it. The headers are `TestHeaders` with a drawn tip, above or below the feed's; the guard's cases add mined words whose root the headers carry, mined words whose root has since moved, paths at heights the headers do not carry, and each word that claims nothing of the chain, under no rule or one of two rules. Every height from 0 to 645 is asked. The Lean's `runUnpruned`, `dropped`, `effDepthFeedTip`, `Tracker.spendable`, `Tracker.stillConfirmed` and `Tracker.rootChecks`, which are not items of the crate, are mirrored in the test under their names, as is the host's page loop (`readC`), which is the host's.

| test | the Lean's theorem (`Doubts.lean` unless said) | what is checked |
|---|---|---|
| `safe_either_lag` | `effDepth_le_depth`, `safe_either_lag`, `effDepth_le_lead_headers` | on a start, its run and its unpruned run: `eff <= depth(headers' tip, x)`; for every held entry forking at or below `x`, `eff` at most the feed's tip, and at most the headers' tip, less its tip (stopped at zero) |
| `no_feed_is_today` | `no_feed_is_today`, `live_no_feed` | cold doubts at any tip, 0 included, and any doubts with no rival, split or summary: `eff == depth`; `spendable_eff` equals today's guard (`Tracker.spendable`); `may_enter` is "the root checks and two deep" |
| `run_safe` | `run_safe` | from cold, any rows, any headers, every `x`: `eff(run) <= eff(unpruned)` |
| `run_safe_from` | `run_safe_from` | the same from any start |
| `run_exact` | `run_exact` | from cold, `N` from 1 to 8: when every dropped entry's tip plus `N` is at most the lesser of the run's tip and the headers' tip, `N <= eff(run)` exactly when `N <= eff(unpruned)`; the hypothesis is met with something dropped in some case (checked) |
| `state_bound` | `state_bound` | from cold, after every prefix: the held entries forking at or below the tip number at most the settled line plus two |
| `reorg_holds_until_headers` | `reorg_holds_until_headers` | from any start, a `reorg` row at `fh` whose removed branch tops at `m`, then any rows, the headers' tip at most `m`: `eff == 0` at every height from `fh` up |
| `stale_feed_only_unseen` | `stale_feed_only_unseen` | two hosts, the stale one's tip not above the fresh one's: where the stale passes `N` and the fresh refuses, the fresh holds an open split (an entry, key and height) at or below `x` the stale lacks, or an entry forking at or below `x`, fewer than `N` below the lesser of its tip and the headers', that no entry of the stale covers; asked at the least such `N`, since the conclusion only weakens as `N` rises |
| `lesser_cost_bound` | `lesser_cost_bound`, `effDepth_le_feedTip`, `effDepth_eq_feedTip` | `eff <= effFeedTip <= eff + (feed tip - headers' tip)`, equal when the feed is not ahead |
| `safety` | `safety`, `spendableEff_imp_spendable` | under any doubts: `spendable_eff` true implies today's guard true; `may_enter` true implies the root checks and the path is two deep under the headers |
| `coldFrom_safe` | `Host.lean` `coldFrom_safe`, `least_covers` | `U` the unpruned run from any start: `Snapshot::least(U)` covers `U` (its tip, every open split, every held entry covered), and the run from its doubts over later rows passes nothing `unpruned(U, later)` refuses |
| `least_enough` | `Host.lean` `least_enough` | the same from cold: against the unpruned run of all the rows |
| `least_fold_coversK` | `Host.lean` `least_fold_tip_splits`, `least_fold_coversK` | the service's fold (`Snapshot::fold`) of the least snapshot has the tip and the open splits of the least snapshot of the run, exactly, and covers that run by key |
| `split_row_holds` | `split_row_holds` | after any `split` row, from any doubts: `eff == 0` from its fork height up, under headers that fault too (`Ok(0)`, no header read) |
| `split_new_key` | `split_new_key` | a key not open goes to the front at the row's height |
| `split_open_key` | `split_open_key` | a key open keeps its place and takes the lower of its height and the row's |
| `split_adds_rival` | `split_adds_rival` | a `split` row's rivals are a `fork` row's at its fork height with its lower witness tip as the competitor; tip and summary stay |
| `splitResolved_mem` | `splitResolved_mem`, `splitResolved_ends_key` | after a `splitResolved` the open entries are the old ones of other keys, in order, and the key is not open |
| `splitResolved_unknown` | `splitResolved_unknown` | for a key not open, `raw` changes nothing |
| `splitResolved_reads_no_height` | `splitResolved_reads_no_height` | two fork heights, one answer |
| `splitResolved_keeps_other` | `splitResolved_keeps_other` | every open entry of another key still holds from its height up |
| `split_holds_until_resolved` | `split_holds_until_resolved`, `run_keeps_hold` | with no `splitResolved` for its key, an open split stays open at its height or lower and holds from its height up, through any rows |
| `splits_never_shrink` | `splits_never_shrink` | no row but a `splitResolved` lowers the count of open splits |
| `run_perm` | `run_perm` | two starts alike but for the order of their rivals and their splits: after any rows the same rivals and splits as sets, one tip, one summary, one depth at every height |
| `trips_iff` | `Host.lean` `trips_iff`, `within_never_trips`, `run_splits` | under a named number `n` (0 to 3): `run_checked` is out exactly when some row leaves more than `n` keys open in the unpruned run; its doubts are `run`'s either way; out in some case (checked) |
| `trips_none` | `Host.lean` `trips_none`, `absent_is_today` | with the number absent: `Checked::Within(run)` |
| `out_passes_nothing` | `Host.lean` `out_passes_nothing` | out: `eff_c == 0` at every height; no `mined` word passes `spendable_c` under a spend depth of 1 or more; a word with no height is today's guard |
| `effC_le` | `Host.lean` `effC_le` | within, `eff_c` is `eff_depth`; out, never above it |
| `accepts_iff` | `Host.lean` `accepts_iff`, `refuse_version`, `refuse_unreadable`, `overfull_refused`, `refuse_tip` | `accepts` takes a served snapshot exactly on its five conditions (at the host's cursor or past it; a version `Params::versions` names; readable through its own cursor; within `Params::bound`; its tip the host's header by height and hash); `start_from` gives its doubts and cursor or builds none; a refusal names the earliest failing condition in the Lean's order (the crate's own promise) |
| `starts_le_reads` | `Host.lean` `starts_le_reads`, `readC_counts`, `in_never_starts`, `leaves_out`, `refused_stays` | over the page loop, mirrored on the crate's `run_checked` and `start_from`: each read counts one read and at most one start; a host in its contract makes no start; an out host that takes a snapshot is in at its doubts and cursor, one refused or unanswered stays as it was; starts at most reads (both a taken and a refused start occur, checked) |
| `registerAncestor_is_onProof` | `registerAncestor_is_onProof` | see below |

[X] The run (the SDK of rule 10 named for the link; the lines in name order):

```text
$ cargo test --locked --test doubts_properties
running 31 tests
test accepts_iff ... ok
test coldFrom_safe ... ok
test effC_le ... ok
test least_enough ... ok
test least_fold_coversK ... ok
test lesser_cost_bound ... ok
test no_feed_is_today ... ok
test out_passes_nothing ... ok
test registerAncestor_is_onProof ... ok
test reorg_holds_until_headers ... ok
test run_exact ... ok
test run_perm ... ok
test run_safe ... ok
test run_safe_from ... ok
test safe_either_lag ... ok
test safety ... ok
test splitResolved_keeps_other ... ok
test splitResolved_mem ... ok
test splitResolved_reads_no_height ... ok
test splitResolved_unknown ... ok
test split_adds_rival ... ok
test split_holds_until_resolved ... ok
test split_new_key ... ok
test split_open_key ... ok
test split_row_holds ... ok
test splits_never_shrink ... ok
test stale_feed_only_unseen ... ok
test starts_le_reads ... ok
test state_bound ... ok
test trips_iff ... ok
test trips_none ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
```

In the full suite (`cargo test --locked`) the total is 161 passed, 0 failed, against 129 at the base: these 31 and the key-case test of the source change (`a_mixed_case_key_is_held_in_lower_case`).

[X] Five faults planted in `src/doubts.rs`, one at a time and reverted after, each fail some of these tests: the summary not re-keyed by a `reorg` row (`run_safe`, `run_safe_from`, `coldFrom_safe`, `least_enough`, `least_fold_coversK` fail); a key already open keeping its stored height (`split_row_holds`, `split_open_key`); the lead read under the feed's tip alone (`safe_either_lag`, `reorg_holds_until_headers`); the readable cursor unchecked (`accepts_iff`); the summary not read by the depth (six fail).

The theorems issue #51 names, each by its test: `safe_either_lag`, `reorg_holds_until_headers`, `run_safe`, `run_exact`, `state_bound` and `lesser_cost_bound` are each a test of the same name. None is left without one.

`registerAncestor` (section 6) is no new item of the crate: it is the tracker's own proof step on a new state, `State::new(txid)` stepped with `Evidence::Proof(path)`. `registerAncestor_is_onProof` pins that: the word is `mined` at the path's height with the path and the header it was checked against as its evidence, not suspect and with no re-ask, exactly when the path's leaf is the txid and the headers carry its root at its height; otherwise the step is an error and the word stays `unknown` with no evidence. The crate's check also asks the headers' tip at or above the height, which the Lean's `onProof` does not; every header the test draws is at or below the tip, where the two agree.

What changed since the plan wrote appendix C, mirrored as the Lean states it now:

- The open splits are entries, a key and a fork height (cut 1f). `stale_feed_only_unseen` reads "an open split the stale host lacks" as an entry `p ∉ ds.splits`; the snapshot's `Covers` reads entries; cut 1g's `CoversK` reads each key at its height or lower (`SplitCov`).
- The lead is read under the lesser of the feed's and the headers' tip (cut 1e): `run_exact`'s hypothesis, `stale_feed_only_unseen`'s bound and `safe_either_lag`'s headers' side read it so.
- `reorg_holds_until_headers` holds from any start, not only from cold; the test draws any start.
- The plan's test names were descriptions; each test here carries its theorem's name (`eff_depth_never_above_the_depth_or_a_lead` is `safe_either_lag`, `spendable_eff_never_passes_what_today_refuses` is `safety`, `cold_from_the_least_snapshot_is_safe` is `coldFrom_safe` and `least_enough`).
- Cut 1f's laws of the keyed split and cut 1g's bound, out host, snapshot conditions and page loop are new rows of the table: the properties a host's safety rests on.
- The Lean's guards name a word `handed` that the crate has no word for; the mirrors read the crate's words. The Lean compares a snapshot's tip hash with `==`; the crate compares it in either case; the draw's hashes are in lower case, where the two agree.

Not mirrored here: `Host.lean`'s hosts over a journal (`host_is_run`, `two_hosts`, `reachC_holds`, `no_silent_drop`, `reachC_safe` and the rest of its `Reach` and `ReachC` statements), the recheck after a gap, the impossibility of section 8 and the load of a page read: they quantify over the host's journals and page reads, which are the host's, and the crate holds no journal. `fold_coversK` from a snapshot other than the least is not drawn. The pair of `Snapshot.least` over a run, which the Lean leaves owed, is not a theorem and has no test.

## The length rule as properties and goldens (GHSA-jjhc-7hvh-c3hm, BRC-237, 2026-11-28)

A transaction of exactly 64 bytes is never taken as proven by a merkle path (BRC-237's condition for an anchor; bsv-rs enforces it in its own verifier, its public advisory GHSA-jjhc-7hvh-c3hm). The crate measures the transaction: `State::tx_len`, the two constructors of `Proof` and the doors that read a length. This section is the rule's randomized statements and its compact goldens; the tables and the witness are `tests/measure.rs` and `tests/proof_length.rs`.

[SRC] The stack: bsv-stack-lean at `901e9dfa` (`901e9dfaac42b5dafdf5d5f2c552933dc1c28cd4`), `docs/DECISIONS-2026-10-19.md` 32.13 for the rule's words. `lean/Tracker.lean` there holds no measured length (no `txLen` in any Lean file): every name in the table below is ahead of the Lean and a proposal, apart from `txLen`, whose name is ruled.

[X] Worktree `bsv-tracker-trk-64-3`, branch `lane/trk-64-3`, from beta with the measure (lane 1) and the constructors and doors (lane 2) landed. A tests-only change: no line of `src` moves. The test is `tests/length_properties.rs`: ten tests, 256 cases each from the explicit ChaCha seed with no failure persistence, as `tests/properties.rs`.

The draw: a subject (`Subject` in `tests/common/mod.rs`: a txid alone, or bytes of 63, 64, 65 or 191, born measured or born from its txid), held by a `Tracker` beside a mined bystander, and up to 59 acts among: its own proof built from its bytes, stated at its length, stated at a length drawn among 0, 63, 64, 65, 190 and 191, stated at a length that is not its own, or taken out of a record; a competitor's proof from its bytes, stated, or out of a record; the measure (`Tracker::track_bytes`); a restore (`Tracker::restore`) of the state's own version 2 bytes, of its version 1 bytes, of a row born apart from its txid and mined on a length that is not its own (through either version), of the state's bytes with the length field set by hand, or of a measured birth; and a host act, a hint, a chain event, a recheck and a verdict. Headers carry each proof's root, or cannot answer. Each statement is asked after every act.

| test | the theorem's proposed name | what is checked |
|---|---|---|
| `ambiguous_never_mined` | I1 `ambiguous_never_mined` | a state whose length reads 64 does not read `mined` or `stale` and holds no record, whatever the road (either constructor, a record's proof, a competitor's, the measure, a restore through either version, a hand-set field); a state that held a proof and is measured at 64 occurs (checked) |
| `txLen_once` | I2 `txLen_once` | a length once held is held after every later act, refusals and restores included |
| `txLen_only_by_bytes` | I3 `txLen_only_by_bytes` | the length moves only by the measure, by a proof built from the subject's bytes (to the count of those bytes), or by a restore of a row that carries a length (to that length) |
| `mined_read_a_length` | I4 `mined_read_a_length` | a proof that writes a fresh `mined` word read a length: on a measured state the stated length equals the measure; on an unmeasured state the length is absent after it exactly when it was stated; a record's proof never mines an unmeasured state (`Unmeasured`); a fresh `mined` word comes from a proof or a restore alone; both unmeasured roads occur (checked) |
| `mismatch_refused` | I5 `mismatch_refused` | a measured state offered a proof stated at another length answers `LengthMismatch { measured, offered }`, keeps its word and record and asks again (`Reask::ProofFailed`); measured at 64 it answers `AmbiguousLength` and is left whole |
| `competitor_needs_bytes` | I6 `competitor_needs_bytes` | a competitor's proof not built from its bytes is `Unmeasured`, the word and the record kept; one built from its bytes meets no length fault, and is taken by a state measured at 64 (checked): the competitor is judged by its own length |
| `measure_moves_no_word` | I7 `measure_moves_no_word` | the measure changes the length alone, except at 64 on an unmeasured state that held a proof: the fault road (the keeper's K4), where the call returns `AmbiguousLength` AND the record is dropped, a `mined` or `stale` word reads `unknown` with no mark and no re-ask, and any other word keeps its word and re-ask (occurs, checked); a state holding another length refuses the measure, `LengthMismatch`, left whole |
| `record_carries_no_length` | I8 `record_carries_no_length` | a record's proof, as built and as restored from the bytes, is refused `Unmeasured` by `Headers::check`, by `doubts::may_enter` and by an unmeasured state; a state measured at its own length takes it (occurs, checked), one measured at 64 answers `AmbiguousLength` |
| `restore_leaves_the_index_describing_the_held_state` | none (the tracker's indexes are the crate's own) | for every pair of a held state and a restored row (the row stepped apart, then through version 2 bytes, version 1 bytes, or a hand-set length field): after `Tracker::restore` the tracker's indexes describe the state it then holds, read through its public doors on a copy (the height a fork reaches it at, the header hash an invalidation reaches it by, whether a new tip reaches it); on `Ok` the replaced state is returned; on `AmbiguousLength` the row is held unsaid; on `LengthMismatch` nothing moved; the bystander's indexes do not move; each of the three roads occurs (checked) |
| `every_act_leaves_the_index_describing_the_state` | none | the same reading of the indexes after every act of the trace |

The generators of `tests/properties.rs` and `tests/compact.rs` draw the same subject, its own proof by either constructor at a length that agrees with it or not, and the measure at random places. The four properties of `tests/properties.rs` keep their names and statements; `mined_implies_checked_proof` also reads that a `mined` state is never measured at 64, `hint_changes_no_chain_word` that a hint moves no length, and `same_evidence_same_word` that two states fed the same evidence hold the same length. In `stale_iff_header_inactive` half of the mined words are tracked from their bytes and measured by their proof and some states are measured at 64, which take no proof. `compact_round_trip_over_256_random_states` reads the length back and reaches unmeasured states, states measured at 64 and measured `mined` states (checked); `every_proper_prefix_is_truncated_and_a_suffix_is_trailing` runs over the generator as well as its fixed state, so a prefix that ends inside the length field is asked.

Three goldens in `tests/compact_golden.rs`, each beside the bytes of a twin born from the same txid without its bytes and stepped alike, which differ in the last field alone:

| golden | the state | the length field |
|---|---|---|
| `measured_unknown` | `State::measured(&tx_bytes(7, 191))` | `01bf01` |
| `measured_at_64` | `State::measured(&tx_bytes(7, 64))`, built | `0140` (no proof beside it, so it is read) |
| `mined_measured_by_its_proof` | born from `txid_of(&tx_bytes(7, 191))`, mined by `proof_of` at height 10; its twin mined by the same path on a stated length | `01bf01`; the record carries no length, so the rest is the twin's |

The txids of the two byte strings were checked apart from the crate with python's hashlib (the double SHA-256 reversed): `b6e3a84f...c109` for 191 bytes and `7f6b1704...98b7` for 64.

The witness, BRC-237's `T64`, as lane 2 recorded it (`tests/proof_length.rs`, `brc237_t64_is_never_proven`). At its base `bd95b1e`, through the constructor that took a txid alone, before the constructors read a length:

```text
BASE WITNESS: len 64 txid 6444bf849ea4136a9608b60996cc7d552d65d94c7183ae4f71a678add2de1e02: step Ok(()), word mined Some(950008)
```

At its tip, `Proof::new(T64, path)` and `Proof::with_stated_len(txid, 64, path)` answer `Err(AmbiguousLength)`, and `State::measured(T64)` refuses a proof stated at 63 or 65 and stays whole:

```text
$ cargo test --locked --test proof_length -- --exact brc237_t64_is_never_proven
test brc237_t64_is_never_proven ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s
```

[X] The run:

```text
$ cargo test --locked --test length_properties
running 10 tests
test ambiguous_never_mined ... ok
test competitor_needs_bytes ... ok
test every_act_leaves_the_index_describing_the_state ... ok
test measure_moves_no_word ... ok
test mined_read_a_length ... ok
test mismatch_refused ... ok
test record_carries_no_length ... ok
test restore_leaves_the_index_describing_the_held_state ... ok
test txLen_once ... ok
test txLen_only_by_bytes ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.57s
```

In the full suite (`cargo test --locked`) the total is 210 passed, 0 failed, against 197 at the base: these ten and the three goldens.

[X] Twelve faults planted in the arms of `src`, one at a time and reverted after (`git diff --stat -- src` empty at the end), each failing some of the tests (the full suite run each time):

| the fault | failed |
|---|---|
| the proof door of a state measured at 64 takes the proof | `ambiguous_never_mined` and the nine other properties; `stale_iff_header_inactive`; `a_state_measured_at_64_refuses_every_proof`, `measure_after_birth`, `the_proof_door_pairings`, `brc237_t64_is_never_proven` |
| `Proof::with_stated_len` takes a stated 64 | `constructors_by_length`, `brc237_t64_is_never_proven` |
| `Proof::new` takes 64 bytes | the ten properties; `mined_implies_checked_proof`, `compact_round_trip_over_256_random_states`, `constructors_by_length`, `brc237_t64_is_never_proven` |
| a measured state takes a proof stated at another length | `mismatch_refused`, `mined_read_a_length`, `the_proof_door_pairings`, `a_measured_state_against_a_stated_length_proof` |
| a proof built from bytes does not measure an unmeasured state | `mined_read_a_length`, `mined_measured_by_its_proof`, `the_proof_door_pairings` |
| the competitor door takes a stated proof | `competitor_needs_bytes`, `the_competitor_door` |
| the compact decoder reads a length of 64 beside a stored proof | `ambiguous_never_mined`, `tx_length_field` |
| a measure of 64 on a state that held a proof does not unsay it | the ten properties; `mined_implies_checked_proof`, `compact_round_trip_over_256_random_states`; `measure_after_birth`, `a_measure_of_64_on_a_tracked_mined_word_returns_the_fault_and_moves_the_indexes`, `a_mined_row_of_version_1_restores_unmeasured_and_stays_mined`, `restore_carries_the_held_measure` |
| `Tracker::restore` does not carry the held measure | `txLen_once`, `txLen_only_by_bytes`, `restore_leaves_the_index_describing_the_held_state`, `restore_carries_the_held_measure` |
| an unmeasured state takes a record's proof | `record_carries_no_length`, `mined_read_a_length`, `the_proof_door_pairings` |
| `Headers::check` takes a record's proof | `record_carries_no_length`, `the_doors_with_no_state` |
| `doubts::may_enter` takes a record's proof | `record_carries_no_length`, `the_doors_with_no_state` |
| `Tracker::restore` takes a row of another length | `txLen_once`, `restore_leaves_the_index_describing_the_held_state`, `restore_carries_the_held_measure` |

What no length rule closes, a transaction of any length posing one level below a real 64-byte leaf, is not a statement here and is not claimed by any test.
