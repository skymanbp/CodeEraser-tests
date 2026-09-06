//! Corpus resolution for the commit-slice instrument family: which
//! repository is under measurement and where each frozen doc family
//! lives. Split from mod.rs when the M5-1 external-validation
//! machinery pushed it past the file budget; the window machinery
//! (tip/base pinning, Corpus struct, SELF_UNIVERSE_TIP) retired with
//! the one-shot generators that read it (git history, 0c7c936 wave).

/// The pinned tips of the four external corpora under
/// `.ce-eval/corpora/<name>` (docs/EVAL-SET-M5-3.md, 语料树钉定): a
/// measurement on any other tree would be a different corpus, so every
/// ignored leg that walks them checks the tip first (eval_mention,
/// eval_dedup_distinct).
pub const PINNED_CORPORA: [(&str, &str); 4] = [
    ("cobra", "adbc8813901bba65827259daa8e22ff94ec1f30e"),
    ("requests", "8068356288978c4f54661ae6f95afe0e0831885e"),
    ("ripgrep", "3fce3b5bb0236da2df6d99672afb8a719642eca7"),
    ("zod", "912f0f51b0ced654d0069741e7160834dca742ee"),
];

/// One pinned corpus's checkout, its tip verified — the prologue of
/// every external-corpus leg.
pub fn pinned_root(name: &str, tip: &str) -> std::path::PathBuf {
    let root = crate::common::repo_root()
        .join(".ce-eval/corpora")
        .join(name);
    assert!(
        root.is_dir(),
        "{}: clone the corpus at {tip}",
        root.display()
    );
    let (ok, head) = crate::common::git_out(&root, &["rev-parse", "HEAD"]);
    assert!(
        ok && head.trim() == tip,
        "{name}: not the pinned tip {tip} ({})",
        head.trim()
    );
    root
}

/// Every committed slice doc paired with its `family` sibling. The
/// sibling MUST exist — a slice without its labels/baseline is an
/// unfinished freeze, and a gate that skips it would go silently
/// blind on a whole corpus.
pub fn corpus_doc_pairs(family: &str) -> Vec<(String, String)> {
    let pairs = doc_pairs(family, true);
    assert!(!pairs.is_empty(), "no committed slice docs");
    pairs
}

/// Like corpus_doc_pairs, but a missing sibling is a PENDING corpus,
/// not an error — for families that freeze later than the slice
/// (an L2 doc lands only once the bar passes on that corpus; the
/// requests doc stayed pending on the invention finding until the
/// anchor floor landed, see EVAL-SET). The self corpus sibling must
/// still exist: that bar is frozen.
pub fn corpus_doc_pairs_frozen(family: &str) -> Vec<(String, String)> {
    let pairs = doc_pairs(family, false);
    let self_doc = format!("../contracts/eval/commit-{family}-v1.json");
    assert!(
        pairs.iter().any(|(_, d)| *d == self_doc),
        "self {family} doc missing"
    );
    pairs
}

/// The corpus-name suffix of a frozen doc path for an arbitrary
/// document stem: "…/{stem}-ripgrep-v1.json" → Some("ripgrep"), the
/// self doc "…/{stem}-v1.json" → None. Shared by the commit families
/// (below) and the graph family (eval_graph.rs).
pub fn doc_suffix(path: &str, stem: &str) -> Option<String> {
    let file = path.rsplit('/').next().expect("file name");
    let mid = file
        .strip_prefix(stem)
        .and_then(|s| s.strip_suffix("-v1.json"))
        .unwrap_or_else(|| panic!("{path}: not a {stem} doc"));
    (!mid.is_empty()).then(|| mid.trim_start_matches('-').to_string())
}

/// doc_suffix for the commit families — how a gate iterating EVERY
/// corpus resolves the matching compiled review record (resolving
/// via the active corpus would silently read the wrong one — Codex
/// review C3 follow-up).
pub fn doc_corpus_name(path: &str, family: &str) -> Option<String> {
    doc_suffix(path, &format!("commit-{family}"))
}

/// A frozen-doc gate's prologue: the corpus name parsed from the
/// family doc's path plus both loaded documents (companion doc
/// first) — the shared opening of every per-corpus CI gate.
pub fn gate_docs(
    family: &str,
    companion: &str,
    doc_path: &str,
) -> (Option<String>, serde_json::Value, serde_json::Value) {
    (
        doc_corpus_name(doc_path, family),
        super::load(companion),
        super::load(doc_path),
    )
}

/// Frozen docs under contracts/eval whose file name starts with
/// `prefix` (and ends -v1.json), sorted — the ONE enumeration every
/// doc-family gate consumes (the dedup ratchet caught this loop's
/// third verbatim copy; the throat is the fix, not the third copy).
pub fn frozen_docs(prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir("../contracts/eval").expect("eval dir") {
        let file = entry.expect("entry").file_name();
        let file = file.to_string_lossy();
        if file.starts_with(prefix) && file.ends_with("-v1.json") {
            out.push(format!("../contracts/eval/{file}"));
        }
    }
    out.sort();
    out
}

fn doc_pairs(family: &str, required: bool) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for path in frozen_docs("commit-slice") {
        let sibling = path.replace("commit-slice", &format!("commit-{family}"));
        match std::fs::exists(&sibling).expect("probe") {
            true => pairs.push((path, sibling)),
            false => assert!(!required, "{path}: missing {family} sibling {sibling}"),
        }
    }
    pairs
}
