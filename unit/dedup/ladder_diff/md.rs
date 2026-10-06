//! Markdown: seeded trees of documents (headings ATX and setext,
//! duplicates, raw-HTML anchors, code spans and fences, definitions —
//! indented, fenced, commented, quoted, duplicated, case and whitespace
//! variants, Greek capital sigmas, a dotted capital I, titlecase
//! digraphs — and reference links) beside code files and walked assets,
//! one leg per site kind: links and images (the tree's own files reached
//! from the site's directory, odd targets, schemes, site-root and
//! protocol-relative forms, percent escapes good and ill-formed,
//! fragments), reference links (the labels a document defines, spelled
//! again, and others), definitions (the targets a document defines, and
//! others) and URLs. The documents are written under the tree root: the
//! frozen rungs read them, the request's facts too.

use super::common::{Tree, World, legs, put};
use super::rng::Rng;
use super::tables::t;
use crate::graph::md::is_md_path;
use crate::graph::roots::join_dir;
use crate::scan::lang::Lang;
use std::collections::BTreeMap;
use std::path::Path;

/// One table word with the leg's escapes undone: `~` a line break, `^` a
/// tab, `°` a no-break space.
fn word(rng: &mut Rng, table: &'static str) -> String {
    rng.pick(t(table))
        .replace('~', "\n")
        .replace('^', "\t")
        .replace('°', "\u{a0}")
}

/// A document: lines of the line table with headings, labels and targets
/// drawn in, its labels from a few of its own (so its reference links
/// meet its definitions); the label and target of each definition-shaped
/// line kept.
fn document(
    rng: &mut Rng,
    (path, own): (&str, &[String]),
    wrote: &mut Vec<(String, String)>,
) -> String {
    let mut text = String::new();
    let labels: Vec<String> = (0..1 + rng.below(4))
        .map(|_| word(rng, "md LABELS"))
        .collect();
    for _ in 0..rng.below(40) {
        let label = labels[rng.below(labels.len())].clone();
        let target = target(rng, path, own);
        let line = word(rng, "md LINES")
            .replace("$H", &word(rng, "md HEADS"))
            .replace("$L", &label)
            .replace("$T", &target);
        if line.contains("]:") {
            wrote.push((label, target));
        }
        text.push_str(&line);
        text.push_str(if rng.chance(8) { "\r\n" } else { "\n" });
    }
    text
}

/// A label spelled another way: its case flipped or its spaces doubled.
fn respelled(rng: &mut Rng, l: String) -> String {
    match rng.below(5) {
        0 => l.to_uppercase(),
        1 => l.to_lowercase(),
        2 => l.replace(' ', "  \t"),
        _ => l,
    }
}

/// A link target: one of the tree's files reached from the site's
/// directory, or a table target behind a prefix; now and then a fragment.
fn target(rng: &mut Rng, from: &str, own: &[String]) -> String {
    let mut s = if rng.chance(50) && !own.is_empty() {
        let up = "../".repeat(from.matches('/').count());
        format!("{up}{}", own[rng.below(own.len())])
    } else {
        format!("{}{}", rng.pick(t("md PREFIXES")), word(rng, "md TARGETS"))
    };
    if rng.chance(40) {
        s = format!("{s}#{}", rng.pick(t("md FRAGS")));
    }
    s
}

/// Mostly (60 %) what the site's document wrote (`pick` of a written
/// pair), else `other`.
fn written(
    rng: &mut Rng,
    wrote: Option<&Vec<(String, String)>>,
    pick: fn(&(String, String)) -> &String,
    other: &'static str,
) -> String {
    match wrote.filter(|w| !w.is_empty() && rng.chance(60)) {
        Some(w) => pick(&w[rng.below(w.len())]).clone(),
        None => word(rng, other),
    }
}

/// The tree of a leg aimed at site kind `k` (the LEGS rung column): six
/// to twenty-five walked files, up to five walked assets.
fn tree(rng: &mut Rng, root: &Path, k: usize) -> Tree {
    let walked = 6 + rng.below(20);
    let files = rng.paths(t("md DIRS"), t("md BASES"), walked);
    let assets = rng.below(6);
    let world = World {
        files,
        assets: rng.paths("|", t("md ASSETS"), assets),
        ..World::default()
    };
    let mut docs: Vec<String> = world
        .files
        .iter()
        .filter(|f| is_md_path(f))
        .cloned()
        .collect();
    for dir in t("md DIRS").split('|') {
        if rng.chance(15) {
            docs.push(join_dir(dir, "zz.md"));
        }
    }
    let own: Vec<String> = world.files.iter().chain(&world.assets).cloned().collect();
    let mut wrote: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for doc in docs {
        let mut pairs = Vec::new();
        put(root, &doc, &document(rng, (&doc, &own), &mut pairs));
        wrote.insert(doc, pairs);
    }
    let kind = ["link", "image", "ref_link", "ref_def", "url"][k];
    let owned = world.sites(rng, is_md_path, (t("md DIRS"), "zz.md"), |rng, from| {
        let spec = match kind {
            "ref_link" => {
                let l = written(rng, wrote.get(&from), |p| &p.0, "md LABELS");
                respelled(rng, l)
            }
            "ref_def" => written(rng, wrote.get(&from), |p| &p.1, "md TARGETS"),
            _ => target(rng, &from, &own),
        };
        (Lang::Markdown, kind, from, spec)
    });
    (world, owned)
}

#[test]
#[ignore = "instrument: needs a core (CE_CORE_BIN); run with --ignored --nocapture"]
fn markdown_random_trees_agree() {
    legs("md", tree);
}
