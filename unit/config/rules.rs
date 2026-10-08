use crate::config::{Config, PackCfg};
use crate::testutil::scratch;
use serde_json::json;

fn pack(toml: &str) -> Result<PackCfg, String> {
    toml::from_str::<Config>(&format!("[rules.pack]\n{toml}"))
        .map(|c| c.rules.pack)
        .map_err(|e| e.to_string())
}

/// Every key reads into its row, in the table's key order; an
/// undeclared or switched-off item sends no row, and an absent table is
/// no rows at all (an undeclared pack sends no `pack` key).
#[test]
fn every_key_reads_into_its_row() {
    let every = pack(
        r#"no_dir_cycles = true
fan_in_max = 30
fan_out_max = 0
layers = ["top/**", "middle/**", "bottom/**"]
forbid = [["a/**", "b/**"], ["c/**", "d/**"]]"#,
    )
    .expect("every key");
    assert_eq!(
        every.rows(),
        [
            json!(["no_dir_cycles"]),
            json!(["fan_in_max", 30]),
            json!(["fan_out_max", 0]),
            json!(["layers", "top/**", "middle/**", "bottom/**"]),
            json!(["forbid", "a/**", "b/**"]),
            json!(["forbid", "c/**", "d/**"]),
        ]
    );
    for one in ["no_dir_cycles = false", "layers = []", "forbid = []", ""] {
        assert!(pack(one).expect(one).rows().is_empty(), "{one:?}");
    }
    assert!(Config::default().rules.pack.rows().is_empty());
    // a lone layer and a pair of the wrong length are read and sent as
    // they stand: the core refuses them by name (CE.Query.Pack)
    let odd = pack(
        r#"layers = ["a/**"]
forbid = [["a/**"], ["a", "b", "c"]]"#,
    )
    .expect("odd shapes");
    assert_eq!(
        odd.rows(),
        [
            json!(["layers", "a/**"]),
            json!(["forbid", "a/**"]),
            json!(["forbid", "a", "b", "c"]),
        ]
    );
}

/// What serde refuses at the load throat, by the key or the shape:
/// an unknown key, a negative count, a count that is not an integer, a
/// `forbid` entry that is not a list, a layer that is not a string —
/// each through the table read alone and through `Config::load`.
#[test]
fn the_load_throat_refuses_by_name() {
    let cases = [
        ("no_cycles = true", "no_cycles"),
        ("fan_in = 3", "fan_in"),
        ("fan_in_max = -1", "fan_in_max"),
        ("fan_out_max = \"3\"", "fan_out_max"),
        ("forbid = [\"a/**\"]", "forbid"),
        ("layers = [1, 2]", "layers"),
        ("no_dir_cycles = 1", "no_dir_cycles"),
    ];
    let dir = scratch("rules-pack-cfg");
    for (toml, key) in cases {
        let read = pack(toml).expect_err(toml);
        std::fs::write(dir.join("ce.toml"), format!("[rules.pack]\n{toml}\n")).expect("ce.toml");
        let loaded = Config::load(&dir).expect_err(toml);
        assert!(
            read.contains(key) && loaded.contains(key),
            "{toml:?}: {read} / {loaded}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}
