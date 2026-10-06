// The differential leg of plan v2.33 W1 item 2: the core's lex reply
// (query/1 `lex` + `inspect`, CE.Query.Lex / CE.Query.Front) against
// the frozen Rust scanner, goal heads and schema (unit/query/oracle/,
// 1324c927) — every token with its kind, value, source, line, column and
// spelling, the variable tables, the program predicates, the globs and
// where each was spelled, the names, the referenced tables, the heads,
// the counts and the fault — over seeded random programs (three seeds,
// 10,000 programs each) and every real program: the prelude, this
// repository's ce.rules and the tests subrepo's, and the questions the
// byte gate asks.
use super::*;
use crate::query::PRELUDE;
use crate::query::oracle::{columns, legend as frozen_legend, program};
use serde_json::{Value, json};

/// Knuth's MMIX linear congruential step, the high half folded down:
/// the leg's seeded stream.
struct Mix(u64);

impl Mix {
    fn draw(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 ^ (self.0 >> 32)
    }

    fn under(&mut self, n: usize) -> usize {
        (self.draw() % n as u64) as usize
    }
}

/// The pieces a random program is spelled from: names (schema, program,
/// keyword and sugar words), variables, numbers (in range and out),
/// strings (closed, open, with non-ASCII), every punctuation, blanks,
/// comments, and bytes that spell nothing.
const PIECES: &str = r#"file§lines§ref§set§in§mention§dead§reach§foo§bar_2§not§count§min§max§sum§assert§X§Y§F§_§_x§Abc9
§0§7§300§-3§-0§99999999999999999999§18446744073709551615§-9223372036854775808§-9223372036854775809§"src/**"§""
§"é中"§"open§:-§,§.§(§)§?-§=§!=§<§<=§>§>=§+§-§*§/§%§:§ § §#§@§$§é§😀§&§in(F, "g")§in(f, "g")§in(F, x)§in(F "g")"#;

/// The pieces a raw string cannot hold: the line ends, the tab, the form
/// feed and a comment running to its line end.
const CONTROL: [&str; 5] = ["\n", "\t", "\r\n", "\x0c", "# a comment\n"];

fn program_text(rng: &mut Mix) -> String {
    let pieces: Vec<&str> = PIECES
        .split('§')
        .map(|s| s.trim_matches('\n'))
        .chain(CONTROL)
        .collect();
    let n = 1 + rng.under(40);
    (0..n)
        .map(|_| {
            let sep = [" ", "", "\n"][rng.under(3)];
            format!("{}{sep}", pieces[rng.under(pieces.len())])
        })
        .collect()
}

/// A wire number as the oracle holds it.
fn num(v: i128) -> Value {
    if v >= 0 {
        json!(v as u64)
    } else {
        json!(v as i64)
    }
}

/// The lexed object the core must answer, from the frozen scanner.
fn expected(texts: [Option<&str>; 3]) -> Value {
    let p = match program::Program::lex(texts[0].unwrap_or(""), texts[1], texts[2]) {
        Err(f) => {
            let at = format!("{} {}:{}", f.src.word(), f.line, f.col);
            let fault = json!([f.src as usize, f.line, f.col, f.what]);
            return json!({"fault": [at, f.what], "inspect": {"fault": fault}});
        }
        Ok(p) => p,
    };
    let goals = columns::goals(&p);
    let set_at: Vec<String> = (0..p.sets.len())
        .map(|i| {
            let first = p
                .tokens
                .iter()
                .position(|t| t.kind == 3 && t.value == i as i128);
            first.map_or_else(|| "?".to_string(), |t| p.locate(t))
        })
        .collect();
    let schema = (0u32..)
        .map_while(frozen_legend::pred_of)
        .map(|q| json!([q.code, q.name]));
    let own = p
        .preds
        .iter()
        .enumerate()
        .map(|(i, n)| json!([1000 + i, n]));
    let tokens: Vec<Value> = p
        .tokens
        .iter()
        .map(|t| json!([t.kind, num(t.value), t.src as usize, t.line, t.col, t.spell]))
        .collect();
    json!({
        "fault": null, "tokens": p.tokens.len(), "clauses": p.clauses, "prelude": p.prelude_clauses,
        "at": (0..p.tokens.len()).map(|i| p.locate(i)).collect::<Vec<_>>(),
        "preds": schema.chain(own).collect::<Vec<_>>(),
        "sets": p.sets.clone(), "setAt": set_at,
        "names": p.names.iter().map(|(h, n)| json!([h, n])).collect::<Vec<_>>(),
        "referenced": p.referenced(),
        "heads": goals.iter().map(|g| json!([g.name, g.columns])).collect::<Vec<_>>(),
        "inspect": {
            "tokens": tokens, "vars": p.vars.clone(), "preds": p.preds.clone(),
            "goals": goals.iter().map(|g| json!([g.clause, g.kind == "assert", g.name, g.columns])).collect::<Vec<_>>(),
        },
    })
}

/// The core's lexed object for the same texts, over one link.
fn answered(link: &mut crate::corelink::Link, texts: [Option<&str>; 3]) -> Value {
    let body = json!({"texts": [texts[0].unwrap_or(""), texts[1], texts[2]], "lex": true, "inspect": true});
    let reply = wire::ask(link, body).expect("a lex reply");
    reply["lexed"].clone()
}

/// Every case of a list held against the oracle; the mismatches named.
fn mismatches(cases: &[[Option<String>; 3]]) -> Vec<String> {
    let core = crate::daemon::judge::core_bin().expect("a core");
    let mut link = crate::document::open(&core).expect("a core link");
    let (mut out, mut n) = (Vec::new(), 0);
    for c in cases {
        let texts = [c[0].as_deref(), c[1].as_deref(), c[2].as_deref()];
        let (want, got) = (expected(texts), answered(&mut link, texts));
        if want != got {
            n += 1;
            if out.len() < 5 {
                out.push(format!("{texts:?}\n  want {want}\n  got  {got}"));
            }
        }
    }
    if n > 0 {
        out.push(format!("{n} of {} cases differ", cases.len()));
    }
    out
}

/// One seed's 10,000 programs: the prelude, a random rules text or none,
/// a random query or none — or, one time in eight, a random prelude.
fn seeded(seed: u64) -> Vec<[Option<String>; 3]> {
    let mut rng = Mix(seed);
    (0..10_000)
        .map(|_| {
            let prelude = if rng.under(8) == 0 {
                program_text(&mut rng)
            } else {
                PRELUDE.to_string()
            };
            let rules = (rng.under(3) > 0).then(|| program_text(&mut rng));
            let query = (rng.under(3) > 0).then(|| program_text(&mut rng));
            [Some(prelude), rules, query]
        })
        .collect()
}

#[test]
fn the_cores_lexer_answers_what_the_frozen_scanner_did_on_three_seeds() {
    for seed in [0x5eed_0001_u64, 0x00c0_ffee_2033, 0x7777_dead_beef] {
        let bad = mismatches(&seeded(seed));
        assert!(bad.is_empty(), "seed {seed:#x}: {}", bad.join("\n"));
    }
}

/// The real programs: the prelude alone, both repositories' rules files,
/// and the questions the byte gate asks (face.rs wraps them).
#[test]
fn the_cores_lexer_answers_what_the_frozen_scanner_did_on_the_real_programs() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let rules: Vec<Option<String>> = ["/ce.rules", "/cli/tests/ce.rules"]
        .iter()
        .map(|p| Some(std::fs::read_to_string(format!("{root}{p}")).expect("a rules file")))
        .chain([None])
        .collect();
    let questions = "file(F), lang(F, rust), lines(F, N), N > 200|unit(U, F), coc(U, C), C > 3|\
ref(F, G, K, _), in(F, \"src/**\")|N = count(F : file(F))|dead(F)|mention(\"main\", F)|file(F|\
file(F), lang(F, \"rust)|lines(F, 99999999999999999999)|in(f, \"x\")|file(F) & x|foo(X)|not file(F)|\
in(F, \"[\")|file(F)) .|file(F), N = 1 +";
    let mut cases = Vec::new();
    for r in &rules {
        cases.push([Some(PRELUDE.to_string()), r.clone(), None]);
        for q in questions.split('|') {
            cases.push([
                Some(PRELUDE.to_string()),
                r.clone(),
                Some(super::super::face::question(q)),
            ]);
        }
    }
    let bad = mismatches(&cases);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
