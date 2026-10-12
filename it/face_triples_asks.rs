//! The asks of the three-face leg (face_triples.rs): every report
//! family, every document option set at least once — the CLI words,
//! the MCP tool with its arguments, and the library face the GUI's
//! Tauri command passes through to, all asking the same thing. The
//! asks are one text table; the third face is read off the same
//! arguments the MCP tool takes (the parity gate holds the GUI's
//! option names equal to them), as the Tauri command reads its own.

use anyhow::Result;
use codeeraser::document::Answer;
use codeeraser::faces::{self, Plumb, doc, gated_doc};
use serde_json::Value;
use std::path::Path;

pub type Face = Box<dyn Fn(&Path, &str) -> Result<Value>>;

/// A library face over the plumbing alone, and one over the root and
/// the core alone.
type Plain = fn(&Path, &str, Plumb) -> Result<Answer>;
type Bare = fn(&Path, &str) -> Result<Answer>;

/// One family asked one way.
pub struct Ask {
    pub cli: Vec<String>,
    pub tool: &'static str,
    pub args: Value,
    pub face: Face,
}

/// The options that name a place in the root (face_triples::picks).
pub struct Picks {
    pub at: String,
    pub impact: String,
    pub unit: Option<String>,
    pub rules: Option<String>,
}

/// The free text a `similar --text` ask searches by: one CLI word.
const TEXT: &str = "parse_the_input";

/// `ce` words | MCP tool | its arguments. `{at}` / `{impact}` / `{unit}`
/// / `{rules}` / `{text}` are the root's picks; a row naming a pick the
/// root has none of is not asked.
const ASKS: &str = r#"
scan | scan | {}
dedup --min-tokens 60 --min-distinct 8 | check_duplication | {"min_tokens": 60, "min_distinct": 8}
clone | clone | {}
clone --units | clone | {"units": true}
docdup | docdup | {}
graph --sites | graph_sites | {}
deadcode | deadcode | {}
merge | merge_suggestions | {}
churn --days 30 | churn | {"days": 30}
join --days 30 | join | {"days": 30}
structure --deep --days 30 --split-candidates | structure | {"deep": true, "days": 30, "split": true}
trend --commits 2 --batch 1 | trend | {"commits": 2, "batch": 1}
check --days 30 --fail-under 500 | check | {"days": 30, "floor": 500}
erase | erase | {}
erase --log | erase_log | {}
doctor | doctor | {}
similar --at {at} | similar_units | {"at": "{at}"}
similar --text {text} --widen | similar_units | {"text": "{text}", "widen": true}
similar --unit {unit} | similar_units | {"unit": "{unit}"}
flow --kind dead_store,unreachable | flow | {"kind": "dead_store,unreachable"}
arch --impact {impact} | architecture | {"impact": ["{impact}"]}
query dead(F) --why | query | {"body": "dead(F)", "why": true}
query --prelude | query | {"prelude": true}
rules --why | rules | {"why": true}
query dead(F) --file {rules} | query | {"body": "dead(F)", "file": "{rules}"}
rules --file {rules} | rules | {"file": "{rules}"}
"#;

/// Every ask on a root with these picks.
pub fn all(p: &Picks) -> Vec<Ask> {
    let picks = [
        ("{at}", Some(&p.at)),
        ("{impact}", Some(&p.impact)),
        ("{unit}", p.unit.as_ref()),
        ("{rules}", p.rules.as_ref()),
    ];
    let mut out = Vec::new();
    'row: for row in ASKS.lines().filter(|l| !l.trim().is_empty()) {
        let (mut words, mut args) = (row.to_string(), row.to_string());
        for (hole, pick) in picks {
            if row.contains(hole) {
                let Some(v) = pick else { continue 'row };
                let quoted = Value::String(v.clone()).to_string();
                words = words.replace(hole, v);
                args = args.replace(&format!("\"{hole}\""), &quoted);
            }
        }
        let (words, args) = (words.replace("{text}", TEXT), args.replace("{text}", TEXT));
        let [cli, tool, _] = cells(&words);
        let args: Value = serde_json::from_str(cells(&args)[2]).expect("an ask's arguments");
        let tool: &'static str = Box::leak(tool.to_string().into_boxed_str());
        out.push(Ask {
            cli: cli.split_whitespace().map(String::from).collect(),
            tool,
            face: face(tool, &args),
            args,
        });
    }
    out
}

fn cells(row: &str) -> [&str; 3] {
    let c: Vec<&str> = row.splitn(3, " | ").map(str::trim).collect();
    [c[0], c[1], c[2]]
}

/// The faces that take the plumbing alone.
fn plain(tool: &str) -> Option<Plain> {
    Some(match tool {
        "deadcode" => faces::deadcode,
        "docdup" => faces::docdup,
        "merge_suggestions" => faces::merge,
        _ => return None,
    })
}

/// The faces that take the root and the core alone.
fn bare(tool: &str) -> Option<Bare> {
    Some(match tool {
        "scan" => faces::scan,
        "graph_sites" => faces::graph_sites,
        "erase_log" => faces::erase_log,
        _ => return None,
    })
}

/// A face over one option value and the plumbing, asked with `option`.
fn with<T: Copy + 'static>(face: fn(&Path, &str, T, Plumb) -> Result<Answer>, option: T) -> Face {
    Box::new(move |r, c| doc(face(r, c, option, Plumb::default())))
}

/// An argument of the ask, read the way the Tauri command reads its own.
struct Args<'a>(&'a Value);

impl Args<'_> {
    fn text(&self, k: &str) -> Option<String> {
        self.0[k].as_str().map(String::from)
    }
    fn on(&self, k: &str) -> bool {
        self.0[k].as_bool().unwrap_or(false)
    }
    fn count(&self, k: &str) -> Option<usize> {
        self.0[k].as_u64().map(|v| v as usize)
    }
    fn window(&self, k: &str) -> Option<u32> {
        self.0[k].as_u64().map(|v| v as u32)
    }
}

/// The library face `tool` reaches with `args`.
fn face(tool: &str, a: &Value) -> Face {
    if let Some(f) = plain(tool) {
        return Box::new(move |r, c| doc(f(r, c, Plumb::default())));
    }
    if let Some(f) = bare(tool) {
        return Box::new(move |r, c| doc(f(r, c)));
    }
    measured(tool, &Args(a)).unwrap_or_else(|| asked(tool, &Args(a)))
}

/// The faces over a window, a threshold or a switch.
fn measured(tool: &str, a: &Args) -> Option<Face> {
    let days = a.window("days");
    Some(match tool {
        "check_duplication" => {
            let t = (a.count("min_tokens"), a.count("min_distinct"));
            Box::new(move |r, c| gated_doc(faces::dedup(r, c, t, Plumb::default())))
        }
        "clone" => with(faces::clone, a.on("units")),
        "churn" => Box::new(move |r, c| doc(faces::churn(r, c, days.unwrap_or(14)))),
        "join" => with(faces::join, days.unwrap_or(14)),
        "structure" => with(faces::structure, (a.on("deep"), days, a.on("split"))),
        "trend" => with(
            faces::trend,
            (a.count("commits").unwrap_or(0), a.count("batch")),
        ),
        "check" => with(faces::check, (days, a.window("floor"))),
        "erase" => Box::new(|r, c| gated_doc(faces::erase(r, c, Plumb::default(), |_, _| None))),
        "doctor" => Box::new(|r, c| Ok(faces::doctor(r, c))),
        _ => return None,
    })
}

/// The faces asked about a place, a text or a question.
fn asked(tool: &str, a: &Args) -> Face {
    let d = Plumb::default;
    let (file, why) = (a.text("file"), a.on("why"));
    match tool {
        "similar_units" => {
            let parts = [a.text("at"), a.text("text"), a.text("unit")];
            let widen = a.on("widen");
            Box::new(move |r, c| {
                let [at, txt, unit] = parts.each_ref().map(Option::as_deref);
                let ask = codeeraser::similar::query::Ask::from_parts(at, txt, unit)?;
                doc(faces::similar(r, c, &ask, widen, d()))
            })
        }
        "flow" => {
            let listed = a.text("kind").unwrap_or_default();
            let kinds: Vec<String> = listed.split(',').map(String::from).collect();
            Box::new(move |r, c| doc(faces::flow(r, c, &kinds, d())))
        }
        "architecture" => {
            let impact: Vec<String> = serde_json::from_value(a.0["impact"].clone()).expect("paths");
            Box::new(move |r, c| doc(faces::arch(r, c, &impact, d())))
        }
        "query" if a.on("prelude") => Box::new(faces::pack_text),
        "query" => {
            let body = a.text("body").expect("a question");
            Box::new(move |r, c| {
                let f = file.as_deref().map(Path::new);
                doc(faces::query(r, c, (&body, why), f, d()))
            })
        }
        "rules" => {
            Box::new(move |r, c| doc(faces::rules(r, c, file.as_deref().map(Path::new), why, d())))
        }
        other => panic!("{other}: no library face for this tool"),
    }
}
