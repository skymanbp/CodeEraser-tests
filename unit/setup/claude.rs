//! The listing reader behind setup's "already registered" probe: the
//! `--json` array first, the prose form (a marker glyph before each
//! name, `Source:` rows under it) for a CLI too old for the flag.

use super::names_in;

#[test]
fn a_json_listing_yields_its_names_in_order() {
    let json = r#"[
      {"name": "cc-memory", "source": "directory", "path": "D:\\p\\cc-memory"},
      {"name": "codeeraser", "source": "github", "repo": "skymanbp/CodeEraser"}
    ]"#;
    assert_eq!(names_in(json), ["cc-memory", "codeeraser"]);
    assert_eq!(names_in("[]"), Vec::<String>::new());
}

#[test]
fn the_prose_listing_keeps_bare_names_and_drops_source_rows() {
    let prose = "Configured marketplaces:\n\n  ❯ cc-memory\n    Source: Directory (D:\\Projects\\cc-memory)\n\n  > codeeraser\n    Source: GitHub (skymanbp/CodeEraser)\n";
    assert_eq!(names_in(prose), ["cc-memory", "codeeraser"]);
    // a name the listing does not carry is not there, whatever the glyph
    assert!(!names_in("  ❯ other\n").iter().any(|n| n == "codeeraser"));
}
