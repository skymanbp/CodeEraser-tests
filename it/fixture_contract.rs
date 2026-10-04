//! O66 (plan v2.18 step #14, 6.4.0): the golden fixture ledger,
//! DERIVED. VERSIONING §3 spells a triple — the request anchor, the
//! count of request lines standing at it, the version the server
//! answers — and each consumer (core/test/Spec.hs, this suite)
//! carries its own list of the files. Two of the three now REACH the
//! page as chips (`count:golden_requests#digits`, `ver:proto#v`);
//! this leg derives them from the files, holds the registry's count
//! to the one Spec.hs's list yields, and reads the prose, so a
//! regeneration that moved a count, a golden pair added to one
//! consumer, or a reply line left at an old version fails by name
//! instead of waiting for a human to recount.

use crate::common::{core_session, repo_root};
use crate::facts::ver::ANCHOR;
use crate::fixture_asked::asked;
use codeeraser::corelink::PROTO;
use std::path::PathBuf;

/// The handshake golden: its request line follows the server (§3),
/// so the regenerator moves it to PROTO before asking.
pub const HELLO: &str = "handshake/hello-ok.ndjson";

/// The wire golden files both consumers read, in Spec.hs's order.
pub const GOLDEN_FILES: [&str; 21] = [
    HELLO,
    "handshake/wire-errors.ndjson",
    "fourclass/golden.ndjson",
    "graph/golden.ndjson",
    "clone/golden.ndjson",
    "docdup/golden.ndjson",
    "verdict/golden.ndjson",
    "scan/golden.ndjson",
    "structure/golden.ndjson",
    "trend/golden.ndjson",
    "erase/golden.ndjson",
    "audit/golden.ndjson",
    "tombstone/golden.ndjson",
    "similar/golden.ndjson",
    "query/golden.ndjson",
    "flow/golden.ndjson",
    "merge/golden.ndjson",
    "arch/golden.ndjson",
    "tables/golden.ndjson",
    "document/golden.ndjson",
    "resolve/golden.ndjson",
    "candidates/golden.ndjson",
];

fn fixture(rel: &str) -> PathBuf {
    repo_root().join("contracts/fixtures").join(rel)
}

fn lines(rel: &str) -> Vec<String> {
    std::fs::read_to_string(fixture(rel))
        .unwrap_or_else(|e| panic!("{rel}: {e}"))
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

/// The file's (request, reply) pairs in order — the round trip
/// (core_wire.rs) and the regenerator read the files through this.
pub fn golden_pairs(rel: &str) -> Vec<(String, String)> {
    let rows = lines(rel);
    assert_eq!(rows.len() % 2, 0, "{rel}: request/reply pairs");
    rows.chunks(2)
        .map(|c| (c[0].clone(), c[1].clone()))
        .collect()
}

/// A generated golden (query_golden.rs, flow_golden.rs): its first
/// request lines are a generator's output and `kept` hand-written pairs
/// follow them as filed. Fails naming every generated line the file
/// disagrees with, and the leg that rewrites them.
pub fn assert_generated(rel: &str, generated: &[String], kept: usize, leg: &str) {
    let filed = golden_pairs(rel);
    let moved: Vec<usize> = generated
        .iter()
        .enumerate()
        .filter(|(i, g)| filed.get(*i).map(|(req, _)| req) != Some(*g))
        .map(|(i, _)| i + 1)
        .collect();
    assert!(
        moved.is_empty() && filed.len() == generated.len() + kept,
        "{rel}: request lines behind their generator: {moved:?} of {} filed / {} generated + \
         {kept} kept — run `{leg}::regen` then `fixture_contract::regen` under CE_BLESS=1",
        filed.len(),
        generated.len()
    );
}

/// A generated golden rewritten: each generated request line beside the
/// reply filed at its place (`{}` where it is new — `regen` below answers
/// it next), then the last `kept` filed pairs as filed. `CE_BLESS=1`
/// writes the file; without it the dry run fails if the file would move.
pub fn rewrite_generated(rel: &str, generated: &[String], kept: usize) {
    let filed = golden_pairs(rel);
    let (made, tail) = filed.split_at(filed.len().saturating_sub(kept));
    let replies = made
        .iter()
        .map(|(_, r)| r.as_str())
        .chain(std::iter::repeat("{}"));
    let mut body: String = generated
        .iter()
        .zip(replies)
        .map(|(req, reply)| format!("{req}\n{reply}\n"))
        .collect();
    body.extend(tail.iter().map(|(req, reply)| format!("{req}\n{reply}\n")));
    if crate::facts::blessing() {
        std::fs::write(fixture(rel), body).expect(rel);
    } else {
        let current = std::fs::read_to_string(fixture(rel)).expect(rel);
        assert_eq!(
            current, body,
            "{rel}: CE_BLESS=1 rewrites the request lines"
        );
    }
}

pub fn proto_of(line: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(line)
        .ok()?
        .get("proto")?
        .as_str()
        .map(str::to_string)
}

/// (anchored request lines, reply lines not at PROTO) over the files.
fn tally() -> (usize, Vec<String>) {
    let (mut anchored, mut stale) = (0, Vec::new());
    for rel in GOLDEN_FILES {
        for (n, (request, reply)) in golden_pairs(rel).into_iter().enumerate() {
            let proto = proto_of(&reply);
            if proto.as_deref() != Some(PROTO) {
                stale.push(format!("{rel} pair {}: {proto:?}", n + 1));
            }
            if rel.ends_with("/golden.ndjson") {
                assert_eq!(
                    proto_of(&request).as_deref(),
                    Some(ANCHOR),
                    "{rel} pair {}",
                    n + 1
                );
                anchored += 1;
            }
        }
    }
    (anchored, stale)
}

/// The regenerator (plan v2.31 step 1; through 7.2.0 it lived in a
/// session scratchpad): one core at CE_CORE_BIN answers every request
/// line in Spec.hs's order and the reply lines are rewritten from its
/// answers, each request asked as the current major reads it (`asked`:
/// the handshake at PROTO, lines a major left behind at ANCHOR, retired
/// keys lifted out of the family goldens). `CE_BLESS=1`
/// writes the files; without it the leg is the dry run and fails
/// naming every pair that would move. `--ignored`: it runs on
/// purpose, at a proto bump, never by accident under a bless.
#[test]
#[ignore]
fn regen() {
    let mut core = core_session();
    let mut moved = Vec::new();
    for rel in GOLDEN_FILES {
        let mut body = String::new();
        for (n, (filed, reply)) in golden_pairs(rel).into_iter().enumerate() {
            let request = asked(rel, &filed);
            let answer = core.ask_line(&request);
            if request != filed || answer != reply {
                moved.push(format!("{rel} pair {}", n + 1));
            }
            body.push_str(&format!("{request}\n{answer}\n"));
        }
        if crate::facts::blessing() {
            std::fs::write(fixture(rel), body).expect(rel);
        }
    }
    let status = core.finish();
    assert!(status.success(), "core exit: {status}");
    assert!(
        crate::facts::blessing() || moved.is_empty(),
        "golden pairs behind the core (CE_BLESS=1 rewrites them): {moved:?}"
    );
}

#[test]
fn every_reply_answers_the_current_proto_and_the_handshake_follows_it() {
    let (_, stale) = tally();
    assert!(stale.is_empty(), "reply lines behind {PROTO}: {stale:?}");
    let hello = lines(HELLO);
    assert_eq!(
        proto_of(&hello[0]).as_deref(),
        Some(PROTO),
        "the handshake request follows the server (§3)"
    );
}

#[test]
fn the_versioning_triple_is_derived_from_the_files() {
    let (anchored, _) = tally();
    // one number, two derivations: the registry globs the fixture
    // directories, this leg counts the files Spec.hs names
    assert_eq!(
        crate::facts::resolve("count:golden_requests#digits"),
        anchored.to_string(),
        "the registry's golden request count is not the one Spec.hs's list yields"
    );
    let text =
        std::fs::read_to_string(repo_root().join("contracts/VERSIONING.md")).expect("VERSIONING");
    // the count and the answered version are chips too (facts_chips.rs
    // renders them); the anchor is the hand-written const this file
    // holds every request line to
    for want in [
        format!("<!--ce:count:golden_requests#digits-->{anchored}<!--/ce--> 行"),
        format!("server 恒答 <!--ce:ver:proto#v-->{PROTO}<!--/ce-->"),
        format!("server 走 <!--ce:ver:proto#v-->{PROTO}<!--/ce-->"),
    ] {
        assert!(text.contains(&want), "VERSIONING carries {want:?}");
    }
}

#[test]
fn both_consumers_read_the_same_files() {
    let spec = std::fs::read_to_string(repo_root().join("core/test/Spec.hs")).expect("Spec.hs");
    let listed: Vec<&str> = spec
        .lines()
        .filter_map(|l| {
            l.trim().strip_prefix("goldenPairs \"").or_else(|| {
                l.trim()
                    .strip_prefix(", goldenPairs \"")
                    .or_else(|| l.trim().strip_prefix("[ goldenPairs \""))
            })
        })
        .filter_map(|rest| rest.split('"').next())
        .collect();
    assert_eq!(
        listed,
        GOLDEN_FILES.to_vec(),
        "Spec.hs's list is this one, in order"
    );
}
