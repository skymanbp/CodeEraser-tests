//! Versions, revisions and the toolchain (the report-id family is
//! report.rs).

use super::{Fact, linked, read, scraped};
use crate::common::repo_root;
use std::collections::BTreeSet;
use std::path::Path;

/// The request anchor (VERSIONING §3): the major every golden request
/// line stands at, re-anchored on each major and deliberately skewed
/// between. fixture_contract.rs derives the rest of the triple from
/// the files against this one.
pub const ANCHOR: &str = "6.0.0";

pub fn facts() -> Vec<Fact> {
    let root = repo_root();
    let mut out = typed();
    out.push(pin(&root));
    out.extend(toolchain(&root));
    out.push(index_schema(&root));
    out
}

fn typed() -> Vec<Fact> {
    use codeeraser::{corelink, daemon, dedup, docdup, graph, mention};
    vec![
        linked(
            "ver:ce#v",
            env!("CARGO_PKG_VERSION"),
            "cli/Cargo.toml::version",
        ),
        linked("ver:proto#v", corelink::PROTO, "cli/src/corelink.rs::PROTO"),
        linked(
            "ver:daemon#v",
            daemon::proto::DAEMON_PROTO,
            "cli/src/daemon/proto.rs::DAEMON_PROTO",
        ),
        linked("ver:anchor#v", ANCHOR, "cli/tests/it/facts/ver.rs::ANCHOR"),
        linked(
            "ver:graph_rev#digits",
            graph::store::GRAPH_REV,
            "cli/src/graph/store.rs::GRAPH_REV",
        ),
        linked(
            "ver:mention_rev#digits",
            mention::MENTION_REV,
            "cli/src/mention/mod.rs::MENTION_REV",
        ),
        linked(
            "ver:docdup_rev#digits",
            docdup::DOCDUP_REV,
            "cli/src/docdup/mod.rs::DOCDUP_REV",
        ),
        linked(
            "ver:tokenizer_rev#digits",
            dedup::tokens::TOKENIZER_REV,
            "cli/src/dedup/tokens.rs::TOKENIZER_REV",
        ),
        linked(
            "ver:struct_rev#digits",
            dedup::struct_fp::STRUCT_REV,
            "cli/src/dedup/struct_fp.rs::STRUCT_REV",
        ),
    ]
}

/// The plugin's pinned release, through the product's manifest parser.
fn pin(root: &Path) -> Fact {
    let map = codeeraser::update::manifest::parse(&read(root, "plugin/bin/manifest.env"));
    linked(
        "ver:pin#v",
        map.get("CE_MANIFEST_VERSION")
            .expect("manifest.env carries CE_MANIFEST_VERSION"),
        "plugin/bin/manifest.env::CE_MANIFEST_VERSION (update::manifest::parse)",
    )
}

/// The first double-quoted value on `line`.
fn quoted(line: &str, rel: &str) -> String {
    let (_, rest) = line
        .split_once('"')
        .unwrap_or_else(|| panic!("{rel}: no quoted value in {line:?}"));
    rest.split('"').next().expect("closing quote").to_string()
}

/// The quoted value on the ONE line of `text` starting with `key`.
fn quoted_after(text: &str, rel: &str, key: &str) -> String {
    let hits: Vec<&str> = text.lines().filter(|l| l.starts_with(key)).collect();
    assert_eq!(hits.len(), 1, "{rel}: exactly one line starts with {key:?}");
    quoted(hits[0], rel)
}

/// Every line of `text` carrying `key` agrees on one quoted value
/// (ci.yml spells the GHC pin once per job).
fn agreed(text: &str, rel: &str, key: &str) -> String {
    let values: BTreeSet<String> = text
        .lines()
        .filter(|l| l.trim_start().starts_with(key))
        .map(|l| quoted(l, rel))
        .collect();
    assert_eq!(values.len(), 1, "{rel}: {key:?} values {values:?}");
    values.into_iter().next().expect("one value")
}

const TOML_DEBT: &str = "read by a line grammar, not a TOML reader; promote = parse the manifest with the toml crate the product already links";

fn toolchain(root: &Path) -> Vec<Fact> {
    let cli = read(root, "cli/Cargo.toml");
    let ts = quoted_after(&cli, "cli/Cargo.toml", "tree-sitter ");
    let minor = ts.rsplit_once('.').expect("a dotted version").0.to_string();
    let dep = |key: &str| quoted_after(&cli, "cli/Cargo.toml", key);
    vec![
        scraped(
            "tool:rust#v",
            quoted_after(
                &read(root, "rust-toolchain.toml"),
                "rust-toolchain.toml",
                "channel",
            ),
            "rust-toolchain.toml::channel",
            TOML_DEBT,
        ),
        scraped(
            "tool:ghc#v",
            agreed(
                &read(root, ".github/workflows/ci.yml"),
                "ci.yml",
                "ghc-version:",
            ),
            ".github/workflows/ci.yml::ghc-version",
            "the pin lives in the workflow alone; promote = one toolchain file CI and the docs both read",
        ),
        scraped(
            "tool:edition#digits",
            dep("edition "),
            "cli/Cargo.toml::edition",
            TOML_DEBT,
        ),
        scraped(
            "tool:tree_sitter#v",
            &ts,
            "cli/Cargo.toml::tree-sitter",
            TOML_DEBT,
        ),
        scraped(
            "tool:tree_sitter#vminor",
            minor,
            "cli/Cargo.toml::tree-sitter",
            TOML_DEBT,
        ),
        scraped(
            "tool:rusqlite#vminor",
            dep("rusqlite "),
            "cli/Cargo.toml::rusqlite",
            TOML_DEBT,
        ),
        scraped(
            "tool:interprocess#v",
            dep("interprocess "),
            "cli/Cargo.toml::interprocess",
            TOML_DEBT,
        ),
        scraped(
            "tool:tauri#digits",
            quoted_after(
                &read(root, "gui/src-tauri/Cargo.toml"),
                "gui/src-tauri/Cargo.toml",
                "tauri ",
            ),
            "gui/src-tauri/Cargo.toml::tauri",
            TOML_DEBT,
        ),
    ]
}

/// The index schema version — a crate-private const, read by line.
fn index_schema(root: &Path) -> Fact {
    let key = "const SCHEMA_VERSION: i64 = ";
    let text = read(root, "cli/src/dedup/schema.rs");
    let line = text
        .lines()
        .find(|l| l.starts_with(key))
        .expect("schema.rs declares SCHEMA_VERSION");
    scraped(
        "ver:schema.index#digits",
        line[key.len()..]
            .split(';')
            .next()
            .expect("terminator")
            .trim(),
        "cli/src/dedup/schema.rs::SCHEMA_VERSION",
        "crate-private const; promote = a pub reader, or the index's own `pragma user_version` face",
    )
}
