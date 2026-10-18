//! The nine Arcade bodies where LOW's door and the tracker read differently
//! (bsv-stack-lean #169; bsv-overlay-cloudflare `crates/overlay-cloudflare/src/door_words.rs`,
//! test `tracker_vocabulary_differs_from_the_door_exactly_here`). Each row is
//! the body verbatim and the 0.4.1 reading: a hint, never a verdict.
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

const TXID: &str = "aa00000000000000000000000000000000000000000000000000000000000001";
const COMPETITOR: &str = "bb00000000000000000000000000000000000000000000000000000000000002";

/// Decode the body, reduce it, and check the hint writes no chain word.
fn read(body: &str) -> Hint {
    let hint = Verdict::from_arc_body(TXID, body.as_bytes())
        .unwrap()
        .into_hint("Arcade", 7);
    let mut s = State::new(TXID);
    step(
        &mut s,
        &TestHeaders::default(),
        Input::Host(HostAction::Build),
    );
    step(&mut s, &TestHeaders::default(), Input::Hint(hint.clone()));
    assert_eq!(s.word().chain(), None, "{body}");
    assert_eq!(s.evidence(), None, "{body}");
    hint
}
fn status_word(hint: &Hint) -> &str {
    &hint.verdict.as_ref().unwrap().status
}

#[test]
fn stump_processing_is_accepted() {
    let hint = read(r#"{"txStatus":"STUMP_PROCESSING"}"#);
    assert_eq!(hint.status, HintStatus::Accepted);
    assert_eq!(status_word(&hint), "STUMP_PROCESSING");
}

#[test]
fn pending_retry_is_accepted() {
    let hint = read(r#"{"txStatus":"PENDING_RETRY"}"#);
    assert_eq!(hint.status, HintStatus::Accepted);
    assert_eq!(status_word(&hint), "PENDING_RETRY");
}

#[test]
fn mined_without_a_height_is_unknown() {
    let hint = read(r#"{"txStatus":"MINED"}"#);
    assert_eq!(hint.status, HintStatus::Unknown);
    assert_eq!(status_word(&hint), "MINED");
}

#[test]
fn immutable_without_a_height_is_unknown() {
    let hint = read(r#"{"txStatus":"IMMUTABLE"}"#);
    assert_eq!(hint.status, HintStatus::Unknown);
    assert_eq!(status_word(&hint), "IMMUTABLE");
}

#[test]
fn seen_with_a_stale_orphan_extra_info_is_seen() {
    let hint = read(r#"{"txStatus":"SEEN_MULTIPLE_NODES","extraInfo":"ORPHAN parents missing"}"#);
    assert_eq!(hint.status, HintStatus::Seen);
    assert_eq!(status_word(&hint), "SEEN_MULTIPLE_NODES");
    assert_eq!(
        hint.verdict.as_ref().unwrap().reason.as_deref(),
        Some("ORPHAN parents missing")
    );
}

#[test]
fn seen_with_competitors_is_a_double_spend() {
    let body = format!(r#"{{"txStatus":"SEEN_ON_NETWORK","competingTxs":["{COMPETITOR}"]}}"#);
    let hint = read(&body);
    assert_eq!(
        hint.status,
        HintStatus::DoubleSpend {
            competitors: vec![COMPETITOR.into()]
        }
    );
    assert_eq!(status_word(&hint), "SEEN_ON_NETWORK");
}

#[test]
fn a_spaced_word_is_trimmed() {
    let hint = read(r#"{"txStatus":" SEEN_ON_NETWORK "}"#);
    assert_eq!(hint.status, HintStatus::Seen);
    assert_eq!(status_word(&hint), "SEEN_ON_NETWORK");
}

#[test]
fn invalid_is_rejected_and_utxo_spent_is_a_double_spend() {
    let hint = read(r#"{"txStatus":"INVALID"}"#);
    assert_eq!(
        hint.status,
        HintStatus::Rejected {
            reason: "INVALID".into()
        }
    );
    assert_eq!(status_word(&hint), "INVALID");
    let hint = read(r#"{"txStatus":"UTXO_SPENT"}"#);
    assert_eq!(
        hint.status,
        HintStatus::DoubleSpend {
            competitors: vec![]
        }
    );
    assert_eq!(status_word(&hint), "UTXO_SPENT");
}

#[test]
fn confirmed_with_a_height_is_mined() {
    let hint = read(r#"{"txStatus":"CONFIRMED","blockHeight":970710}"#);
    assert_eq!(hint.status, HintStatus::Mined { height: 970_710 });
    assert_eq!(status_word(&hint), "CONFIRMED");
}
