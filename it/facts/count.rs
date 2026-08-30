//! The count family: every number the prose spells as a word — axes,
//! languages, grammars, screens, hooks, MCP tools, wire families,
//! booklets, fail conditions, erase names, verdict codes, release
//! binaries, dogfood gates — plus the one large cap the README spells
//! with a comma. Counted off a typed path or the tree where one
//! exists; scraped, with the debt named, where the product keeps the
//! list private or in another language.

use super::{Fact, linked, read, scraped};
use std::collections::BTreeSet;
use std::path::Path;

pub fn facts(root: &Path) -> Vec<Fact> {
    let mut out = typed();
    out.extend(tree(root));
    out.extend(manifest(root));
    out.extend(scrapes(root));
    out
}

fn typed() -> Vec<Fact> {
    use codeeraser::{erase, graph, join, scan, score};
    vec![
        linked(
            "count:axes#word",
            score::knobs::AXES.len(),
            "cli/src/score/knobs.rs::AXES",
        ),
        linked(
            "count:langs#word",
            scan::lang::Lang::judged_mask().count_ones(),
            "cli/src/scan/lang.rs::Lang::judged_mask (set bits)",
        ),
        linked(
            "count:erase_reasons#word",
            erase::REASON_NAMES.len(),
            "cli/src/erase/model.rs::REASON_NAMES",
        ),
        linked(
            "count:erase_classes#word",
            erase::CLASS_NAMES
                .iter()
                .filter(|n| **n != "(retired)")
                .count(),
            "cli/src/erase/model.rs::CLASS_NAMES (live names)",
        ),
        linked(
            "count:join_codes#word",
            join::verdicts::VERDICT_NAMES.len(),
            "cli/src/join/verdicts.rs::VERDICT_NAMES",
        ),
        linked(
            "count:deadcode_codes#word",
            graph::deadcode::VERDICT_NAMES.len(),
            "cli/src/graph/deadcode.rs::VERDICT_NAMES",
        ),
    ]
}

fn json(root: &Path, rel: &str) -> serde_json::Value {
    serde_json::from_str(&read(root, rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// Files with `ext` directly under `dir` (not recursive).
fn entries(root: &Path, dir: &str, ext: Option<&str>) -> usize {
    std::fs::read_dir(root.join(dir))
        .unwrap_or_else(|e| panic!("{dir}: {e}"))
        .flatten()
        .map(|e| e.path())
        .filter(|p| match ext {
            Some(x) => p.extension().is_some_and(|got| got == x),
            None => p.is_dir(),
        })
        .count()
}

/// Counts off the tree and the contract JSONs.
fn tree(root: &Path) -> Vec<Fact> {
    let hooks = json(root, "plugin/hooks/hooks.json");
    let hello = read(root, "contracts/fixtures/handshake/hello-ok.ndjson");
    let reply: serde_json::Value =
        serde_json::from_str(hello.lines().nth(1).expect("hello reply")).expect("hello reply json");
    let capabilities = reply["capabilities"].as_array().expect("capabilities");
    vec![
        linked(
            "count:families#word",
            capabilities.iter().filter(|c| c != &"hello").count(),
            "contracts/fixtures/handshake/hello-ok.ndjson::capabilities (minus hello)",
        ),
        linked(
            "count:hooks#word",
            hooks["hooks"].as_object().expect("hooks object").len(),
            "plugin/hooks/hooks.json::hooks (events)",
        ),
        linked(
            "count:skills#word",
            entries(root, "plugin/skills", None),
            "plugin/skills/* (directories)",
        ),
        linked(
            "count:commands#word",
            entries(root, "plugin/commands", Some("md")),
            "plugin/commands/*.md",
        ),
        linked(
            "count:booklets#word",
            entries(root, "docs/reference/methodology", Some("md")),
            "docs/reference/methodology/*.md",
        ),
    ]
}

/// The release pins: `CE_SHA256_<PLATFORM>_<ASSET>` keys.
fn manifest(root: &Path) -> Vec<Fact> {
    let map = codeeraser::update::manifest::parse(&read(root, "plugin/bin/manifest.env"));
    let pins: Vec<&str> = map
        .keys()
        .filter_map(|k| k.strip_prefix("CE_SHA256_"))
        .collect();
    let platforms: BTreeSet<&str> = pins
        .iter()
        .map(|k| k.rsplit_once('_').expect("platform_asset").0)
        .collect();
    let installers = pins
        .iter()
        .filter(|k| k.ends_with("_SETUP") || k.ends_with("_APPIMAGE") || k.ends_with("_DMG"))
        .count();
    let src = "plugin/bin/manifest.env::CE_SHA256_* (update::manifest::parse)";
    vec![
        linked("count:binaries#word", pins.len(), src),
        linked("count:platforms#word", platforms.len(), src),
        linked("count:installers#word", installers, src),
    ]
}

/// The arms of `CE.Structure.Axes.axes` — one per structure axis.
/// The check score's axes are a DIFFERENT roster (`count:axes#word`,
/// linked to score::knobs::AXES); the two are equal today and are not
/// the same list, which is the whole reason this fact exists.
fn structure_axis_arms(axes_hs: &str) -> usize {
    between(axes_hs, "axes :: Knobs -> Facts", "\n where")
        .matches("count (")
        .count()
}

/// The `Some(...)` arms of `Lang::grammar` — one per tree-sitter
/// grammar (TypeScript and TSX are two grammars from one crate, so
/// the Cargo dependency lines undercount).
fn grammar_arms(lang_rs: &str) -> usize {
    between(lang_rs, "pub fn grammar(self)", "_ => None")
        .matches("=> Some(")
        .count()
}

/// The slice of `text` after the first `start` and before the next
/// `end` — the one cut every source-slice scrape makes.
fn between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let after = text
        .split(start)
        .nth(1)
        .unwrap_or_else(|| panic!("no {start:?} in the scraped source"));
    after.split(end).next().expect("split yields a head")
}

fn scrapes(root: &Path) -> Vec<Fact> {
    let tools = read(root, "cli/src/mcp/tools.rs");
    let table = between(&tools, "pub const TOOLS", "];");
    let faces = read(root, "core/app/CE/Verdict/Faces.hs");
    let conds = between(&faces, "failConditions r ", "]");
    let ci = read(root, ".github/workflows/ci.yml");
    let dogfood = between(&ci, "name: Dogfood (", "- name:");
    let cost = read(root, "core/app/CE/Erase/Cost.hs");
    let cap = cost
        .lines()
        .find_map(|l| l.strip_prefix("eraseRowCap = "))
        .expect("Cost.hs declares eraseRowCap")
        .trim();
    vec![
        scraped(
            "count:grammars#word",
            grammar_arms(&read(root, "cli/src/scan/lang.rs")),
            "cli/src/scan/lang.rs::Lang::grammar (Some arms)",
            "Lang has no variant iterator; promote = Lang::judged() + grammar().is_some()",
        ),
        scraped(
            "count:structure_axes#word",
            structure_axis_arms(&read(root, "core/app/CE/Structure/Axes.hs")),
            "core/app/CE/Structure/Axes.hs::axes (count arms)",
            "a Haskell list read by line; promote = the axis roster on the structure wire",
        ),
        scraped(
            "count:screens#word",
            read(root, "gui/ui/index.html")
                .matches("data-tab=\"")
                .count(),
            "gui/ui/index.html::data-tab",
            "the tab strip is HTML; promote = a tab roster the GUI and the registry both read",
        ),
        scraped(
            "count:mcp_tools#word",
            table.matches("tool!(").count(),
            "cli/src/mcp/tools.rs::TOOLS (tool! rows)",
            "mcp::tools is a private module; promote = a pub names() face",
        ),
        scraped(
            "count:fail_conditions#word",
            conds.matches("(\"").count(),
            "core/app/CE/Verdict/Faces.hs::failConditions",
            "a Haskell list read by line; promote = the names on the hello wire",
        ),
        scraped(
            "count:gates#word",
            dogfood.matches("cargo run --locked -- ").count(),
            ".github/workflows/ci.yml::Dogfood step (cargo run lines)",
            "promote = one gate list CI and the docs both read",
        ),
        scraped(
            "gate:erase.row_cap#digits",
            cap,
            "core/app/CE/Erase/Cost.hs::eraseRowCap",
            "a Haskell const read by line; promote = the cap on the hello wire",
        ),
    ]
}
