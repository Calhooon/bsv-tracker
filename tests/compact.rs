//! The compact State form (bsv-stack-lean #92): the round trip over 256
//! reproducible random states, the refusals, and the restore rule that a
//! restored mined word is suspect until its proof is rechecked.
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
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
use std::cell::Cell;

fn runner() -> TestRunner {
    TestRunner::new_with_rng(
        Config {
            cases: 256,
            failure_persistence: None,
            ..Config::default()
        },
        TestRng::deterministic_rng(RngAlgorithm::ChaCha),
    )
}

/// Every hint shape, text that is and is not an id, and ARC metadata.
fn status(code: u8, height: u32, n: u64) -> Hint {
    let status = match code % 11 {
        0 => HintStatus::Accepted,
        1 => HintStatus::Seen,
        2 => HintStatus::Mined { height },
        3 => HintStatus::StaleBlock,
        4 => HintStatus::Rejected {
            reason: format!("reason \u{e9} {n}"),
        },
        5 => HintStatus::DoubleSpend {
            competitors: vec![hash(n), hash(n).to_ascii_uppercase(), "short".into()],
        },
        6 => HintStatus::OrphanMempool,
        7 => HintStatus::Unknown,
        8 => HintStatus::MempoolPresent {
            source: format!("courier-{n}"),
        },
        9 => {
            let body =
                format!(r#"{{"txStatus":"MINED","blockHeight":{height},"extraInfo":"x{n}"}}"#);
            return Verdict::from_arc_body(&hash(1), body.as_bytes())
                .unwrap()
                .into_hint("arc", n);
        }
        _ => {
            let body = format!(
                r#"{{"txStatus":"SEEN_ON_NETWORK","competingTxs":["{}"]}}"#,
                hash(n)
            );
            return Verdict::from_arc_body(&hash(1), body.as_bytes())
                .unwrap()
                .into_hint("arc", n);
        }
    };
    Hint::new(format!("source-{code}"), status, n)
}

fn input(h: &mut TestHeaders, code: u8, height: u32, n: u64) -> Input {
    h.fault = false;
    let own = proof(&hash(1), height, n);
    let other = proof(&hash(2), height, n);
    match code % 14 {
        0 | 1 => {
            h.insert(&own, u64::from(height));
            Input::Evidence(Evidence::Proof(own))
        }
        2 => Input::Evidence(Evidence::Proof(own)),
        3 => {
            h.insert(&other, u64::from(height));
            Input::Evidence(Evidence::Competitor(other))
        }
        4 => Input::Evidence(Evidence::Chain(reorg(height))),
        5 => Input::Evidence(Evidence::Chain(fork(height))),
        6 => Input::Evidence(Evidence::Chain(split(height))),
        7 => Input::Evidence(Evidence::Verdict(NodeVerdict::Rejected {
            reason: format!("node-{n}"),
        })),
        8 => Input::Evidence(Evidence::Verdict(NodeVerdict::Conflicted {
            competitors: vec![hash(n), format!("Not-An-Id-{n}")],
        })),
        9 => Input::Host(HostAction::Build),
        10 => Input::Host(HostAction::SpendAttempt),
        11 => Input::Host(HostAction::Tick(n + 1_000)),
        12 => Input::Host(HostAction::Abandon),
        _ => Input::Hint(status(n as u8, height, n)),
    }
}

/// A state reached by a random trace, so every field is one the machine writes.
fn states() -> impl Strategy<Value = Vec<(u8, u32, u64, u8)>> {
    prop::collection::vec(
        (any::<u8>(), 1u32..300_000, 0u64..1 << 40, any::<u8>()),
        0..40,
    )
}

fn build(trace: Vec<(u8, u32, u64, u8)>) -> State {
    let mut s = State::new(hash(1));
    let mut h = TestHeaders::default();
    for (code, height, n, hint) in trace {
        let i = input(&mut h, code, height, n);
        let _ = s.step(&params(), &h, i);
        if hint % 3 == 0 {
            let _ = s.step(&params(), &h, Input::Hint(status(hint, height, n)));
        }
    }
    s
}

#[test]
fn compact_round_trip_over_256_random_states() {
    let mined = Cell::new(0);
    runner()
        .run(&states(), |trace| {
            let s = build(trace);
            let bytes = s.to_compact();
            prop_assert_eq!(bytes[0], COMPACT_VERSION);
            let restored = State::from_compact(&bytes).unwrap();
            if let Word::Mined(m) = s.word() {
                mined.set(mined.get() + 1);
                // The word and its record come back; the recheck falls due.
                prop_assert_eq!(restored.word(), s.word());
                prop_assert_eq!(restored.evidence(), s.evidence());
                prop_assert_eq!(restored.evidence(), Some(m.checked()));
                prop_assert!(restored.suspect());
                prop_assert_eq!(restored.reask(), s.reask().or(Some(&Reask::Recheck)));
                prop_assert_eq!(restored.hints(), s.hints());
                prop_assert_eq!(restored.last_hint_at(), s.last_hint_at());
                prop_assert_eq!(restored.txid(), s.txid());
            } else {
                prop_assert_eq!(&restored, &s);
                prop_assert_eq!(restored.to_compact(), bytes);
            }
            // A restored state is a fixed point: its bytes restore it exactly.
            let again = restored.to_compact();
            prop_assert_eq!(State::from_compact(&again).unwrap(), restored.clone());
            prop_assert_eq!(State::from_compact(&again).unwrap().to_compact(), again);
            Ok(())
        })
        .unwrap();
    // Both arms run: the generator reaches mined words and every other kind.
    assert!((1..256).contains(&mined.get()), "mined: {}", mined.get());
}

#[test]
fn every_proper_prefix_is_truncated_and_a_suffix_is_trailing() {
    let s = build(vec![
        (9, 10, 5, 0),
        (13, 10, 9, 3),
        (0, 10, 7, 0),
        (10, 10, 7, 1),
    ]);
    assert!(matches!(s.word(), Word::Mined(_)));
    let bytes = s.to_compact();
    for n in 0..bytes.len() {
        assert_eq!(
            State::from_compact(&bytes[..n]),
            Err(CompactError::Truncated),
            "prefix {n}"
        );
    }
    let mut long = bytes.clone();
    long.push(0);
    assert_eq!(State::from_compact(&long), Err(CompactError::TrailingBytes));
    let mut v3 = bytes;
    v3[0] = 3;
    assert_eq!(
        State::from_compact(&v3),
        Err(CompactError::UnknownVersion(3))
    );
}

#[test]
fn bytes_that_forge_a_mined_word_are_refused() {
    let txid = hash(1);
    let mut s = State::new(&txid);
    let mut h = TestHeaders::default();
    let p = proof(&txid, 10, 2);
    h.insert(&p, 10);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
    let bytes = s.to_compact();
    // version, txid id (tag and 32 bytes), word tag
    assert_eq!(bytes[34], 4);

    // A mined word with no stored proof.
    let mut no_proof = State::new(&txid).to_compact();
    no_proof[34] = 4;
    assert_eq!(
        State::from_compact(&no_proof),
        Err(CompactError::Malformed("mined word with no stored proof"))
    );

    // The stored header's root altered: the record no longer holds together.
    let checked = s.evidence().unwrap();
    let root = hex_bytes(&checked.header().merkle_root);
    let at = find(&bytes, &root, 0);
    let mut forged = bytes.clone();
    forged[at] ^= 1;
    assert_eq!(
        State::from_compact(&forged),
        Err(CompactError::Evidence(CheckError::RootMismatch(10)))
    );

    // The stored proof bound to another transaction.
    let other = hash(9);
    let mut s9 = State::new(&other);
    let p9 = proof(&other, 10, 2);
    let mut h9 = TestHeaders::default();
    h9.insert(&p9, 10);
    step(&mut s9, &h9, Input::Evidence(Evidence::Proof(p9)));
    let mut swapped = s9.to_compact();
    swapped[2..34].copy_from_slice(&hex_bytes(&txid));
    assert_eq!(
        State::from_compact(&swapped),
        Err(CompactError::Evidence(CheckError::TxidMismatch))
    );

    // An unknown tag and a non-canonical varint.
    let mut tag = State::new(&txid).to_compact();
    tag[34] = 10;
    assert_eq!(
        State::from_compact(&tag),
        Err(CompactError::Malformed("word tag"))
    );
    let mut fresh = State::new(&txid).to_compact();
    // Version 2: the hints, the last hint time and the re-ask, then the
    // instances, the competitors and the prior word.
    let hints_at = fresh.len() - 6;
    assert_eq!(&fresh[hints_at..], &[0, 0, 0, 0, 0, 0]);
    fresh.splice(hints_at..hints_at + 1, [0x80, 0x00]);
    assert_eq!(
        State::from_compact(&fresh),
        Err(CompactError::Malformed("varint not shortest"))
    );
}

#[test]
fn a_restored_mined_word_is_rechecked_before_it_counts() {
    let txid = hash(1);
    let mut h = TestHeaders::default();
    let p = proof(&txid, 10, 2);
    h.insert(&p, 10);
    let mut tracker = Tracker::new(params());
    tracker
        .apply(&txid, &h, Input::Evidence(Evidence::Proof(p)))
        .unwrap();
    let bytes = tracker.get(&txid).unwrap().to_compact();

    // A fresh host (a Worker's next request) restores it: suspect, recheck due.
    let mut host = Tracker::new(params());
    let restored = State::from_compact(&bytes).unwrap();
    mined(&restored, 10);
    assert!(restored.suspect());
    assert_eq!(restored.reask(), Some(&Reask::Recheck));
    assert_eq!(host.restore(restored.clone()), None);
    assert_eq!(
        host.restore(restored),
        Some(State::from_compact(&bytes).unwrap())
    );

    // The next tip reaches it through the suspect set; the headers agree.
    let updates = host.on_chain(&h, tip(11, 90));
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].result, Ok(()));
    let after = host.get(&txid).unwrap();
    mined(after, 10);
    assert!(!after.suspect());
    assert_eq!(after.reask(), None);

    // Restored again while the chain moved under it: the recheck makes it stale.
    let mut moved = TestHeaders::default();
    moved.insert(&proof(&txid, 10, 3), 77);
    let mut s = State::from_compact(&bytes).unwrap();
    step(&mut s, &moved, Input::Evidence(Evidence::Recheck));
    assert_eq!(s.word(), &Word::Stale);
    assert_eq!(s.reask(), Some(&Reask::Reorg { fork_height: 10 }));
    assert_eq!(
        s.evidence(),
        State::from_compact(&bytes).unwrap().evidence()
    );
}

/// Version 2's refusals (for #56): each byte string the encoder never writes.
#[test]
fn version_2_refusals() {
    let h = TestHeaders::default();
    let txid = hash(1);
    let mut s = State::new(&txid);
    let hand = |instance: &str, at| {
        Input::Host(HostAction::HandOver {
            instance: instance.into(),
            at,
        })
    };
    step(&mut s, &h, hand("a", 5));
    step(&mut s, &h, hand("b", 6));
    step(&mut s, &h, Input::Host(HostAction::Tick(65)));
    assert_eq!(s.reask(), Some(&Reask::Reannounce));
    let bytes = s.to_compact();
    assert_eq!(State::from_compact(&bytes).unwrap(), s);
    // version, txid (34 bytes), word tag 9 handed, evidence 0, suspect 0,
    // hints 0, last hint 0, re-ask 1 and tag 7, two instances ("a" at 5,
    // "b" at 6), no competitor, no prior word.
    assert_eq!(bytes[34], 9);
    let tail = [0, 0, 0, 0, 1, 7, 2, 1, b'a', 5, 1, b'b', 6, 0, 0];
    assert_eq!(&bytes[35..], &tail);
    let malformed = |b: &[u8], why| {
        assert_eq!(State::from_compact(b), Err(CompactError::Malformed(why)));
    };

    // Under version 1, word tag 9 and re-ask tag 7 do not exist.
    let mut v1 = bytes[..35].to_vec();
    v1[0] = 1;
    v1.extend_from_slice(&[0, 0, 0, 0, 0]);
    malformed(&v1, "word tag");
    let mut v1 = State::new(&txid).to_compact();
    v1.truncate(v1.len() - 4);
    v1[0] = 1;
    v1.extend_from_slice(&[1, 7]);
    malformed(&v1, "re-ask tag");
    // Re-ask tag 8 is not read by this release.
    let mut eight = bytes.clone();
    eight[40] = 8;
    malformed(&eight, "re-ask tag");

    // A handed word with no instance.
    let mut none = bytes[..41].to_vec();
    none.extend_from_slice(&[0, 0, 0]);
    malformed(&none, "handed word with no instance");
    // An instance named twice.
    let mut twice = bytes.clone();
    twice[46] = b'a';
    malformed(&twice, "instance named twice");
    // Instances beside a word that holds none.
    for tag in [0, 1, 2, 3, 5, 8] {
        let mut beside = bytes.clone();
        beside[34] = tag;
        malformed(&beside, "instances beside a word that holds none");
    }
    // The record of competitors and the prior word are empty in this release.
    let mut rivals = bytes.clone();
    rivals[48] = 1;
    malformed(&rivals, "competitors");
    let mut prior = bytes.clone();
    prior[49] = 1;
    malformed(&prior, "prior word");
    // A version 1 row with the three fields is trailing.
    let mut long = State::new(&txid).to_compact();
    long[0] = 1;
    assert_eq!(State::from_compact(&long), Err(CompactError::TrailingBytes));
}

/// A conflict that began from `handed` keeps its instances, and the bytes
/// hold them (`Tracker.verdictHanded`).
#[test]
fn conflicted_from_handed_keeps_its_instances_through_the_bytes() {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(
        &mut s,
        &h,
        Input::Host(HostAction::HandOver {
            instance: "a".into(),
            at: 5,
        }),
    );
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Verdict(NodeVerdict::Conflicted {
            competitors: vec![hash(9)],
        })),
    );
    assert_eq!(s.handed_to(), &[HandedTo::new("a", 5)]);
    assert_eq!(State::from_compact(&s.to_compact()).unwrap(), s);
}

/// The measured length, version 2's last field: `0`, or `1` and a varint. A
/// transaction of exactly 64 bytes is never taken as proven by a merkle path,
/// so that length beside a stored proof is refused.
#[test]
fn tx_length_field() {
    let txid = hash(1);
    let fresh = State::new(&txid).to_compact();
    let with = |row: &[u8], field: &[u8]| {
        let mut bytes = row[..row.len() - 1].to_vec();
        bytes.extend_from_slice(field);
        bytes
    };
    let read = |bytes: &[u8]| State::from_compact(bytes).map(|s| s.tx_len());

    // `00`: the host never handed the bytes.
    assert_eq!(fresh.last(), Some(&0));
    assert_eq!(read(&fresh), Ok(None));
    // `01` and a varint: a measure, written back as it was read.
    for (field, len) in [
        (&[1, 0x3f][..], 63),
        (&[1, 0x41][..], 65),
        (&[1, 0xbf, 0x01][..], 191),
        // A length of zero is a measure, never absence.
        (&[1, 0x00][..], 0),
        // 64 with no stored proof: a state measured at 64.
        (&[1, 0x40][..], AMBIGUOUS_TX_LEN),
    ] {
        let bytes = with(&fresh, field);
        assert_eq!(read(&bytes), Ok(Some(len)));
        let restored = State::from_compact(&bytes).unwrap();
        assert_eq!(restored.to_compact(), bytes);
        assert_ne!(restored, State::new(&txid));
    }

    // Beside a stored proof: 63 is read, 64 is refused.
    let mut h = TestHeaders::default();
    let p = proof(&txid, 10, 2);
    h.insert(&p, 10);
    let mut s = State::new(&txid);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
    let mined_row = s.to_compact();
    assert_eq!(read(&mined_row), Ok(None));
    assert_eq!(read(&with(&mined_row, &[1, 0x3f])), Ok(Some(63)));
    assert_eq!(
        read(&with(&mined_row, &[1, 0x40])),
        Err(CompactError::Evidence(CheckError::AmbiguousLength))
    );
    // The same for a stale word, which keeps its record.
    let mut moved = h.clone();
    moved.move_root(10, 77);
    step(&mut s, &moved, Input::Evidence(Evidence::Recheck));
    assert_eq!(s.word(), &Word::Stale);
    assert_eq!(
        read(&with(&s.to_compact(), &[1, 0x40])),
        Err(CompactError::Evidence(CheckError::AmbiguousLength))
    );

    // The refusals of the field's own bytes.
    assert_eq!(
        read(&with(&fresh, &[2])),
        Err(CompactError::Malformed("flag"))
    );
    assert_eq!(
        read(&with(&fresh, &[1, 0x80, 0x00])),
        Err(CompactError::Malformed("varint not shortest"))
    );
    // Version 2 bytes without the field, or with its flag alone.
    assert_eq!(read(&with(&fresh, &[])), Err(CompactError::Truncated));
    assert_eq!(read(&with(&fresh, &[1])), Err(CompactError::Truncated));
    assert_eq!(
        read(&with(&fresh, &[1, 0x3f, 0])),
        Err(CompactError::TrailingBytes)
    );

    // A version 1 row has no such field and reads unmeasured.
    let mut v1 = fresh[..fresh.len() - 4].to_vec();
    v1[0] = 1;
    assert_eq!(read(&v1), Ok(None));
}

fn hex_bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn find(haystack: &[u8], needle: &[u8], from: usize) -> usize {
    from + haystack[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .unwrap()
}
