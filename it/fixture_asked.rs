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
/// every retired key lifted out, and a structure request's retired
/// `patterns` table re-spelled as `patternShapes`. Text surgery, never a
/// re-serialization of the whole line:
/// the filed key order is part of the bytes both consumers read.
pub fn asked(rel: &str, filed: &str) -> String {
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
    if rel == "structure/golden.ndjson" && !line.contains("\"patternShapes\":") {
        line = as_shapes(&line);
    }
    with_required(rel, &line)
}

/// Request keys the 9.0.0 major made required (plan v2.33 step 11a: the
/// core refuses a request without one by name, `<key>: required at
/// 9.0.0`): a filed family request without one is asked with it — the
/// graph's `unres` ledger empty, the erase target table one entry per
/// fact row in key order, a whole file (`[i,0,0]`) or, for a
/// verbatim_doc row (class 1), a one-line span (`[i,1,1]`). A line
/// without a `rows` table, or one already carrying the key, comes back
/// as is.
fn with_required(rel: &str, line: &str) -> String {
    let add = match rel {
        "graph/golden.ndjson" if !line.contains("\"unres\":") => "\"unres\":[]".to_string(),
        "erase/golden.ndjson" if !line.contains("\"targets\":") => match targets_for(line) {
            Some(table) => format!("\"targets\":{table}"),
            None => return line.to_string(),
        },
        _ => return line.to_string(),
    };
    let body = line.strip_suffix('}').expect("a closed object");
    let out = format!("{body},{add}}}");
    serde_json::from_str::<serde_json::Value>(&out).expect("still one json object");
    out
}

/// The erase target table a filed request's fact rows name.
fn targets_for(line: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let rows = v.get("rows")?.as_array()?;
    let table: Vec<[u64; 3]> = (0..)
        .zip(rows)
        .map(
            |(i, r)| match r.get(0).and_then(serde_json::Value::as_i64) {
                Some(1) => [i, 1, 1],
                _ => [i, 0, 0],
            },
        )
        .collect();
    serde_json::to_string(&table).ok()
}

/// One stem's shape bits per retired pattern code (CE.Structure.Shape):
/// lower_snake = lower + underscore, lower_kebab = lower + dash, camel =
/// upper + lower, pascal = camel + first char upper, upper_snake =
/// upper + underscore, digit_led = digit-led + lower, other =
/// unclassifiable.
/// Distinct codes, distinct bits: the folded distribution is the filed one.
const SHAPE_OF_CODE: [u64; 7] = [5, 6, 12, 44, 9, 20, 64];

/// A structure request's retired `patterns` table (8.0.0, plan v2.32
/// step 6: the core classifies `patternShapes` itself) re-spelled as
/// the shape facts it folds from, rows ascending; a line without the
/// key, or with both roads (the golden that pins the refusal), comes
/// back as is.
fn as_shapes(line: &str) -> String {
    let needle = "\"patterns\":";
    let Some(at) = line.find(needle) else {
        return line.to_string();
    };
    let start = at + needle.len();
    let end = start + line[start..].find("]]").expect("a closed table") + 2;
    let rows: Vec<[u64; 3]> = serde_json::from_str(&line[start..end]).expect("patterns rows");
    let mut shapes: Vec<[u64; 3]> = rows
        .iter()
        .map(|&[dir, code, n]| [dir, SHAPE_OF_CODE[code as usize], n])
        .collect();
    shapes.sort();
    let table = serde_json::to_string(&shapes).expect("shape rows");
    let out = format!("{}\"patternShapes\":{table}{}", &line[..at], &line[end..]);
    serde_json::from_str::<serde_json::Value>(&out).expect("still one json object");
    out
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
