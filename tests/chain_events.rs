//! Wire witnesses from rust-chaintracks@a62f9ed docs/CHAIN-EVENTS.md:265-285.
//! JSON field order is not a schema rule. The two short examples also match
//! byte for byte; the four header-bearing examples use JSON value equality.
mod common;
use bsv_tracker::*;
use common::*;
use serde_json::{json, Value};

fn round_trip(wire: &str, byte_equal: bool) {
    let wire = wire.trim_end();
    let event = decode_envelope(wire.as_bytes()).expect("the pinned envelope must decode");
    let direct: ChainEvent = serde_json::from_str(wire).expect("ChainEvent serde must gate v:1");
    assert_eq!(direct, event);
    let envelope: ChainEnvelope = serde_json::from_str(wire).expect("typed envelope must decode");
    assert_eq!(envelope.version(), 1);
    assert_eq!(envelope.event(), &event);
    let encoded = serde_json::to_string(&envelope).unwrap();
    assert_eq!(serde_json::to_string(&event).unwrap(), encoded);
    assert_eq!(decode_envelope(encoded.as_bytes()), Ok(event.clone()));
    assert_eq!(envelope.into_event(), event);
    if byte_equal {
        assert_eq!(encoded, wire);
    } else {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&encoded).unwrap(),
            serde_json::from_str::<serde_json::Value>(wire).unwrap()
        );
    }
}

#[test]
fn pinned_tip_round_trip() {
    round_trip(include_str!("vectors/chain_events/tip.json"), false);
}

#[test]
fn pinned_fork_round_trip() {
    round_trip(include_str!("vectors/chain_events/fork.json"), false);
}

#[test]
fn pinned_reorg_round_trip() {
    round_trip(include_str!("vectors/chain_events/reorg.json"), false);
}

#[test]
fn pinned_invalidated_round_trip() {
    round_trip(include_str!("vectors/chain_events/invalidated.json"), true);
}

#[test]
fn pinned_frozen_round_trip() {
    round_trip(include_str!("vectors/chain_events/frozen.json"), true);
}

#[test]
fn pinned_tip_age_round_trip() {
    round_trip(include_str!("vectors/chain_events/tip_age.json"), false);
}

#[test]
fn future_kind_and_version_are_typed_host_faults() {
    // A host reports the Err and delivers no event. The version gate precedes
    // even an unknown kind or a malformed known-version body, in either order.
    for (wire, expected) in [
        (
            r#"{"v":1,"kind":"futureKind"}"#,
            DecodeError::UnknownKind("futureKind".into()),
        ),
        (r#"{"kind":"tip","v":2}"#, DecodeError::UnknownVersion(2)),
        (
            r#"{"v":2,"kind":"futureKind"}"#,
            DecodeError::UnknownVersion(2),
        ),
    ] {
        assert_eq!(decode_envelope(wire.as_bytes()), Err(expected.clone()));
        let direct = serde_json::from_str::<ChainEvent>(wire).unwrap_err();
        let envelope = serde_json::from_str::<ChainEnvelope>(wire).unwrap_err();
        assert!(direct.to_string().contains(&expected.to_string()));
        assert!(envelope.to_string().contains(&expected.to_string()));
    }
}

fn rejects_shape(value: &Value) {
    let wire = serde_json::to_vec(value).unwrap();
    assert!(
        matches!(
            decode_envelope(&wire),
            Err(DecodeError::UnknownShape { .. })
        ),
        "malformed envelope was accepted: {value}"
    );
    assert!(serde_json::from_slice::<ChainEvent>(&wire).is_err());
    assert!(serde_json::from_slice::<ChainEnvelope>(&wire).is_err());
}

#[test]
fn known_shapes_refuse_legacy_fields_and_malformed_headers() {
    let tip: Value = serde_json::from_str(include_str!("vectors/chain_events/tip.json")).unwrap();
    let fork: Value = serde_json::from_str(include_str!("vectors/chain_events/fork.json")).unwrap();
    let reorg: Value =
        serde_json::from_str(include_str!("vectors/chain_events/reorg.json")).unwrap();
    let frozen: Value =
        serde_json::from_str(include_str!("vectors/chain_events/frozen.json")).unwrap();
    for legacy in [
        json!({"v": 1, "kind": "tip", "height": 102, "hash": hash(1)}),
        json!({"v": 1, "kind": "fork", "height": 101, "competingTips": [hash(1), hash(2)], "depth": 1}),
        json!({"v": 1, "kind": "reorg", "forkHeight": 101, "depth": 1, "deactivated": [hash(1)], "newTip": hash(2)}),
        json!({"v": 1, "kind": "frozen", "outpoint": format!("{}:0", hash(1))}),
        json!({"v": 1, "kind": "tipAge", "seconds": 600}),
    ] {
        rejects_shape(&legacy);
    }
    for field in [
        "version",
        "previousHash",
        "merkleRoot",
        "time",
        "bits",
        "nonce",
        "height",
        "hash",
        "chainWork",
    ] {
        let mut missing = tip.clone();
        missing["header"].as_object_mut().unwrap().remove(field);
        rejects_shape(&missing);
    }
    for field in ["version", "time", "bits", "nonce", "height"] {
        let mut overflow = tip.clone();
        overflow["header"][field] = json!(u64::from(u32::MAX) + 1);
        rejects_shape(&overflow);
    }
    for field in ["previousHash", "merkleRoot", "hash", "chainWork"] {
        let mut malformed = tip.clone();
        malformed["header"][field] = json!("z".repeat(64));
        rejects_shape(&malformed);
    }
    let mut missing_version = tip.clone();
    missing_version.as_object_mut().unwrap().remove("v");
    rejects_shape(&missing_version);
    let mut malformed_version = tip.clone();
    malformed_version["v"] = json!("1");
    rejects_shape(&malformed_version);
    let mut malformed_kind = tip.clone();
    malformed_kind["kind"] = json!(null);
    rejects_shape(&malformed_kind);
    let mut extra = tip.clone();
    extra["header"]["isActive"] = json!(true);
    rejects_shape(&extra);
    let mut extra = tip.clone();
    extra["cursor"] = json!("42");
    rejects_shape(&extra);
    for field in ["height", "hash", "time"] {
        let mut inconsistent = tip.clone();
        inconsistent[field] = if field == "hash" {
            json!(hash(0))
        } else {
            json!(0)
        };
        rejects_shape(&inconsistent);
    }
    let mut wrong_count = fork.clone();
    wrong_count["competingTips"].as_array_mut().unwrap().pop();
    rejects_shape(&wrong_count);
    let mut wrong_count = fork.clone();
    wrong_count["competingTips"]
        .as_array_mut()
        .unwrap()
        .push(fork["competingTips"][0].clone());
    rejects_shape(&wrong_count);
    let mut repeated_tip = fork.clone();
    repeated_tip["competingTips"][1] = fork["competingTips"][0].clone();
    rejects_shape(&repeated_tip);
    let mut wrong_depth = reorg.clone();
    wrong_depth["depth"] = json!(2);
    rejects_shape(&wrong_depth);
    let mut wrong_boundary = reorg.clone();
    wrong_boundary["forkHeight"] = json!(100);
    rejects_shape(&wrong_boundary);
    let mut old_name = reorg.clone();
    let removed = old_name
        .as_object_mut()
        .unwrap()
        .remove("deactivatedHeaders")
        .unwrap();
    old_name["deactivated"] = removed;
    rejects_shape(&old_name);
    let mut malformed_outpoint = frozen.clone();
    malformed_outpoint["outpoint"]["vout"] = json!(-1);
    rejects_shape(&malformed_outpoint);
    let mut extra = frozen.clone();
    extra["outpoint"]["height"] = json!(101);
    rejects_shape(&extra);
}

#[test]
fn pinned_reorg_keeps_the_inclusive_height_routing_and_evidence_record() {
    let event = decode_envelope(include_bytes!("vectors/chain_events/reorg.json")).unwrap();
    let ChainEvent::Reorg {
        fork_height,
        deactivated_headers,
        new_tip,
        ..
    } = &event
    else {
        panic!("expected the pinned reorg");
    };
    assert_eq!(*fork_height, 101);
    assert_eq!(
        deactivated_headers
            .iter()
            .map(|h| h.height)
            .collect::<Vec<_>>(),
        [101]
    );
    assert_eq!(new_tip.height, 102);
    let mut headers = TestHeaders::default();
    let mut tracker = Tracker::new(params());
    let mut saved = Vec::new();
    for height in 100..=102 {
        let txid = hash(u64::from(height));
        let proof = proof(&txid, height, 1);
        headers.insert(&proof, u64::from(height));
        tracker
            .apply(&txid, &headers, Input::Evidence(Evidence::Proof(proof)))
            .unwrap();
        saved.push(tracker.get(&txid).unwrap().evidence().cloned());
    }
    let updates = tracker.on_chain(&headers, event.clone());
    assert_eq!(
        updates.iter().map(|u| u.txid.clone()).collect::<Vec<_>>(),
        [hash(101), hash(102)]
    );
    for update in &updates {
        assert_eq!(update.result, Ok(()));
        assert_eq!(update.reask, Some(Reask::Reorg { fork_height: 101 }));
        assert_eq!(tracker.get(&update.txid).unwrap().word(), &Word::Stale);
    }
    for (index, height) in (100..=102).enumerate() {
        assert_eq!(
            tracker.get(&hash(height)).unwrap().evidence(),
            saved[index].as_ref()
        );
    }
    mined(tracker.get(&hash(100)).unwrap(), 100);
    assert_eq!(tracker.get(&hash(100)).unwrap().reask(), None);
    assert!(tracker.on_chain(&headers, event).is_empty());
}

#[test]
fn pinned_tip_header_cannot_bypass_the_checked_snapshot() {
    let tip = decode_envelope(include_bytes!("vectors/chain_events/tip.json")).unwrap();
    let fork = decode_envelope(include_bytes!("vectors/chain_events/fork.json")).unwrap();
    let txid = hash(1);
    let proof = proof(&txid, 101, 2);
    let mut headers = TestHeaders::default();
    let mut tracker = Tracker::new(params());
    tracker.track(&txid);
    assert!(tracker.on_chain(&headers, tip.clone()).is_empty());
    assert_eq!(tracker.get(&txid).unwrap().word(), &Word::Unknown);
    headers.insert(&proof, 101);
    tracker
        .apply(&txid, &headers, Input::Evidence(Evidence::Proof(proof)))
        .unwrap();
    let saved = tracker.get(&txid).unwrap().evidence().cloned();
    assert_eq!(tracker.on_chain(&headers, fork).len(), 1);
    headers.fault = true;
    let updates = tracker.on_chain(&headers, tip);
    assert_eq!(updates.len(), 1);
    assert!(matches!(updates[0].result, Err(CheckError::Headers(_))));
    assert_eq!(updates[0].reask, Some(Reask::Recheck));
    mined(tracker.get(&txid).unwrap(), 101);
    assert!(tracker.get(&txid).unwrap().suspect());
    assert_eq!(tracker.get(&txid).unwrap().evidence(), saved.as_ref());
}
