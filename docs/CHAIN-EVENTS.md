# Chain event envelope, version 1

[SRC] The envelope of record is `rust-chaintracks@a62f9ed docs/CHAIN-EVENTS.md:13-35`, read through `git show`. Exact Rust widths and names are `src/events.rs:13-93` at the same pin. The stack's copy is `docs/p0/a3-32-chain-event-envelope.md`, sections "Schema and version rule" and "One example per kind". The Lean transition projection remains `bsv-stack-lean@f95520fcd85e066ec341faffa9dd8f1f8f35a37e lean/Tracker.lean:132`.

Every event has `v: 1` and exactly one `kind`. [SRC] `rust-chaintracks@a62f9ed docs/CHAIN-EVENTS.md:24-35`:

| kind | required fields beside `v` and `kind` |
|---|---|
| `tip` | `height: u32`, `hash: string`, `time: u32`, `header: H` |
| `fork` | `height: u32`, `competingTips: [H, H]`, `depth: u32` |
| `reorg` | `forkHeight: u32`, `depth: u32`, `deactivatedHeaders: H[]`, `newTip: H` |
| `invalidated` | `blockHash: string` |
| `frozen` | `outpoint: {txid: string, vout: u32}` |
| `tipAge` | `seconds: u64`, `tip: H` |

`H` is the public `EventHeader`, separate from the tracker's checked `Header` projection. All nine fields are required; storage flags and row ids are refused. [SRC] `rust-chaintracks@a62f9ed docs/CHAIN-EVENTS.md:13-23`, `src/events.rs:13-25,43-85`:

| wire field | Rust field | type |
|---|---|---|
| `version` | `version` | `u32` |
| `previousHash` | `previous_hash` | `String` |
| `merkleRoot` | `merkle_root` | `String` |
| `time` | `time` | `u32` (UNIX seconds) |
| `bits` | `bits` | `u32` |
| `nonce` | `nonce` | `u32` |
| `height` | `height` | `u32` |
| `hash` | `hash` | `String` |
| `chainWork` | `chain_work` | `String` (256-bit hex integer) |

[SRC] Hashes, txids and work contain 64 hex characters; hashes are in display byte order. The decoder matches the emitter's shape and relationship checks at `rust-chaintracks@a62f9ed src/events.rs:125-185`: header heights at most `i32::MAX`, redundant tip fields agree with the header, fork tips are distinct and depth locates the inclusive fork height, and reorg depth is 1 through 400 and matches linked, descending deactivated headers ending at `forkHeight`. Parsing these fields establishes no proof of work, active-chain membership or transaction inclusion (`docs/CHAIN-EVENTS.md:35-48`).

[X] `ChainEvent` and `ChainEnvelope` serialize the same flat versioned object and deserialize through the same version/shape gate. `decode_envelope` exposes `DecodeError::UnknownVersion(v)`, `UnknownKind(kind)` and `UnknownShape { version, detail }` to the host. Version is checked before kind/body, independent of key order; missing/malformed fields and extra fields are faults. Direct serde reports the same fault through the deserializer's error type. [SRC] Hosts report a fault through their status, metric or listener, stop at its cursor and do not acknowledge or skip it (`rust-chaintracks@a62f9ed docs/CHAIN-EVENTS.md:200-206`). Live transport/cursor handling is host work [D].

[X] Envelope data never constructs `CheckedProof`: the immutable capability still comes through `Headers::check`, and `src/evidence.rs`, `src/state.rs` and `src/tracker.rs` are unchanged by this reconciliation. Reorgs select the mined height range at or above `forkHeight`; descending deactivated header heights describe the removed suffix, and the resulting `Reask::Reorg` retains `forkHeight`. Neither `newTip.height`, tip payload roots nor advertised `chainWork` substitutes for the host's checked snapshot. A tip rechecks only suspect txids, and frozen/tip-age input writes no mined word.

[X] `tests/vectors/chain_events/` preserves the six JSON examples verbatim from `rust-chaintracks@a62f9ed docs/CHAIN-EVENTS.md:265,269,273,277,281,285`, checked against the stack's copy. `tests/chain_events.rs` parses and reserializes each through both public serde types and the typed decoder. Invalidated/frozen also compare bytes (apart from the fixture's trailing newline); the four header-bearing examples compare JSON values because field order is not a schema rule. These tests also reject the old assumed names/types, future kinds/versions and malformed shapes, preserve targeted reorg scheduling/evidence, and prove that rich tip data cannot bypass an unavailable checked snapshot. [D] This is a local codec/machine replay, not live emitter delivery or a host integration.

[D] Migration for callers constructing events: supply `tip.time/header`; replace `fork.competingTips` hash vectors with exactly two `EventHeader`s; replace `reorg.deactivated` with `deactivated_headers` serialized as `deactivatedHeaders`, and its hash-only `new_tip` with an `EventHeader`; replace string outpoints with `Outpoint { txid, vout }`; supply `tipAge.tip`; handle `UnknownKind`. Direct `ChainEvent` serde now requires/emits `v`, like `ChainEnvelope`. This corrects an unpublished 0.1.0 API; after publication the Rust/accepted-shape break would require a minor release. There is no storage migration or unchecked trusted-state deserializer. Stored proofs still enter through targeted `Headers::check` calls, bounded by the host's tracked rows. Roll back by returning a host to its previous adapter; do not connect the old hash-only projection to this emitter.
