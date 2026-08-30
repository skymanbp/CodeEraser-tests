//! Every sentence the PreToolUse guard speaks, asked in both languages
//! (`cli/src/guard/say.rs`). Its own file rather than a stanza in
//! guard_hook: that battery is about what the rules DECIDE, this one is
//! about what they SAY, and the pair stood at 452 lines against a
//! 300-line soft line.
//!
//! The Stop audit has answered `CE_LANG=zh` in Chinese since M8-G3b;
//! this face — the only one that ever refuses a write, and so the one a
//! person reads most — answered in English on every line it had,
//! because the templates sat inline at six call sites across two files
//! and nothing ever asked them as a set. Three of the rows below had no
//! test in any language: the degraded note, the clip marker, and the
//! unreadable-baseline note.

use std::path::Path;

use crate::common;
use crate::common::pretooluse_envelope as envelope;
use crate::common::{rust_fn, seed_project, tmp};

/// Ask one write twice — the default road and `CE_LANG=zh` — and
/// hold both halves of the sentence. The last assert is the one that
/// matters: a face passes "contains the Chinese" trivially by never
/// having translated at all, so the ENGLISH needle must be gone from
/// the Chinese answer.
fn both_languages(dir: &Path, envelope: &str, want: &str, en: &str, zh: &str) -> String {
    // each language is its own SESSION: B4 suppresses a second warn
    // for the same (rule, file, session), so asking twice inside one
    // session answers the second time with silence — which is the
    // correct product behaviour and the wrong question here.
    let ask = |lang: &[(&str, &str)], session: &str| {
        let mut e: serde_json::Value = serde_json::from_str(envelope).expect("envelope");
        e["session_id"] = session.into();
        let out = common::run_hook_env(dir, &["probe", "--hook"], &e.to_string(), lang);
        let v: serde_json::Value = serde_json::from_str(out.trim()).expect("decision json");
        assert_eq!(v["hookSpecificOutput"]["permissionDecision"], want, "{out}");
        v["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .expect("reason")
            .to_string()
    };
    let english = ask(&[], "en");
    assert!(english.contains(en), "en: {english}");
    let chinese = ask(&[("CE_LANG", "zh")], "zh");
    assert!(chinese.contains(zh), "zh: {chinese}");
    assert!(!chinese.contains(en), "zh answered in English: {chinese}");
    chinese
}

/// One bilingual scene: what the tree declares, what is written into
/// it, and the two halves of the sentence the guard must answer with.
/// A table rather than one call stanza per scene — five stanzas that
/// differ only in their literals are a clone block under this repo's
/// own T2 normalization, and its dedup gate said so.
struct Scene {
    tag: &'static str,
    indexed: bool,
    declare: Vec<(&'static str, String)>,
    write: String,
    want: &'static str,
    en: &'static str,
    zh: &'static str,
    /// Asserted on the Chinese answer when the scene also drives the
    /// reason past its token budget.
    tail: Option<&'static str>,
}

/// A scene with nothing declared, nothing indexed and nothing written:
/// every row states only what it changes.
fn scene(tag: &'static str, want: &'static str, en: &'static str, zh: &'static str) -> Scene {
    Scene {
        tag,
        indexed: false,
        declare: Vec::new(),
        write: String::new(),
        want,
        en,
        zh,
        tail: None,
    }
}

/// Every sentence guard/say.rs holds, one row each — the two rules that
/// judge the write, the graded-zone advisory, and the two degraded
/// notes. The broken-config row carries a 900-character parse error, so
/// its reason also runs past the 200-token warn budget: that is where
/// hookio's clip marker gets asked in Chinese, since i18n pins the
/// language once per PROCESS and no in-process test can ask twice.
fn scenes() -> Vec<Scene> {
    let toml = |t: &str| vec![("ce.toml", t.to_string())];
    let filler = |n: usize| "// filler\n".repeat(n);
    vec![
        Scene {
            indexed: true,
            write: rust_fn(1),
            ..scene(
                "guard-say-dup",
                "deny",
                "indexed region(s): ",
                "已索引区域重复",
            )
        },
        Scene {
            write: filler(751),
            ..scene(
                "guard-say-budget",
                "deny",
                "past the hard budget of 750",
                "越过 750 行的硬预算",
            )
        },
        Scene {
            declare: toml("[guard]\nzone_tiers = true\n"),
            write: filler(600),
            ..scene(
                "guard-say-zone",
                "allow",
                "666‰ into the graded zone",
                "进入分级区 666‰",
            )
        },
        Scene {
            declare: toml(&format!("not_a_table = {}\n", "x".repeat(900))),
            write: "// one line\n".to_string(),
            tail: Some("…（已截断；完整记录见 .ce/observe.ndjson）"),
            ..scene(
                "guard-say-broken",
                "allow",
                "guard degraded to observe",
                "守卫已降级为 observe",
            )
        },
        Scene {
            declare: [
                ("ce.toml", "[thresholds]\nfile_lines_fail = 0\n"),
                ("ce-baseline.json", "{not json"),
            ]
            .map(|(n, t)| (n, t.to_string()))
            .to_vec(),
            write: filler(900),
            ..scene(
                "guard-say-baseline",
                "deny",
                "the committed baseline is unreadable",
                "提交在库的基线不可读",
            )
        },
    ]
}

/// The whole set, asked in both languages (guard/say.rs). The Stop
/// audit has answered `CE_LANG=zh` in Chinese since M8-G3b; this face —
/// the only one that ever refuses a write, and so the one a person
/// reads most — answered in English on every line it had, because the
/// templates sat inline at six call sites across two files and nothing
/// ever asked them as a set. Three of these rows had no test in any
/// language: the degraded note, the clip marker, the unreadable
/// baseline.
#[test]
fn every_sentence_the_guard_speaks_answers_in_both_languages() {
    for s in scenes() {
        let dir = tmp(s.tag);
        if s.indexed {
            seed_project(&dir, "deny");
        }
        for (name, text) in &s.declare {
            std::fs::write(dir.join(name), text).expect(name);
        }
        let chinese = both_languages(&dir, &envelope(&dir, "Write", &s.write), s.want, s.en, s.zh);
        if let Some(tail) = s.tail {
            assert!(chinese.ends_with(tail), "{}: {chinese}", s.tag);
        }
    }
}
