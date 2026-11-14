//! The GUI's copies of the core's console sentences, held to the console
//! (plan v2.35, the dead-file amendment, phase 5C). The graph and check
//! screens print a few of the core's sentences from their own table
//! (gui/ui/i18n.js): the two reasons by code, the state words, a root's
//! fall count and a dependent's roots, the gate row's three readings, the
//! unreferenced axis's name with its off mark and the `dead_added` hint.
//! Nothing read the two side by side, so when 5A reworded the core's
//! code-1 reason the GUI kept the old one. Here each copy, its markers
//! filled (cli/tests/gui/words.js), must appear in the line the real `ce`
//! prints for a real tree, in both languages — the core's sentences are
//! read off the console, since the definitions package carries no console
//! text.

use crate::common::{self, WHOLESALE, core_bin, tmp};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Each language's console punctuation around the copied sentences, one
/// line per language: the language, the dash before a row's suffix, a
/// note's head, the check line's head, the `dead_added` line's head, a
/// state mark's opening (the row word with it) and closing (to the path),
/// and the brackets around a reason.
const PUNCT: &str = "en\t — \tnote: \tdeadcode check: \tdead_added: \tunreferenced (\t): \t(\t)\n\
                     zh\t——\t注：\tdeadcode check：\tdead_added：\t无引用（\t）：\t（\t）";

/// One language's punctuation, by column.
fn punct() -> Vec<Vec<&'static str>> {
    PUNCT
        .lines()
        .map(|l| l.trim().split('\t').collect())
        .collect()
}

/// The GUI's words, both languages, markers unfilled.
fn gui() -> Value {
    let out = common::gates::node(&["cli/tests/gui/words.js"], &[]);
    common::gates::expect_ok(&out, "node cli/tests/gui/words.js");
    serde_json::from_slice(&out.stdout).expect("the words")
}

/// One copied sentence of `lang`, its markers filled in order.
fn copy(w: &Value, lang: &str, key: &str, fill: &[&str]) -> String {
    let mut s = w[lang][key]
        .as_str()
        .unwrap_or_else(|| panic!("{lang}.{key}"))
        .to_string();
    for (i, f) in fill.iter().enumerate() {
        s = s.replace(&format!("{{{i}}}"), f);
    }
    s
}

/// A fresh project tree from `path: text` lines, `toml` its ce.toml.
fn tree(name: &str, files: &str, toml: &str) -> PathBuf {
    let dir = tmp(name);
    put(&dir, files);
    common::declare(&dir, toml);
    dir
}

fn put(dir: &Path, files: &str) {
    for l in files.lines() {
        let (path, text) = l.split_once(": ").expect("path: text");
        std::fs::write(dir.join(path), format!("{text}\n")).expect(path);
    }
}

/// The committed baseline, written by the named act.
fn establish(dir: &Path) {
    let (code, _, err) =
        common::ce_triple(dir, &["baseline", ".", "--core", &core_bin()], WHOLESALE);
    assert_eq!(code, Some(0), "{dir:?}: the baseline is written: {err}");
}

/// `ce` in `dir` under `lang`, stdout and stderr together.
fn ce(dir: &Path, lang: &str, args: &[&str]) -> String {
    let core = core_bin();
    let args: Vec<&str> = args.iter().copied().chain(["--core", &core]).collect();
    let (_, out, err) = common::ce_triple(dir, &args, &[("CE_LANG", lang)]);
    format!("{out}{err}")
}

/// `text` carries the GUI's copy of `key` (its markers filled by `fill`)
/// after the console's own lead `lead`.
fn holds(text: &str, w: &Value, (lang, lead): (&str, &str), key: &str, fill: &[&str]) {
    let line = format!("{lead}{}", copy(w, lang, key, fill));
    assert!(text.contains(&line), "{lang}: {line} in {text}");
}

/// The console's row for `path` (the verdict names are English in both
/// languages).
fn row<'a>(text: &'a str, path: &str) -> &'a str {
    text.lines()
        .find(|l| l.contains(&format!("{path}  un")))
        .unwrap_or_else(|| panic!("no row for {path}: {text}"))
}

/// The two reasons, a root's fall count, a dependent's roots and the
/// listed-not-blocked line: `p.ts` is a root (code 0), `d.ts` falls with
/// it (code 1), nothing is declared.
#[test]
fn the_reasons_and_the_grouping_words_are_the_consoles() {
    let w = gui();
    let dir = tree(
        "gui-words-grouping",
        "p.ts: import './d';\nd.ts: export const d = 1;",
        "",
    );
    for p in punct() {
        let (lang, text) = (p[0], ce(&dir, p[0], &["deadcode", ".", "--check"]));
        let why = |code: &str| {
            format!(
                "{}{}{}",
                p[7],
                w[lang]["deadWhy"][code].as_str().unwrap_or("?"),
                p[8]
            )
        };
        assert!(row(&text, "p.ts").contains(&why("0")), "{lang}: {text}");
        assert!(row(&text, "d.ts").contains(&why("1")), "{lang}: {text}");
        let falls = format!("{}{}", p[1], copy(&w, lang, "graphFalls", &["1"]));
        assert!(row(&text, "p.ts").ends_with(&falls), "{lang}: {text}");
        let under = format!(
            "{}{} p.ts",
            p[1],
            w[lang]["graphUnder"].as_str().unwrap_or("?")
        );
        assert!(row(&text, "d.ts").ends_with(&under), "{lang}: {text}");
        holds(&text, &w, (lang, p[3]), "gateListed", &["2"]);
    }
}

/// The three state words on the rows of a compared tree, the blocked
/// line, the `dead_added` hint and the off axis: `s.ts` was unreferenced
/// when the baseline was written (standing), `lib.ts` lost its last
/// reference since (orphaned), `n.ts` is new (born); `born` is blocked.
#[test]
fn the_state_words_the_gate_and_the_check_hint_are_the_consoles() {
    let w = gui();
    let dir = tree(
        "gui-words-states",
        "main.ts: import './lib';\nlib.ts: export const v = 1;\ns.ts: export const s = 1;",
        "[graph]\nentry_globs = [\"main.ts\"]\n[deadcode]\nblock = [\"born\"]\n",
    );
    establish(&dir);
    put(
        &dir,
        "main.ts: export const m = 1;\nn.ts: export const z = 1;",
    );
    let core = core_bin();
    let (_, out, err) = common::ce_triple(
        &dir,
        &["check", ".", "--format", "json", "--core", &core],
        &[],
    );
    let doc: Value = serde_json::from_str(&out).expect(&err);
    let off = doc["axes"][4][1].to_string();
    for p in punct() {
        let lang = p[0];
        let text = ce(&dir, lang, &["deadcode", ".", "--check"]);
        let words = &w[lang]["stateWords"];
        for (state, path) in [
            ("standing", "s.ts"),
            ("orphaned", "lib.ts"),
            ("born", "n.ts"),
        ] {
            let mark = format!(
                "{}{}{}{path}",
                p[5],
                words[state].as_str().unwrap_or("?"),
                p[6]
            );
            assert!(text.contains(&mark), "{lang}: {mark} in {text}");
        }
        let born = words["born"].as_str().unwrap_or("?");
        holds(&text, &w, (lang, p[3]), "gateBlocked", &["1", born]);
        let check = ce(&dir, lang, &["check", "."]);
        holds(&check, &w, (lang, p[4]), "deadAddedHint", &["1"]);
        let axis = format!(
            "{} {}",
            copy(&w, lang, "axis4", &[]),
            copy(&w, lang, "axisOff", &[&off])
        );
        assert!(check.contains(&axis), "{lang}: {axis} in {check}");
    }
}

/// The gate's other two readings: declared and answered with nothing
/// blocked (a standing root under `block = ["born"]`), and declared with
/// no comparison point (no baseline).
#[test]
fn the_gate_rows_none_blocked_and_unanswered_are_the_consoles() {
    let w = gui();
    let toml = "[deadcode]\nblock = [\"born\"]\n";
    let answered = tree("gui-words-answered", "s.ts: export const s = 1;", toml);
    establish(&answered);
    let unanswered = tree("gui-words-unanswered", "a.ts: export const a = 1;", toml);
    for p in punct() {
        let lang = p[0];
        let born = w[lang]["stateWords"]["born"].as_str().unwrap_or("?");
        let readings = [
            (
                &answered,
                "--check",
                p[3],
                "gateNoneBlocked",
                vec!["1", born],
            ),
            (
                &unanswered,
                "--format=console",
                p[2],
                "gateUnanswered",
                vec![born],
            ),
        ];
        for (dir, flag, lead, key, fill) in readings {
            holds(
                &ce(dir, lang, &["deadcode", ".", flag]),
                &w,
                (lang, lead),
                key,
                &fill,
            );
        }
    }
}
