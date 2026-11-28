//! The doubts' rows of record: `tests/vectors/doubts/rows_of_record.json`,
//! cases taken from the `#guard` rows of bsv-stack-lean (`lean/Headers/Doubts.lean`
//! and `lean/Headers/Host.lean`; the file's `source` names the commit and the
//! sections), and the stored form of `Doubts`. No case is a journal row.
//!
//! Each case names a settled line, a fold (`run`, or `raw` alone for the
//! Lean's `runUnpruned`), a start, its rows in the Lean's helpers' terms and
//! the printed line after the last row. Its `asks` are read by
//! `doubts::eff_depth` under headers that carry no header (the Lean's `hdAt`).
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
use bsv_tracker::{ChainEvent, Params};
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
            // The check after every row: under the case's number, its `out`;
            // under no number, within, and the doubts are `run`'s.
            let (bound, out) = match case.get("bound") {
                Some(Value::Null) => (None, case["out"].as_bool().unwrap()),
                Some(n) => (Some(height(n)), case["out"].as_bool().unwrap()),
                None => (None, false),
            };
            let params = Params::new(60, 1).with_bound(bound);
            let checked = doubts::run_checked(&bounds, &params, &from, &rows);
            assert_eq!(checked.is_out(), out, "{name}: out under {bound:?}");
            assert_eq!(checked.doubts(), &after, "{name}: the check drops nothing");
            // `run_checked` is `step_checked` folded: out when some step is.
            let mut stepped = from.clone();
            let mut some_out = false;
            for e in &rows {
                let one = doubts::step_checked(&bounds, &params, e, &stepped);
                some_out |= one.is_out();
                assert_eq!(one.is_out(), doubts::over(&params, one.doubts()));
                stepped = one.into_doubts();
            }
            assert_eq!(some_out, out, "{name}: the check is after every row");
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
    assert_eq!(ran, 80, "the rows of record hold 80 cases");
    assert_eq!(file["counts"]["cases"], 80);
}

/// The fold a case names, from its start.
fn fold_of(case: &Value) -> Doubts {
    let bounds = Bounds::new(height(&case["settled"]));
    let from = start(&case["start"]);
    let rows: Vec<ChainEvent> = case["rows"].as_array().unwrap().iter().map(row).collect();
    match case["fold"].as_str().unwrap() {
        "run" => doubts::run(&bounds, &from, &rows),
        "raw" => rows.iter().fold(from, |ds, e| doubts::raw(e, &ds)),
        other => panic!("a fold this test does not know: {other}"),
    }
}

/// Every ask of every case: `[headers_tip, x, eff]` is `eff_depth` at `x`
/// under headers at `headers_tip` that carry no header, after the case's
/// rows. The depth reads the headers' tip alone; it asks for no header.
#[test]
fn rows_of_record_asks() {
    let file: Value = serde_json::from_str(ROWS_OF_RECORD).unwrap();
    let mut asked = 0;
    for case in file["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let after = fold_of(case);
        for ask in case["asks"].as_array().unwrap() {
            let ask = ask.as_array().expect("an ask is [headers_tip, x, eff]");
            assert_eq!(ask.len(), 3, "{name}: an ask is [headers_tip, x, eff]");
            let headers = TestHeaders {
                tip: height(&ask[0]),
                ..TestHeaders::default()
            };
            let x = height(&ask[1]);
            let eff = ask[2].as_u64().unwrap();
            assert_eq!(
                doubts::eff_depth(&headers, &after, x),
                Ok(eff),
                "{name}: eff_depth at {x} under headers at {}",
                headers.tip
            );
            // The lead is read under the lesser tip.
            assert_eq!(
                doubts::lead_tip(&headers, &after),
                Ok(after.tip().min(headers.tip)),
                "{name}: lead_tip"
            );
            asked += 1;
        }
    }
    assert_eq!(asked, 184, "the rows of record hold 184 asks");
    assert_eq!(file["counts"]["asks"], 184);
}

const SPLIT_CASES: &str = include_str!("vectors/doubts/split_cases.txt");

/// A row of the stack's oracle: `T <height>`, `S <key> <fork> <tip> <tip>`
/// or `E <key> <fork>`, the key without its `split:` prefix.
fn oracle_row(fields: &[&str]) -> ChainEvent {
    let n = |text: &str| text.parse::<u32>().expect("a height");
    match fields {
        ["T", h] => tip_row(n(h)),
        ["S", key, fh, a, b] => keyed_split_row(key, n(fh), n(a), n(b)),
        ["E", key, fh] => keyed_resolved_row(key, n(fh)),
        _ => panic!("an oracle row of a shape this test does not know: {fields:?}"),
    }
}

/// The stack's reducer's own run over twelve rows
/// (`tests/vectors/doubts/split_cases.txt`): replayed through `step` from a
/// cold start, every state line byte for byte.
#[test]
fn split_cases_oracle() {
    let mut lines = SPLIT_CASES.lines().filter(|line| !line.starts_with('#'));
    let mut ds = Doubts::cold(0);
    let mut cursor = 0;
    while let Some(row) = lines.next() {
        cursor += 1;
        let state = lines.next().expect("every row has its state line");
        let row = row
            .strip_prefix(&format!("row {cursor} "))
            .expect("the rows are numbered from 1");
        let fields: Vec<&str> = row.split(' ').collect();
        ds = doubts::step(&Bounds::B400, &oracle_row(&fields), &ds);
        assert_eq!(format!("state {cursor} {ds}"), state, "row {cursor}: {row}");
    }
    assert_eq!(cursor, 12, "the oracle holds twelve rows");
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
            "one key named twice",
            r#"{"rivals":[],"splits":[["ka",5],["ka",5]],"tip":6,"gone":null}"#,
        ),
        (
            "one key named twice at two heights",
            r#"{"rivals":[],"splits":[["ka",5],["kb",7],["ka",9]],"tip":6,"gone":null}"#,
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

/// The count against the named number: more than it is over, as many is
/// within; the start is not counted; no number is no check.
#[test]
fn bound_counts_after_rows() {
    let with = |bound| Params::new(60, 1).with_bound(bound);
    let two = Doubts::new(vec![], vec![("ka".into(), 50), ("kb".into(), 50)], 60, None);
    assert!(!doubts::over(&with(None), &two));
    assert!(!doubts::over(&with(Some(2)), &two));
    assert!(doubts::over(&with(Some(1)), &two));
    assert!(doubts::over(&with(Some(0)), &two));
    assert!(!doubts::over(&with(Some(0)), &Doubts::cold(60)));
    // A start over the number with no row read: `trips []` is false.
    let none: [ChainEvent; 0] = [];
    let checked = doubts::run_checked(&Bounds::B400, &with(Some(0)), &two, &none);
    assert_eq!(checked, doubts::Checked::Within(two.clone()));
    // A row that leaves it over is out, though it opens no key.
    let one = doubts::step_checked(&Bounds::B400, &with(Some(1)), &tip_row(61), &two);
    assert!(one.is_out());
    assert_eq!(one.doubts().splits(), two.splits());
    // The same split row twice adds nothing: a repeat never trips the check.
    let rows = [
        keyed_split_row("ka", 50, 52, 53),
        keyed_split_row("ka", 50, 52, 53),
    ];
    let checked = doubts::run_checked(&Bounds::B400, &with(Some(1)), &Doubts::cold(60), &rows);
    assert!(!checked.is_out());
    assert_eq!(checked.doubts().splits(), [("ka".to_owned(), 50)]);
}

/// A key in mixed case is held in lower case through every way in: a row
/// (`raw` lower-cases it), `Doubts::new` and the stored form's reader. A key
/// that came in through either of the last two in upper case once stayed so,
/// and a `splitResolved` row for it, read in lower case, never closed it: the
/// split held for ever (the repository's issue #57). Two spellings of one key
/// in a stored text are one key named twice, and refused.
#[test]
fn a_mixed_case_key_is_held_in_lower_case() {
    let upper = "AB".repeat(32);
    let lower = "ab".repeat(32);
    let built = Doubts::new(vec![], vec![(upper.clone(), 50)], 60, None);
    assert_eq!(built.splits(), [(lower.clone(), 50)]);
    let text = format!(r#"{{"rivals":[],"splits":[["{upper}",50]],"tip":60,"gone":null}}"#);
    let read: Doubts = serde_json::from_str(&text).unwrap();
    assert_eq!(read, built);
    assert_eq!(
        serde_json::to_string(&read).unwrap(),
        serde_json::to_string(&built).unwrap()
    );
    for ds in [&built, &read] {
        let closed = doubts::step(&Bounds::B400, &keyed_resolved_row(&upper, 50), ds);
        assert!(
            closed.splits().is_empty(),
            "the upper-case resolve: {closed}"
        );
        let closed = doubts::step(&Bounds::B400, &keyed_resolved_row(&lower, 50), ds);
        assert!(
            closed.splits().is_empty(),
            "the lower-case resolve: {closed}"
        );
    }
    let twice =
        format!(r#"{{"rivals":[],"splits":[["{upper}",50],["{lower}",52]],"tip":60,"gone":null}}"#);
    assert!(
        serde_json::from_str::<Doubts>(&twice).is_err(),
        "two spellings of one key must be refused: {twice}"
    );
}
