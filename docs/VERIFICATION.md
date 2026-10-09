# A6 #30 verification record

[SRC] Initial specification: bsv-stack-lean@24f22f808b1fb44ce9bf128ec89ac5c8839d9d60, `docs/charters/tracker.md`, `maps/tracker.json`, `lean/Tracker.lean`, `docs/APP-CONTRACT.md` and the seven named scenarios. Final machine pin: `f95520fcd85e066ec341faffa9dd8f1f8f35a37e`, including the landed #31 successful-recheck correction recorded at the end. SDK and ARC vocabulary: bsv-rs@7bc623c, `tests/vectors/arc_tx_status_verdicts.json`; the actual dependency is crates.io `bsv-rs` at the caret requirement `0.3.35` (the tracked `Cargo.lock` holds 0.3.35) with only `transaction`, and `wasm` under this crate's `wasm` feature.

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
