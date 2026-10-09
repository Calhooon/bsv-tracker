//! Branch witnesses for the charter's transition table and host boundary.
mod common;
use bsv_rs::transaction::MerklePath;
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
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Chain(ChainEvent::Fork {
            height: 10,
            competing_tips: vec![],
            depth: 1,
        })),
    );
    mined(&s, 10);
    assert!(s.suspect());
    h.fault = true;
    assert!(s
        .step(
            &params(),
            &h,
            Input::Evidence(Evidence::Chain(ChainEvent::Tip {
                height: 11,
                hash: hash(11)
            }))
        )
        .is_err());
    mined(&s, 10);
    assert!(s.suspect());
    assert_eq!(s.reask(), Some(&Reask::Recheck));
    h.fault = false;
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Chain(ChainEvent::Tip {
            height: 11,
            hash: hash(11),
        })),
    );
    mined(&s, 10);
    assert!(!s.suspect());
    assert_eq!(s.reask(), Some(&Reask::Recheck));
    step(
        &mut s,
        &h,
        Input::Evidence(Evidence::Chain(ChainEvent::Invalidated {
            block_hash: hash(9),
        })),
    );
    mined(&s, 10);
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
    let h = CountHeaders {
        inner: h,
        lookups: Cell::new(0),
    };
    assert!(tracker
        .on_chain(
            &h,
            ChainEvent::Tip {
                height: 5,
                hash: hash(5)
            }
        )
        .is_empty());
    assert_eq!(h.lookups.get(), 0);
    let fork = tracker.on_chain(
        &h,
        ChainEvent::Fork {
            height: 3,
            competing_tips: vec![],
            depth: 1,
        },
    );
    assert_eq!(fork.len(), 2);
    assert_eq!(h.lookups.get(), 0);
    let tip = tracker.on_chain(
        &h,
        ChainEvent::Tip {
            height: 5,
            hash: hash(5),
        },
    );
    assert_eq!(tip.len(), 2);
    assert_eq!(h.lookups.get(), 2);
    let invalid = tracker.on_chain(
        &h,
        ChainEvent::Invalidated {
            block_hash: hash(2),
        },
    );
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0].txid, hash(2));
    let reorg_updates = tracker.on_chain(&h, reorg(3));
    assert_eq!(reorg_updates.len(), 2);
    assert_eq!(h.lookups.get(), 2);
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
        ChainEvent::Tip {
            height: 1,
            hash: hash(1),
        },
        ChainEvent::Fork {
            height: 1,
            competing_tips: vec![hash(1), hash(2)],
            depth: 1,
        },
        reorg(1),
        ChainEvent::Invalidated {
            block_hash: hash(1),
        },
        ChainEvent::Frozen {
            outpoint: format!("{}:0", hash(1)),
        },
        ChainEvent::TipAge { seconds: 600 },
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
        br#"{"v":1,"kind":"new_kind"}"#,
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
        decode_envelope(br#"{"kind":"unknown","v":2}"#),
        Err(DecodeError::UnknownVersion(2))
    );
    assert!(serde_json::from_slice::<ChainEnvelope>(
        br#"{"v":2,"kind":"tip","height":1,"hash":"x"}"#
    )
    .is_err());
}
