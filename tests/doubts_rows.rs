//! The doubts' rows of record: `tests/vectors/doubts/rows_of_record.json`,
//! cases taken from the `#guard` rows of bsv-stack-lean (`lean/Headers/Doubts.lean`
//! and `lean/Headers/Host.lean`; the file's `source` names the commit and the
//! sections), and the stored form of `Doubts`. No case is a journal row.
//!
//! Each case names a settled line, a fold (`run`, or `raw` alone for the
//! Lean's `runUnpruned`), a start, its rows in the Lean's helpers' terms and
//! the printed line after the last row. Its `asks` are read by the depth
//! functions, which this file does not test yet.
// The lint bar binds the library; a test may unwrap, and its fixtures fail loudly
// (bsv-stack-lean #134).
#![allow(
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod common;
use bsv_tracker::doubts::{self, Bounds, Doubts, Rival};
use bsv_tracker::ChainEvent;
use common::*;
use serde_json::Value;

const ROWS_OF_RECORD: &str = include_str!("vectors/doubts/rows_of_record.json");

fn height(value: &Value) -> u32 {
    u32::try_from(value.as_u64().expect("a height is a number")).expect("a height fits u32")
}

fn pair(value: &Value) -> Rival {
    let pair = value.as_array().expect("a rival is [fork, tip]");
    assert_eq!(pair.len(), 2, "a rival is [fork, tip]: {value}");
    Rival::new(height(&pair[0]), height(&pair[1]))
}

fn split(value: &Value) -> (String, u32) {
    let pair = value.as_array().expect("an open split is [key, fork]");
    assert_eq!(pair.len(), 2, "an open split is [key, fork]: {value}");
    (
        pair[0].as_str().expect("a key").to_owned(),
        height(&pair[1]),
    )
}

fn start(value: &Value) -> Doubts {
    let start = value.as_object().expect("a start is an object");
    assert_eq!(start.len(), 1, "a start is cold or a doubts value: {value}");
    if let Some(tip) = start.get("cold") {
        return Doubts::cold(height(tip));
    }
    let ds = start["doubts"].as_object().expect("a doubts value");
    assert_eq!(ds.len(), 4, "a doubts value has four parts: {value}");
    Doubts::new(
        ds["rivals"].as_array().unwrap().iter().map(pair).collect(),
        ds["splits"].as_array().unwrap().iter().map(split).collect(),
        height(&ds["tip"]),
        match &ds["gone"] {
            Value::Null => None,
            gone => Some(pair(gone)),
        },
    )
}

fn row(value: &Value) -> ChainEvent {
    let row = value.as_array().expect("a row is an array");
    let kind = row[0].as_str().expect("a row begins with its kind");
    if let ("split" | "resolved", Some(key)) = (kind, row.get(1).and_then(Value::as_str)) {
        let n: Vec<u32> = row[2..].iter().map(height).collect();
        return match (kind, n.as_slice()) {
            ("split", [fh, lo, hi]) => keyed_split_row(key, *fh, *lo, *hi),
            ("resolved", [fh]) => keyed_resolved_row(key, *fh),
            _ => panic!("a row of a shape this test does not know: {value}"),
        };
    }
    let n: Vec<u32> = row[1..].iter().map(height).collect();
    match (kind, n.as_slice()) {
        ("tip", [h]) => tip_row(*h),
        ("fork", [f, t, competitor]) => fork_row(*f, *t, *competitor),
        ("reorg", [fh, old, nt]) => reorg_row(*fh, *old, *nt),
        ("reorg_none", [fh, nt]) => reorg_none(*fh, *nt),
        _ => panic!("a row of a shape this test does not know: {value}"),
    }
}

/// The state line of every case, through `Display`.
#[test]
fn rows_of_record_states() {
    let file: Value = serde_json::from_str(ROWS_OF_RECORD).unwrap();
    assert_eq!(file["schema"], "bsv-tracker-doubts-rows/2");
    let cases = file["cases"].as_array().unwrap();
    let mut ran = 0;
    let mut names = std::collections::BTreeSet::new();
    for case in cases {
        let name = case["name"].as_str().unwrap();
        assert!(names.insert(name), "{name}: named twice");
        let bounds = Bounds::new(height(&case["settled"]));
        let from = start(&case["start"]);
        let rows: Vec<ChainEvent> = case["rows"].as_array().unwrap().iter().map(row).collect();
        let after = match case["fold"].as_str().unwrap() {
            "run" => doubts::run(&bounds, &from, &rows),
            "raw" => rows.iter().fold(from.clone(), |ds, e| doubts::raw(e, &ds)),
            other => panic!("{name}: a fold this test does not know: {other}"),
        };
        assert_eq!(
            after.to_string(),
            case["state"].as_str().unwrap(),
            "{name}: the state after the last row"
        );
        if case["fold"] == "run" {
            // `run` is `step` folded from the left, and `step` is `prune` of `raw`.
            let by_step = rows.iter().fold(from.clone(), |ds, e| {
                let stepped = doubts::step(&bounds, e, &ds);
                assert_eq!(stepped, doubts::prune(&bounds, &doubts::raw(e, &ds)));
                stepped
            });
            assert_eq!(by_step, after, "{name}: run is the fold of step");
        }
        ran += 1;
    }
    assert_eq!(ran, 53, "the rows of record hold 53 cases");
    assert_eq!(file["counts"]["cases"], 53);
}

/// Three golden texts, each read and written back identical.
#[test]
fn stored_form_round_trips() {
    let golden = [
        (
            r#"{"rivals":[],"splits":[],"tip":0,"gone":null}"#,
            Doubts::cold(0),
        ),
        (
            r#"{"rivals":[{"fork":101,"tip":101},{"fork":95,"tip":99}],"splits":[["k95",95]],"tip":103,"gone":null}"#,
            Doubts::new(
                vec![Rival::new(101, 101), Rival::new(95, 99)],
                vec![("k95".into(), 95)],
                103,
                None,
            ),
        ),
        (
            r#"{"rivals":[{"fork":405,"tip":406}],"splits":[],"tip":6,"gone":{"fork":5,"tip":5}}"#,
            Doubts::new(
                vec![Rival::new(405, 406)],
                vec![],
                6,
                Some(Rival::new(5, 5)),
            ),
        ),
    ];
    for (text, value) in golden {
        let read: Doubts = serde_json::from_str(text).expect("the golden text must read");
        assert_eq!(read, value, "{text}");
        assert_eq!(serde_json::to_string(&read).unwrap(), text);
    }
    // The readers give back what `new` was given.
    let (_, ds) = &golden_parts();
    assert_eq!(ds.rivals(), [Rival::new(405, 406)]);
    assert_eq!(ds.splits(), [("kb".into(), 7), ("ka".into(), 7)]);
    assert_eq!(ds.tip(), 6);
    assert_eq!(ds.gone(), Some(&Rival::new(5, 5)));
    assert_eq!(
        ds.held().cloned().collect::<Vec<_>>(),
        [Rival::new(5, 5), Rival::new(405, 406)],
        "the summary, then the rivals"
    );
    assert_eq!(
        ds.to_string(),
        "tip=6 splits=[kb:7 ka:7] rivals=[405:406] summary=5:5"
    );
    assert_eq!(Bounds::B400, Bounds::new(400));
}

fn golden_parts() -> (&'static str, Doubts) {
    (
        r#"{"rivals":[{"fork":405,"tip":406}],"splits":[["kb",7],["ka",7]],"tip":6,"gone":{"fork":5,"tip":5}}"#,
        Doubts::new(
            vec![Rival::new(405, 406)],
            vec![("kb".into(), 7), ("ka".into(), 7)],
            6,
            Some(Rival::new(5, 5)),
        ),
    )
}

/// A text the reader does not know whole is refused, never defaulted.
#[test]
fn stored_form_refuses() {
    let (whole, value) = golden_parts();
    assert_eq!(serde_json::from_str::<Doubts>(whole).unwrap(), value);
    let refused = [
        (
            "an unknown field",
            r#"{"rivals":[],"splits":[],"tip":6,"gone":null,"cursor":1}"#,
        ),
        ("a missing gone", r#"{"rivals":[],"splits":[],"tip":6}"#),
        ("a missing tip", r#"{"rivals":[],"splits":[],"gone":null}"#),
        ("a missing rivals", r#"{"splits":[],"tip":6,"gone":null}"#),
        ("a missing splits", r#"{"rivals":[],"tip":6,"gone":null}"#),
        (
            "a rival with a third field",
            r#"{"rivals":[{"fork":1,"tip":2,"work":3}],"splits":[],"tip":6,"gone":null}"#,
        ),
        (
            "a summary with a third field",
            r#"{"rivals":[],"splits":[],"tip":6,"gone":{"fork":1,"tip":2,"work":3}}"#,
        ),
        (
            "a rival without its tip",
            r#"{"rivals":[{"fork":1}],"splits":[],"tip":6,"gone":null}"#,
        ),
        (
            "a height below zero",
            r#"{"rivals":[],"splits":[],"tip":-1,"gone":null}"#,
        ),
        (
            "a height past u32",
            r#"{"rivals":[],"splits":[["ka",4294967296]],"tip":6,"gone":null}"#,
        ),
        (
            "an open split as a bare fork height (lane 1's form)",
            r#"{"rivals":[],"splits":[95],"tip":6,"gone":null}"#,
        ),
        (
            "an open split without its fork height",
            r#"{"rivals":[],"splits":[["ka"]],"tip":6,"gone":null}"#,
        ),
        (
            "an open split with a third part",
            r#"{"rivals":[],"splits":[["ka",5,6]],"tip":6,"gone":null}"#,
        ),
        (
            "an open split whose key is not text",
            r#"{"rivals":[],"splits":[[5,5]],"tip":6,"gone":null}"#,
        ),
    ];
    for (why, text) in refused {
        assert!(
            serde_json::from_str::<Doubts>(text).is_err(),
            "{why} must be refused: {text}"
        );
    }
}
