//! How the golden regenerator asks a filed request (fixture_contract.rs
//! `regen`): split from that file when the 8.0.0 re-anchoring (plan v2.32
//! step 6) took it past the E01 300-line line.

use crate::facts::ver::ANCHOR;
use crate::fixture_contract::{HELLO, golden_pairs, proto_of};
use codeeraser::corelink::PROTO;

/// The wire-error golden: every pair in it is a request the server
/// refuses, so every reply is an error.
const WIRE_ERRORS: &str = "handshake/wire-errors.ndjson";

/// The major-mismatch probes, by (file, request id): asked one major
/// AHEAD of the server, derived from PROTO at every regeneration. A
/// literal major ahead of the server stops being one the day the server
/// reaches it — at 10.0.0 the filed `"proto":"10.0.0"` became the
/// server's own and the pair a `fourclass.result`.
const SKEWED: [(&str, u64); 1] = [(WIRE_ERRORS, 10)];

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

/// One major past the server's: the version a major-mismatch probe asks at.
fn skewed() -> String {
    let m: u32 = major(PROTO).parse().expect("a numeric major");
    format!("{}.0.0", m + 1)
}

/// The request id of a filed line, when it has a numeric one.
fn id_of(line: &str) -> Option<u64> {
    serde_json::from_str::<serde_json::Value>(line)
        .ok()?
        .get("id")?
        .as_u64()
}

/// A filed request as the regenerator asks it (§3): the handshake at
/// PROTO; a major-mismatch probe (SKEWED) one major past PROTO; any
/// other line a major left behind (its proto's major below the
/// anchor's) re-anchored at ANCHOR; every other line as filed. Text
/// surgery, never a re-serialization of the whole line: the filed key
/// order is part of the bytes both consumers read.
pub fn asked(rel: &str, filed: &str) -> String {
    if rel == HELLO {
        return at_proto(filed, PROTO);
    }
    if SKEWED.contains(&(rel, id_of(filed).unwrap_or(u64::MAX))) {
        return at_proto(filed, &skewed());
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

/// The wire-error golden holds refusals only: a pair whose reply is a
/// result is a probe that stopped probing (the 10.0.0 regeneration
/// turned the major-mismatch pair into a `fourclass.result`, and the
/// golden check compared it to itself and stayed green). Each skew probe
/// is filed one major past the server.
#[test]
fn every_wire_error_reply_is_an_error_and_every_skew_probe_is_ahead() {
    for (n, (request, reply)) in golden_pairs(WIRE_ERRORS).iter().enumerate() {
        let r: serde_json::Value = serde_json::from_str(reply).expect("a JSON reply");
        assert_eq!(r["type"], "error", "{WIRE_ERRORS} pair {}: {reply}", n + 1);
        if SKEWED.contains(&(WIRE_ERRORS, id_of(request).unwrap_or(u64::MAX))) {
            assert_eq!(proto_of(request), Some(skewed()), "pair {}", n + 1);
        }
    }
}
