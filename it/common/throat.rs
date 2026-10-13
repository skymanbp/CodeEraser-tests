//! The one process-construction throat, enforced (audit C row 14):
//! every leg outside common/ reaches the real binary through
//! `run_ce_env`, `ce_triple` or `ce_command`, which scrub the acts,
//! the console language and the update notice before any env the leg
//! lays over. A leg that builds its own `Command` on the binary
//! inherits whatever the developer's shell exported.

use std::path::PathBuf;

/// The raw form, assembled so this file does not spell it.
fn raw() -> String {
    format!("Command::new(env!(\"{}\"))", "CARGO_BIN_EXE_ce")
}

/// `file:line` of every raw construction under it/, common/ excepted.
fn raw_ce_commands(it: &std::path::Path) -> Vec<String> {
    let (mut files, needle) = (Vec::<PathBuf>::new(), raw());
    super::files_with_ext(it, "rs", &mut files);
    files.sort();
    let mut out = Vec::new();
    for f in files.iter().filter(|f| !f.starts_with(it.join("common"))) {
        let text = std::fs::read_to_string(f).expect("a test source");
        for (i, l) in text.lines().enumerate() {
            if l.contains(&needle) {
                out.push(format!("{}:{}", f.display(), i + 1));
            }
        }
    }
    out
}

#[test]
fn raw_ce_commands_go_through_the_throat() {
    let it = super::repo_root().join("cli/tests/it");
    let found = raw_ce_commands(&it);
    assert!(
        found.is_empty(),
        "build the ce process through common::run_ce_env / ce_triple / ce_command:\n{}",
        found.join("\n")
    );
}
