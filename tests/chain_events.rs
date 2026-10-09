//! Wire witnesses from rust-chaintracks@a62f9ed docs/CHAIN-EVENTS.md:265-285.
//! JSON field order is not a schema rule. The two short examples also match
//! byte for byte; the four header-bearing examples use JSON value equality.
use bsv_tracker::{decode_envelope, ChainEnvelope, ChainEvent};

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
