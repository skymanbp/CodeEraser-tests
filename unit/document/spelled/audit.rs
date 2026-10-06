//! The W7 differential's audit leg (plan v2.33 W7; mounted inside the
//! frozen audit speech): each shown block's two paths, each shown
//! site's path, the leg's error, the mount and the unreadable message
//! spelled by the core and by the frozen `Strings`; a single string the
//! face does not hold is not sent, as `speech::body` sends it.

use super::super::kit::{Leg, leg};
use super::Strings;
use serde_json::{Value, json};

/// One of the three single strings, or none.
fn maybe(leg: &mut Leg) -> Option<String> {
    leg.rng.chance(50).then(|| leg.word())
}

/// The request's strings as the face sends them.
fn sent(s: &Strings) -> Value {
    let (a, b): (Vec<&String>, Vec<&String>) = s.blocks.iter().map(|(a, b)| (a, b)).unzip();
    let error: Vec<&String> = s.error.iter().collect();
    let mut out = json!({"block_a": a, "block_b": b, "place_file": s.places, "error": error});
    if let Some(m) = &s.mount {
        out["mount"] = json!(m);
    }
    if let Some(m) = &s.message {
        out["message"] = json!(m);
    }
    out
}

leg!(audit_spells_as_the_frozen_speech, "audit", 0x7718, |leg| {
    let blocks = (0..leg.rng.below(4))
        .map(|_| (leg.path(), leg.path()))
        .collect();
    let places = leg.paths(0, 4);
    let (error, mount, message) = (maybe(leg), maybe(leg), maybe(leg));
    let frozen = Strings {
        blocks,
        places,
        error,
        mount,
        message,
    };
    let nb = frozen.blocks.len();
    let drawn = [
        ("error", vec![1]),
        ("block_a", vec![nb]),
        ("block_b", vec![nb]),
        ("mount", vec![]),
        ("place_file", vec![frozen.places.len()]),
        ("message", vec![]),
    ];
    leg.spell_drawn("audit", &sent(&frozen), &frozen, &drawn);
});
