//! Four randomized statements named for the Lean theorems they exercise.
//! 256 reproducible traces per theorem, using an explicit ChaCha seed and no
//! failure persistence, network, clock or storage.
mod common;
use bsv_tracker::*;
use common::*;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};

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

fn trace() -> impl Strategy<Value = Vec<(u8, u32, u64, u8, u8)>> {
    prop::collection::vec(
        (0u8..16, 1u32..12, 20u64..40, any::<u8>(), any::<u8>()),
        1..80,
    )
}

fn status(code: u8, height: u32, observed: u64) -> Input {
    hint(
        match code % 8 {
            0 => HintStatus::Accepted,
            1 => HintStatus::Seen,
            2 => HintStatus::Mined { height },
            3 => HintStatus::StaleBlock,
            4 => HintStatus::Rejected {
                reason: format!("reason-{code}"),
            },
            5 => HintStatus::DoubleSpend {
                competitors: vec![hash(2)],
            },
            6 => HintStatus::OrphanMempool,
            _ => HintStatus::Unknown,
        },
        observed,
    )
}

fn host(code: u8, observed: u64) -> Input {
    Input::Host(match code % 4 {
        0 => HostAction::Build,
        1 => HostAction::Abandon,
        2 => HostAction::SpendAttempt,
        _ => HostAction::Tick(observed),
    })
}

/// Each step supplies exactly one immutable snapshot, as in Tracker.Trace.
fn input(h: &mut TestHeaders, code: u8, height: u32, sibling: u64) -> Input {
    h.fault = false;
    let own = proof(&hash(1), height, sibling);
    let other = proof(&hash(2), height, sibling);
    Input::Evidence(match code {
        0 => {
            h.insert(&own, u64::from(height));
            Evidence::Proof(own)
        }
        1 => {
            h.insert(&other, u64::from(height));
            Evidence::Proof(own)
        }
        2 => {
            h.fault = true;
            Evidence::Proof(own)
        }
        3 => {
            h.insert(&own, u64::from(height));
            Evidence::Recheck
        }
        4 => {
            h.insert(&other, u64::from(height));
            Evidence::Competitor(other)
        }
        5 => {
            h.insert(&own, u64::from(height));
            Evidence::Competitor(other)
        }
        6 => Evidence::Chain(reorg(height)),
        7 => Evidence::Chain(ChainEvent::Fork {
            height,
            competing_tips: vec![hash(90), hash(91)],
            depth: 1,
        }),
        8 => Evidence::Chain(ChainEvent::Tip {
            height,
            hash: hash(90),
        }),
        9 => Evidence::Chain(ChainEvent::Invalidated {
            block_hash: hash(u64::from(height)),
        }),
        10 => Evidence::Chain(ChainEvent::Frozen {
            outpoint: format!("{}:0", hash(1)),
        }),
        11 => Evidence::Chain(ChainEvent::TipAge { seconds: sibling }),
        12 => Evidence::Verdict(NodeVerdict::Rejected {
            reason: format!("node-{height}"),
        }),
        13 => Evidence::Verdict(NodeVerdict::Conflicted {
            competitors: vec![hash(sibling)],
        }),
        14 => {
            h.headers.clear();
            Evidence::Recheck
        }
        _ => {
            h.fault = true;
            Evidence::Recheck
        }
    })
}

#[test]
fn mined_implies_checked_proof() {
    runner()
        .run(&trace(), |trace| {
            let mut s = State::new(hash(1));
            let mut h = TestHeaders::default();
            for (code, height, sibling, noise, time) in trace {
                let i = input(&mut h, code, height, sibling);
                let _ = s.step(&params(), &h, i);
                step(&mut s, &h, status(noise, height, u64::from(time)));
                step(&mut s, &h, host(noise, u64::from(time)));
                if let Word::Mined(mined) = s.word() {
                    let checked = s.evidence().unwrap();
                    prop_assert_eq!(mined.height(), checked.proof().height());
                    prop_assert_eq!(mined.checked(), checked);
                    prop_assert_eq!(checked.proof().txid(), s.txid());
                    prop_assert_eq!(
                        checked.proof().path().compute_root(Some(s.txid())).unwrap(),
                        checked.header().merkle_root.clone()
                    );
                    prop_assert!(checked.depth() >= 1);
                }
            }
            Ok(())
        })
        .unwrap();
}

#[test]
fn reorg_reasks_at_or_above() {
    let states = prop::collection::vec((1u32..20, 0u8..5), 1..40);
    runner()
        .run(&(states, 0u32..22), |(states, fork_height)| {
            let mut tracker = Tracker::new(params());
            let mut h = TestHeaders::default();
            let mut before = vec![];
            for (n, (height, kind)) in states.into_iter().enumerate() {
                let txid = hash(n as u64 + 100);
                tracker.track(&txid);
                match kind {
                    0 | 1 => {
                        let p = proof(&txid, height, 1_000);
                        h.insert(&p, u64::from(height));
                        tracker
                            .apply(&txid, &h, Input::Evidence(Evidence::Proof(p)))
                            .unwrap();
                    }
                    2 => {
                        tracker
                            .apply(&txid, &h, Input::Host(HostAction::Build))
                            .unwrap();
                    }
                    3 => {
                        tracker
                            .apply(
                                &txid,
                                &h,
                                Input::Evidence(Evidence::Verdict(NodeVerdict::Rejected {
                                    reason: "node".into(),
                                })),
                            )
                            .unwrap();
                    }
                    _ => {}
                }
                before.push(tracker.get(&txid).unwrap().clone());
            }
            let affected = tracker.on_chain(&h, reorg(fork_height));
            let expected = before
                .iter()
                .filter(|s| s.word().height().is_some_and(|h| h >= fork_height))
                .count();
            prop_assert_eq!(affected.len(), expected);
            prop_assert!(affected.len() <= before.len());
            for old in before {
                let after = tracker.get(old.txid()).unwrap();
                if old.word().height().is_some_and(|h| h >= fork_height) {
                    prop_assert_eq!(after.word(), &Word::Stale);
                    prop_assert_eq!(after.reask(), Some(&Reask::Reorg { fork_height }));
                    prop_assert_eq!(after.evidence(), old.evidence());
                    prop_assert!(!after.suspect());
                    prop_assert!(affected
                        .iter()
                        .any(|u| u.txid == old.txid() && u.result.is_ok()));
                } else {
                    prop_assert_eq!(after, &old);
                }
            }
            Ok(())
        })
        .unwrap();
}

#[test]
fn hint_changes_no_chain_word() {
    runner()
        .run(&trace(), |trace| {
            let mut s = State::new(hash(1));
            let mut h = TestHeaders::default();
            for (code, height, sibling, noise, time) in trace {
                let i = input(&mut h, code, height, sibling);
                let _ = s.step(&params(), &h, i);
                step(&mut s, &h, host(noise, u64::from(time)));
                let before = s.clone();
                step(&mut s, &h, status(noise, height, u64::from(time)));
                prop_assert_eq!(s.word().chain(), before.word().chain());
                prop_assert_eq!(s.evidence(), before.evidence());
                prop_assert_eq!(s.suspect(), before.suspect());
                if s.word() != before.word() {
                    prop_assert!(matches!(
                        (before.word(), s.word()),
                        (Word::Built, Word::Announced | Word::Seen) | (Word::Announced, Word::Seen)
                    ));
                }
            }
            Ok(())
        })
        .unwrap();
}

#[test]
fn same_evidence_same_word() {
    runner()
        .run(&trace(), |trace| {
            let mut a = State::new(hash(1));
            let mut b = a.clone();
            let mut h = TestHeaders::default();
            for (code, height, sibling, noise_a, noise_b) in trace {
                let i = input(&mut h, code, height, sibling);
                step(&mut a, &h, status(noise_a, height, u64::from(noise_a)));
                step(&mut b, &h, status(noise_b, height, u64::from(noise_b)));
                step(&mut a, &h, host(noise_a, u64::from(noise_a)));
                step(&mut b, &h, host(noise_b, u64::from(noise_b)));
                let result_a = a.step(&params(), &h, i.clone());
                let result_b = b.step(&params(), &h, i);
                prop_assert_eq!(result_a, result_b);
                prop_assert_eq!(a.word().chain(), b.word().chain());
                prop_assert_eq!(a.evidence(), b.evidence());
                prop_assert_eq!(a.suspect(), b.suspect());
            }
            Ok(())
        })
        .unwrap();
}
