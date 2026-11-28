//! The length rule as randomized statements (GHSA-jjhc-7hvh-c3hm, BRC-237):
//! a transaction of exactly 64 bytes is never taken as proven by a merkle
//! path. Eight properties named I1 to I8 for the theorems the Lean will hold,
//! and two over `Tracker::restore`. 256 reproducible traces each, from the
//! explicit ChaCha seed with no failure persistence, network, clock or
//! storage, as `tests/properties.rs`.
//!
//! Ahead of the Lean: bsv-stack-lean `lean/Tracker.lean` at `901e9dfa` has no
//! measured length yet (its name there will be `State.txLen`); the rule's
//! words are its `docs/DECISIONS-2026-10-19.md` 32.13 at that commit. The
//! theorem names below are proposals.
//!
//! The trace: a subject (a txid alone, or bytes of 63, 64, 65 or 191, born
//! measured or born from its txid), held by a `Tracker` beside a mined
//! bystander, and acts drawn among: its own proof built from its bytes, stated
//! at its length, stated at another, stated at 64, or taken out of a record;
//! a competitor's proof from bytes, stated, or out of a record; the measure
//! (`Tracker::track_bytes`); a restore of the state's own version 2 bytes, of
//! its version 1 bytes, of a row born apart and mined on a stated length, of
//! a row whose length field is set by hand, or of a measured birth; and every
//! other input: a host act, a hint, a chain event, a recheck, a verdict.
// The lint bar binds the library; a test may unwrap, and its fixtures fail loudly
// (bsv-stack-lean #134).
#![allow(
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod common;
use bsv_tracker::doubts::{self, Doubts};
use bsv_tracker::*;
use common::*;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestCaseError, TestRng, TestRunner};
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

/// The lengths a stated proof or a hand-set field is drawn among.
const OTHER: [u64; 6] = [0, 63, 64, 65, 190, 191];
/// The bystander: a txid nobody has bytes for, mined at height 5.
const BYSTANDER: u64 = 9;

/// A tracker holding the subject at its birth beside the mined bystander,
/// and the headers that carry the bystander's root.
fn tracker_of(s: &Subject) -> (Tracker, TestHeaders) {
    let mut t = Tracker::new(params());
    let mut h = TestHeaders::default();
    let by = hash(BYSTANDER);
    let p = proof(&by, 5, 3);
    h.insert(&p, 5);
    t.track(&by);
    t.apply(&by, &h, Input::Evidence(Evidence::Proof(p)))
        .unwrap();
    match &s.bytes {
        Some(b) if s.born_measured => {
            t.track_bytes(b).unwrap();
        }
        _ => {
            t.track(&s.txid);
        }
    }
    (t, h)
}

/// A proof stated for `txid` at `len`, at `height` with `sibling`.
fn stated(txid: &str, len: u64, height: u32, sibling: u64) -> Result<Proof, CheckError> {
    Proof::with_stated_len(txid, len, path_of(txid, height, sibling))
}

/// The proof kept in the record of a state born from `txid` and mined on a
/// stated `len`: it carries no length of its own.
fn record(txid: &str, len: u64, height: u32, sibling: u64) -> Proof {
    let p = stated(txid, len, height, sibling).unwrap();
    let mut h = TestHeaders::default();
    h.insert(&p, u64::from(height));
    let mut s = State::new(txid);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
    s.evidence().unwrap().proof().clone()
}

/// The version 1 bytes of a state version 1 can hold (no instance, no
/// measure): version 2's with the opening byte `01` and its last four
/// fields, each empty, cut.
fn version_1(s: &State) -> Option<Vec<u8>> {
    if !s.handed_to().is_empty() || s.tx_len().is_some() {
        return None;
    }
    let mut b = s.to_compact();
    assert_eq!(b[b.len() - 4..], [0, 0, 0, 0]);
    b.truncate(b.len() - 4);
    b[0] = 1;
    Some(b)
}

/// The version 2 bytes of an unmeasured state with its length field set by
/// hand to `len`.
fn with_len_field(s: &State, len: u64) -> Option<Vec<u8>> {
    if s.tx_len().is_some() {
        return None;
    }
    let mut b = s.to_compact();
    assert_eq!(b.pop(), Some(0));
    b.push(1);
    let mut n = len;
    loop {
        let byte = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            b.push(byte);
            break;
        }
        b.push(byte | 0x80);
    }
    Some(b)
}

/// What an act was, as the properties read it.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Act {
    /// The subject's own proof, built from its bytes.
    ProofFromBytes,
    /// The subject's own proof, stated at this length.
    ProofStated(u64),
    /// The subject's own proof, taken out of a record.
    ProofFromRecord,
    /// A competitor's proof built from the competitor's bytes.
    CompetitorFromBytes,
    /// A competitor's proof stated, or out of a record.
    CompetitorUnmeasured,
    /// The subject's bytes handed to the tracker.
    Measure,
    /// A restore of a row whose own length is this.
    Restore(Option<u64>),
    /// Anything that is neither a proof, a measure nor a restore, or an act
    /// that could not be built (and then did nothing).
    Other,
}

/// One act's record: the subject's state on both sides of it, its answer,
/// and the tracker after it.
struct Seen<'a> {
    subject: &'a Subject,
    act: Act,
    before: State,
    after: State,
    result: Result<(), CheckError>,
    tracker: &'a Tracker,
}

fn trace() -> impl Strategy<Value = (u8, Vec<(u8, u32, u64, u8)>)> {
    (
        any::<u8>(),
        prop::collection::vec((0u8..20, 1u32..12, 20u64..40, any::<u8>()), 1..60),
    )
}

fn hint_of(code: u8, height: u32, observed: u64) -> Input {
    hint(
        match code % 6 {
            0 => HintStatus::Accepted,
            1 => HintStatus::Seen,
            2 => HintStatus::Mined { height },
            3 => HintStatus::DoubleSpend {
                competitors: vec![hash(2)],
            },
            4 => HintStatus::Rejected {
                reason: format!("reason-{code}"),
            },
            _ => HintStatus::Unknown,
        },
        observed,
    )
}

/// Play one act on the tracker; answer what it was and what it returned.
fn play(
    s: &Subject,
    t: &mut Tracker,
    h: &mut TestHeaders,
    (code, height, sibling, aux): (u8, u32, u64, u8),
) -> (Act, Result<(), CheckError>) {
    h.fault = false;
    let txid = s.txid.clone();
    let apply =
        |t: &mut Tracker, h: &TestHeaders, e: Evidence| t.apply(&txid, h, Input::Evidence(e));
    let carry = |h: &mut TestHeaders, p: &Proof| {
        if aux % 4 == 0 {
            // Headers that cannot answer.
            h.fault = true;
        } else {
            h.insert(p, u64::from(height));
        }
    };
    match code {
        0 | 1 => {
            let Some(bytes) = &s.bytes else {
                return (Act::Other, Ok(()));
            };
            let path = path_of(&txid, height, sibling);
            match Proof::new(bytes, path) {
                Err(e) => {
                    assert_eq!(bytes.len() as u64, AMBIGUOUS_TX_LEN);
                    assert_eq!(e, CheckError::AmbiguousLength);
                    (Act::Other, Ok(()))
                }
                Ok(p) => {
                    carry(h, &p);
                    (Act::ProofFromBytes, apply(t, h, Evidence::Proof(p)))
                }
            }
        }
        2 | 3 => {
            // Stated at its own length, or at a length drawn among `OTHER`.
            let len = if code == 2 {
                s.len()
            } else {
                OTHER[usize::from(aux) % OTHER.len()]
            };
            match stated(&txid, len, height, sibling) {
                Err(e) => {
                    assert_eq!(len, AMBIGUOUS_TX_LEN);
                    assert_eq!(e, CheckError::AmbiguousLength);
                    (Act::Other, Ok(()))
                }
                Ok(p) => {
                    carry(h, &p);
                    (Act::ProofStated(len), apply(t, h, Evidence::Proof(p)))
                }
            }
        }
        4 => {
            // A lie: a length that is not the subject's, never 64.
            let p = stated(&txid, s.lie(), height, sibling).unwrap();
            carry(h, &p);
            (Act::ProofStated(s.lie()), apply(t, h, Evidence::Proof(p)))
        }
        5 => {
            let p = record(&txid, s.lie(), height, sibling);
            carry(h, &p);
            (Act::ProofFromRecord, apply(t, h, Evidence::Proof(p)))
        }
        6 => {
            let p = competitor(2, height, sibling);
            carry(h, &p);
            (
                Act::CompetitorFromBytes,
                apply(t, h, Evidence::Competitor(p)),
            )
        }
        7 | 8 => {
            let p = if code == 7 {
                stated(&hash(2), 191, height, sibling).unwrap()
            } else {
                record(&hash(2), 191, height, sibling)
            };
            carry(h, &p);
            (
                Act::CompetitorUnmeasured,
                apply(t, h, Evidence::Competitor(p)),
            )
        }
        9 => match &s.bytes {
            Some(bytes) => (Act::Measure, t.track_bytes(bytes).map(|_| ())),
            None => (Act::Other, Ok(())),
        },
        10..=14 => {
            let held = t.get(&txid).unwrap().clone();
            let bytes = match code {
                10 => Some(held.to_compact()),
                11 => version_1(&held),
                12 => {
                    // A row born apart from its txid and mined on a lie.
                    let p = stated(&txid, s.lie(), height, sibling).unwrap();
                    h.insert(&p, u64::from(height));
                    let mut row = State::new(&txid);
                    step(&mut row, h, Input::Evidence(Evidence::Proof(p)));
                    if aux % 2 == 0 {
                        version_1(&row)
                    } else {
                        Some(row.to_compact())
                    }
                }
                13 => with_len_field(&held, OTHER[usize::from(aux) % OTHER.len()]),
                _ => s.bytes.as_ref().map(|b| State::measured(b).to_compact()),
            };
            let Some(bytes) = bytes else {
                return (Act::Other, Ok(()));
            };
            match State::from_compact(&bytes) {
                Err(e) => {
                    // The one refusal a hand-set field meets: 64 beside a
                    // stored proof.
                    assert_eq!(code, 13);
                    assert_eq!(e, CompactError::Evidence(CheckError::AmbiguousLength));
                    assert!(held.evidence().is_some());
                    (Act::Other, Ok(()))
                }
                Ok(row) => {
                    let len = row.tx_len();
                    (Act::Restore(len), t.restore(row).map(|_| ()))
                }
            }
        }
        15 => (
            Act::Other,
            t.apply(
                &txid,
                h,
                Input::Host(match aux % 4 {
                    0 => HostAction::Build,
                    1 => HostAction::Abandon,
                    2 => HostAction::SpendAttempt,
                    _ => HostAction::Tick(sibling),
                }),
            ),
        ),
        16 => (
            Act::Other,
            t.apply(&txid, h, hint_of(aux, height, u64::from(aux))),
        ),
        17 => {
            if aux % 2 == 0 {
                h.move_root(height, u64::from(height) + 500);
            }
            let event = match aux % 4 {
                0 | 1 => reorg(height),
                2 => fork(height),
                _ => tip(height, 90),
            };
            let _ = t.on_chain(h, event);
            (Act::Other, Ok(()))
        }
        18 => (Act::Other, apply(t, h, Evidence::Recheck)),
        _ => (
            Act::Other,
            apply(
                t,
                h,
                Evidence::Verdict(if aux % 2 == 0 {
                    NodeVerdict::Conflicted {
                        competitors: vec![hash(2)],
                    }
                } else {
                    NodeVerdict::Rejected {
                        reason: "node".into(),
                    }
                }),
            ),
        ),
    }
}

/// Run 256 traces and hand every act's record to `check`.
fn each_act(check: impl Fn(&Seen<'_>) -> Result<(), TestCaseError>) {
    runner()
        .run(&trace(), |(kind, acts)| {
            let subject = Subject::of(kind);
            let (mut t, mut h) = tracker_of(&subject);
            for a in acts {
                let before = t.get(&subject.txid).unwrap().clone();
                let (act, result) = play(&subject, &mut t, &mut h, a);
                let after = t.get(&subject.txid).unwrap().clone();
                check(&Seen {
                    subject: &subject,
                    act,
                    before,
                    after,
                    result,
                    tracker: &t,
                })?;
            }
            Ok(())
        })
        .unwrap();
}

/// Whether a state holds a proof of its own transaction: a record, or a word
/// only a proof writes.
fn holds_a_proof(s: &State) -> bool {
    s.evidence().is_some() || matches!(s.word(), Word::Mined(_) | Word::Stale)
}

/// I1 `ambiguous_never_mined`, and the captain's row (b): along every trace,
/// whatever the road (a proof by either constructor, a record's proof, a
/// competitor's, the measure, a restore through either version of the
/// compact form, a hand-set length field), a state whose length reads 64
/// does not read `mined` and holds no record.
#[test]
fn ambiguous_never_mined() {
    let reached = Cell::new(0u32);
    each_act(|e| {
        if e.after.tx_len() == Some(AMBIGUOUS_TX_LEN) {
            prop_assert!(!matches!(e.after.word(), Word::Mined(_)), "{:?}", e.act);
            prop_assert!(e.after.evidence().is_none(), "{:?}", e.act);
            prop_assert!(e.after.word() != &Word::Stale, "{:?}", e.act);
            if holds_a_proof(&e.before) {
                reached.set(reached.get() + 1);
            }
        }
        Ok(())
    });
    // The fault road is reached: a state that held a proof is measured at 64.
    assert!(reached.get() > 0);
}

/// I2 `txLen_once`: a length, once held, is held after every later act,
/// restores and refusals included.
#[test]
#[allow(non_snake_case)]
fn txLen_once() {
    each_act(|e| {
        if let Some(n) = e.before.tx_len() {
            prop_assert_eq!(e.after.tx_len(), Some(n), "{:?}", e.act);
        }
        Ok(())
    });
}

/// I3 `txLen_only_by_bytes`: the length is written only by bytes the crate
/// hashed: the measure, a proof built from the subject's bytes, or a restore
/// of a row that carries a length. A stated length, a record's proof, a
/// competitor and every other input write none.
#[test]
#[allow(non_snake_case)]
fn txLen_only_by_bytes() {
    each_act(|e| {
        if e.after.tx_len() != e.before.tx_len() {
            match &e.act {
                Act::Measure | Act::ProofFromBytes => {
                    prop_assert_eq!(e.after.tx_len(), Some(e.subject.len()));
                }
                Act::Restore(Some(n)) => prop_assert_eq!(e.after.tx_len(), Some(*n)),
                other => prop_assert!(false, "{other:?} wrote the length"),
            }
        }
        Ok(())
    });
}

/// I4 `mined_read_a_length`: a step that writes a fresh `mined` word read a
/// length, the state's measure or, for an unmeasured state, the proof's own;
/// after it the length is absent exactly when it was stated. A record's
/// proof never mines an unmeasured state.
#[test]
#[allow(non_snake_case)]
fn mined_read_a_length() {
    let both = (Cell::new(0u32), Cell::new(0u32));
    each_act(|e| {
        let fresh =
            matches!(e.after.word(), Word::Mined(_)) && e.after.evidence() != e.before.evidence();
        let a_proof = matches!(
            e.act,
            Act::ProofFromBytes | Act::ProofStated(_) | Act::ProofFromRecord
        );
        if e.act == Act::ProofFromRecord && e.before.tx_len().is_none() {
            prop_assert_eq!(&e.result, &Err(CheckError::Unmeasured));
        }
        if !fresh || !a_proof {
            if fresh {
                prop_assert!(matches!(e.act, Act::Restore(_)), "{:?}", e.act);
            }
            return Ok(());
        }
        prop_assert_eq!(&e.result, &Ok(()));
        match (e.before.tx_len(), &e.act) {
            (Some(n), Act::ProofStated(m)) => {
                prop_assert_eq!(n, *m);
                prop_assert_eq!(e.after.tx_len(), Some(n));
            }
            (Some(n), _) => prop_assert_eq!(e.after.tx_len(), Some(n)),
            (None, Act::ProofFromBytes) => {
                both.0.set(both.0.get() + 1);
                prop_assert_eq!(e.after.tx_len(), Some(e.subject.len()));
            }
            (None, Act::ProofStated(_)) => {
                both.1.set(both.1.get() + 1);
                prop_assert_eq!(e.after.tx_len(), None);
            }
            (None, other) => prop_assert!(false, "{other:?} mined an unmeasured state"),
        }
        Ok(())
    });
    assert!(both.0.get() > 0 && both.1.get() > 0, "{both:?}");
}

/// I5 `mismatch_refused`: a measured state takes no proof of another
/// length. The answer names both lengths, the word and the record are kept
/// and the proof is asked again; a state measured at 64 answers
/// `AmbiguousLength` and is left whole.
#[test]
fn mismatch_refused() {
    let refused = Cell::new(0u32);
    each_act(|e| {
        let (Some(n), Act::ProofStated(m)) = (e.before.tx_len(), &e.act) else {
            return Ok(());
        };
        if n == AMBIGUOUS_TX_LEN {
            prop_assert_eq!(&e.result, &Err(CheckError::AmbiguousLength));
            prop_assert_eq!(&e.after, &e.before);
        } else if n != *m {
            refused.set(refused.get() + 1);
            prop_assert_eq!(
                &e.result,
                &Err(CheckError::LengthMismatch {
                    measured: n,
                    offered: *m
                })
            );
            prop_assert_eq!(e.after.word(), e.before.word());
            prop_assert_eq!(e.after.evidence(), e.before.evidence());
            prop_assert!(
                matches!(e.after.reask(), Some(Reask::ProofFailed { .. })),
                "re-ask: {:?}",
                e.after.reask()
            );
        }
        Ok(())
    });
    assert!(refused.get() > 0);
}

/// I6 `competitor_needs_bytes`: no `conflicted` word is written from a
/// competitor's proof that was not built from the competitor's bytes: the
/// door answers `Unmeasured` and keeps the word and the record. A
/// competitor is judged by its own length, never by the subject's measure.
#[test]
fn competitor_needs_bytes() {
    let taken_at_64 = Cell::new(0u32);
    each_act(|e| {
        match e.act {
            Act::CompetitorUnmeasured => {
                prop_assert_eq!(&e.result, &Err(CheckError::Unmeasured));
                prop_assert_eq!(e.after.word(), e.before.word());
                prop_assert_eq!(e.after.evidence(), e.before.evidence());
            }
            Act::CompetitorFromBytes => {
                prop_assert!(
                    !matches!(
                        e.result,
                        Err(CheckError::Unmeasured
                            | CheckError::AmbiguousLength
                            | CheckError::LengthMismatch { .. })
                    ),
                    "competitor from bytes: {:?}",
                    e.result
                );
                if e.result.is_ok() && e.before.tx_len() == Some(AMBIGUOUS_TX_LEN) {
                    taken_at_64.set(taken_at_64.get() + 1);
                }
            }
            _ => {}
        }
        // A `conflicted` word newly written names a competitor only through
        // a proof from bytes, a verdict or a restore.
        if let Word::Conflicted { competitors } = e.after.word() {
            let was = match e.before.word() {
                Word::Conflicted { competitors } => competitors.clone(),
                _ => vec![],
            };
            if competitors != &was {
                prop_assert!(
                    matches!(
                        e.act,
                        Act::CompetitorFromBytes | Act::Other | Act::Restore(_)
                    ),
                    "{:?}",
                    e.act
                );
            }
        }
        Ok(())
    });
    assert!(taken_at_64.get() > 0);
}

/// I7 `measure_moves_no_word`: off 64, the measure changes the length alone.
/// At 64 on a state that held a proof it is the fault road (K4): the fault
/// is returned AND the proof is unsaid; at 64 on a state that held none, it
/// changes the length alone too. A state that holds another length refuses
/// the measure, `LengthMismatch`, and is left whole.
#[test]
fn measure_moves_no_word() {
    let fault = Cell::new(0u32);
    each_act(|e| {
        if e.act != Act::Measure {
            return Ok(());
        }
        let (b, a) = (&e.before, &e.after);
        let len = e.subject.len();
        // A state that holds another length (a row whose length field was
        // set by hand) refuses the measure and is left whole.
        if let Some(held) = b.tx_len().filter(|&held| held != len) {
            prop_assert_eq!(
                &e.result,
                &Err(CheckError::LengthMismatch {
                    measured: held,
                    offered: len
                })
            );
            prop_assert_eq!(a, b);
            return Ok(());
        }
        let unsaid = len == AMBIGUOUS_TX_LEN && b.tx_len().is_none() && holds_a_proof(b);
        prop_assert_eq!(a.tx_len(), Some(len));
        if unsaid {
            fault.set(fault.get() + 1);
            prop_assert_eq!(&e.result, &Err(CheckError::AmbiguousLength));
            prop_assert!(a.evidence().is_none());
            match b.word() {
                Word::Mined(_) | Word::Stale => {
                    prop_assert_eq!(a.word(), &Word::Unknown);
                    prop_assert!(!a.suspect());
                    prop_assert_eq!(a.reask(), None);
                }
                w => {
                    prop_assert_eq!(a.word(), w);
                    prop_assert_eq!(a.reask(), b.reask());
                }
            }
        } else {
            prop_assert_eq!(&e.result, &Ok(()));
            prop_assert_eq!(a.word(), b.word());
            prop_assert_eq!(a.evidence(), b.evidence());
            prop_assert_eq!(a.suspect(), b.suspect());
            prop_assert_eq!(a.reask(), b.reask());
            prop_assert_eq!(a.hints(), b.hints());
            prop_assert_eq!(a.last_hint_at(), b.last_hint_at());
            prop_assert_eq!(a.handed_to(), b.handed_to());
        }
        Ok(())
    });
    assert!(fault.get() > 0);
}

/// I8 `record_carries_no_length`: a proof taken out of a record, built by
/// steps or restored from its bytes, is refused `Unmeasured` by
/// `Headers::check`, by `doubts::may_enter` and by an unmeasured state, and
/// is taken by a state measured at its own length (refused `AmbiguousLength`
/// by one measured at 64).
#[test]
#[allow(non_snake_case)]
fn record_carries_no_length() {
    let taken = Cell::new(0u32);
    each_act(|e| {
        let Some(checked) = e.after.evidence() else {
            return Ok(());
        };
        let restored = State::from_compact(&e.after.to_compact()).unwrap();
        for p in [
            checked.proof().clone(),
            restored.evidence().unwrap().proof().clone(),
        ] {
            let mut h = TestHeaders::default();
            h.insert(&p, 1);
            h.tip = p.height() + 1;
            prop_assert_eq!(h.check(p.clone()), Err(CheckError::Unmeasured));
            let ds = Doubts::cold(h.tip);
            prop_assert_eq!(doubts::may_enter(&h, &ds, &p), Err(CheckError::Unmeasured));
            let mut bare = State::new(&e.subject.txid);
            let answer = bare.step(&params(), &h, Input::Evidence(Evidence::Proof(p.clone())));
            prop_assert_eq!(answer, Err(CheckError::Unmeasured));
            if let Some(bytes) = &e.subject.bytes {
                let mut measured = State::measured(bytes);
                let answer = measured.step(&params(), &h, Input::Evidence(Evidence::Proof(p)));
                if bytes.len() as u64 == AMBIGUOUS_TX_LEN {
                    prop_assert_eq!(answer, Err(CheckError::AmbiguousLength));
                } else {
                    taken.set(taken.get() + 1);
                    prop_assert_eq!(answer, Ok(()));
                    prop_assert!(matches!(measured.word(), Word::Mined(_)));
                }
            }
        }
        Ok(())
    });
    assert!(taken.get() > 0);
}

/// What the tracker's indexes say of `txid`, read through its public doors
/// on a copy: the heights a fork reaches it at, the header hashes an
/// invalidation reaches it by, and whether a new tip reaches it (the suspect
/// set).
fn index_of(
    t: &Tracker,
    txid: &str,
    heights: &[u32],
    hashes: &[String],
) -> (Vec<u32>, Vec<String>, bool) {
    let h = TestHeaders::default();
    let reaches = |event: ChainEvent| t.clone().on_chain(&h, event).iter().any(|u| u.txid == txid);
    let at: Vec<u32> = heights
        .iter()
        .copied()
        .filter(|&height| reaches(fork(height)) && !reaches(fork(height + 1)))
        .collect();
    let by: Vec<String> = hashes
        .iter()
        .filter(|hash| {
            reaches(ChainEvent::Invalidated {
                block_hash: (*hash).clone(),
            })
        })
        .cloned()
        .collect();
    (at, by, reaches(tip(1, 90)))
}

/// What the indexes must say of a state: its word's height, its `mined`
/// word's header hash, and its suspect mark.
fn index_wanted(s: &State) -> (Vec<u32>, Vec<String>, bool) {
    let height = s.word().height().into_iter().collect();
    let header = match s.word() {
        Word::Mined(m) => vec![m.checked().header().hash.clone()],
        _ => vec![],
    };
    (height, header, s.suspect())
}

/// Every height and header hash a trace can write, to ask the indexes by.
fn probes(states: &[&State]) -> (Vec<u32>, Vec<String>) {
    let mut heights: Vec<u32> = (0..13).collect();
    let mut hashes: Vec<String> = (0..13).map(hash).collect();
    for s in states {
        if let Some(c) = s.evidence() {
            heights.push(c.height());
            hashes.push(c.header().hash.clone());
        }
    }
    heights.sort_unstable();
    heights.dedup();
    hashes.sort();
    hashes.dedup();
    (heights, hashes)
}

/// The captain's row (a): for every pair of a held state and a restored row,
/// the tracker's index after `Tracker::restore` describes the state it then
/// holds, on the `Ok` road and on both `Err` roads (`AmbiguousLength`, the
/// row held unsaid; `LengthMismatch`, nothing moved). The bystander's index
/// is untouched by every road.
#[test]
fn restore_leaves_the_index_describing_the_held_state() {
    let roads = [Cell::new(0u32), Cell::new(0u32), Cell::new(0u32)];
    let pair = (
        any::<u8>(),
        prop::collection::vec((0u8..20, 1u32..12, 20u64..40, any::<u8>()), 0..30),
        0u8..6,
        prop::collection::vec((0u8..9, 1u32..12, 20u64..40, any::<u8>()), 0..12),
        any::<u8>(),
    );
    runner()
        .run(&pair, |(kind, held_acts, row_kind, row_acts, aux)| {
            let s = Subject::of(kind);
            let (mut t, mut h) = tracker_of(&s);
            for a in held_acts {
                let _ = play(&s, &mut t, &mut h, a);
            }
            // The row: born from its txid or measured, stepped by its own
            // proofs and inputs on a tracker of its own, then through bytes.
            let mut rt = Tracker::new(params());
            match (&s.bytes, row_kind % 2) {
                (Some(b), 0) => {
                    rt.track_bytes(b).unwrap();
                }
                _ => {
                    rt.track(&s.txid);
                }
            }
            let mut rh = TestHeaders::default();
            for a in row_acts {
                let _ = play(&s, &mut rt, &mut rh, a);
            }
            let row = rt.get(&s.txid).unwrap().clone();
            let bytes = match row_kind / 2 {
                0 => Some(row.to_compact()),
                1 => version_1(&row),
                _ => with_len_field(&row, OTHER[usize::from(aux) % OTHER.len()]),
            };
            let Some(row) = bytes.and_then(|b| State::from_compact(&b).ok()) else {
                return Ok(());
            };
            let held = t.get(&s.txid).unwrap().clone();
            let by = hash(BYSTANDER);
            let (heights, hashes) = probes(&[&held, &row, t.get(&by).unwrap()]);
            let bystander = index_of(&t, &by, &heights, &hashes);
            let answer = t.restore(row.clone());
            let now = t.get(&s.txid).unwrap();
            prop_assert_eq!(index_of(&t, &s.txid, &heights, &hashes), index_wanted(now));
            prop_assert_eq!(index_of(&t, &by, &heights, &hashes), bystander);
            match answer {
                Ok(previous) => {
                    roads[0].set(roads[0].get() + 1);
                    prop_assert_eq!(previous, Some(held));
                    prop_assert_eq!(now.txid(), row.txid());
                }
                Err(CheckError::AmbiguousLength) => {
                    roads[1].set(roads[1].get() + 1);
                    prop_assert_eq!(held.tx_len(), Some(AMBIGUOUS_TX_LEN));
                    prop_assert_eq!(row.tx_len(), None);
                    prop_assert!(holds_a_proof(&row));
                    prop_assert!(now.evidence().is_none());
                    prop_assert!(
                        !matches!(now.word(), Word::Mined(_) | Word::Stale),
                        "word: {:?}",
                        now.word()
                    );
                    prop_assert_eq!(now.tx_len(), Some(AMBIGUOUS_TX_LEN));
                }
                Err(CheckError::LengthMismatch { measured, offered }) => {
                    roads[2].set(roads[2].get() + 1);
                    prop_assert_eq!(held.tx_len(), Some(measured));
                    prop_assert_eq!(row.tx_len(), Some(offered));
                    prop_assert_eq!(now, &held);
                }
                Err(other) => prop_assert!(false, "restore answered {other:?}"),
            }
            Ok(())
        })
        .unwrap();
    let counts: Vec<u32> = roads.iter().map(Cell::get).collect();
    assert!(counts.iter().all(|&n| n > 0), "the three roads: {counts:?}");
}

/// The tracker's index describes the subject after every act of a trace,
/// not only after a restore: the measure's fault road, a refused proof and
/// every input leave no row the index does not describe.
#[test]
fn every_act_leaves_the_index_describing_the_state() {
    each_act(|e| {
        let (heights, hashes) = probes(&[&e.before, &e.after]);
        prop_assert_eq!(
            index_of(e.tracker, &e.subject.txid, &heights, &hashes),
            index_wanted(&e.after),
            "{:?}",
            e.act
        );
        Ok(())
    });
}
