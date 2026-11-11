//! BARS.md §3 — a convention becomes an entry name. The candidates named
//! before the audit (DESIGN.md (ii), BARS §3's list) and the three
//! project conventions that stay out by rule 1 are built in as NAMES and
//! tool words only. Where a candidate sits in a tree comes from the same
//! input that carries its citation (`CE_DEADFILE_CONVENTION_DOCS`, one
//! object per candidate: `url`, `quote`, `position`), so a position is
//! never wider than the documentation it is read from (rule 1's letter).
//! A tool candidate passes when (1) the input cites the documentation
//! and gives the position rule, (2) every audited row carrying it is
//! `in_use` and at least one answer names the tool, (3) at least one
//! audited row carries it. A candidate with no position rule carries no
//! row and does not pass.
//!
//! The position rule, a closed set of four kinds (a name may hold `*`,
//! any run of characters inside the basename; nothing else is special):
//!   {"kind": "exact", "paths": ["CONTRIBUTING.md", ".github/CONTRIBUTING.md"]}  root-relative paths;
//!       with `"nocase": true` compared without ASCII case
//!   {"kind": "basename", "names": ["package-info.java"]}                        the basename, at any depth
//!   {"kind": "under", "segment": "spec", "names": ["*_spec.lua"]}               the basename, below a directory so named;
//!       with `"no_later_prefix": "_"` the FIRST such directory decides and
//!       no directory segment after it may begin with the prefix
//!   {"kind": "sequence", "dirs": ["src", "test", "resources"]}                  the directory segments hold that run,
//!       consecutively, anywhere; the file at any depth below it
//! A key a kind does not list (or an unknown kind) is refused by the
//! candidate's name; so is a field of the wrong type.

use serde_json::{Value, json};
use std::collections::BTreeMap;

/// `kind | name | corpus | tool words` per line.
const CANDIDATES: &str = "
tool | package-info.java | * | javac
tool | module-info.java | * | javac
tool | vitest.config | * | vitest
tool | next-app-entry | * | next.js nextjs
tool | setup.py | * | setuptools pip
tool | sphinx-conf.py | * | sphinx
tool | tlconfig.lua | * | teal tlconfig
tool | busted-spec | * | busted
tool | maven-test-resources | * | maven surefire
tool | CONTRIBUTING.md | * | github forge
tool | SECURITY.md | * | github forge
tool | CODE_OF_CONDUCT.md | * | github forge
project | koreader-plugin-meta | koreader | -
project | lua-testes | lua | -
project | zod-bench | zod | -
";

/// The candidate input: candidate name -> {url, quote, position}.
pub type Docs = BTreeMap<String, Value>;

/// One built-in candidate.
pub struct Candidate {
    pub kind: &'static str,
    pub name: &'static str,
    corpus: &'static str,
    tools: Vec<&'static str>,
}

pub fn candidates() -> Vec<Candidate> {
    CANDIDATES
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut column = l.split(" | ").map(str::trim);
            let mut next = || column.next().expect("candidate column");
            let (kind, name, corpus) = (next(), next(), next());
            let tools = next().split_whitespace().filter(|w| *w != "-");
            Candidate {
                kind,
                name,
                corpus,
                tools: tools.collect(),
            }
        })
        .collect()
}

/// `*` matches any run of characters; every other character itself.
fn wild(pattern: &str, s: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    let (first, last) = (parts[0], parts[parts.len() - 1]);
    if parts.len() == 1 {
        return pattern == s;
    }
    if s.len() < first.len() + last.len() || !s.starts_with(first) || !s.ends_with(last) {
        return false;
    }
    let mut rest = &s[first.len()..s.len() - last.len()];
    for mid in &parts[1..parts.len() - 1] {
        match rest.find(mid) {
            Some(i) => rest = &rest[i + mid.len()..],
            None => return false,
        }
    }
    true
}

fn words<'a>(name: &str, rule: &'a Value, key: &str) -> Vec<&'a str> {
    let list = rule[key].as_array();
    let list = list.unwrap_or_else(|| panic!("{name}: position rule without `{key}`"));
    list.iter()
        .map(|w| {
            w.as_str()
                .unwrap_or_else(|| panic!("{name}: `{key}` holds a non-string"))
        })
        .collect()
}

/// The keys each kind of rule may hold.
const KEYS: &str = "exact: kind paths nocase | basename: kind names | under: kind segment names no_later_prefix | sequence: kind dirs";

/// The rule's kind, after refusing an unknown kind or key.
fn kind_of<'a>(name: &str, rule: &'a Value) -> &'a str {
    let kind = rule["kind"].as_str().unwrap_or("");
    let allowed = KEYS
        .split(" | ")
        .find_map(|k| k.strip_prefix(kind)?.strip_prefix(": "));
    let allowed = allowed.unwrap_or_else(|| panic!("{name}: unknown position kind {kind:?}"));
    let keys = rule
        .as_object()
        .map(|o| o.keys().collect::<Vec<_>>())
        .unwrap_or_default();
    if let Some(k) = keys
        .iter()
        .find(|k| !allowed.split(' ').any(|a| a == k.as_str()))
    {
        panic!("{name}: position kind {kind} takes no `{k}`");
    }
    kind
}

/// An optional field of one JSON type: absent or null reads `None`.
fn optional<'a, T>(
    name: &str,
    rule: &'a Value,
    key: &str,
    read: fn(&'a Value) -> Option<T>,
) -> Option<T> {
    let v = &rule[key];
    (!v.is_null()).then(|| read(v).unwrap_or_else(|| panic!("{name}: `{key}` has the wrong type")))
}

/// A root-relative path's basename and its directory segments.
fn parts(path: &str) -> (&str, Vec<&str>) {
    match path.rsplit_once('/') {
        Some((dir, base)) => (base, dir.split('/').collect()),
        None => (path, Vec::new()),
    }
}

/// Whether a root-relative path matches a candidate's position rule; an
/// unknown kind or a missing field is refused by the candidate's name.
pub fn matches(name: &str, rule: &Value, path: &str) -> bool {
    let (base, dirs) = parts(path);
    let named = |key| words(name, rule, key).iter().any(|n| wild(n, base));
    match kind_of(name, rule) {
        "exact" => {
            let nocase = optional(name, rule, "nocase", Value::as_bool).unwrap_or(false);
            let same = |p: &&str| {
                if nocase {
                    p.eq_ignore_ascii_case(path)
                } else {
                    *p == path
                }
            };
            words(name, rule, "paths").iter().any(same)
        }
        "basename" => named("names"),
        "under" => {
            let seg = rule["segment"].as_str();
            let seg = seg.unwrap_or_else(|| panic!("{name}: `under` without `segment`"));
            let prefix = optional(name, rule, "no_later_prefix", Value::as_str);
            let first = dirs.iter().position(|d| *d == seg);
            let later = |i: usize| {
                dirs[i + 1..]
                    .iter()
                    .any(|d| prefix.is_some_and(|p| d.starts_with(p)))
            };
            first.is_some_and(|i| !later(i)) && named("names")
        }
        _ => {
            let run = words(name, rule, "dirs");
            !run.is_empty() && dirs.windows(run.len()).any(|w| w == run.as_slice())
        }
    }
}

/// A candidate's position rule from the input, when given.
pub fn rule<'a>(c: &Candidate, docs: &'a Docs) -> Option<&'a Value> {
    docs.get(c.name)
        .map(|d| &d["position"])
        .filter(|p| !p.is_null())
}

/// Whether a row (corpus, path) carries a candidate under the input's
/// position rule; no rule, no row.
pub fn carries(c: &Candidate, docs: &Docs, corpus: &str, path: &str) -> bool {
    held(c, rule(c, docs), corpus, path)
}

/// Whether a row carries a candidate under a given rule (None: no row).
fn held(c: &Candidate, rule: Option<&Value>, corpus: &str, path: &str) -> bool {
    (c.corpus == "*" || c.corpus == corpus) && rule.is_some_and(|r| matches(c.name, r, path))
}

/// The selector the second-reading packets were drawn with (the main
/// session's make_second_packets.py, `CONVENTION`): an `unused` answer on
/// a row it selects is read twice (BARS §1). A record of how the second
/// reading was drawn — never a §3 position rule.
const SECOND_READ: &str = "package-info.java module-info.java vitest.config.ts setup.py conf.py \
    tlconfig.lua CONTRIBUTING.md SECURITY.md CODE_OF_CONDUCT.md page.tsx layout.tsx route.ts";

pub fn drawn_for_second_reading(path: &str) -> bool {
    let (base, dirs) = parts(path);
    SECOND_READ.split_whitespace().any(|n| n == base)
        || dirs.contains(&"spec")
        || format!("/{path}").contains("/src/test/resources/")
}

/// Whether an answer names one of the tool words, as a whole word.
fn names(text: &str, word: &str) -> bool {
    let text = text.to_lowercase();
    let edge = |c: Option<char>| c.is_none_or(|c| !c.is_ascii_alphanumeric());
    text.match_indices(word)
        .any(|(i, w)| edge(text[..i].chars().last()) && edge(text[i + w.len()..].chars().next()))
}

/// One candidate under one position rule (None: no rule given): its
/// rows, answers, the three rules, and which of them refuse it.
fn judged(c: &Candidate, doc: Option<&Value>, rule: Option<&Value>, rows: &[Value]) -> Value {
    let at = |r: &Value, k: &str| r[k].as_str().unwrap_or("").to_string();
    let carried = |r: &&Value| held(c, rule, &at(r, "corpus"), &at(r, "path"));
    let mine: Vec<&Value> = rows.iter().filter(carried).collect();
    let count = |w: &str| mine.iter().filter(|r| r["answer"] == w).count();
    let text = |r: &Value| format!("{} {}", r["first"]["mechanism"], r["first"]["evidence"]);
    let names_tool = mine
        .iter()
        .any(|r| c.tools.iter().any(|w| names(&text(r), w)));
    let given = |k: &str| {
        doc.and_then(|d| d[k].as_str())
            .is_some_and(|s| !s.trim().is_empty())
    };
    let (documented, position) = (given("url") && given("quote"), rule.is_some());
    let rule2 = count("in_use") == mine.len() && names_tool;
    let refused: Vec<u8> = [
        (1, c.kind == "tool" && documented && position),
        (2, mine.is_empty() || rule2),
        (3, !mine.is_empty()),
    ]
    .into_iter()
    .filter_map(|(n, holds)| (!holds).then_some(n))
    .collect();
    let listed: Vec<String> = mine
        .iter()
        .map(|r| format!("{} {} {}", at(r, "corpus"), at(r, "path"), at(r, "answer")))
        .collect();
    json!({
        "name": c.name, "kind": c.kind, "rows": mine.len(), "in_use": count("in_use"),
        "unused": count("unused"), "unsure": count("unsure"), "names_tool": names_tool,
        "documented": documented, "position": position, "refused_by": refused,
        "passes": refused.is_empty(), "listed": listed,
    })
}

/// §3 over the review rows: per candidate the judgment under its
/// `position`; beside it, when the input gives one, the judgment under
/// its `beside` rule (printed for comparison, never deciding), its rows
/// by name when the input asks (`print_rows`), and its `gap` sentence.
pub fn evaluate(rows: &[Value], docs: &Docs) -> Vec<Value> {
    candidates()
        .iter()
        .map(|c| {
            let doc = docs.get(c.name);
            let mut one = judged(c, doc, rule(c, docs), rows);
            let extra = |k: &str| doc.map(|d| &d[k]).filter(|v| !v.is_null());
            if let Some(b) = extra("beside") {
                one["beside"] = judged(c, doc, Some(b), rows);
            }
            if extra("print_rows") != Some(&json!(true)) {
                one["listed"] = Value::Null;
            }
            one["gap"] = extra("gap").cloned().unwrap_or(Value::Null);
            one
        })
        .collect()
}
