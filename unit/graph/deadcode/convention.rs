use super::*;

/// `kind-json | path => holds` per line: each kind's positive case and
/// the nearest negative — the same basename where the rule does not
/// name it, another case, a later barred segment.
const CASES: &str = r#"
{"kind": "basename", "names": ["setup.py"]} | setup.py => true
{"kind": "basename", "names": ["setup.py"]} | pkg/tools/setup.py => true
{"kind": "basename", "names": ["setup.py"]} | pkg/Setup.py => false
{"kind": "basename", "names": ["setup.py"]} | pkg/setup.py.bak => false
{"kind": "under", "segment": "app", "no_later_prefix": "_", "names": ["page.tsx"]} | app/page.tsx => true
{"kind": "under", "segment": "app", "no_later_prefix": "_", "names": ["page.tsx"]} | web/app/blog/page.tsx => true
{"kind": "under", "segment": "app", "no_later_prefix": "_", "names": ["page.tsx"]} | pages/page.tsx => false
{"kind": "under", "segment": "app", "no_later_prefix": "_", "names": ["page.tsx"]} | page.tsx => false
{"kind": "under", "segment": "app", "no_later_prefix": "_", "names": ["page.tsx"]} | app/_lib/page.tsx => false
{"kind": "under", "segment": "app", "no_later_prefix": "_", "names": ["page.tsx"]} | app/a/app/_b/page.tsx => false
{"kind": "under", "segment": "app", "no_later_prefix": "_", "names": ["page.tsx"]} | _app/page.tsx => false
{"kind": "under", "segment": "app", "names": ["page.tsx"]} | app/_lib/page.tsx => true
{"kind": "exact", "nocase": true, "paths": ["CONTRIBUTING.md", ".github/CONTRIBUTING.md"]} | .github/contributing.md => true
{"kind": "exact", "nocase": true, "paths": ["CONTRIBUTING.md", ".github/CONTRIBUTING.md"]} | pkg/CONTRIBUTING.md => false
{"kind": "exact", "paths": ["SECURITY.md", "docs/SECURITY.md"]} | docs/SECURITY.md => true
{"kind": "exact", "paths": ["SECURITY.md", "docs/SECURITY.md"]} | docs/security.md => false
{"kind": "exact", "paths": ["SECURITY.md", "docs/SECURITY.md"]} | src/SECURITY.md => false"#;

#[test]
fn each_kind_reads_its_position_and_no_wider() {
    for row in CASES.lines().filter(|l| !l.trim().is_empty()) {
        let (rule, rest) = row.rsplit_once(" | ").expect("rule | case");
        let (path, want) = rest.split_once(" => ").expect("path => holds");
        let position: Position = serde_json::from_str(rule).expect(rule);
        assert_eq!(position.holds(path).to_string(), want, "{row}");
    }
}

/// A kind the reader does not know, or a key its kind does not take,
/// does not read: the package carrying it is refused, never half-read.
#[test]
fn an_unknown_kind_or_key_does_not_read() {
    for rule in [
        r#"{"kind": "glob", "names": ["x"]}"#,
        r#"{"kind": "basename", "names": ["x"], "nocase": true}"#,
        r#"{"kind": "exact", "names": ["x"]}"#,
    ] {
        assert!(serde_json::from_str::<Position>(rule).is_err(), "{rule}");
    }
}

/// The package's own rows read, and every one names a file: the eight
/// conventions of generation 1 and Neovim's root `init.lua`.
#[test]
fn the_package_rows_are_the_registered_conventions() {
    let names: Vec<&str> = crate::tables::get()
        .flags
        .entry_conventions
        .iter()
        .map(|c| c.name)
        .collect();
    assert_eq!(
        names,
        [
            "package-info.java",
            "vitest.config",
            "next-app-entry",
            "setup.py",
            "sphinx-conf.py",
            "CONTRIBUTING.md",
            "SECURITY.md",
            "CODE_OF_CONDUCT.md",
            "init.lua",
        ]
    );
    for path in [
        "src/a/package-info.java",
        "app/page.tsx",
        "docs/conf.py",
        ".github/SECURITY.md",
        "init.lua",
    ] {
        assert!(conventional(path), "{path}");
    }
    assert!(!conventional("lua/foo/init.lua"));
}
