//! The Java legs' trees (plan v2.33 W2-text stage C): walked files under
//! standard-layout source sets (main, test, another module's) and plain
//! directories, each with the header the walk would hand over — a
//! package drawn from its directory or from a table, single and star
//! imports (static ones too) on lines that sometimes share one, and type
//! declarations with member types, supertypes written simple, nested or
//! qualified, and line ranges a site may fall inside — some headers for
//! files the walk does not hold; and the specifiers a detector writes:
//! an import the header read (whole, or cut at a dot as a folded
//! declaration leaves it), drawn names, `static` spelled with each kind
//! of space, extra White_Space, empty segments and type annotations
//! (their argument lists with strings, text blocks, char literals and
//! comments) — the annotation reader is on the path of every one.

use super::rng::Rng;
use super::tables::t;
use crate::graph::ladder::java_header::{Header, Import, TypeDecl};
use crate::graph::roots::parent_dir;
use std::collections::{BTreeMap, BTreeSet};

/// What drawn names are drawn against: every package a header declares
/// and every type chain (`C`, `C.N`) one declares.
#[derive(Default)]
pub(super) struct Names {
    packages: Vec<String>,
    types: Vec<String>,
}

/// The headers of a tree's files: most of its Java files (and now and
/// then one it does not walk), imports drawn once every package and
/// type is known.
pub(super) fn headers(
    rng: &mut Rng,
    files: &BTreeSet<String>,
) -> (BTreeMap<String, Header>, Names) {
    let mut java: Vec<String> = files
        .iter()
        .filter(|f| f.ends_with(".java") && !rng.chance(8))
        .cloned()
        .collect();
    if rng.chance(15) {
        java.push(format!("{}/Ghost.java", rng.pick(t("java DIRS"))));
    }
    let mut out = BTreeMap::new();
    let mut names = Names::default();
    for path in java {
        let header = bare(rng, &path);
        names.packages.push(header.package.clone());
        chains(&header.types, "", &mut names.types);
        out.insert(path, header);
    }
    for header in out.values_mut() {
        let lines = 2 + rng.below(8);
        header.imports = (0..rng.below(7))
            .map(|_| {
                let line = 2 + rng.below(lines);
                import(rng, &names, line)
            })
            .collect();
    }
    (out, names)
}

/// Every type chain a declaration list holds, `C` then `C.N`.
fn chains(types: &[TypeDecl], outer: &str, out: &mut Vec<String>) {
    for t in types {
        let chain = if outer.is_empty() {
            t.name.clone()
        } else {
            format!("{outer}.{}", t.name)
        };
        chains(&t.members, &chain, out);
        out.push(chain);
    }
}

/// A header without imports: its package and its types.
fn bare(rng: &mut Rng, path: &str) -> Header {
    let package = if rng.chance(70) {
        package_of(path)
    } else {
        rng.pick(t("java PACKAGES")).to_string()
    };
    let stem = path.rsplit('/').next().unwrap_or(path);
    let stem = stem.strip_suffix(".java").unwrap_or(stem).to_string();
    let count = if rng.chance(15) { 0 } else { 1 + rng.below(2) };
    let types = (0..count)
        .map(|i| {
            let name = if i == 0 && rng.chance(70) {
                stem.clone()
            } else {
                rng.pick(t("java CLASSES")).to_string()
            };
            let first = 1 + rng.below(12);
            decl(rng, name, first, 0)
        })
        .collect();
    Header {
        package,
        imports: Vec::new(),
        types,
    }
}

/// The package a file's directory spells: what follows a `java/`
/// segment, else the whole directory, slashes as dots.
fn package_of(path: &str) -> String {
    let dir = parent_dir(path);
    let below = match dir.find("java/") {
        Some(at) => &dir[at + 5..],
        None if dir.ends_with("java") => "",
        None => &dir,
    };
    below.replace('/', ".")
}

/// One type declaration from `first`, its supertypes and (two levels
/// down at most) its member types.
fn decl(rng: &mut Rng, name: String, first: usize, depth: usize) -> TypeDecl {
    let last = first + rng.below(30);
    let supers = (0..rng.below(3))
        .map(|_| rng.pick(t("java SUPERS")).to_string())
        .collect();
    let members = if depth < 2 {
        (0..rng.below(3))
            .map(|_| {
                let name = rng.pick(t("java CLASSES")).to_string();
                let from = first + rng.below(last - first + 2);
                decl(rng, name, from, depth + 1)
            })
            .collect()
    } else {
        Vec::new()
    };
    TypeDecl {
        name,
        supers,
        members,
        lines: (first, last),
    }
}

/// One header import: a class (a member after it when static), or —
/// star — a package or a class.
fn import(rng: &mut Rng, names: &Names, line: usize) -> Import {
    let (star, is_static) = (rng.chance(25), rng.chance(20));
    let name = if star && !is_static && rng.chance(65) {
        package(rng, names)
    } else if is_static && !star {
        format!("{}.{}", class(rng, names), rng.pick(t("java MEMBERS")))
    } else {
        class(rng, names)
    };
    Import {
        name,
        star,
        is_static,
        line,
    }
}

/// A package: one a header declares, the JDK's or a table's.
fn package(rng: &mut Rng, names: &Names) -> String {
    match rng.below(10) {
        0..6 if !names.packages.is_empty() => {
            names.packages[rng.below(names.packages.len())].clone()
        }
        6..8 => rng.pick(t("java JDK")).to_string(),
        _ => rng.pick(t("java PACKAGES")).to_string(),
    }
}

/// A qualified class: a package then a type chain.
pub(super) fn class(rng: &mut Rng, names: &Names) -> String {
    let package = package(rng, names);
    let chain = simple(rng, names);
    if package.is_empty() {
        chain
    } else {
        format!("{package}.{chain}")
    }
}

/// A type name as a body writes it: a chain a header declares, or a
/// table's word (a JDK type, an unknown one).
pub(super) fn simple(rng: &mut Rng, names: &Names) -> String {
    if !names.types.is_empty() && rng.chance(65) {
        names.types[rng.below(names.types.len())].clone()
    } else {
        rng.pick(t("java CLASSES")).to_string()
    }
}
