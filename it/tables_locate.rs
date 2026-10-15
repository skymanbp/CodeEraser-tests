//! The core named by a path without `.exe` (1.9.2 D1). Git Bash answers
//! `command -v ce-core` as `/c/Program Files/CodeEraser/ce-core`; the
//! plugin starter bound and stamped that string as CE_CORE_BIN, the
//! spawner ran it (Windows appends `.exe`) while the package loader looked
//! for the literal file and refused every measuring command — `ce doctor`
//! said handshake OK and every hook went inert from 1.9.0. The loader now
//! reads a given path as the spawner does. Runs `ce` bare, as the
//! `--core` battery does: no core anywhere but the one named.

use crate::tables_core_flag::{bare, named_core, passed, refused};
use crate::tables_package::seeded;

#[test]
fn the_package_loader_reads_a_core_path_as_the_spawner_does() {
    let core = named_core();
    let stem = core.strip_suffix(".exe").unwrap_or(&core).to_string();
    let nothing = std::env::temp_dir()
        .join("ce-no-core-here")
        .display()
        .to_string();
    let [by_stem, by_exe, by_nothing] =
        [("stem", &stem), ("exe", &core), ("none", &nothing)].map(|(tag, named)| {
            let dir = seeded(&format!("core-locate-{tag}"));
            bare(
                &dir,
                &["scan", ".", "--format", "json"],
                &[("CE_CORE_BIN", named)],
            )
        });
    passed(&by_stem);
    passed(&by_exe);
    assert_eq!(by_stem.stdout, by_exe.stdout, "one core, two spellings");
    refused(&by_nothing, &["core unavailable", "ce-no-core-here"]);
}
