//! The kinds' words and the stored form of a hint status (the make-room pull
//! request of the rulings of 2026-11-17, bsv-stack-lean
//! `docs/DECISIONS-2026-10-19.md` 28.8.1 (c)): each kind's word, the two
//! lists in order, the serde form of every status as text, the refusals, and
//! the host's shape, read and written byte for byte as bsv-wallet-cli stores
//! its hints today.
// The lint bar binds the library; a test may unwrap, and its fixtures fail loudly
// (bsv-stack-lean #134).
#![allow(
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
use bsv_tracker::*;
use serde::{Deserialize, Serialize};

fn id() -> String {
    "ab".repeat(32)
}

/// Each status with its word and its stored form, in declaration order.
fn statuses() -> Vec<(HintStatus, &'static str, String)> {
    vec![
        (
            HintStatus::Accepted,
            "accepted",
            r#"{"status":"accepted"}"#.into(),
        ),
        (HintStatus::Seen, "seen", r#"{"status":"seen"}"#.into()),
        (
            HintStatus::Mined { height: 900 },
            "mined",
            r#"{"status":"mined","height":900}"#.into(),
        ),
        (
            HintStatus::StaleBlock,
            "stale_block",
            r#"{"status":"stale_block"}"#.into(),
        ),
        (
            HintStatus::Rejected { reason: "r".into() },
            "rejected",
            r#"{"status":"rejected","reason":"r"}"#.into(),
        ),
        (
            HintStatus::DoubleSpend {
                competitors: vec![id()],
            },
            "double_spend",
            format!(r#"{{"status":"double_spend","competitors":["{}"]}}"#, id()),
        ),
        (
            HintStatus::DoubleSpend {
                competitors: vec![],
            },
            "double_spend",
            r#"{"status":"double_spend"}"#.into(),
        ),
        (
            HintStatus::OrphanMempool,
            "orphan_mempool",
            r#"{"status":"orphan_mempool"}"#.into(),
        ),
        (
            HintStatus::Unknown,
            "unknown",
            r#"{"status":"unknown"}"#.into(),
        ),
        (
            HintStatus::MempoolPresent { source: "c".into() },
            "mempool_present",
            r#"{"status":"mempool_present","courier":"c"}"#.into(),
        ),
    ]
}

fn reasks() -> Vec<(Reask, &'static str)> {
    vec![
        (Reask::Reorg { fork_height: 10 }, "reorg"),
        (Reask::Fork { height: 10 }, "fork"),
        (Reask::Hint(Hint::new("s", HintStatus::Accepted, 1)), "hint"),
        (Reask::ProofFailed { height: 10 }, "proof_failed"),
        (Reask::Recheck, "recheck"),
        (Reask::Age, "age"),
        (Reask::Spend, "spend"),
    ]
}

#[test]
fn every_kind_has_its_word() {
    for (status, word, _) in statuses() {
        assert_eq!(status.label(), word, "{status:?}");
    }
    for (reask, word) in reasks() {
        assert_eq!(reask.label(), word, "{reask:?}");
    }
}

#[test]
fn the_lists_are_the_words_in_declaration_order() {
    assert_eq!(
        HintStatus::LABELS,
        &[
            "accepted",
            "seen",
            "mined",
            "stale_block",
            "rejected",
            "double_spend",
            "orphan_mempool",
            "unknown",
            "mempool_present",
        ]
    );
    assert_eq!(
        Reask::LABELS,
        &[
            "reorg",
            "fork",
            "hint",
            "proof_failed",
            "recheck",
            "age",
            "spend"
        ]
    );
    let mut words: Vec<&str> = statuses().iter().map(|(_, w, _)| *w).collect();
    words.dedup();
    assert_eq!(words, HintStatus::LABELS);
    let words: Vec<&str> = reasks().iter().map(|(_, w)| *w).collect();
    assert_eq!(words, Reask::LABELS);
}

#[test]
fn every_status_round_trips_through_its_stored_form() {
    for (status, word, text) in statuses() {
        assert_eq!(serde_json::to_string(&status).unwrap(), text, "{word}");
        let back: HintStatus = serde_json::from_str(&text).unwrap();
        assert_eq!(back, status, "{word}");
        assert_eq!(serde_json::to_string(&back).unwrap(), text, "{word}");
        // The serde tag is the label.
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["status"], status.label(), "{word}");
    }
}

#[test]
fn a_word_this_release_does_not_know_and_a_missing_payload_are_refused() {
    for text in [
        r#"{"status":"non_final"}"#,
        r#"{"status":"mined"}"#,
        r#"{"status":"mempool_present"}"#,
        r#"{"status":"rejected"}"#,
        r#"{"status":"Accepted"}"#,
        r#"{"height":900}"#,
    ] {
        assert!(
            serde_json::from_str::<HintStatus>(text).is_err(),
            "{text} must be refused, never read as unknown"
        );
    }
}

/// The host's shape: a hint's source, its status flattened beside it, and
/// its observed time, the keys in the order bsv-wallet-cli writes them.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct HostHint {
    source: String,
    #[serde(flatten)]
    status: HintStatus,
    observed: Timestamp,
}

/// bsv-wallet-cli's `StoredHint` and its `of`, copied from its beta line
/// (`src/tracker_host.rs:408-452` at 108f80e), so the texts below are what
/// that host has already stored. Its match gains the arm a host's sentence
/// asks for: a kind it does not name fails aloud.
#[derive(Serialize)]
struct CliStoredHint {
    source: String,
    status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    height: Option<Height>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    competitors: Vec<TxId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    courier: Option<String>,
    observed: Timestamp,
}
impl CliStoredHint {
    fn of(hint: &Hint) -> Self {
        let mut courier = None;
        let (status, height, reason, competitors) = match &hint.status {
            HintStatus::Accepted => ("accepted", None, None, vec![]),
            HintStatus::Seen => ("seen", None, None, vec![]),
            HintStatus::Mined { height } => ("mined", Some(*height), None, vec![]),
            HintStatus::StaleBlock => ("stale_block", None, None, vec![]),
            HintStatus::Rejected { reason } => ("rejected", None, Some(reason.clone()), vec![]),
            HintStatus::DoubleSpend { competitors } => {
                ("double_spend", None, None, competitors.clone())
            }
            HintStatus::OrphanMempool => ("orphan_mempool", None, None, vec![]),
            HintStatus::Unknown => ("unknown", None, None, vec![]),
            HintStatus::MempoolPresent { source } => {
                courier = Some(source.clone());
                ("mempool_present", None, None, vec![])
            }
            other => panic!("a kind the copy of the CLI's map does not name: {other:?}"),
        };
        Self {
            source: hint.source.clone(),
            status: status.to_string(),
            height,
            reason,
            competitors,
            courier,
            observed: hint.observed,
        }
    }
}

#[test]
fn the_hosts_shape_reads_and_writes_the_rows_the_cli_has_stored() {
    let rows = [
        r#"{"source":"arc","status":"accepted","observed":1700000000}"#.to_string(),
        r#"{"source":"arc","status":"seen","observed":1700000000}"#.to_string(),
        r#"{"source":"arc","status":"mined","height":900,"observed":1700000000}"#.to_string(),
        r#"{"source":"arc","status":"stale_block","observed":1700000000}"#.to_string(),
        r#"{"source":"arc","status":"rejected","reason":"r","observed":1700000000}"#.to_string(),
        r#"{"source":"arc","status":"rejected","reason":"","observed":1700000000}"#.to_string(),
        format!(
            r#"{{"source":"arc","status":"double_spend","competitors":["{}"],"observed":1700000000}}"#,
            id()
        ),
        r#"{"source":"arc","status":"double_spend","observed":1700000000}"#.to_string(),
        r#"{"source":"arc","status":"orphan_mempool","observed":1700000000}"#.to_string(),
        r#"{"source":"arc","status":"unknown","observed":1700000000}"#.to_string(),
        r#"{"source":"arc","status":"mempool_present","courier":"c","observed":1700000000}"#
            .to_string(),
    ];
    let mut statuses: Vec<HintStatus> = statuses().into_iter().map(|(s, _, _)| s).collect();
    statuses.insert(5, HintStatus::Rejected { reason: "".into() });
    assert_eq!(rows.len(), statuses.len());
    for (text, status) in rows.iter().zip(statuses) {
        let hint = Hint::new("arc", status.clone(), 1_700_000_000);
        // The row is the text the CLI writes for this hint.
        assert_eq!(
            &serde_json::to_string(&CliStoredHint::of(&hint)).unwrap(),
            text
        );
        // The host's shape reads it as the status, and writes it back unchanged.
        let read: HostHint = serde_json::from_str(text).unwrap();
        assert_eq!(read.source, "arc");
        assert_eq!(read.status, status, "{text}");
        assert_eq!(read.observed, 1_700_000_000);
        assert_eq!(&serde_json::to_string(&read).unwrap(), text);
    }
    // A row whose word this release does not know is refused in the shape too.
    assert!(serde_json::from_str::<HostHint>(
        r#"{"source":"arc","status":"non_final","observed":1}"#
    )
    .is_err());
}
