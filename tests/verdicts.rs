//! The bytes of the P0-2 vector at bsv-rs@7bc623c are fixtures, never live calls.
mod common;
use bsv_tracker::*;
use common::*;

#[test]
fn arc_p0_2_twenty_bodies_are_only_hints() {
    let vector: serde_json::Value =
        serde_json::from_str(include_str!("vectors/arc_tx_status_verdicts.json")).unwrap();
    let txid = vector["txid"].as_str().unwrap();
    let h = TestHeaders::default();
    let cases = vector["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 20);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let body = serde_json::to_vec(&case["body"]).unwrap();
        let decoded = Verdict::from_arc_body(txid, &body);
        if matches!(
            name,
            "200_no_tx_status"
                | "200_rejected_for_another_txid"
                | "200_double_spend_attempted_bad_competing_txs"
                | "200_unknown_status_word"
        ) {
            assert!(decoded.is_err(), "{name}");
            continue;
        }
        let verdict = decoded.unwrap();
        if name == "200_seen_on_network_competing_txs" {
            assert_eq!(
                verdict.competitors,
                vec![vector["competing_txid"].as_str().unwrap()]
            );
        }
        let hint = verdict.into_hint("ARC vector", 123);
        assert_eq!(hint.source, "ARC vector");
        assert_eq!(hint.observed, 123);
        assert_eq!(
            hint.verdict.as_ref().unwrap().status,
            case["body"]["txStatus"].as_str().unwrap()
        );
        let mut s = State::new(txid);
        step(&mut s, &h, Input::Host(HostAction::Build));
        step(&mut s, &h, Input::Hint(hint));
        assert_eq!(s.word().chain(), None, "{name}");
        assert_eq!(s.evidence(), None, "{name}");
        if case["expect"]["result"] == "failure" {
            assert_eq!(s.word(), &Word::Built, "{name}");
            assert!(matches!(s.reask(), Some(Reask::Hint(_))), "{name}");
        }
    }
}

#[test]
fn arcade_twelve_words_and_reorg_markers_remain_hints() {
    // [SRC] arcade@1ae1208 models/transaction.go:89-126,319-332.
    let h = TestHeaders::default();
    for word in [
        "UNKNOWN",
        "RECEIVED",
        "SENT_TO_NETWORK",
        "ACCEPTED_BY_NETWORK",
        "SEEN_ON_NETWORK",
        "SEEN_MULTIPLE_NODES",
        "DOUBLE_SPEND_ATTEMPTED",
        "REJECTED",
        "PENDING_RETRY",
        "STUMP_PROCESSING",
        "MINED",
        "IMMUTABLE",
    ] {
        let verdict = Verdict {
            status: word.into(),
            height: Some(10),
            reason: None,
            competitors: vec![],
        };
        let mut s = State::new(hash(1));
        step(&mut s, &h, Input::Host(HostAction::Build));
        step(&mut s, &h, Input::Hint(verdict.into_hint("Arcade", 10)));
        assert_eq!(s.word().chain(), None, "{word}");
        assert_eq!(s.evidence(), None);
    }
    let unmine = Verdict {
        status: "SEEN_ON_NETWORK".into(),
        height: None,
        reason: Some("reorg_unmined".into()),
        competitors: vec![],
    }
    .into_hint("Arcade", 10);
    assert_eq!(unmine.status, HintStatus::StaleBlock);
    let reanchor = Verdict {
        status: "MINED".into(),
        height: Some(11),
        reason: Some("reorg_reanchor".into()),
        competitors: vec![],
    }
    .into_hint("Arcade", 11);
    assert_eq!(reanchor.status, HintStatus::Mined { height: 11 });
}
