//! The third console-language selector (plan v2.29 step 9, O61):
//! `[ui] lang` in the project's ce.toml, read where the console face
//! loads its config — `--lang` first, CE_LANG second, the file third.
//!
//! Process roads only, on purpose: the selector is a per-process pin,
//! and the in-process suite must keep the English bytes every other
//! leg asserts — which is exactly what the unarmed library does
//! (i18n::init_from_config reads progress::armed, set by `ce`'s main
//! alone), so a config loaded by a test, the GUI or the MCP server
//! never speaks. The sentence read is `ce erase --log`'s absent-trail
//! line: no core, no git, no index — the cheapest command that speaks.

use crate::common;

const EN: &str = "erase log: no .ce/erase-log.ndjson yet";
const ZH: &str = "擦除日志：尚无 .ce/erase-log.ndjson";

/// One selector arrangement: (ce.toml, extra args, env, the sentence
/// the console must speak — and the other one must not).
type Row = (
    &'static str,
    &'static [&'static str],
    &'static [(&'static str, &'static str)],
    &'static str,
);

#[test]
fn the_file_speaks_third_after_the_flag_and_the_variable() {
    let dir = common::tmp("ui-lang");
    let rows: &[Row] = &[
        // nothing declared anywhere: English, as it always was
        ("", &[], &[], EN),
        // the file alone
        ("[ui]\nlang = \"zh\"\n", &[], &[], ZH),
        ("[ui]\nlang = \"en\"\n", &[], &[], EN),
        // the flag beats the file, both ways
        ("[ui]\nlang = \"zh\"\n", &["--lang", "en"], &[], EN),
        ("[ui]\nlang = \"en\"\n", &["--lang", "zh"], &[], ZH),
        // the variable beats the file, both ways
        ("[ui]\nlang = \"zh\"\n", &[], &[("CE_LANG", "en")], EN),
        ("[ui]\nlang = \"en\"\n", &[], &[("CE_LANG", "zh")], ZH),
        // the flag beats the variable, with the file saying a third thing
        (
            "[ui]\nlang = \"zh\"\n",
            &["--lang", "en"],
            &[("CE_LANG", "zh")],
            EN,
        ),
    ];
    for (i, (toml, extra, env, want)) in rows.iter().enumerate() {
        common::declare(&dir, toml);
        let mut args = vec!["erase", "--log"];
        args.extend_from_slice(extra);
        let (code, out, err) = common::ce_triple(&dir, &args, env);
        assert_eq!(code, Some(0), "row {i}: {err}");
        let other = if *want == EN { ZH } else { EN };
        assert!(
            out.contains(want) && !out.contains(other),
            "row {i} ({toml:?} {extra:?} {env:?}): {out:?}"
        );
    }
}

/// `--help` renders before any project is known: the file never
/// reaches it, only the flag and the variable do (README, ce-toml.md).
#[test]
fn help_reads_only_the_flag_and_the_variable() {
    let dir = common::tmp("ui-lang-help");
    common::declare(&dir, "[ui]\nlang = \"zh\"\n");
    let (_, plain, _) = common::ce_triple(&dir, &["--help"], &[]);
    let (_, flagged, _) = common::ce_triple(&dir, &["--help", "--lang", "zh"], &[]);
    assert!(plain.contains("Console language"), "{plain:?}");
    assert!(flagged.contains("控制台语言"), "{flagged:?}");
}

/// The hooks reach the same pin through their own Config::load: the
/// SessionStart health line — the one line every session shows —
/// answers in the project's language with no flag and no variable.
#[test]
fn the_session_start_hook_answers_in_the_projects_language() {
    let dir = common::tmp("ui-lang-hook");
    common::declare(&dir, "[ui]\nlang = \"zh\"\n");
    let (_, ctx) = common::session_start_line(&dir);
    assert!(ctx.contains("守卫："), "the health line in Chinese: {ctx}");
    assert!(!ctx.contains("guard:"), "no English twin beside it: {ctx}");
}
