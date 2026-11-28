//! The compact bytes pinned (bsv-stack-lean #92; the make-room pull request
//! of the rulings of 2026-11-17, the stack's 28.8.1 (c)): fixed states that
//! between them hold every word, a checked proof, every hint status with and
//! without a verdict and every re-ask, each written by `State::to_compact`
//! and compared with bytes spelled out here as hex. A change that moves one
//! byte the crate writes fails here, whatever else it keeps.
// The lint bar binds the library; a test may unwrap, and its fixtures fail loudly
// (bsv-stack-lean #134).
#![allow(
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod common;
use bsv_tracker::*;
use common::*;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The bytes the state writes, against the golden; a state whose word is not
/// `mined` also restores equal to itself (a mined word restores suspect with
/// its recheck due, `State::from_compact`'s rule, so it is not compared).
fn golden(name: &str, s: &State, expected: &str) {
    let bytes = s.to_compact();
    assert_eq!(hex(&bytes), expected, "{name}: the compact bytes moved");
    let restored = State::from_compact(&bytes).unwrap();
    if !matches!(s.word(), Word::Mined(_)) {
        assert_eq!(&restored, s, "{name}");
    }
}

/// A verdict read from an ARC body, so the test builds none by literal.
fn arc(body: &str, source: &str, observed: u64) -> Hint {
    Verdict::from_arc_body(&hash(1), body.as_bytes())
        .unwrap()
        .into_hint(source, observed)
}

/// Every hint status: nine without a verdict, nine with one.
fn every_hint() -> Vec<Hint> {
    let plain = [
        HintStatus::Accepted,
        HintStatus::Seen,
        HintStatus::Mined { height: 900 },
        HintStatus::StaleBlock,
        HintStatus::Rejected {
            reason: "r \u{e9}".into(),
        },
        HintStatus::DoubleSpend {
            competitors: vec![hash(7), "not-an-id".into()],
        },
        HintStatus::DoubleSpend {
            competitors: vec![],
        },
        HintStatus::OrphanMempool,
        HintStatus::Unknown,
        HintStatus::MempoolPresent {
            source: "courier-a".into(),
        },
    ];
    let mut hints: Vec<Hint> = plain
        .into_iter()
        .enumerate()
        .map(|(i, status)| Hint::new(format!("plain-{i}"), status, 100 + i as u64))
        .collect();
    let competitor = hash(7);
    let bodies = [
        r#"{"txStatus":"ACCEPTED_BY_NETWORK"}"#.to_string(),
        r#"{"txStatus":"SEEN_ON_NETWORK","extraInfo":"note"}"#.to_string(),
        r#"{"txStatus":"MINED","blockHeight":900}"#.to_string(),
        r#"{"txStatus":"MINED_IN_STALE_BLOCK","blockHeight":899}"#.to_string(),
        r#"{"txStatus":"REJECTED","extraInfo":"bad script"}"#.to_string(),
        format!(r#"{{"txStatus":"DOUBLE_SPEND_ATTEMPTED","competingTxs":["{competitor}"]}}"#),
        r#"{"txStatus":"SEEN_IN_ORPHAN_MEMPOOL"}"#.to_string(),
        r#"{"txStatus":"UNKNOWN"}"#.to_string(),
    ];
    for (i, body) in bodies.iter().enumerate() {
        hints.push(arc(body, &format!("arc-{i}"), 200 + i as u64));
    }
    // The presence hint is never a broadcaster's reduction; a host may still
    // keep a verdict beside it.
    let mut present = arc(r#"{"txStatus":"SEEN_ON_NETWORK"}"#, "courier", 300);
    present.status = HintStatus::MempoolPresent {
        source: "courier-b".into(),
    };
    hints.push(present);
    hints
}

fn mined_state(h: &mut TestHeaders) -> State {
    let txid = hash(1);
    let p = proof(&txid, 10, 2);
    h.insert(&p, 10);
    let mut s = State::new(&txid);
    step(&mut s, h, Input::Evidence(Evidence::Proof(p)));
    s
}

#[test]
fn every_hint_status_with_and_without_a_verdict_is_held() {
    let hints = every_hint();
    for (code, want) in [
        (0, 2),
        (1, 2),
        (2, 2),
        (3, 2),
        (4, 2),
        (5, 3),
        (6, 2),
        (7, 2),
        (8, 2),
    ] {
        let n = hints
            .iter()
            .filter(|h| match code {
                0 => h.status == HintStatus::Accepted,
                1 => h.status == HintStatus::Seen,
                2 => matches!(h.status, HintStatus::Mined { .. }),
                3 => h.status == HintStatus::StaleBlock,
                4 => matches!(h.status, HintStatus::Rejected { .. }),
                5 => matches!(h.status, HintStatus::DoubleSpend { .. }),
                6 => h.status == HintStatus::OrphanMempool,
                7 => h.status == HintStatus::Unknown,
                _ => matches!(h.status, HintStatus::MempoolPresent { .. }),
            })
            .count();
        assert_eq!(n, want, "status {code}");
    }
    assert_eq!(hints.iter().filter(|h| h.verdict.is_some()).count(), 9);
}

fn unknown_and_no_reask_state() -> State {
    State::new(hash(1))
}

#[test]
fn unknown_and_no_reask() {
    golden(
        "unknown",
        &unknown_and_no_reask_state(),
        "0200000000000000000000000000000000000000000000000000000000000000000100000000000000000000",
    );
}

fn built_with_spend_state() -> State {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(&mut s, &h, Input::Host(HostAction::Build));
    step(&mut s, &h, Input::Host(HostAction::SpendAttempt));
    assert_eq!(s.reask(), Some(&Reask::Spend));
    s
}

#[test]
fn built_with_spend() {
    golden(
        "built",
        &built_with_spend_state(),
        concat!(
            "02000000000000000000000000000000000000000000000000000000000000000001010000000001",
            "0600000000",
        ),
    );
}

fn announced_with_age_state() -> State {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(&mut s, &h, Input::Host(HostAction::Build));
    step(&mut s, &h, hint(HintStatus::Accepted, 10));
    step(&mut s, &h, Input::Host(HostAction::Tick(70)));
    assert_eq!(s.word(), &Word::Announced);
    assert_eq!(s.reask(), Some(&Reask::Age));
    s
}

#[test]
fn announced_with_age() {
    golden(
        "announced",
        &announced_with_age_state(),
        concat!(
            "02000000000000000000000000000000000000000000000000000000000000000001020000011073",
            "7475622062726f6164636173746572000a00010a010500000000",
        ),
    );
}

fn seen_with_every_hint_and_a_hint_reask_state() -> State {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(&mut s, &h, Input::Host(HostAction::Build));
    for hint in every_hint() {
        step(&mut s, &h, Input::Hint(hint));
    }
    assert_eq!(s.word(), &Word::Seen);
    assert_eq!(s.hints().len(), 19);
    assert!(matches!(s.reask(), Some(Reask::Hint(_))));
    s
}

#[test]
fn seen_with_every_hint_and_a_hint_reask() {
    golden(
        "seen",
        &seen_with_every_hint_and_a_hint_reask_state(),
        concat!(
            "02000000000000000000000000000000000000000000000000000000000000000001030000130763",
            "6f75726965720809636f75726965722d62ac02010f5345454e5f4f4e5f4e4554574f524b00000005",
            "6172632d3707cf010107554e4b4e4f574e000000056172632d3606ce0101165345454e5f494e5f4f",
            "525048414e5f4d454d504f4f4c000000056172632d35050100000000000000000000000000000000",
            "0000000000000000000000000000000007cd010116444f55424c455f5350454e445f415454454d50",
            "54454400000100000000000000000000000000000000000000000000000000000000000000000705",
            "6172632d34040a62616420736372697074cc01010852454a454354454400010a6261642073637269",
            "707400056172632d3303cb0101144d494e45445f494e5f5354414c455f424c4f434b018307000005",
            "6172632d32028407ca0101054d494e45440184070000056172632d3101c901010f5345454e5f4f4e",
            "5f4e4554574f524b0001046e6f746500056172632d3000c801011341434345505445445f42595f4e",
            "4554574f524b00000007706c61696e2d390809636f75726965722d616d0007706c61696e2d38076c",
            "0007706c61696e2d37066b0007706c61696e2d3605006a0007706c61696e2d350502000000000000",
            "00000000000000000000000000000000000000000000000000000701096e6f742d616e2d69646900",
            "07706c61696e2d3404047220c3a9680007706c61696e2d3303670007706c61696e2d320284076600",
            "07706c61696e2d3101650007706c61696e2d3000640001ac020102056172632d3707cf010107554e",
            "4b4e4f574e00000000000000",
        ),
    );
}

fn mined_with_recheck_state() -> State {
    let mut h = TestHeaders::default();
    let mut s = mined_state(&mut h);
    let other = competitor(2, 11, 3);
    h.insert(&other, 11);
    step(&mut s, &h, Input::Evidence(Evidence::Competitor(other)));
    assert!(matches!(s.word(), Word::Mined(_)));
    assert!(s.evidence().is_some());
    assert_eq!(s.reask(), Some(&Reask::Recheck));
    s
}

#[test]
fn mined_with_recheck() {
    golden(
        "mined",
        &mined_with_recheck_state(),
        concat!(
            "02000000000000000000000000000000000000000000000000000000000000000001040100000000",
            "00000000000000000000000000000000000000000000000000000000010a01020006000000000000",
            "00000000000000000000000000000000000000000000000000000101040000000000000000000000",
            "00000000000000000000000000000000000000000002000000000000000000000000000000000000",
            "00000000000000000000000000000a00ff8f6e7ac7aa0bfc98af8be24ed6ae1ab243f64aaa065366",
            "b2215e697fddb64b00ff8f6e7ac7aa0bfc98af8be24ed6ae1ab243f64aaa065366b2215e697fddb6",
            "4b01000000010400000000",
        ),
    );
}

fn mined_suspect_with_fork_state() -> State {
    let mut h = TestHeaders::default();
    let mut s = mined_state(&mut h);
    step(&mut s, &h, Input::Evidence(Evidence::Chain(fork(9))));
    assert!(s.suspect());
    assert_eq!(s.reask(), Some(&Reask::Fork { height: 9 }));
    s
}

#[test]
fn mined_suspect_with_fork() {
    golden(
        "fork",
        &mined_suspect_with_fork_state(),
        concat!(
            "02000000000000000000000000000000000000000000000000000000000000000001040100000000",
            "00000000000000000000000000000000000000000000000000000000010a01020006000000000000",
            "00000000000000000000000000000000000000000000000000000101040000000000000000000000",
            "00000000000000000000000000000000000000000002000000000000000000000000000000000000",
            "00000000000000000000000000000a00ff8f6e7ac7aa0bfc98af8be24ed6ae1ab243f64aaa065366",
            "b2215e697fddb64b00ff8f6e7ac7aa0bfc98af8be24ed6ae1ab243f64aaa065366b2215e697fddb6",
            "4b0101000001010900000000",
        ),
    );
}

fn stale_with_reorg_state() -> State {
    let mut h = TestHeaders::default();
    let mut s = mined_state(&mut h);
    h.move_root(10, 50);
    step(&mut s, &h, Input::Evidence(Evidence::Chain(reorg(10))));
    assert_eq!(s.word(), &Word::Stale);
    assert_eq!(s.reask(), Some(&Reask::Reorg { fork_height: 10 }));
    s
}

#[test]
fn stale_with_reorg() {
    golden(
        "stale",
        &stale_with_reorg_state(),
        concat!(
            "02000000000000000000000000000000000000000000000000000000000000000001050100000000",
            "00000000000000000000000000000000000000000000000000000000010a01020006000000000000",
            "00000000000000000000000000000000000000000000000000000101040000000000000000000000",
            "00000000000000000000000000000000000000000002000000000000000000000000000000000000",
            "00000000000000000000000000000a00ff8f6e7ac7aa0bfc98af8be24ed6ae1ab243f64aaa065366",
            "b2215e697fddb64b00ff8f6e7ac7aa0bfc98af8be24ed6ae1ab243f64aaa065366b2215e697fddb6",
            "4b0100000001000a00000000",
        ),
    );
}

fn rejected_with_proof_failed_state() -> State {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Verdict(NodeVerdict::Rejected {
            reason: "node said no".into(),
        })),
    );
    let wrong = proof(&hash(3), 12, 4);
    assert_eq!(
        s.step(&params(), &h, Input::Evidence(Evidence::Proof(wrong))),
        Err(CheckError::TxidMismatch)
    );
    assert_eq!(s.reask(), Some(&Reask::ProofFailed { height: 12 }));
    s
}

#[test]
fn rejected_with_proof_failed() {
    golden(
        "rejected",
        &rejected_with_proof_failed_state(),
        concat!(
            "02000000000000000000000000000000000000000000000000000000000000000001060c6e6f6465",
            "2073616964206e6f0000000001030c00000000",
        ),
    );
}

fn conflicted_by_ids_and_text_state() -> State {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Verdict(NodeVerdict::Conflicted {
            competitors: vec![hash(8), "Not-An-Id".into()],
        })),
    );
    assert!(matches!(s.word(), Word::Conflicted { .. }));
    s
}

#[test]
fn conflicted_by_ids_and_text() {
    golden(
        "conflicted",
        &conflicted_by_ids_and_text_state(),
        concat!(
            "02000000000000000000000000000000000000000000000000000000000000000001070200000000",
            "000000000000000000000000000000000000000000000000000000000801094e6f742d416e2d4964",
            "000000000000000000",
        ),
    );
}

fn abandoned_state() -> State {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(&mut s, &h, Input::Host(HostAction::Build));
    step(&mut s, &h, Input::Host(HostAction::Abandon));
    assert_eq!(s.word(), &Word::Abandoned);
    s
}

#[test]
fn abandoned() {
    golden(
        "abandoned",
        &abandoned_state(),
        "0200000000000000000000000000000000000000000000000000000000000000000108000000000000000000",
    );
}

/// The ten states of record with the version 1 bytes each wrote up to
/// 0.9.0-beta.11, verbatim. They are the reader's record and never move: a
/// version 1 row a host stored still restores.
type Record = (&'static str, fn() -> State, &'static str);
const VERSION_1: [Record; 10] = [
    (
        "unknown",
        unknown_and_no_reask_state,
        "01000000000000000000000000000000000000000000000000000000000000000001000000000000",
    ),
    (
        "built",
        built_with_spend_state,
        concat!(
            "01000000000000000000000000000000000000000000000000000000000000000001010000000001",
            "06",
        ),
    ),
    (
        "announced",
        announced_with_age_state,
        concat!(
            "01000000000000000000000000000000000000000000000000000000000000000001020000011073",
            "7475622062726f6164636173746572000a00010a0105",
        ),
    ),
    (
        "seen",
        seen_with_every_hint_and_a_hint_reask_state,
        concat!(
            "01000000000000000000000000000000000000000000000000000000000000000001030000130763",
            "6f75726965720809636f75726965722d62ac02010f5345454e5f4f4e5f4e4554574f524b00000005",
            "6172632d3707cf010107554e4b4e4f574e000000056172632d3606ce0101165345454e5f494e5f4f",
            "525048414e5f4d454d504f4f4c000000056172632d35050100000000000000000000000000000000",
            "0000000000000000000000000000000007cd010116444f55424c455f5350454e445f415454454d50",
            "54454400000100000000000000000000000000000000000000000000000000000000000000000705",
            "6172632d34040a62616420736372697074cc01010852454a454354454400010a6261642073637269",
            "707400056172632d3303cb0101144d494e45445f494e5f5354414c455f424c4f434b018307000005",
            "6172632d32028407ca0101054d494e45440184070000056172632d3101c901010f5345454e5f4f4e",
            "5f4e4554574f524b0001046e6f746500056172632d3000c801011341434345505445445f42595f4e",
            "4554574f524b00000007706c61696e2d390809636f75726965722d616d0007706c61696e2d38076c",
            "0007706c61696e2d37066b0007706c61696e2d3605006a0007706c61696e2d350502000000000000",
            "00000000000000000000000000000000000000000000000000000701096e6f742d616e2d69646900",
            "07706c61696e2d3404047220c3a9680007706c61696e2d3303670007706c61696e2d320284076600",
            "07706c61696e2d3101650007706c61696e2d3000640001ac020102056172632d3707cf010107554e",
            "4b4e4f574e000000",
        ),
    ),
    (
        "mined",
        mined_with_recheck_state,
        concat!(
            "01000000000000000000000000000000000000000000000000000000000000000001040100000000",
            "00000000000000000000000000000000000000000000000000000000010a01020006000000000000",
            "00000000000000000000000000000000000000000000000000000101040000000000000000000000",
            "00000000000000000000000000000000000000000002000000000000000000000000000000000000",
            "00000000000000000000000000000a00ff8f6e7ac7aa0bfc98af8be24ed6ae1ab243f64aaa065366",
            "b2215e697fddb64b00ff8f6e7ac7aa0bfc98af8be24ed6ae1ab243f64aaa065366b2215e697fddb6",
            "4b010000000104",
        ),
    ),
    (
        "fork",
        mined_suspect_with_fork_state,
        concat!(
            "01000000000000000000000000000000000000000000000000000000000000000001040100000000",
            "00000000000000000000000000000000000000000000000000000000010a01020006000000000000",
            "00000000000000000000000000000000000000000000000000000101040000000000000000000000",
            "00000000000000000000000000000000000000000002000000000000000000000000000000000000",
            "00000000000000000000000000000a00ff8f6e7ac7aa0bfc98af8be24ed6ae1ab243f64aaa065366",
            "b2215e697fddb64b00ff8f6e7ac7aa0bfc98af8be24ed6ae1ab243f64aaa065366b2215e697fddb6",
            "4b01010000010109",
        ),
    ),
    (
        "stale",
        stale_with_reorg_state,
        concat!(
            "01000000000000000000000000000000000000000000000000000000000000000001050100000000",
            "00000000000000000000000000000000000000000000000000000000010a01020006000000000000",
            "00000000000000000000000000000000000000000000000000000101040000000000000000000000",
            "00000000000000000000000000000000000000000002000000000000000000000000000000000000",
            "00000000000000000000000000000a00ff8f6e7ac7aa0bfc98af8be24ed6ae1ab243f64aaa065366",
            "b2215e697fddb64b00ff8f6e7ac7aa0bfc98af8be24ed6ae1ab243f64aaa065366b2215e697fddb6",
            "4b0100000001000a",
        ),
    ),
    (
        "rejected",
        rejected_with_proof_failed_state,
        concat!(
            "01000000000000000000000000000000000000000000000000000000000000000001060c6e6f6465",
            "2073616964206e6f0000000001030c",
        ),
    ),
    (
        "conflicted",
        conflicted_by_ids_and_text_state,
        concat!(
            "01000000000000000000000000000000000000000000000000000000000000000001070200000000",
            "000000000000000000000000000000000000000000000000000000000801094e6f742d416e2d4964",
            "0000000000",
        ),
    ),
    (
        "abandoned",
        abandoned_state,
        "01000000000000000000000000000000000000000000000000000000000000000001080000000000",
    ),
];

/// The relation of the two versions on a state with no instance, no
/// competitor, no prior word and no measured length: version 2's bytes are
/// version 1's with the opening byte `02` and four bytes `00000000` after the
/// re-ask.
fn version_2_of(version_1: &str) -> String {
    assert!(version_1.starts_with("01"));
    format!("02{}00000000", &version_1[2..])
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn version_1_bytes_still_restore() {
    for (name, build, v1) in VERSION_1 {
        let built = build();
        let restored = State::from_compact(&unhex(v1)).unwrap();
        if matches!(built.word(), Word::Mined(_)) {
            // `State::from_compact`'s rule: a restored mined word is suspect
            // with its recheck due; every other field is the built state's.
            assert_eq!(restored.word(), built.word(), "{name}");
            assert_eq!(restored.evidence(), built.evidence(), "{name}");
            assert!(restored.suspect(), "{name}");
            assert_eq!(
                restored.reask(),
                built.reask().or(Some(&Reask::Recheck)),
                "{name}"
            );
            assert_eq!(restored.hints(), built.hints(), "{name}");
            assert_eq!(restored.last_hint_at(), built.last_hint_at(), "{name}");
            assert_eq!(restored.txid(), built.txid(), "{name}");
        } else {
            assert_eq!(restored, built, "{name}");
        }
        assert!(restored.handed_to().is_empty(), "{name}");
    }
}

#[test]
fn version_2_is_version_1_with_four_empty_fields() {
    for (name, build, v1) in VERSION_1 {
        let built = build();
        assert_eq!(hex(&built.to_compact()), version_2_of(v1), "{name}");
        let from_1 = State::from_compact(&unhex(v1)).unwrap();
        let from_2 = State::from_compact(&unhex(&version_2_of(v1))).unwrap();
        assert_eq!(from_2, from_1, "{name}");
        if !matches!(built.word(), Word::Mined(_)) {
            // A version 1 row re-encoded writes the relation.
            assert_eq!(hex(&from_1.to_compact()), version_2_of(v1), "{name}");
        }
    }
}
