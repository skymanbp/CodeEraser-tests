//! The dead-file exam's review side (BARS.md §1 with addendum 1): the
//! mechanical evidence check of every answer, addendum 1's `named` /
//! `generic` tag, the second-reading set and its agreement, and the
//! review table `deadfile-review-<lang>` built from first readings,
//! second readings and the main session's settlements. Nothing here
//! repairs an answer: a failing answer is replaced only by a passing
//! second reading, two differing answers only by a settlement row.
//! Answers: one JSON object per line, `{corpus, path, answer,
//! mechanism, evidence, searched}` (the auditor brief's format).

use serde_json::{Value, json};

/// The three answer words.
pub const WORDS: [&str; 3] = ["in_use", "unused", "unsure"];
/// An `unused` answer's search record is at least this long.
pub const SEARCH_MIN: usize = 20;
/// BARS §1's second-reading domain and modulus; the agreement floor.
pub const DOUBLE_READ: (&str, u32, f64) = ("ce-deadfile-doubleread-v1", 5, 0.90);

/// The text of a file at a pinned tip, by (corpus, commit, path).
pub type Tip<'a> = &'a dyn Fn(&str, &str, &str) -> Option<String>;

/// The two facts of one answer on one row that need the pinned tree:
/// BARS §1's check and addendum 1 B's tag. The review docs' generator
/// reads them off the clones (`clone_facts`); the CI gate reads them
/// off the filed review docs and rebuilds every other field.
pub type Facts<'a> = &'a dyn Fn(&Value, &Value) -> (String, Value);

/// The facts read off the clones.
pub fn clone_facts<'a>(tip: Tip<'a>) -> impl Fn(&Value, &Value) -> (String, Value) + 'a {
    move |row, a| (check(tip, row, a), json!(tag(tip, row, a)))
}

/// A file reader over clones under `root` (`git show <tip>:<path>`).
pub fn clone_reader(root: String) -> impl Fn(&str, &str, &str) -> Option<String> {
    move |corpus, tip, path| {
        let out = std::process::Command::new("git")
            .args([
                "-C",
                &format!("{root}/{corpus}"),
                "show",
                &format!("{tip}:{path}"),
            ])
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

/// `path:line` tokens of a text (the main session's check_answers.py
/// pattern `([^\s:;,`'"]+):(\d+)`: a run of characters other than
/// space, colon and `;,` quotes, then a colon and digits; brackets stay
/// inside the token, a wrapping one is peeled by the caller).
pub fn citations(text: &str) -> Vec<(String, usize)> {
    let stop = |c: char| c.is_whitespace() || ";,`'\"".contains(c);
    let mut out = Vec::new();
    for chunk in text.split(stop) {
        let parts: Vec<&str> = chunk.split(':').collect();
        for pair in parts.windows(2) {
            let digits: String = pair[1].chars().take_while(char::is_ascii_digit).collect();
            if !pair[0].is_empty() && !digits.is_empty() {
                out.push((pair[0].to_string(), digits.parse().unwrap_or(0)));
            }
        }
    }
    out
}

/// Whether `word` occurs in `line` as a whole word.
fn has_word(line: &str, word: &str) -> bool {
    let inner = |c: char| c.is_alphanumeric() || c == '_';
    line.match_indices(word).any(|(i, _)| {
        let before = line[..i].chars().next_back().is_none_or(|c| !inner(c));
        let after = line[i + word.len()..]
            .chars()
            .next()
            .is_none_or(|c| !inner(c));
        before && after
    })
}

/// The cited lines that exist at the tip: (path, line, the file's text).
fn existing(tip: Tip, corpus: &str, commit: &str, text: &str) -> Vec<(String, usize, String)> {
    let mut out = Vec::new();
    for (raw, line) in citations(text) {
        let peeled = raw.trim_start_matches(['(', '[', '{']);
        for cand in [
            raw.as_str(),
            peeled,
            peeled.trim_end_matches([')', ']', '}']),
        ] {
            let body = tip(corpus, commit, cand);
            if let Some(body) = body.filter(|b| line >= 1 && line <= b.lines().count().max(1)) {
                out.push((cand.to_string(), line, body));
                break;
            }
        }
    }
    out
}

fn field<'a>(a: &'a Value, k: &str) -> &'a str {
    a[k].as_str().unwrap_or("")
}

/// BARS §1's mechanical check of one answer: `ok`, `convention` (an
/// `in_use` naming a mechanism with no existing citation — a named tool
/// convention for the second auditor to confirm) or `fail: <why>`.
pub fn check(tip: Tip, row: &Value, a: &Value) -> String {
    let (corpus, commit) = (field(row, "corpus"), field(row, "commit"));
    match field(a, "answer") {
        "in_use" => {
            let text = format!("{} {}", field(a, "evidence"), field(a, "mechanism"));
            if !existing(tip, corpus, commit, &text).is_empty() {
                "ok".into()
            } else if field(a, "mechanism").trim().is_empty() {
                "fail: in_use names no mechanism and cites no existing path:line".into()
            } else {
                "convention".into()
            }
        }
        "unused" if field(a, "searched").trim().len() < SEARCH_MIN => {
            "fail: unused without a search record".into()
        }
        word if WORDS.contains(&word) => "ok".into(),
        _ => "fail: not one of the three answer words".into(),
    }
}

/// Addendum 1 B: `named` when a cited line that exists, or one of the
/// two lines either side, holds the row's basename or its stem as a
/// word; `generic` otherwise. Only an `in_use` answer is tagged.
pub fn tag(tip: Tip, row: &Value, a: &Value) -> Option<&'static str> {
    if field(a, "answer") != "in_use" {
        return None;
    }
    let path = field(row, "path");
    let (base, stem) = crate::eval_deadfile_parts::features::names(path);
    let text = format!("{} {}", field(a, "evidence"), field(a, "mechanism"));
    let named = existing(tip, field(row, "corpus"), field(row, "commit"), &text)
        .iter()
        .any(|(_, line, body)| {
            let lines: Vec<&str> = body.lines().collect();
            let (lo, hi) = (line.saturating_sub(3), (line + 2).min(lines.len()));
            lines[lo..hi]
                .iter()
                .any(|l| l.contains(base) || has_word(l, stem))
        });
    Some(if named { "named" } else { "generic" })
}

/// Why a row is read twice (BARS §1, addendum 1 C), empty when once.
pub fn twice_reasons(row: &Value, first: &Value, candidate: bool) -> Vec<&'static str> {
    let key = format!(
        "{}|{}|{}|{}",
        DOUBLE_READ.0,
        field(row, "corpus"),
        field(row, "commit"),
        field(row, "path")
    );
    let h = u32::from_str_radix(&crate::eval_support::content_sha(&key)[..8], 16).expect("hex");
    let mut out = Vec::new();
    if h.is_multiple_of(DOUBLE_READ.1) {
        out.push("fixed");
    }
    if field(first, "answer") == "unsure" {
        out.push("unsure");
    }
    if field(first, "answer") == "unused" && candidate {
        out.push("unused_convention");
    }
    out
}

/// One answer as the review table stores it.
fn reading(facts: Facts, row: &Value, a: &Value) -> Value {
    let (check, tag) = facts(row, a);
    json!({
        "answer": field(a, "answer"), "check": check, "tag": tag,
        "mechanism": field(a, "mechanism"), "evidence": field(a, "evidence"), "searched": field(a, "searched"),
    })
}

/// (final answer, status) of a row from its readings and settlement.
fn settle(first: &Value, second: Option<&Value>, settled: Option<&str>) -> (Value, String) {
    let passes = |r: &Value| r["check"] == "ok" || r["check"] == "convention";
    match (passes(first), second) {
        (false, Some(s)) if passes(s) => (
            s["answer"].clone(),
            "replaced: first answer failed the check".into(),
        ),
        (false, _) => (
            Value::Null,
            "blocked: answer failed the check, no passing second reading".into(),
        ),
        // BARS §1: two different answers (any two words) are settled
        // by the evidence, never by a vote
        (true, Some(s)) if first["answer"] != s["answer"] => match settled {
            Some(word) => (json!(word), "settled by evidence".into()),
            None => (Value::Null, "unsettled: two different answers".into()),
        },
        (true, _) => (first["answer"].clone(), "ok".into()),
    }
}

/// A row's readings: the first and second answers, the main session's
/// settlement word, whether the row carries a §3 tool candidate.
#[derive(Clone, Copy, Default)]
pub struct Readings<'a> {
    pub first: Option<&'a Value>,
    pub second: Option<&'a Value>,
    pub settled: Option<&'a str>,
    pub candidate: bool,
}

/// One review row: the primary sample row, its readings, final answer.
pub fn review_row(facts: Facts, row: &Value, given: Readings) -> Value {
    let Readings {
        first,
        second,
        settled,
        candidate,
    } = given;
    let Some(first) = first else {
        return json!({"corpus": row["corpus"], "commit": row["commit"], "path": row["path"], "status": "blocked: no first reading"});
    };
    let one = reading(facts, row, first);
    let two = second.map(|s| reading(facts, row, s));
    let (answer, status) = settle(&one, two.as_ref(), settled);
    let tag = if answer == "in_use" {
        two.as_ref()
            .filter(|t| t["answer"] == answer && one["answer"] != answer)
            .map_or(&one["tag"], |t| &t["tag"])
            .clone()
    } else {
        Value::Null
    };
    json!({
        "corpus": row["corpus"], "commit": row["commit"], "path": row["path"],
        "twice": twice_reasons(row, first, candidate), "first": one, "second": two,
        "answer": answer, "tag": tag, "status": status,
    })
}

/// Agreement over the twice-read rows (same side of in_use / not), and
/// over those whose first answer is tagged `generic`.
pub fn agreement(rows: &[Value]) -> Value {
    let read2: Vec<&Value> = rows.iter().filter(|r| !r["second"].is_null()).collect();
    let share = |sel: &dyn Fn(&&Value) -> bool| {
        let pick: Vec<&&Value> = read2.iter().filter(|r| sel(r)).collect();
        let same = pick
            .iter()
            .filter(|r| (r["first"]["answer"] == "in_use") == (r["second"]["answer"] == "in_use"))
            .count();
        json!({"rows": pick.len(), "same_side": same, "share": if pick.is_empty() { Value::Null } else { json!(same as f64 / pick.len() as f64) }})
    };
    let overall = share(&|_| true);
    let below = overall["share"].as_f64().is_some_and(|s| s < DOUBLE_READ.2);
    json!({
        "overall": overall,
        "first_generic": share(&|r| r["first"]["tag"] == "generic"),
        "first_named": share(&|r| r["first"]["tag"] == "named"),
        "first_not_in_use": share(&|r| r["first"]["answer"] != "in_use"),
        "every_row_twice": below,
    })
}
