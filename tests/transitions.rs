//! Branch witnesses for the charter's transition table and host boundary.
// The lint bar binds the library; a test may unwrap, and its fixtures fail loudly
// (bsv-stack-lean #134).
#![allow(
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod common;
use bsv_rs::transaction::{MerklePath, MerklePathLeaf};
use bsv_tracker::*;
use common::*;
use std::cell::Cell;

#[test]
fn proof_checks_fail_closed_and_bind_the_tracked_txid() {
    let txid = hash(1);
    let mut s = State::new(&txid);
    let mut h = TestHeaders::default();
    announce(&mut s, &h);
    let p = proof(&txid, 10, 2);
    assert_eq!(
        s.step(&params(), &h, Input::Evidence(Evidence::Proof(p.clone()))),
        Err(CheckError::Unavailable(10))
    );
    assert_eq!(s.word(), &Word::Seen);
    assert_eq!(s.evidence(), None);
    h.fault = true;
    assert!(matches!(
        s.step(&params(), &h, Input::Evidence(Evidence::Proof(p.clone()))),
        Err(CheckError::Headers(_))
    ));
    assert_eq!(s.word(), &Word::Seen);
    h.fault = false;
    h.insert(&p, 10);
    h.tip = 9;
    assert_eq!(
        s.step(&params(), &h, Input::Evidence(Evidence::Proof(p.clone()))),
        Err(CheckError::Unavailable(10))
    );
    h.tip = 14;
    step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
    mined(&s, 10);
    assert_eq!(s.evidence().unwrap().depth(), 5);
    let saved = s.evidence().cloned();
    let other = proof(&hash(2), 10, 1);
    assert_eq!(
        s.step(&params(), &h, Input::Evidence(Evidence::Proof(other))),
        Err(CheckError::TxidMismatch)
    );
    mined(&s, 10);
    assert_eq!(s.evidence(), saved.as_ref());
}

#[test]
fn malformed_sdk_paths_are_rejected_before_root_reduction() {
    let txid = hash(1);
    for path in [
        MerklePath {
            block_height: 1,
            path: vec![],
        },
        MerklePath {
            block_height: 1,
            path: vec![vec![]],
        },
        MerklePath {
            block_height: 1,
            path: vec![vec![]; 65],
        },
    ] {
        assert!(matches!(
            Proof::new(&txid, path),
            Err(CheckError::InvalidProof(_))
        ));
    }
    assert!(Proof::new("not a txid", MerklePath::from_coinbase_txid(&txid, 1)).is_err());
    assert!(Proof::new(hash(2), MerklePath::from_coinbase_txid(&txid, 1)).is_err());
    // bsv-rs 0.9.0-beta.8 (#817): a walk that pairs a hash holding a character
    // that is not hex is refused, where 0.9.0-beta.7 dropped that side of the
    // pair. The path is an invalid proof at the constructor and reaches no
    // header check.
    let with_sibling = |sibling: String| MerklePath {
        block_height: 1,
        path: vec![vec![
            MerklePathLeaf::new_txid(0, txid.clone()),
            MerklePathLeaf::new(1, sibling),
        ]],
    };
    assert!(matches!(
        Proof::new(&txid, with_sibling("zz".repeat(32))),
        Err(CheckError::InvalidProof(_))
    ));
    // A sibling that is hex of another length is paired as bsv-rs pairs it
    // (the reference's concatenation): the path reduces, to a root the
    // well-formed proof of the txid does not have, so a true header answers
    // RootMismatch for it and no word moves.
    let good = Proof::new(&txid, with_sibling(hash(2)))
        .unwrap()
        .root()
        .unwrap();
    for sibling in [hash(2)[..63].to_string(), "ab".repeat(31)] {
        let root = Proof::new(&txid, with_sibling(sibling))
            .unwrap()
            .root()
            .unwrap();
        assert_ne!(root, good);
    }
}

#[test]
fn age_abandon_and_agreeing_hints_follow_the_lean_table() {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(&mut s, &h, Input::Host(HostAction::Abandon));
    assert_eq!(s.word(), &Word::Abandoned);
    step(&mut s, &h, Input::Host(HostAction::Build));
    assert_eq!(s.word(), &Word::Abandoned);
    let mut s = State::new(hash(1));
    step(&mut s, &h, Input::Host(HostAction::Build));
    step(&mut s, &h, hint(HintStatus::Accepted, 10));
    assert_eq!(s.word(), &Word::Announced);
    assert_eq!(s.reask(), None);
    step(&mut s, &h, Input::Host(HostAction::Abandon));
    assert_eq!(s.word(), &Word::Announced);
    step(&mut s, &h, Input::Host(HostAction::Tick(69)));
    assert_eq!(s.reask(), None);
    step(&mut s, &h, Input::Host(HostAction::Tick(70)));
    assert_eq!(s.reask(), Some(&Reask::Age));
    step(&mut s, &h, hint(HintStatus::Accepted, 71));
    assert_eq!(s.reask(), Some(&Reask::Age));
    step(&mut s, &h, hint(HintStatus::Seen, 72));
    assert_eq!(s.word(), &Word::Seen);
    step(&mut s, &h, hint(HintStatus::Seen, u64::MAX));
    step(&mut s, &h, Input::Host(HostAction::SpendAttempt));
    step(&mut s, &h, Input::Host(HostAction::Tick(u64::MAX)));
    assert_eq!(s.reask(), Some(&Reask::Spend));
    step(&mut s, &h, hint(HintStatus::Unknown, 0));
    assert_eq!(s.last_hint_at(), Some(0));
    assert_eq!(s.hints()[0].status, HintStatus::Unknown);
}

#[test]
fn fork_tip_recheck_and_invalidation_retain_the_evidence_record() {
    let txid = hash(1);
    let p = proof(&txid, 10, 2);
    let mut h = TestHeaders::default();
    h.insert(&p, 10);
    let mut s = State::new(&txid);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
    let saved = s.evidence().cloned();
    step(&mut s, &h, Input::Evidence(Evidence::Chain(fork(10))));
    mined(&s, 10);
    assert!(s.suspect());
    h.fault = true;
    assert!(s
        .step(&params(), &h, Input::Evidence(Evidence::Chain(tip(11, 11))))
        .is_err());
    mined(&s, 10);
    assert!(s.suspect());
    assert_eq!(s.reask(), Some(&Reask::Recheck));
    h.fault = false;
    step(&mut s, &h, Input::Evidence(Evidence::Chain(tip(11, 11))));
    mined(&s, 10);
    assert!(!s.suspect());
    assert_eq!(s.reask(), None);
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Chain(ChainEvent::Invalidated {
            block_hash: hash(9),
        })),
    );
    mined(&s, 10);
    // The ruling (5) of 2026-10-20 19:4xZ (bsv-stack-lean docs/RULINGS-2026-10-12.md):
    // a `mined` word is `stale` when, and only when, its stored proof's root is
    // not the active header's root at its height; the invalidation selects the
    // word by its checked header, and the host's headers have moved the root.
    h.move_root(10, 30);
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Chain(ChainEvent::Invalidated {
            block_hash: hash(10),
        })),
    );
    assert_eq!(s.word(), &Word::Stale);
    assert_eq!(s.evidence(), saved.as_ref());
    assert_eq!(s.reask(), Some(&Reask::Reorg { fork_height: 10 }));
}

#[test]
fn successful_recheck_clears_completed_spend_and_fork_triggers() {
    let txid = hash(1);
    let p = proof(&txid, 10, 2);
    let mut h = TestHeaders::default();
    h.insert(&p, 10);
    let mut s = State::new(&txid);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
    let evidence = s.evidence().cloned();
    step(&mut s, &h, Input::Host(HostAction::SpendAttempt));
    assert_eq!(s.reask(), Some(&Reask::Spend));
    step(&mut s, &h, Input::Evidence(Evidence::Recheck));
    mined(&s, 10);
    assert_eq!(s.reask(), None);
    assert_eq!(s.evidence(), evidence.as_ref());
    step(&mut s, &h, Input::Evidence(Evidence::Chain(fork(10))));
    assert!(s.suspect());
    assert_eq!(s.reask(), Some(&Reask::Fork { height: 10 }));
    step(&mut s, &h, Input::Evidence(Evidence::Chain(tip(11, 11))));
    mined(&s, 10);
    assert!(!s.suspect());
    assert_eq!(s.reask(), None);
    assert_eq!(s.evidence(), evidence.as_ref());
}

#[test]
fn node_and_competitor_evidence_outrank_hints_but_reask_a_mined_word() {
    let mut h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Verdict(NodeVerdict::Rejected {
            reason: "node policy".into(),
        })),
    );
    assert_eq!(
        s.word(),
        &Word::Rejected {
            reason: "node policy".into()
        }
    );
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Verdict(NodeVerdict::Conflicted {
            competitors: vec![hash(4)],
        })),
    );
    let c = proof(&hash(2), 10, 3);
    h.insert(&c, 10);
    step(&mut s, &h, Input::Evidence(Evidence::Competitor(c.clone())));
    assert_eq!(
        s.word(),
        &Word::Conflicted {
            competitors: vec![hash(2), hash(4)]
        }
    );
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Verdict(NodeVerdict::Conflicted {
            competitors: vec![hash(5)],
        })),
    );
    assert_eq!(
        s.word(),
        &Word::Conflicted {
            competitors: vec![hash(5), hash(2), hash(4)]
        }
    );
    let own = proof(&hash(1), 11, 3);
    h.insert(&own, 11);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(own)));
    mined(&s, 11);
    let saved = s.evidence().cloned();
    step(&mut s, &h, Input::Evidence(Evidence::Competitor(c)));
    mined(&s, 11);
    assert_eq!(s.reask(), Some(&Reask::Recheck));
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Verdict(NodeVerdict::Rejected {
            reason: "mempool".into(),
        })),
    );
    mined(&s, 11);
    assert_eq!(s.evidence(), saved.as_ref());
    let failed = proof(&hash(3), 99, 4);
    assert!(s
        .step(&params(), &h, Input::Evidence(Evidence::Competitor(failed)))
        .is_err());
    mined(&s, 11);
    let mut unmined = State::new(hash(1));
    let competitor = proof(&hash(2), 10, 3);
    step(
        &mut unmined,
        &h,
        Input::Evidence(Evidence::Competitor(competitor)),
    );
    assert_eq!(
        unmined.word(),
        &Word::Conflicted {
            competitors: vec![hash(2)]
        }
    );
    assert_eq!(unmined.evidence(), None);
}

struct CountHeaders {
    inner: TestHeaders,
    lookups: Cell<usize>,
}

#[test]
fn cached_host_check_cannot_bypass_fail_closed_snapshot_lookup() {
    struct CachedCheck {
        current: TestHeaders,
        old: CheckedProof,
    }
    impl Headers for CachedCheck {
        fn header_at(&self, height: u32) -> Result<Option<Header>, HeaderError> {
            self.current.header_at(height)
        }
        fn tip_height(&self) -> Result<u32, HeaderError> {
            self.current.tip_height()
        }
        fn check(&self, _proof: Proof) -> Result<CheckedProof, CheckError> {
            Ok(self.old.clone())
        }
    }
    let txid = hash(1);
    let p = proof(&txid, 10, 2);
    let mut current = TestHeaders::default();
    current.insert(&p, 10);
    let old = current.check(p.clone()).unwrap();
    current.fault = true;
    let headers = CachedCheck { current, old };
    let mut s = State::new(&txid);
    let result = s.step(&params(), &headers, Input::Evidence(Evidence::Proof(p)));
    assert!(matches!(result, Err(CheckError::Headers(_))), "{result:?}");
    assert_eq!(s.word(), &Word::Unknown);
    assert_eq!(s.evidence(), None);
    assert_eq!(s.reask(), Some(&Reask::ProofFailed { height: 10 }));

    let mut competitor = State::new(hash(2));
    let result = competitor.step(
        &params(),
        &headers,
        Input::Evidence(Evidence::Competitor(proof(&txid, 10, 2))),
    );
    assert!(matches!(result, Err(CheckError::Headers(_))), "{result:?}");
    assert_eq!(competitor.word(), &Word::Unknown);
    assert_eq!(competitor.evidence(), None);
    assert_eq!(competitor.reask(), Some(&Reask::ProofFailed { height: 10 }));
}
impl Headers for CountHeaders {
    fn header_at(&self, height: u32) -> Result<Option<Header>, HeaderError> {
        self.lookups.set(self.lookups.get() + 1);
        self.inner.header_at(height)
    }
    fn tip_height(&self) -> Result<u32, HeaderError> {
        self.inner.tip_height()
    }
}

#[test]
fn collection_routes_only_affected_heights_headers_and_suspects() {
    let mut h = TestHeaders::default();
    let mut tracker = Tracker::new(params());
    for n in 1..=4 {
        let p = proof(&hash(n), n as u32, 10);
        h.insert(&p, n);
        tracker
            .apply(&hash(n), &h, Input::Evidence(Evidence::Proof(p)))
            .unwrap();
    }
    tracker.track(hash(5));
    let mut h = CountHeaders {
        inner: h,
        lookups: Cell::new(0),
    };
    assert!(tracker.on_chain(&h, tip(5, 5)).is_empty());
    assert_eq!(h.lookups.get(), 0);
    let fork_updates = tracker.on_chain(&h, fork(3));
    assert_eq!(fork_updates.len(), 2);
    assert_eq!(h.lookups.get(), 0);
    let tip_updates = tracker.on_chain(&h, tip(5, 5));
    assert_eq!(tip_updates.len(), 2);
    assert_eq!(h.lookups.get(), 2);
    // The ruling (5) of 2026-10-20 19:4xZ (bsv-stack-lean docs/RULINGS-2026-10-12.md):
    // a `mined` word is `stale` when, and only when, its stored proof's root is
    // not the active header's root at its height. The event selects the words;
    // the recheck reads one header for each selected word and fetches nothing.
    h.inner.move_root(2, 22);
    let invalid = tracker.on_chain(
        &h,
        ChainEvent::Invalidated {
            block_hash: hash(2),
        },
    );
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0].txid, hash(2));
    assert_eq!(h.lookups.get(), 3);
    h.inner.move_root(3, 23);
    h.inner.move_root(4, 24);
    let reorg_updates = tracker.on_chain(&h, reorg(3));
    assert_eq!(reorg_updates.len(), 2);
    assert_eq!(h.lookups.get(), 5);
    assert!(tracker.on_chain(&h, reorg(3)).is_empty());
    assert_eq!(tracker.get(&hash(1)).unwrap().word().height(), Some(1));
    assert_eq!(tracker.get(&hash(5)).unwrap().word(), &Word::Unknown);
    assert_eq!(tracker.len(), 5);
    assert!(!tracker.is_empty());
    let p = proof(&hash(3), 1, 20);
    let mut inner = h.inner.clone();
    inner.insert(&p, 20);
    tracker
        .apply(&hash(3), &inner, Input::Evidence(Evidence::Proof(p)))
        .unwrap();
    assert!(tracker.on_chain(&inner, reorg(3)).is_empty());
    assert_eq!(
        tracker
            .on_chain(
                &inner,
                ChainEvent::Invalidated {
                    block_hash: hash(20)
                }
            )
            .len(),
        1
    );
}

#[test]
fn envelopes_round_trip_and_reject_every_unknown_shape() {
    let events = [
        tip(1, 1),
        fork(1),
        reorg(1),
        ChainEvent::Invalidated {
            block_hash: hash(1),
        },
        ChainEvent::Frozen {
            outpoint: Outpoint {
                txid: hash(1),
                vout: 0,
            },
        },
        ChainEvent::TipAge {
            seconds: 600,
            tip: event_header(1, 1),
        },
    ];
    for event in events {
        let wire = serde_json::to_vec(&ChainEnvelope::new(event.clone())).unwrap();
        assert_eq!(decode_envelope(&wire), Ok(event.clone()));
        let envelope: ChainEnvelope = serde_json::from_slice(&wire).unwrap();
        assert_eq!(envelope.version(), 1);
        assert_eq!(envelope.into_event(), event);
    }
    for body in [
        br#"{}"#.as_slice(),
        br#"{"v":1,"kind":"tip","height":1,"hash":"x","extra":1}"#,
        br#"{"v":"1","kind":"tip","height":1,"hash":"x"}"#,
        br#"{"v":1,"kind":"tip","height":-1,"hash":"x"}"#,
    ] {
        assert!(matches!(
            decode_envelope(body),
            Err(DecodeError::UnknownShape { .. })
        ));
        assert!(serde_json::from_slice::<ChainEnvelope>(body).is_err());
    }
    assert_eq!(
        decode_envelope(br#"{"kind":"unknown","v":3}"#),
        Err(DecodeError::UnknownVersion(3))
    );
    assert!(serde_json::from_slice::<ChainEnvelope>(
        br#"{"v":2,"kind":"tip","height":1,"hash":"x"}"#
    )
    .is_err());
}

#[test]
fn a_couriers_mempool_presence_announces_a_built_word_and_writes_no_chain_word() {
    let txid = hash(1);
    let mut h = TestHeaders::default();
    let present = |source: &str, at| {
        hint(
            HintStatus::MempoolPresent {
                source: source.into(),
            },
            at,
        )
    };

    // Built: the ruling (2) of 2026-10-20 19:4xZ (bsv-stack-lean
    // docs/RULINGS-2026-10-12.md): a courier's mempool presence on a `built`
    // transaction reads `announced`; a node we did not broadcast through
    // holds the bytes, so someone announced it. The word agrees: no re-ask.
    let mut s = State::new(&txid);
    step(&mut s, &h, Input::Host(HostAction::Build));
    step(&mut s, &h, present("woc", 5));
    assert_eq!(s.word(), &Word::Announced);
    assert_eq!(s.reask(), None);
    assert_eq!(s.last_hint_at(), Some(5));
    assert_eq!(
        s.hints()[0].status,
        HintStatus::MempoolPresent {
            source: "woc".into()
        }
    );

    // The same hint gives the same word whether or not this host announced
    // the transaction (the ruling (2)'s stronger statement): `announced`
    // either way, and a second presence over the presence's own `announced`
    // leaves it there.
    let announced_by_us = {
        let mut s = State::new(&txid);
        step(&mut s, &h, Input::Host(HostAction::Build));
        step(&mut s, &h, hint(HintStatus::Accepted, 0));
        step(&mut s, &h, present("woc", 5));
        s
    };
    assert_eq!(announced_by_us.word(), s.word());
    assert_eq!(announced_by_us.reask(), s.reask());
    step(&mut s, &h, present("woc", 6));
    assert_eq!(s.word(), &Word::Announced);
    assert_eq!(s.reask(), None);

    // Announced: it agrees, moves nothing and restarts the age clock.
    let mut s = State::new(&txid);
    step(&mut s, &h, Input::Host(HostAction::Build));
    step(&mut s, &h, hint(HintStatus::Accepted, 0));
    step(&mut s, &h, present("woc", 30));
    assert_eq!(s.word(), &Word::Announced);
    assert_eq!(s.reask(), None);
    assert_eq!(s.last_hint_at(), Some(30));
    step(&mut s, &h, Input::Host(HostAction::Tick(89)));
    assert_eq!(s.reask(), None);
    step(&mut s, &h, Input::Host(HostAction::Tick(90)));
    assert_eq!(s.reask(), Some(&Reask::Age));
    assert_eq!(s.word(), &Word::Announced);

    // Mined: the chain word and its evidence stay; the hint is re-asked.
    let mut s = State::new(&txid);
    let p = proof(&txid, 10, 2);
    h.insert(&p, 10);
    step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
    let before = s.clone();
    step(&mut s, &h, present("woc", 40));
    mined(&s, 10);
    assert_eq!(s.evidence(), before.evidence());
    assert_eq!(s.suspect(), before.suspect());
    assert!(matches!(s.reask(), Some(Reask::Hint(_))));

    // No ARC word reduces to it: a broadcaster's seen stays seen.
    let seen = Verdict::from_arc_body(&txid, br#"{"txStatus":"SEEN_ON_NETWORK"}"#)
        .unwrap()
        .into_hint("arc", 0);
    assert_eq!(seen.status, HintStatus::Seen);
}

// The word `handed` (for #56): one example per arm of the proof page's
// section 3 (bsv-stack-lean `docs/p0/tracker-handed-proof-2026-11-27.md`),
// over `lean/Tracker.lean`'s `onHandOver`, `onDischarge`, `onEndHold`,
// `onTick`, `onHint`, `onProof` and `onVerdict`.

fn host(action: HostAction) -> Input {
    Input::Host(action)
}
fn hand(instance: &str, at: u64) -> Input {
    host(HostAction::HandOver {
        instance: instance.into(),
        at,
    })
}
fn discharge(instance: &str) -> Input {
    host(HostAction::Discharge {
        instance: instance.into(),
    })
}

#[test]
fn handed_arm_hand_over_writes_handed_from_unknown_and_built() {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(&mut s, &h, hand("a", 5));
    assert_eq!(s.word(), &Word::Handed);
    assert_eq!(s.handed_to(), &[HandedTo::new("a", 5)]);
    let mut b = State::new(hash(1));
    step(&mut b, &h, host(HostAction::Build));
    step(&mut b, &h, hand("a", 5));
    assert_eq!(b.word(), &Word::Handed);
    assert_eq!(b.handed_to(), &[HandedTo::new("a", 5)]);
    assert_eq!(Word::Handed.chain(), None);
}

#[test]
fn handed_arm_hand_over_renews_in_place() {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(&mut s, &h, hand("a", 5));
    step(&mut s, &h, hand("b", 6));
    step(&mut s, &h, hand("a", 9));
    assert_eq!(
        s.handed_to(),
        &[HandedTo::new("a", 9), HandedTo::new("b", 6)]
    );
}

#[test]
fn handed_arm_hand_over_leaves_abandoned_whole_and_writes_no_other_word() {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(&mut s, &h, host(HostAction::Abandon));
    let before = s.clone();
    step(&mut s, &h, hand("a", 5));
    assert_eq!(s, before);
    let mut a = State::new(hash(1));
    step(&mut a, &h, hand("a", 5));
    step(&mut a, &h, host(HostAction::Tick(65)));
    assert_eq!(a.reask(), Some(&Reask::Reannounce));
    step(&mut a, &h, hint(HintStatus::Accepted, 70));
    assert_eq!(a.word(), &Word::Announced);
    assert!(a.handed_to().is_empty());
    assert_eq!(a.reask(), Some(&Reask::Reannounce));
    step(&mut a, &h, hand("b", 71));
    assert_eq!(a.word(), &Word::Announced);
    assert!(a.handed_to().is_empty());
    assert_eq!(a.reask(), None);
}

#[test]
fn handed_arm_hints_raise_and_never_lower() {
    let h = TestHeaders::default();
    for (status, word) in [
        (HintStatus::Accepted, Word::Announced),
        (
            HintStatus::MempoolPresent {
                source: "courier".into(),
            },
            Word::Announced,
        ),
        (HintStatus::Seen, Word::Seen),
        (HintStatus::Rejected { reason: "r".into() }, Word::Handed),
        (HintStatus::StaleBlock, Word::Handed),
        (HintStatus::Unknown, Word::Handed),
    ] {
        let mut s = State::new(hash(1));
        step(&mut s, &h, hand("a", 5));
        step(&mut s, &h, hint(status.clone(), 6));
        assert_eq!(s.word(), &word, "{status:?}");
        assert_eq!(s.handed_to().is_empty(), word != Word::Handed, "{status:?}");
    }
}

#[test]
fn handed_arm_discharge_keeps_while_held_then_returns_built() {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(&mut s, &h, hand("a", 5));
    step(&mut s, &h, hand("b", 6));
    step(&mut s, &h, discharge("a"));
    assert_eq!(s.word(), &Word::Handed);
    assert_eq!(s.handed_to(), &[HandedTo::new("b", 6)]);
    step(&mut s, &h, discharge("b"));
    assert_eq!(s.word(), &Word::Built);
    assert!(s.handed_to().is_empty());
    let mut other = State::new(hash(1));
    step(&mut other, &h, host(HostAction::Build));
    let before = other.clone();
    step(&mut other, &h, discharge("a"));
    assert_eq!(other, before);
}

#[test]
fn handed_arm_abandon_refused_end_hold_ends() {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(&mut s, &h, hand("a", 5));
    step(&mut s, &h, host(HostAction::Abandon));
    assert_eq!(s.word(), &Word::Handed);
    step(&mut s, &h, host(HostAction::Tick(65)));
    step(&mut s, &h, host(HostAction::EndHold));
    assert_eq!(s.word(), &Word::Abandoned);
    assert!(s.handed_to().is_empty());
    assert_eq!(s.reask(), None);
    let mut taken = State::new(hash(1));
    step(&mut taken, &h, hand("a", 5));
    step(&mut taken, &h, hint(HintStatus::Accepted, 6));
    let before = taken.clone();
    step(&mut taken, &h, host(HostAction::EndHold));
    assert_eq!(taken, before);
}

#[test]
fn handed_arm_tick_reannounces_on_the_oldest_and_never_on_built() {
    let h = TestHeaders::default();
    let mut s = State::new(hash(1));
    step(&mut s, &h, hand("a", 10));
    step(&mut s, &h, hand("b", 30));
    step(&mut s, &h, host(HostAction::Tick(69)));
    assert_eq!(s.reask(), None);
    step(&mut s, &h, host(HostAction::Tick(70)));
    assert_eq!(s.reask(), Some(&Reask::Reannounce));
    assert_eq!(Reask::Reannounce.label(), "reannounce");
    let mut built = State::new(hash(1));
    step(&mut built, &h, host(HostAction::Build));
    step(&mut built, &h, host(HostAction::Tick(u64::MAX)));
    assert_eq!(built.reask(), None);
    let mut late = State::new(hash(1));
    step(&mut late, &h, hand("a", u64::MAX - 1));
    step(&mut late, &h, host(HostAction::Tick(u64::MAX)));
    assert_eq!(late.reask(), None);
}

#[test]
fn handed_arm_evidence_leaves_and_the_list_follows() {
    let txid = hash(1);
    let mut h = TestHeaders::default();
    let p = proof(&txid, 10, 2);
    h.insert(&p, 10);
    let mut s = State::new(&txid);
    step(&mut s, &h, hand("a", 5));
    step(&mut s, &h, Input::Evidence(Evidence::Proof(p)));
    assert!(matches!(s.word(), Word::Mined(_)));
    assert!(s.handed_to().is_empty());

    let mut r = State::new(&txid);
    step(&mut r, &h, hand("a", 5));
    step(
        &mut r,
        &h,
        Input::Evidence(Evidence::Verdict(NodeVerdict::Rejected {
            reason: "no".into(),
        })),
    );
    assert_eq!(
        r.word(),
        &Word::Rejected {
            reason: "no".into()
        }
    );
    assert!(r.handed_to().is_empty());

    let mut c = State::new(&txid);
    step(&mut c, &h, hand("a", 5));
    step(
        &mut c,
        &h,
        Input::Evidence(Evidence::Verdict(NodeVerdict::Conflicted {
            competitors: vec![hash(9)],
        })),
    );
    assert!(matches!(c.word(), Word::Conflicted { .. }));
    assert_eq!(c.handed_to(), &[HandedTo::new("a", 5)]);

    let rival = proof(&hash(8), 11, 3);
    h.insert(&rival, 11);
    let mut k = State::new(&txid);
    step(&mut k, &h, hand("a", 5));
    step(&mut k, &h, Input::Evidence(Evidence::Competitor(rival)));
    assert!(matches!(k.word(), Word::Conflicted { .. }));
    assert_eq!(k.handed_to(), &[HandedTo::new("a", 5)]);
}

#[test]
fn handed_to_is_stored_by_its_serde_form() {
    let held = HandedTo::new("arcade", 1_700_000_000);
    let text = serde_json::to_string(&held).unwrap();
    assert_eq!(text, r#"{"instance":"arcade","at":1700000000}"#);
    assert_eq!(serde_json::from_str::<HandedTo>(&text).unwrap(), held);
}
