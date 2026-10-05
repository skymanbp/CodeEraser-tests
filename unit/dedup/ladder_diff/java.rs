//! Java (plan v2.33 W2-text stage C): seeded trees (java_gen.rs draws
//! the files and their headers), declared `java` roots, and four legs —
//! one per rung kind (`import`, `import_star`, `type_ref`, thirty sites
//! a tree) and one whose specifiers mostly carry type annotations over
//! all three kinds, the annotation reader's leg. A site stands on a line:
//! an import's own line when it is one the header read, inside one of the
//! file's type declarations most of the time for a `type_ref`.

use super::common::{Tree, World};
use super::java_gen::{Names, class, headers, simple};
use super::rng::Rng;
use super::tables::t;
use crate::graph::ladder::java_header::Header;
use crate::scan::lang::Lang;

/// The White_Space a detector's spec may hold and a look-alike that is
/// not (U+200B); a `\n` stands for a folded line.
const SPACES: [&str; 8] = [
    " ", "\t", "  ", "\n", "\u{3000}", "\u{a0}", "\u{200b}", "\u{85}",
];

/// `static` as a spec may begin: each kind of space after it, none, and
/// two words that are not the keyword.
const STATIC: [&str; 7] = [
    "static ",
    "static\t",
    "static  ",
    "static\u{3000}",
    "static",
    "staticx ",
    "Static ",
];

/// A tree: its files, their headers, its declared roots, thirty sites of
/// the kinds `kind` draws, `annotate` % of them annotated.
fn tree(rng: &mut Rng, kind: fn(&mut Rng) -> &'static str, annotate: usize) -> Tree {
    let mut world = World::seeded(rng, (6, 22), t("java DIRS"), t("java BASES"));
    world.declare(rng, "java", t("java ROOTS"));
    let (java, names) = headers(rng, &world.files);
    world.java = java;
    let keep = |f: &str| f.ends_with(".java");
    let mut lines = Vec::new();
    let owned = world.sites(rng, keep, (t("java DIRS"), "Zz.java"), |rng, from| {
        let kind = kind(rng);
        let (spec, line) = spec(rng, kind, &names, world.java.get(&from));
        lines.push(line);
        (Lang::Java, kind, from, noise(rng, spec, annotate))
    });
    world.lines = lines;
    (world, owned)
}

/// A spec and the line it stands on, before noise.
fn spec(rng: &mut Rng, kind: &str, names: &Names, header: Option<&Header>) -> (String, usize) {
    if kind == "type_ref" {
        let name = if rng.chance(55) {
            simple(rng, names)
        } else {
            class(rng, names)
        };
        return (name, type_line(rng, header));
    }
    let star = kind == "import_star";
    let read: Vec<_> = header
        .into_iter()
        .flat_map(|h| &h.imports)
        .filter(|i| i.star == star)
        .collect();
    if !read.is_empty() && rng.chance(45) {
        let one = read[rng.below(read.len())];
        let prefix = if one.is_static { "static " } else { "" };
        return (format!("{prefix}{}", cut(rng, &one.name)), one.line);
    }
    let name = if star && rng.chance(50) {
        rng.pick(t("java JDK")).to_string()
    } else {
        class(rng, names)
    };
    let prefix = if rng.chance(20) {
        STATIC[rng.below(STATIC.len())]
    } else {
        ""
    };
    (format!("{prefix}{name}"), 1 + rng.below(12))
}

/// An import's name whole, or (40 %) cut after one of its dots — the
/// first line of a declaration folded over lines.
fn cut(rng: &mut Rng, name: &str) -> String {
    let dots: Vec<usize> = name.match_indices('.').map(|(at, _)| at).collect();
    if dots.is_empty() || !rng.chance(40) {
        return name.to_string();
    }
    name[..=dots[rng.below(dots.len())]].to_string()
}

/// A `type_ref`'s line: inside one of the file's type declarations (a
/// member's, sometimes) most of the time, else anywhere.
fn type_line(rng: &mut Rng, header: Option<&Header>) -> usize {
    let types = header.map(|h| h.types.as_slice()).unwrap_or_default();
    if types.is_empty() || rng.chance(25) {
        return 1 + rng.below(40);
    }
    let mut decl = &types[rng.below(types.len())];
    while !decl.members.is_empty() && rng.chance(50) {
        decl = &decl.members[rng.below(decl.members.len())];
    }
    let (first, last) = decl.lines;
    first + rng.below(last - first + 1)
}

/// The spellings a detector's spec may carry: an annotation before a
/// segment (`annotate` %), White_Space after a dot, an empty segment.
fn noise(rng: &mut Rng, mut spec: String, annotate: usize) -> String {
    if rng.chance(annotate) {
        let note = rng.pick(t("java ANNOTATIONS")).replace('~', "\n");
        let at = segment_start(rng, &spec);
        spec.insert_str(at, &note);
    }
    if rng.chance(8) {
        let at = segment_start(rng, &spec);
        spec.insert_str(at, SPACES[rng.below(SPACES.len())]);
    }
    match rng.below(100) {
        0..2 => spec.push('.'),
        2..4 => spec = spec.replacen('.', "..", 1),
        _ => {}
    }
    spec
}

/// The start of the spec or a place just after one of its dots.
fn segment_start(rng: &mut Rng, spec: &str) -> usize {
    let mut places: Vec<usize> = spec.match_indices('.').map(|(at, _)| at + 1).collect();
    places.push(spec.find(|c: char| !c.is_whitespace()).unwrap_or(0));
    places[rng.below(places.len())]
}

/// One kind for the whole leg.
fn import(_: &mut Rng) -> &'static str {
    "import"
}

fn import_star(_: &mut Rng) -> &'static str {
    "import_star"
}

fn type_ref(_: &mut Rng) -> &'static str {
    "type_ref"
}

/// Any of the three, and (2 %) a kind Java has no rung for.
fn any(rng: &mut Rng) -> &'static str {
    match rng.below(100) {
        0..2 => "export_from",
        n => ["import", "import_star", "type_ref"][n % 3],
    }
}

/// The four legs, each with its own tally: a name, a salt, the kinds it
/// draws, the share of annotated specifiers.
type Leg = (&'static str, u64, fn(&mut Rng) -> &'static str, usize);

const LEGS: [Leg; 4] = [
    ("java-import", 7, import, 6),
    ("java-import-star", 8, import_star, 6),
    ("java-type-ref", 9, type_ref, 6),
    ("java-annotated", 10, any, 80),
];

#[test]
#[ignore = "instrument: needs a core (CE_CORE_BIN); run with --ignored --nocapture"]
fn java_random_trees_agree() {
    for (name, salt, kind, annotate) in LEGS {
        super::common::leg(name, salt, false, |rng, _root| tree(rng, kind, annotate));
    }
}
