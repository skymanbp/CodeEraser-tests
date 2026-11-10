//! How the golden regenerator asks a filed request (fixture_contract.rs
//! `regen`): split from that file when the 8.0.0 re-anchoring (plan v2.32
//! step 6) took it past the E01 300-line line.

use crate::facts::ver::ANCHOR;
use crate::fixture_contract::{HELLO, proto_of};
use codeeraser::corelink::PROTO;

/// The line with its `proto` value at `version`.
fn at_proto(line: &str, version: &str) -> String {
    let (head, rest) = line.split_once("\"proto\":\"").expect("a proto field");
    let (_, tail) = rest.split_once('"').expect("a closed proto value");
    format!("{head}\"proto\":\"{version}\"{tail}")
}

/// The major of a version string.
fn major(version: &str) -> &str {
    version.split('.').next().unwrap_or(version)
}

/// A filed request as the regenerator asks it (§3): the handshake at
/// PROTO; any other line a major left behind (its proto's major below
/// the anchor's) re-anchored at ANCHOR; every other line as filed. Text
/// surgery, never a re-serialization of the whole line: the filed key
/// order is part of the bytes both consumers read.
pub fn asked(rel: &str, filed: &str) -> String {
    if rel == HELLO {
        return at_proto(filed, PROTO);
    }
    let behind = proto_of(filed).is_some_and(|v| {
        let (v, a) = (major(&v).parse::<u32>(), major(ANCHOR).parse::<u32>());
        matches!((v, a), (Ok(v), Ok(a)) if v < a)
    });
    if behind {
        at_proto(filed, ANCHOR)
    } else {
        filed.to_string()
    }
}
