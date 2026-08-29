//! The frozen-universe gate skeleton — ONE binding for the slice
//! (eval_graph.rs) and t3 (eval_t3_universe.rs) families: the doc
//! stem, the gate opening, the nine-key envelope checks and the
//! working-tree drift walk. The generator half (tree walker, doc
//! assembly) retired with the one-shot instruments (git history;
//! resurrection takes cli/tests/eval_support at the same commit).

use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The doc stem of one corpus in a family: "graph-slice-zod" /
/// "graph-slice" (self). Shared by generators and cross-family
/// lookups so file naming can never fork.
pub fn doc_stem(family: &str, name: &Option<String>) -> String {
    match name {
        Some(n) => format!("{family}-{n}"),
        None => family.into(),
    }
}

/// The shared opening of every universe gate: iterate the frozen docs
/// of one family (set anchored to FROZEN_CORPORA) through a per-doc
/// check.
pub fn each_frozen_doc(family: &str, mut check: impl FnMut(&str, &Value)) {
    for path in super::assert_frozen_corpus_set(family) {
        let doc = super::load(&path);
        check(&path, &doc);
    }
}

/// The frozen-doc envelope every per-corpus universe doc carries:
/// summary re-derived by the family's own scorer (G1), frozen
/// constants, frozen scope, well-formed exclusion ledger, pinned
/// full-OID tip, embedded corpus name matching the file name, rows
/// sorted and duplicate-free. Returns the corpus name.
pub fn assert_doc_envelope(
    path: &str,
    doc: &Value,
    family: &str,
    scorer: fn(&[Value]) -> Value,
    constants: fn() -> Value,
) -> Option<String> {
    let files = doc["files"].as_array().expect("files");
    assert_eq!(doc["summary"], scorer(files), "{path}: summary drifted");
    assert_eq!(doc["constants"], constants(), "{path}: constants drifted");
    assert_eq!(
        doc["scope"]["extensions"],
        json!(super::SCOPE_EXTS),
        "{path}"
    );
    assert_eq!(
        doc["scope"]["excludes"],
        json!(super::SCOPE_EXCLUDES),
        "{path}"
    );
    for (category, n) in doc["excluded"].as_object().expect("excluded") {
        assert!(
            ["excluded_prefix", "variant_extension", "other_extension"]
                .contains(&category.as_str())
                && n.as_u64().is_some_and(|v| v > 0),
            "{path}: malformed excluded row {category}"
        );
    }
    let tip = doc["corpus"]["tip"].as_str().expect("tip");
    assert!(
        tip.len() == 40 && tip.chars().all(|c| c.is_ascii_hexdigit()),
        "{path}: tip is not a pinned full OID"
    );
    let name = doc["corpus"]["name"].as_str().map(str::to_string);
    assert_eq!(
        name,
        super::doc_suffix(path, family),
        "{path}: embedded corpus name does not match the file name"
    );
    for pair in files.windows(2) {
        assert!(
            pair[0]["path"].as_str() < pair[1]["path"].as_str(),
            "{path}: rows unsorted or duplicated"
        );
    }
    name
}

/// Sum of a JSON object's u64 values — the map-total idiom every
/// per-key summary gate re-derives (site totals, survivor strata).
pub fn sum_obj(v: &Value) -> u64 {
    v.as_object()
        .expect("object")
        .values()
        .map(|x| x.as_u64().expect("u64"))
        .sum()
}

/// Accumulate a JSON object's u64 values into a cross-corpus map and
/// return the object's own total — the one accumulation walk every
/// universe gate and summarizer shares.
pub fn sum_obj_into(obj: &Value, into: &mut BTreeMap<String, u64>) -> u64 {
    let mut total = 0;
    for (k, v) in obj.as_object().expect("object") {
        let n = v.as_u64().expect("u64");
        *into.entry(k.clone()).or_insert(0) += n;
        total += n;
    }
    total
}

/// Where a frozen row's file lives now, and its LF text: the path
/// as frozen, or its relocation — `cli/tests/<x>` moved under `it/`
/// when the suite became a submodule (K+1), and every `#[cfg(test)]`
/// module of cli/src rides at `cli/tests/unit/<x>` since plan v2.18
/// step #13. None when the file is gone. The frozen identity is
/// git-blob text (LF); the Windows CI runner checks out with
/// autocrlf=true, so normalize before comparing — first CI run of
/// the slice gate was 0-row vacuous on windows-latest for exactly
/// this reason.
fn live_text(path: &str) -> Option<(String, String)> {
    let mut candidates = vec![path.to_string()];
    if let Some(rest) = path.strip_prefix("cli/tests/") {
        candidates.push(format!("cli/tests/it/{rest}"));
    }
    if let Some(rest) = path.strip_prefix("cli/src/") {
        let name = rest.rsplit('/').next().unwrap_or(rest);
        if name == "testutil.rs" || name.ends_with("_tests.rs") || name.starts_with("tests") {
            candidates.push(format!("cli/tests/unit/{rest}"));
        }
    }
    candidates.into_iter().find_map(|live| {
        let bytes = std::fs::read(format!("../{live}")).ok()?;
        Some((live, String::from_utf8_lossy(&bytes).replace("\r\n", "\n")))
    })
}

/// The self working-tree drift walk: every frozen row whose file
/// still carries the frozen content identity is handed to `check`
/// with its LF text; the count of such rows is the gate's evidence.
pub fn each_frozen_match(doc: &Value, mut check: impl FnMut(&Value, &str, &str, &str)) -> usize {
    let mut verified = 0;
    for row in doc["files"].as_array().expect("files") {
        let path = row["path"].as_str().expect("path");
        let Some((_, text)) = live_text(path) else {
            continue;
        };
        if super::content_sha(&text) != row["sha256"].as_str().expect("sha256") {
            continue;
        }
        check(row, path, row["lang"].as_str().expect("lang"), &text);
        verified += 1;
    }
    verified
}

/// CE_BLESS=1 with CE_REFREEZE=<frozen path,…>: re-sign exactly the
/// named rows — each re-derived through the family's own row throat
/// at the file's live path — then re-derive the summary with the
/// family's scorer (the envelope the gate re-checks). Named, never
/// "every changed row": the self views are a pinned denominator (the
/// t3-candidates doc anchors its admitted units to the universe
/// bands and the t3 sample to its pool digest), so a step re-signs
/// the rows it touched and patches that anchor by hand, as steps #5
/// and #8 did by hand-edit; a wholesale re-sign is a re-freeze at a
/// new tip. The self views must be re-signed together: the sibling
/// anchor holds them to one path→sha inventory. A named row whose
/// file is gone or unchanged is never touched, so a detector drift
/// can never be blessed away.
pub fn refreeze_self(
    doc_path: &str,
    row: fn(&str, &str, &str) -> Value,
    summarize: fn(&[Value]) -> Value,
) {
    let named: Vec<String> = std::env::var("CE_REFREEZE")
        .unwrap_or_default()
        .split(',')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();
    let mut doc = super::load(doc_path);
    let files = doc["files"].as_array_mut().expect("files");
    for frozen in files.iter_mut() {
        let path = frozen["path"].as_str().expect("path");
        if !named.iter().any(|n| n == path) {
            continue;
        }
        let Some((live, text)) = live_text(path) else {
            continue;
        };
        if super::content_sha(&text) == frozen["sha256"].as_str().expect("sha256") {
            continue;
        }
        *frozen = row(&live, frozen["lang"].as_str().expect("lang"), &text);
    }
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let summary = summarize(files);
    doc["summary"] = summary;
    let text = serde_json::to_string_pretty(&doc).expect("frozen doc json") + "\n";
    std::fs::write(doc_path, text).expect(doc_path);
}
