//! Shared primitives for the GENERATED bench surfaces (the render
//! tests): one owner for the parsed contract, field access, the
//! latest-version read, the measured-at label and the
//! iterate-a-table shape — the dedup gate caught every one of them
//! growing a twin when the dashboard renderer landed beside the
//! chip renderer.

use serde_json::Value;

/// contracts/bench/bench.json, parsed.
pub fn doc() -> Value {
    let text = std::fs::read_to_string(super::bench_path()).expect("bench.json");
    serde_json::from_str(&text).expect("bench.json parses")
}

/// String field or empty — absent fields render blank, never panic.
pub fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}

/// One field of the newest series row. Rows are sorted by version, so
/// the last one is the newest; two readers asked it two ways until the
/// dedup gate said they were one.
fn newest<'a>(d: &'a Value, key: &str) -> &'a str {
    d["rows"]
        .as_array()
        .and_then(|rows| rows.last())
        .map(|row| s(row, key))
        .unwrap_or("")
}

/// The newest series version present.
pub fn latest(d: &Value) -> &str {
    newest(d, "version")
}

/// The commit the newest series row was measured at — the tree any
/// candidate row must differ from to earn one of its own.
pub fn newest_row_commit(d: &Value) -> &str {
    newest(d, "commit")
}

/// WHY the series holds no row for the release this build IS. There are
/// two reasons and a reader must be able to tell them apart.
pub enum NoRow {
    /// The rule turned this release away: it ships the same measured
    /// code as the newest row, and a second measurement of the same
    /// program is machine drift wearing a version number.
    NothingNew,
    /// It earns a row and does not have one yet — the whole series is
    /// replayed in one sitting, and that happens after the tag.
    ReplayOwed,
}

/// The release this build IS, when the series holds no row for it.
///
/// The heading used to call `latest` the "latest version", which was
/// true only while every release joined the series. v1.3.1 shipped the
/// same `cli/src` and `core/app` as v1.3.0 and so did not join — a row
/// for an identical program publishes machine drift as if it were a
/// version delta, which the BENCH header warns is wider than most
/// deltas a reader would try to read. The heading then quietly went on
/// naming v1.3.0 as the latest, which it no longer was. Nothing but
/// this comparison can notice that, so every surface asks it.
///
/// v1.4.1 then shipped in the OTHER case and no surface could say so:
/// it changed `cli/src`, earned a row, and the replay had not run yet,
/// so every page printed the sentence written for a release that ships
/// the same program — which a reader joins to the rule stated three
/// lines above and reads as "no code changed". The case is asked here,
/// with the same predicate the two row writers use.
pub fn release_without_a_row(d: &Value) -> Option<(&'static str, NoRow)> {
    let release = env!("CARGO_PKG_VERSION");
    if latest(d) == release {
        return None;
    }
    let newest = newest_row_commit(d).to_string();
    let case = if newest.is_empty() || super::brings_something_new(&newest, "HEAD") {
        NoRow::ReplayOwed
    } else {
        NoRow::NothingNew
    };
    Some((release, case))
}

/// What joins a sentence to whatever else shares its line. English
/// takes a space; Chinese takes nothing at all — a space after 。 is a
/// typographic error, not a nicety. Every bilingual surface that
/// concatenates prose asks this, so it is asked in one place.
pub fn join(zh: bool) -> &'static str {
    if zh { "" } else { " " }
}

/// That fact as one sentence, for any surface that shows the numbers.
/// Plain text: it goes into a Markdown paragraph and an HTML caption
/// unchanged, and carries no character either of them would escape.
/// Empty when the release did join — the sentence must not appear on a
/// page whose heading already names the current version.
pub fn unmeasured_note(d: &Value, zh: bool) -> String {
    let Some((v, why)) = release_without_a_row(d) else {
        return String::new();
    };
    format!("{}{}", join(zh), no_row_sentence(&why, v, zh))
}

/// The four sentences — two reasons × two languages — apart from the
/// reading that picks one, so a gate can ask all four at once. They
/// are line-continued literals, and a lost `\` leaves the source
/// indentation inside the string; it then reads as a broken build on
/// the page, in both READMEs and on four site pages, where no byte
/// gate can see it because every one of those compares a file with
/// this generator (`bench_render::no_generated_sentence_…`).
pub fn no_row_sentence(why: &NoRow, v: &str, zh: bool) -> String {
    match (why, zh) {
        (NoRow::NothingNew, true) => {
            format!("当前发布 v{v} 与最新一行是同一份被测代码，故没有自己的行。")
        }
        (NoRow::NothingNew, false) => format!(
            "The current release, v{v}, ships the same measured code as \
             the newest row and gets none of its own."
        ),
        (NoRow::ReplayOwed, true) => {
            format!("当前发布 v{v} 该有自己的行，而全序列重跑在打 tag 之后，尚未落表。")
        }
        (NoRow::ReplayOwed, false) => format!(
            "The current release, v{v}, earns a row and does not have one \
             yet: the whole series is replayed in one sitting after the tag."
        ),
    }
}

/// The sentence docs/BENCH.md owes its reader either way: which release
/// the newest row is, or — when this release did not join — that it has
/// no row at all. The README and the site name the version in their
/// heading instead; this page's heading is a link anchor other documents
/// point at, so it stays stable and the version lives in prose.
pub fn series_note(d: &Value) -> String {
    match release_without_a_row(d) {
        Some(_) => unmeasured_note(d, false).trim().to_string(),
        None => format!(
            "The newest row, v{}, is the release this build is.",
            latest(d)
        ),
    }
}

/// Which surfaces owe the reader that sentence: every one that prints
/// these numbers beside a version. The list is here rather than at the
/// call sites because it was five of the seven for a release — the two
/// dashboard pages, the most detailed public latency surface, printed
/// 1.4.0 as the newest row while the site shipped 1.4.1 and said
/// nothing about the gap.
pub const VERSION_BEARING_SURFACES: [&str; 7] = [
    "docs/BENCH.md",
    "README.md",
    "README.zh.md",
    "site/index.html",
    "site/zh/index.html",
    "site/bench/index.html",
    "site/zh/bench/index.html",
];

/// Every surface printing a version beside these numbers must name the
/// release this build IS — in its heading when that release joined the
/// series, in `series_note` / `unmeasured_note` when it did not.
///
/// v1.3.1 shipped with all four surfaces headed "v1.3.0" and every byte
/// gate green: a gate that compares a file to its own generator cannot
/// notice that the generator stopped telling the truth. This is the one
/// assertion that reads the crate version instead of the contract.
pub fn names_the_release(surface: &str, text: &str) {
    let release = env!("CARGO_PKG_VERSION");
    assert!(
        text.contains(&format!("v{release}")),
        "{surface} prints the latency numbers without naming v{release}, the \
         release this build is. Either the series gained a row for it and the \
         heading should say so, or it did not and the surface owes the reader \
         the sentence saying which version these numbers came from."
    );
}

/// The measured-at label with the dirty suffix — one spelling on
/// every surface (dates are digits and dashes, safe in md and html).
pub fn measured(row: &Value) -> String {
    format!(
        "{}{}",
        s(row, "measured_at"),
        if row["dirty"] == Value::Bool(true) {
            " (dirty)"
        } else {
            ""
        }
    )
}

/// Concatenate one rendered line per row of `d[key]` — the table
/// shape every generated surface repeats.
pub fn rows_with(d: &Value, key: &str, f: impl Fn(&Value) -> String) -> String {
    d[key].as_array().expect(key).iter().map(f).collect()
}
