# A6 #30 verification record

[SRC] Specification: bsv-stack-lean@24f22f808b1fb44ce9bf128ec89ac5c8839d9d60, `docs/charters/tracker.md`, `maps/tracker.json`, `lean/Tracker.lean`, `docs/APP-CONTRACT.md` and the seven named scenarios. SDK and ARC vocabulary: bsv-rs@7bc623c, `tests/vectors/arc_tx_status_verdicts.json`; the actual dependency is crates.io `bsv-rs =0.3.35` with only `transaction`, and `wasm` under this crate's `wasm` feature.

[X] New repository, `main`, no base commit, no remote. This is the lane's explicit exception to the shared worktree rule. No host integration or deployment is authorized by this lane.

## Witness before the transitions

[X] The seven tests compile and fail because the transition function is missing. Five encounter `Unknown` instead of `Built` or `Mined`; the complete output below records every failure. Fixtures use synthetic two-leaf SDK paths and in-memory active header snapshots, no node or production data.

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

