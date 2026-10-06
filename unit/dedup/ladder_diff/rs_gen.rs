//! The Rust legs' texts (plan v2.33 W2-text stage F): Cargo.toml files
//! and Rust sources written the way crates write them — `mod`
//! declarations bare, behind a `#[path]` and `#[cfg(test)]` stack and
//! inside bodied modules and function bodies; `use` items of every head
//! (`crate`, `self`, `super` chains, a local module, a crate name, the
//! global `::` form) with brace groups, globs, renames and hand-folded
//! lines; `pub use` surfaces, globs and `extern crate` bindings; item
//! definitions that share a name with a module; comments and string
//! literals that spell items, and a broken item now and then. Every word
//! comes from the `rs` rows of tables.rs.

use super::rng::Rng;
use super::tables::t;

/// One Cargo.toml's text: a `[package]` (mostly named), now and then a
/// `[lib]` path and `[[bin]]` paths, dependency tables; rarely broken.
pub(super) fn manifest(rng: &mut Rng) -> String {
    let mut out = String::new();
    if rng.chance(85) {
        out.push_str("[package]\n");
        if rng.chance(90) {
            out.push_str(&format!("name = \"{}\"\n", rng.pick(t("rs PKG_NAMES"))));
        }
    }
    if rng.chance(25) {
        out.push_str(&format!(
            "[lib]\npath = \"{}\"\n",
            rng.pick(t("rs LIB_PATHS"))
        ));
    }
    for _ in 0..rng.below(3) {
        let path = rng.pick(t("rs BIN_PATHS"));
        out.push_str(&format!("[[bin]]\nname = \"b\"\npath = \"{path}\"\n"));
    }
    for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if rng.chance(35) {
            out.push_str(&format!("[{table}]\n"));
            for _ in 0..1 + rng.below(3) {
                out.push_str(&format!("{} = \"1\"\n", rng.pick(t("rs DEPS"))));
            }
        }
    }
    if rng.chance(4) {
        out.push_str("[package\nbroken");
    }
    out
}

/// A workspace root's text.
pub(super) fn workspace() -> String {
    "[workspace]\nmembers = [\"crates/a\", \"crates/b\"]\n".to_string()
}

/// One source file: three to ten items, aimed at `rung` (0 `mod`, 1
/// `crate::`, 2 `self` / `super` / a local module, 3 a crate name, 4 the
/// re-export surface), three in ten aimed elsewhere.
pub(super) fn source(rng: &mut Rng, rung: usize) -> String {
    let n = 3 + rng.below(8);
    let items: Vec<String> = (0..n).map(|_| item(rng, rung, 0)).collect();
    items.join("\n") + "\n"
}

fn item(rng: &mut Rng, rung: usize, depth: usize) -> String {
    let aim = if rng.chance(70) { rung } else { rng.below(5) };
    let roll = rng.below(20);
    if roll < 2 && depth < 2 {
        return inline(rng, rung, depth);
    }
    if roll < 5 {
        return rng.pick(t("rs DEFS")).replace('$', rng.pick(t("rs ITEMS")));
    }
    if roll == 5 {
        return rng.pick(t("rs NOISE")).replace('~', "\n");
    }
    match aim {
        0 => module(rng),
        4 if roll < 12 => surface(rng),
        _ => use_item(rng, aim),
    }
}

/// A `mod x;` declaration, behind attributes now and then.
fn module(rng: &mut Rng) -> String {
    let mut attrs = String::new();
    if rng.chance(15) {
        attrs.push_str("#[cfg(test)]\n");
    }
    if rng.chance(30) {
        attrs.push_str(&format!("#[path = \"{}\"]\n", rng.pick(t("rs PATHS"))));
    }
    if rng.chance(10) {
        attrs.push_str("#[allow(unused)]\n");
    }
    let vis = rng.pick(t("rs VIS"));
    format!("{attrs}{vis}mod {};", rng.pick(t("rs NAMES")))
}

/// A bodied module (or a function body) holding one to four items.
fn inline(rng: &mut Rng, rung: usize, depth: usize) -> String {
    let body: Vec<String> = (0..1 + rng.below(4))
        .map(|_| item(rng, rung, depth + 1))
        .collect();
    let body = body.join("\n");
    match rng.below(6) {
        0 => format!("fn f() {{\n{body}\n}}"),
        1 => format!("#[cfg(test)]\nmod tests {{\n    use super::*;\n{body}\n}}"),
        _ => {
            let vis = rng.pick(t("rs VIS"));
            format!("{vis}mod {} {{\n{body}\n}}", rng.pick(t("rs NAMES")))
        }
    }
}

/// A path aimed at a rung: its head, zero to three module segments, an
/// item now and then, and a tail (a glob, a group, a rename) or none.
fn path(rng: &mut Rng, aim: usize) -> String {
    let head = match aim {
        1 => "crate",
        2 if rng.chance(70) => rng.pick(t("rs LOCAL")),
        3 => rng.pick(t("rs EXTERN")),
        0 | 2 => rng.pick(t("rs NAMES")),
        _ => rng.pick("crate|self|super|a|std|::a|::core|b|x"),
    };
    let global = if aim == 3 && rng.chance(15) { "::" } else { "" };
    let mut segs = vec![format!("{global}{head}")];
    for _ in 0..rng.below(4) {
        segs.push(rng.pick(t("rs NAMES")).to_string());
    }
    if rng.chance(50) {
        segs.push(rng.pick(t("rs ITEMS")).to_string());
    }
    let mut out = segs.join("::");
    match rng.below(20) {
        0..3 => out.push_str("::*"),
        3..5 => out.push_str("::{Thing, a::X, self}"),
        5 => out.push_str("::{\n    a,\n    b::Other,\n}"),
        6 | 7 => out.push_str(" as Z"),
        _ => {}
    }
    out
}

/// A `use` item: its visibility, its path, and now and then a hand fold
/// that cuts the path after a `::`, or a prefix-less group.
fn use_item(rng: &mut Rng, aim: usize) -> String {
    let vis = rng.pick(t("rs VIS"));
    if rng.chance(4) {
        return format!("{vis}use {{{}, {}}};", path(rng, aim), path(rng, aim));
    }
    let mut p = path(rng, aim);
    if rng.chance(12)
        && let Some(at) = p.find("::")
    {
        p.insert_str(at + 2, "\n    ");
    }
    format!("{vis}use {p};")
}

/// The re-export surface: a `pub use` (a glob now and then) or an
/// `extern crate` binding, pub or not, renamed or not.
fn surface(rng: &mut Rng) -> String {
    let aim = rng.below(4);
    match rng.below(6) {
        0 => {
            let vis = rng.pick("pub |");
            let alias = if rng.chance(40) { " as y" } else { "" };
            format!("{vis}extern crate {}{alias};", rng.pick(t("rs EXTERN")))
        }
        1 => format!("pub use {}::*;", path(rng, aim)),
        _ => format!("pub use {};", path(rng, aim)),
    }
}
