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
use codeeraser::corelink::PROTO;
use std::path::PathBuf;

/// The handshake golden: its request line follows the server (§3),
/// so the regenerator moves it to PROTO before asking.
const HELLO: &str = "handshake/hello-ok.ndjson";

/// The wire golden files both consumers read, in Spec.hs's order.
pub const GOLDEN_FILES: [&str; 20] = [
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

fn proto_of(line: &str) -> Option<String> {
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

/// The line with its `proto` value at `version`.
fn at_proto(line: &str, version: &str) -> String {
    let (head, rest) = line.split_once("\"proto\":\"").expect("a proto field");
    let (_, tail) = rest.split_once('"').expect("a closed proto value");
    format!("{head}\"proto\":\"{version}\"{tail}")
}

/// Request keys a major retired (8.0.0, plan v2.32 step 6: the core
/// reads the judged set off its own language table). The core refuses
/// a request that carries one by name, so a family golden's request is
/// asked without it; the refusal itself is pinned once, in
/// `handshake/wire-errors.ndjson`, whose requests are asked as filed.
const RETIRED_KEYS: [&str; 1] = ["judgedMask"];

/// The major of a version string.
fn major(version: &str) -> &str {
    version.split('.').next().unwrap_or(version)
}

/// A filed request as the regenerator asks it (§3): the handshake at
/// PROTO; any other line a major left behind (its proto's major below
/// the anchor's) re-anchored at ANCHOR; a family golden's request with
/// every retired key lifted out. Text surgery, never a re-serialization:
/// the filed key order is part of the bytes both consumers read.
fn asked(rel: &str, filed: &str) -> String {
    if rel == HELLO {
        return at_proto(filed, PROTO);
    }
    let mut line = filed.to_string();
    let behind = proto_of(filed).is_some_and(|v| {
        let (v, a) = (major(&v).parse::<u32>(), major(ANCHOR).parse::<u32>());
        matches!((v, a), (Ok(v), Ok(a)) if v < a)
    });
    if behind {
        line = at_proto(&line, ANCHOR);
    }
    if rel.ends_with("/golden.ndjson") {
        for key in RETIRED_KEYS {
            line = without_key(&line, key);
        }
    }
    line
}

/// The line without its top-level integer-valued `key`, the comma that
/// joined it dropped with it; a line without the key comes back as is.
fn without_key(line: &str, key: &str) -> String {
    let needle = format!("\"{key}\":");
    let Some(at) = line.find(&needle) else {
        return line.to_string();
    };
    let value_end = line[at + needle.len()..]
        .find([',', '}'])
        .map(|i| at + needle.len() + i)
        .expect("a closed value");
    let value = &line[at + needle.len()..value_end];
    assert!(
        value.parse::<i128>().is_ok(),
        "{key}: {value:?} is not an integer"
    );
    let out = if line[..at].ends_with(',') {
        format!("{}{}", &line[..at - 1], &line[value_end..])
    } else {
        let rest = &line[value_end..];
        format!("{}{}", &line[..at], rest.strip_prefix(',').unwrap_or(rest))
    };
    serde_json::from_str::<serde_json::Value>(&out).expect("still one json object");
    out
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
