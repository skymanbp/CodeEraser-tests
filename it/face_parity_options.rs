//! Three-face parity, the half a capability table cannot see (plan
//! v2.33 step 10): the OPTIONS each face takes. A row claiming `ce
//! check`, the MCP `check` tool and the Tauri `check_report` command
//! said nothing about which knobs each reached, and the CLI's `--days`
//! reached neither machine face. Here each report pair's option sets
//! are derived from the shipped code — the CLI's from `ce <sub>
//! --help` (clap is the parser, so its help is the truth), the MCP
//! tool's from the catalog in `cli/src/mcp/tools.rs`, the Tauri
//! command's from its signature in `gui/src-tauri/src/commands*.rs` —
//! and compared after subtracting `faces::exempt::EXEMPT` (the one list
//! of CLI flags that are not document options) and renaming by
//! `faces::exempt::ALIASES`. Also here: every GUI command a row claims
//! is called by the webview, and every CLI claim exists whole. A
//! negative probe points `CE_PARITY_ROOT` at a scratch copy of the tree
//! (and `CE_PARITY_BIN` at a ce built from it).

use crate::face_parity_sources::{cli_names, mcp_names, parity_root, tauri_names};
use crate::face_parity_table::rows;
use codeeraser::faces::exempt::{ALIASES, EXEMPT};
use std::collections::BTreeSet;

/// The report pairs, one per line: the CLI subcommand, the MCP tool,
/// the Tauri command, then the subcommand's flags this pair sets aside
/// — a flag that selects ANOTHER claim's document (`erase --log`,
/// `graph --sites`), itself a claim in the table.
pub const PAIRS: &str = "
scan scan scan_report
dedup check_duplication dedup_report
clone clone clone_report
docdup docdup docdup_report
graph graph_sites sites_report sites
deadcode deadcode deadcode_report
churn churn churn_report
join join join_report
structure structure structure_report
trend trend trend_report
check check check_report
similar similar_units similar_report
query query query_report
rules rules rules_report
flow flow flow_report
merge merge_suggestions merge_report
arch architecture arch_report
erase erase erase_preview log
erase erase_log erase_log_report log
doctor doctor doctor_report
update update_check update_check
";

/// A name as every face compares it: the CLI's spelling renamed by
/// ALIASES, `-` read as `_`.
fn norm(name: &str) -> String {
    let named = ALIASES
        .iter()
        .find(|(cli, _)| *cli == name)
        .map_or(name, |(_, machine)| *machine);
    named.replace('-', "_")
}

/// What no face compares: the exempt flags, the root, clap's own, and
/// the flags a pair sets aside.
fn kept(names: impl IntoIterator<Item = String>, aside: &[&str]) -> BTreeSet<String> {
    let dropped = ["root", "path", "win", "help", "version"];
    names
        .into_iter()
        .filter(|n| !dropped.contains(&n.as_str()))
        .filter(|n| !EXEMPT.iter().any(|(f, _)| norm(f) == *n))
        .filter(|n| !aside.iter().any(|a| norm(a) == *n))
        .collect()
}

/// One pair's three option sets, by face.
fn sets(line: &str) -> (String, [BTreeSet<String>; 3]) {
    let words: Vec<&str> = line.split_whitespace().collect();
    let (sub, tool, cmd, aside) = (words[0], words[1], words[2], &words[3..]);
    let normed = |names: Vec<String>| kept(names.iter().map(|n| norm(n)), aside);
    let label = format!("ce {sub} / MCP {tool} / {cmd}");
    let faces = [
        normed(cli_names(sub)),
        normed(mcp_names(tool)),
        normed(tauri_names(cmd)),
    ];
    (label, faces)
}

/// The pair rows, blank lines skipped.
pub fn pairs() -> impl Iterator<Item = &'static str> {
    PAIRS.lines().filter(|l| !l.trim().is_empty())
}

/// A parity gate's drift, empty — or failed with every line of it.
pub fn none(drift: Vec<String>) {
    assert!(drift.is_empty(), "{}", drift.join("\n"));
}

/// The option-set gate: per report pair, the CLI's document options
/// (help, minus EXEMPT) equal the MCP tool's properties and the Tauri
/// command's parameters, each missing option named with its face.
pub fn option_mismatches() -> Vec<String> {
    let mut out = Vec::new();
    for line in pairs() {
        let (label, [cli, mcp, tauri]) = sets(line);
        let all: BTreeSet<&String> = cli.iter().chain(&mcp).chain(&tauri).collect();
        for name in all {
            for (face, set) in [("CLI", &cli), ("MCP", &mcp), ("GUI", &tauri)] {
                if !set.contains(name) {
                    out.push(format!("{label}: `{name}` missing on the {face} face"));
                }
            }
        }
    }
    out
}

#[test]
fn every_document_option_reaches_all_three_faces() {
    none(option_mismatches());
}

/// Each pair's three members sit in one row, and every flag it sets
/// aside is claimed whole in the table (`<sub> --<flag>`).
#[test]
fn every_pair_sits_in_one_row_and_its_set_aside_flags_are_claims() {
    let table = rows();
    for line in pairs() {
        let w: Vec<&str> = line.split_whitespace().collect();
        let first = |c: &String| c.split(' ').next() == Some(w[0]);
        let home = table.iter().find(|r| {
            r.cli.iter().any(first)
                && r.plugin.contains(&format!("mcp:{}", w[1]))
                && r.gui.contains(&w[2].to_string())
        });
        assert!(home.is_some(), "{line}: no row claims all three");
        for aside in &w[3..] {
            let claim = format!("{} --{aside}", w[0]);
            let claimed = table.iter().any(|r| r.cli.contains(&claim));
            assert!(
                claimed,
                "{line}: sets `--{aside}` aside, no row claims `{claim}`"
            );
        }
    }
}

/// Every EXEMPT row names a flag some subcommand still takes: a stale
/// exemption is a hole the next flag of that name walks through.
#[test]
fn every_exempt_flag_is_one_the_cli_takes() {
    let subs = crate::face_parity::cli_subcommands();
    let taken: BTreeSet<String> = subs.iter().flat_map(|s| cli_names(s)).collect();
    for (flag, why) in EXEMPT {
        assert!(
            taken.contains(*flag),
            "exempt `--{flag}` ({why}) is no flag"
        );
        assert!(!why.is_empty());
    }
}

/// Every GUI command a row claims is called by the webview — an
/// `invoke("<cmd>"`, a hub entry `cmd: "<cmd>"`, or the screens' one
/// load road (app.js `load`, whose call names the command on its first
/// line) in a `gui/ui` script — not merely registered with Tauri.
pub fn uncalled_commands() -> Vec<String> {
    let dir = parity_root().join("gui/ui");
    let scripts: String = std::fs::read_dir(&dir)
        .expect("gui/ui")
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "js"))
        .map(|e| std::fs::read_to_string(e.path()).expect("script"))
        .collect();
    let claimed: BTreeSet<String> = rows()
        .iter()
        .flat_map(|r| r.gui.clone())
        .filter(|g| !g.starts_with("tab:"))
        .collect();
    claimed
        .into_iter()
        .filter(|c| {
            let quoted = format!("\"{c}\"");
            !scripts.contains(&format!("invoke({quoted}"))
                && !scripts.contains(&format!("cmd: {quoted}"))
                && !scripts
                    .lines()
                    .any(|l| l.contains("load(") && l.contains(&quoted))
        })
        .collect()
}

#[test]
fn every_claimed_gui_command_is_called_by_the_webview() {
    let uncalled = uncalled_commands();
    assert!(
        uncalled.is_empty(),
        "registered, never called: {uncalled:?}"
    );
}

/// A CLI claim is compared whole: every `--flag` it names is one its
/// subcommand's help lists (a hook's `[class+…]` legs are
/// face_parity_legs.rs's).
#[test]
fn every_cli_claim_exists_whole() {
    for item in rows().iter().flat_map(|r| r.cli.clone()) {
        let claim = item.split(" [").next().expect("claim");
        let mut words = claim.split(' ');
        let sub = words.next().expect("sub");
        let names = cli_names(sub);
        for flag in words {
            let flag = flag
                .strip_prefix("--")
                .expect("a claim's later words are flags");
            assert!(names.iter().any(|n| n == flag), "`ce {claim}`: no --{flag}");
        }
    }
}
