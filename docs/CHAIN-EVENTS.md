# Chain event envelope, versions 1 and 2

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

## Version 2: `split` and `splitResolved`

[SRC] The emitter of record is `chaintracks-cloudflare@v0.4.1` (`74cc110`), read through `git show v0.4.1:<path>`: shapes `src/events.rs:50-57,95-107` and `docs/SPLITS.md:75-83`, the version gate `src/events.rs:130-146`, the relationship checks `src/events.rs:157-161,210-232`, the key's form and lifecycle `docs/SPLITS.md:85-101`, and "Readers accept both versions; each envelope selects its schema" (`docs/CHAIN-EVENTS.md:6-8`). A split reports that the configured rungs follow different branches, each at least `SPLIT_DEPTH` blocks past their common ancestor; it does not change the service's tip (`docs/SPLITS.md:3-8`).

| kind | `v` | required fields beside `v` and `kind` |
|---|---|---|
| `split` | 2 | `key: string`, `forkHeight: u32`, `tips: [SplitTip, SplitTip]` |
| `splitResolved` | 2 | `key: string`, `forkHeight: u32` |

`SplitTip` is `{hash: string, height: u32, chainWork: string, rungs: string[]}` (Rust `SplitTip { hash, height, chain_work, rungs }`); unknown fields are refused as in version 1.

[X] The gate: `v: 1` admits exactly the six version-1 kinds and `v: 2` admits exactly the two split kinds. Any other `v` is `UnknownVersion(v)`; a known version with a kind it does not carry is `UnknownKind(kind)`, so `v: 2` with `tip` and `v: 1` with `split` are refused whatever their bodies. The emitter's own reader also admits a version-1 kind under `v: 2` (`src/events.rs:139-146`); this decoder is stricter, because the emitter writes the six original kinds as version 1 only (`docs/SPLITS.md:116`). The relationship checks mirror the emitter's: `key` is `split:` and 64 hex characters (the common ancestor's display-order hash), `forkHeight` is 1 through `i32::MAX`; a split has exactly two tips with distinct hashes, each hash and `chainWork` 64 hex characters, each height at or above `forkHeight` and at most `i32::MAX`, each `rungs` non-empty, strictly sorted and free of empty names, and no rung on both tips. `ChainEvent::version()` names the version a kind is carried in; `ChainEvent` and `ChainEnvelope` serialize that version. The six version-1 kinds decode and serialize exactly as before.

[SRC] The state rule follows the owner's ruling of 2026-10-16 (`bsv-stack-lean docs/RULINGS-2026-10-12.md:323-329`, 19:3xZ): the service answers the most-work chain of valid headers it has seen over every rung, and depth is the consumer's. [X] A split is an observation, not a reorg. `Tracker::on_chain` delivers both kinds to the mined words at or above `forkHeight` only. `split` marks those words suspect and sets `Reask::Fork { height: forkHeight }` (the fork's re-ask is reused: the host's act, a recheck under its own depth rule, is the same); it moves no word and retains the evidence. `splitResolved` rechecks the suspect words at or above `forkHeight` through `Headers::check`'s snapshot, as a tip does for the suspect set: a matching root clears the mark, a replaced root makes the word stale with `Reask::Reorg`, an unavailable snapshot keeps the word and schedules `Recheck`. `frozen` and `tipAge` are unchanged.

**A split never writes a word; a resolved split moves a word only through a checked proof.** [X] `tests/splits.rs` (`split_writes_no_word`, 256 deterministic traces over an emptied snapshot, and three scenario tests), `tests/chain_events.rs` (`pinned_split_round_trip`, `pinned_split_resolved_round_trip`, the refused combinations and malformed shapes). [D] `tests/vectors/chain_events/split.json` and `split_resolved.json` are synthetic, written in the documented shape and the key order of `migrations/0010_rung_tips.sql:63-74,82-87`; the emitter's docs carry no literal example, and no live beta envelope was captured. The Lean machine at the stack's pin has no split input; this rule is a Rust projection of the ruling, not a refinement of a Lean theorem.
