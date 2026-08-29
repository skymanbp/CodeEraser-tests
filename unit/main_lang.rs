use super::*;
use clap::CommandFactory;

/// A `judge.<id>` key is alive exactly when some command's arg
/// RESOLVES through it — declares the id and names no key of its
/// own. A fallback nobody reaches is as dead as a stranded
/// per-command key, and the two-way rule may not have a hole.
fn falls_back(cmd: &clap::Command, m: &HashMap<&'static str, &'static str>, id: &str) -> bool {
    cmd.get_subcommands().any(|sc| {
        !m.contains_key(format!("{}.{id}", sc.get_name()).as_str())
            && sc.get_arguments().any(|a| a.get_id().as_str() == id)
    })
}

/// The charter's G3 acceptance for this face: the zh lookup is
/// complete in BOTH directions — a subcommand or helped arg
/// without a zh entry is red (missing key, now via the SAME
/// resolution localize uses), and a key naming a node that no
/// longer exists is red too (dead key), so a flag rename cannot
/// silently strand its translation.
#[test]
fn zh_lookup_is_complete_and_alive() {
    let cmd = crate::main_cli::Cli::command();
    let m = zh_map();
    assert!(m.contains_key("ce") && m.contains_key("ce.lang"));
    for sc in cmd.get_subcommands() {
        let name = sc.get_name();
        if name == "help" {
            continue;
        }
        assert!(m.contains_key(name), "subcommand {name} has no zh about");
        for a in sc.get_arguments() {
            let id = a.get_id().as_str();
            if matches!(id, "help" | "version" | "lang") || a.get_help().is_none() {
                continue; // format carries no help by design
            }
            assert!(
                zh_help(m, name, id).is_some(),
                "arg {name}.{id} has no zh help"
            );
        }
    }
    for id in SHARED_ARGS {
        let k = format!("judge.{id}");
        assert!(m.contains_key(k.as_str()), "shared arg {k} has no zh help");
        assert!(
            falls_back(&cmd, m, id),
            "dead key {k}: nobody resolves to it"
        );
    }
    for k in m.keys() {
        if matches!(*k, "ce" | "ce.lang") {
            continue;
        }
        match k.split_once('.') {
            None => assert!(cmd.find_subcommand(k).is_some(), "dead key {k}"),
            // the fallback names no subcommand; its own liveness
            // is the SHARED_ARGS loop above
            Some(("judge", _)) => {}
            Some((c, a)) => {
                let sc = cmd
                    .find_subcommand(c)
                    .unwrap_or_else(|| panic!("dead key {k}: no subcommand {c}"));
                assert!(
                    sc.get_arguments().any(|x| x.get_id().as_str() == a),
                    "dead key {k}: no arg {a}"
                );
            }
        }
    }
}

/// The shared-arg keys, in the fallback's own resolution order.
const SHARED_ARGS: [&str; 3] = ["root", "core", "db"];
