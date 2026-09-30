//! The frozen-ledger shapes the FPR instruments share — the graded
//! zone's and the per-language one's: a measured row merged into its
//! doc, the rows read back in a named order, a row's counts and
//! intercepts, the table a ledger page quotes, a row's arithmetic
//! closing against its own raw counts, the page-quotes check and the
//! two-commit synthetic history an e2e walks. One owner, so a second
//! ledger brings a header line, a column spec and its own counts, not
//! a copy of the loop (the guard named the copies at the first write,
//! the tests repo's clone gate the rest).

use super::stats::{cp_upper_ppm, rate_ppm};
use super::{commit_all, init_and_commit, repo_root, tmp, write_all};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The newest first-parent commits each per-language ledger reads a
/// corpus over — one window for the duplicate-write ledger
/// (fpr_lang_replay) and the flow one (fpr_flow_replay).
pub const WINDOW: usize = 400;

/// Where a frozen doc is read and written: the directory `var` names
/// when it is set (the doc under its file name there), else `home`.
/// Two readers: CE_FLOW_OUT for the flow exams' universe, sample and
/// precision docs, CE_FPR_OUT for the three FPR ledgers' frozen docs
/// (fpr-lang, fpr-zone, fpr-flow). A measurement run thus leaves the
/// tree clean — generated_from() reads `git status --porcelain`, so a
/// doc written into the tree marks every later run dirty — and the
/// docs are copied in together once all are measured.
pub fn out_file(var: &str, home: &str) -> String {
    match std::env::var(var) {
        Ok(dir) => {
            let name = Path::new(home).file_name().expect("name");
            format!("{dir}/{}", name.to_string_lossy())
        }
        Err(_) => home.to_string(),
    }
}

/// A frozen ledger doc: where it lives, its schema, its writer, the
/// row field that names a row, and the admission line it records
/// (None for a ledger recorded beside a gate that reads another doc:
/// the flow ledger, whose admission is the precision docs').
pub struct Ledger {
    pub rel: &'static str,
    pub schema: &'static str,
    pub generated_from: &'static str,
    pub key: &'static str,
    pub gate_ppm: Option<u64>,
}

impl Ledger {
    /// A ledger keyed by corpus name with an admission line — the
    /// graded zone's and the per-language duplicate-write one's.
    pub const fn gated(
        rel: &'static str,
        schema: &'static str,
        generated_from: &'static str,
        gate_ppm: u64,
    ) -> Ledger {
        Ledger {
            rel,
            schema,
            generated_from,
            key: "name",
            gate_ppm: Some(gate_ppm),
        }
    }

    /// The doc's rows, loaded from the repository with the header
    /// checked.
    pub fn corpora(&self) -> Vec<Value> {
        let doc: Value =
            serde_json::from_str(&crate::facts::read(&repo_root(), self.rel)).expect(self.rel);
        assert_eq!(doc["schema"], self.schema, "{}: schema", self.rel);
        assert_eq!(
            doc["gate_ppm"].as_u64(),
            self.gate_ppm,
            "{}: gate_ppm",
            self.rel
        );
        doc["corpora"].as_array().expect("a corpora array").clone()
    }

    /// The rows, one per named corpus in that order — every measured
    /// corpus once, none missing, none extra.
    pub fn rows(&self, names: &[&str]) -> Vec<Value> {
        let rows = self.corpora();
        let found: Vec<&str> = rows.iter().filter_map(|c| c[self.key].as_str()).collect();
        assert_eq!(found, names, "{}: one row per corpus, in order", self.rel);
        rows
    }

    /// The measured row into the frozen doc: the row of the same name
    /// replaced, every other row kept as its own run left it, `extra`
    /// header fields written beside the standing ones, rows in `rank`
    /// order. The doc is read and written at one path: CE_FPR_OUT's
    /// directory when set (out_file), so successive blesses accumulate
    /// outside the tree.
    pub fn merge(
        &self,
        root: &Path,
        row: Value,
        extra: &[(&str, Value)],
        rank: impl Fn(&Value) -> (usize, String),
    ) {
        let home = root.join(self.rel);
        let path = out_file("CE_FPR_OUT", &home.to_string_lossy());
        let mut doc: Value = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_else(|| json!({ "corpora": [] }));
        doc["schema"] = json!(self.schema);
        doc["generated_from"] = json!(self.generated_from);
        match self.gate_ppm {
            Some(ppm) => doc["gate_ppm"] = json!(ppm),
            None => _ = doc.as_object_mut().map(|o| o.remove("gate_ppm")),
        }
        for (k, v) in extra {
            doc[*k] = v.clone();
        }
        let corpora = doc["corpora"].as_array_mut().expect("a corpora array");
        corpora.retain(|c| c[self.key] != row[self.key]);
        corpora.push(row);
        corpora.sort_by_key(|c| rank(c));
        let text = serde_json::to_string_pretty(&doc).expect("the frozen doc");
        std::fs::write(&path, format!("{text}\n")).expect("write the frozen doc");
    }

    /// The ledger page quotes the frozen table as the instrument
    /// prints it (the page is hand-maintained prose, not a generated
    /// block).
    pub fn page_quotes(&self, page: &str, printed: &str) {
        let text = crate::facts::read(&repo_root(), page);
        assert!(
            text.contains(printed.trim()),
            "{page}: paste the instrument's measured table for {}",
            self.rel
        );
    }
}

/// A row's count under `k`, the row named when it is absent (by its
/// `name`, or its `lang` in a ledger keyed by language).
pub fn count(c: &Value, k: &str) -> u64 {
    let row = c.get("name").unwrap_or(&c["lang"]);
    c[k].as_u64().unwrap_or_else(|| panic!("{row}: no {k}"))
}

/// A row's frozen intercept rows.
pub fn intercepts(c: &Value) -> &Vec<Value> {
    c["intercepts"].as_array().expect("an intercepts array")
}

/// `name @ tip`, the corpus cell every ledger table leads with.
pub fn corpus_at(c: &Value) -> String {
    format!(
        "{} @ {}",
        c["name"].as_str().unwrap_or("?"),
        c["tip"].as_str().unwrap_or("?")
    )
}

/// A ppm as a percentage with four decimals, integer arithmetic only.
pub fn pct(ppm: u64) -> String {
    format!("{}.{:04} %", ppm / 10_000, ppm % 10_000)
}

/// The table a ledger page carries: the header lines, then one row
/// per corpus — its leading cells from `lead`, then the columns of
/// `spec`, row keys separated by spaces, a `%` prefix printing the
/// key's ppm as a percentage.
pub fn table(
    header: &str,
    lead: impl Fn(&Value) -> String,
    spec: &str,
    corpora: &[Value],
) -> String {
    let mut s = String::from(header);
    for c in corpora {
        let n = |k: &str| c[k].as_u64().unwrap_or_default();
        let cells: Vec<String> = spec
            .split_whitespace()
            .map(|col| match col.strip_prefix('%') {
                Some(k) => pct(n(k)),
                None => n(col).to_string(),
            })
            .collect();
        s += &format!("| {} | {} |\n", lead(c), cells.join(" | "));
    }
    s
}

/// A row's derived numbers against the ones recomputed from its raw
/// counts: every pair equal, or the first unequal key named.
pub fn closes(c: &Value, expected: &[(&str, u64)]) {
    for (k, want) in expected {
        assert_eq!(
            c[*k].as_u64(),
            Some(*want),
            "{}: {k} — re-run the instrument; the row's arithmetic does not close",
            c["name"]
        );
    }
}

/// The rate and its Clopper–Pearson bound for `k` of `n`, under the
/// two keys a ledger row stores them by.
pub fn rate_pair(
    k: usize,
    n: usize,
    keys: (&'static str, &'static str),
) -> [(&'static str, u64); 2] {
    [(keys.0, rate_ppm(k, n)), (keys.1, cp_upper_ppm(k, n))]
}

/// A synthetic two-commit history under a scratch directory: the
/// seed files committed, then the second set written over them and
/// committed — the smallest history a replay can walk.
pub fn two_commits(tag: &str, seed: &[(&str, &str)], then: &[(&str, &str)]) -> PathBuf {
    let repo = tmp(tag);
    write_all(&repo, seed);
    init_and_commit(&repo, "seed");
    write_all(&repo, then);
    commit_all(&repo, "second");
    repo
}
