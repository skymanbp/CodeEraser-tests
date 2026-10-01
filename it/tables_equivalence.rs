//! The core's definition package against the Rust text it was taken
//! from (plan v2.32 step 1; design booklet
//! docs/reference/authority-track.md §4.3). `tables/1` answers every
//! language and product definition the core holds; native_pack renders
//! today's Rust tables in the same shape (cli/src/tables/native.rs, gone
//! in step 2 with the text it reads). Equal means the transcription is
//! the original, table by table; a difference is named by its path and
//! both values, one per top-level family. CE_TABLES_DUMP=<dir> writes
//! both packages there for a reader with a diff tool.

use crate::common::core_bin;
use codeeraser::corelink::Link;
use codeeraser::tables::native::native_pack;
use serde_json::{Map, Value, json};

/// The reply keys that are the envelope's, not the package's.
const ENVELOPE: [&str; 4] = ["proto", "type", "id", "digest"];

/// One `tables/1` round trip: the hello's digest and the reply.
fn ask() -> (Option<u64>, Value) {
    let (mut link, hello) = Link::open(&core_bin()).expect("open");
    let reply = link.request("tables", json!({})).expect("tables/1 answers");
    (hello.tables_digest, reply)
}

/// The reply less its envelope: the package itself.
fn package(reply: &Value) -> Value {
    let mut keys = reply.as_object().expect("an object reply").clone();
    for k in ENVELOPE {
        keys.remove(k);
    }
    Value::Object(keys)
}

/// The first place two values part, as `path: core … / native …`.
fn first_difference(path: &str, core: &Value, native: &Value) -> Option<String> {
    match (core, native) {
        (Value::Object(a), Value::Object(b)) => object_difference(path, a, b),
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => a
            .iter()
            .zip(b)
            .enumerate()
            .find_map(|(i, (x, y))| first_difference(&format!("{path}[{i}]"), x, y)),
        _ if core == native => None,
        _ => Some(format!(
            "{path}: core {} / native {}",
            clip(core),
            clip(native)
        )),
    }
}

fn object_difference(path: &str, a: &Map<String, Value>, b: &Map<String, Value>) -> Option<String> {
    let keys: std::collections::BTreeSet<&String> = a.keys().chain(b.keys()).collect();
    keys.into_iter().find_map(|k| {
        let here = format!("{path}.{k}");
        match (a.get(k), b.get(k)) {
            (Some(x), Some(y)) => first_difference(&here, x, y),
            (x, y) => Some(format!(
                "{here}: core {:?} / native {:?}",
                x.is_some(),
                y.is_some()
            )),
        }
    })
}

/// A value short enough for a failure line.
fn clip(v: &Value) -> String {
    let text = v.to_string();
    match text.char_indices().nth(160) {
        Some((at, _)) => format!("{}…", &text[..at]),
        None => text,
    }
}

fn dump(core: &Value, native: &Value) {
    let Ok(dir) = std::env::var("CE_TABLES_DUMP") else {
        return;
    };
    for (name, v) in [("core.json", core), ("native.json", native)] {
        let text = serde_json::to_string_pretty(v).expect("render");
        std::fs::write(std::path::Path::new(&dir).join(name), text).expect("dump");
    }
}

/// Leg 1: every family, every language, every table equal — one first
/// difference reported per top-level key, never two 250 KB blobs.
#[test]
fn the_core_package_is_the_rust_text() {
    let (_, reply) = ask();
    let (core, native) = (package(&reply), native_pack());
    dump(&core, &native);
    let families: std::collections::BTreeSet<&String> = core
        .as_object()
        .unwrap()
        .keys()
        .chain(native.as_object().unwrap().keys())
        .collect();
    let parted: Vec<String> = families
        .into_iter()
        .filter_map(|k| first_difference(k, &core[k.as_str()], &native[k.as_str()]))
        .collect();
    assert!(
        parted.is_empty(),
        "the package parts from the Rust text:\n{}",
        parted.join("\n")
    );
}

/// Leg 2: three readings of one number — the hello's `tablesDigest`,
/// the reply's `digest`, and the fnv1a64 this side computes over the
/// package's canonical bytes (keys sorted, no whitespace: the core's
/// encoding and serde_json's agree).
#[test]
fn the_digest_is_one_number_three_ways() {
    let (hello, reply) = ask();
    let bytes = serde_json::to_vec(&package(&reply)).expect("render");
    let ours = codeeraser::dedup::tokens::fnv1a(&bytes);
    assert_eq!(reply["digest"].as_u64(), Some(ours), "reply digest");
    assert_eq!(hello, Some(ours), "hello tablesDigest");
}

/// Leg 3: the request is the envelope alone — any other key is refused
/// by name, not ignored.
#[test]
fn an_extra_key_is_refused_by_name() {
    let (mut link, _) = Link::open(&core_bin()).expect("open");
    let err = link
        .request("tables", json!({"lang": "python"}))
        .expect_err("an extra key must not be answered");
    assert!(
        err.contains("contract: tables: unexpected key lang"),
        "got: {err}"
    );
}
