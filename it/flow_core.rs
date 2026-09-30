//! The flow/1 leg end to end through the real core (plan v2.31 step 4
//! A3): every unit of the nine code-language crosscheck corpora lowered,
//! sent and judged — the core refuses none, its counts echo what was
//! sent (wire::consume holds them), and every finding reads back through
//! the legend to a line and a name. `--nocapture` prints each file's
//! findings as `kind line name` and each corpus's tally. A second leg
//! mixes one illegal unit into a legal batch: the core names it, the
//! batch drops it and asks again, and the rest judge as they would alone.

use crate::common::{core_bin, repo_root};
use codeeraser::corelink::Link;
use codeeraser::flow::lower::{Legend, Lowered, Unit, lower_file};
use codeeraser::flow::wire::{self, Verdict};
use codeeraser::scan::lang::Lang;
use std::path::PathBuf;

const CORPORA: [&str; 9] = [
    "python",
    "typescript",
    "rust",
    "go",
    "c",
    "cpp",
    "java",
    "lua",
    "r",
];

const KINDS: [&str; 4] = ["unreachable", "dead_store", "unused_local", "unused_param"];

/// A corpus's files, lowered, beside their paths.
fn corpus(name: &str) -> (Vec<PathBuf>, Vec<Lowered>) {
    let dir = repo_root().join("contracts/fixtures/crosscheck").join(name);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("a corpus")
        .flatten()
        .map(|e| e.path())
        .collect();
    paths.sort();
    paths
        .into_iter()
        .filter_map(|p| {
            let lang = Lang::from_path(&p)?;
            let lowered = lower_file(&std::fs::read_to_string(&p).expect("a file"), lang)?;
            Some((p, lowered))
        })
        .unzip()
}

fn unit_at(files: &[Lowered], file: usize, nth: usize) -> &Unit {
    let found = files[file].units.iter().find(|u| u.nth == nth);
    found.expect("a finding names a unit that was sent")
}

/// A finding's place and name through the legend: an unreachable run's
/// first statement, a store's statement, a variable's declaring place.
fn read_back(u: &Unit, kind: u8, seq: i64, v: i64) -> (u32, String) {
    let name = usize::try_from(v).map_or(String::new(), |v| u.legend.var_name[v].clone());
    let line = match usize::try_from(seq) {
        Ok(s) if kind <= 1 => u.legend.stmt_at[s].0,
        _ => u.legend.var_at[v as usize].0,
    };
    (line, name)
}

#[test]
fn every_crosscheck_unit_is_judged_by_the_core_and_reads_back() {
    let (mut link, _) = Link::open(&core_bin()).expect("open core");
    let mut refused = Vec::new();
    for name in CORPORA {
        let (paths, files) = corpus(name);
        let v: Verdict = wire::judge(&mut link, &files).expect("a judgment");
        let dynamic = files.iter().flat_map(|f| &f.units).filter(|u| u.dynamic);
        assert_eq!(
            v.dynamic_units,
            dynamic.count() as u64,
            "{name}: dynamic units"
        );
        let mut by_kind = [0usize; 4];
        for (f, path) in paths.iter().enumerate() {
            let here = v.findings.iter().filter(|(file, ..)| *file == f);
            for (_, nth, finding) in here {
                let u = unit_at(&files, f, *nth);
                let (line, var) = read_back(u, finding.kind, finding.seq, finding.v);
                assert!(
                    line >= u.start_line && line <= u.end_line,
                    "{path:?}: {finding:?}"
                );
                by_kind[finding.kind as usize] += 1;
                println!(
                    "  {} {line} {var} ({})",
                    KINDS[finding.kind as usize], u.name
                );
            }
        }
        for (f, nth, reason) in &v.refused {
            refused.push(format!(
                "{:?}: {} {reason}",
                paths[*f],
                unit_at(&files, *f, *nth).name
            ));
        }
        let units: usize = files.iter().map(|f| f.units.len()).sum();
        println!(
            "flow_core {name}: {units} units, findings {by_kind:?} ({}), {} refused",
            KINDS.join(" / "),
            v.refused.len()
        );
    }
    assert!(
        refused.is_empty(),
        "the core refused lowered units:\n{}",
        refused.join("\n")
    );
}

/// An if flagged has_else with one child: a tree the core refuses.
fn illegal() -> Unit {
    Unit {
        nth: 999,
        name: "illegal".into(),
        start_line: 1,
        end_line: 1,
        params: 0,
        dynamic: false,
        stmts: vec![[0, -1, 2, 1, 0], [1, 0, 1, 0, 0]],
        vars: Vec::new(),
        uses: Vec::new(),
        legend: Legend {
            stmt_at: vec![(1, 1), (1, 1)],
            stmt_end: vec![1, 1],
            stmt_text: vec![String::new(); 2],
            var_name: Vec::new(),
            var_at: Vec::new(),
        },
    }
}

#[test]
fn a_refused_unit_leaves_the_batch_and_the_rest_are_judged() {
    let (mut link, _) = Link::open(&core_bin()).expect("open core");
    let (_, alone) = corpus("python");
    let (_, mut mixed) = corpus("python");
    let at = mixed[0].units.len() / 2;
    mixed[0].units.insert(at, illegal());
    let (clean, dirty) = (
        wire::judge(&mut link, &alone).expect("a judgment"),
        wire::judge(&mut link, &mixed).expect("a judgment"),
    );
    assert_eq!(dirty.refused.len(), 1, "{:?}", dirty.refused);
    let (file, nth, reason) = &dirty.refused[0];
    assert_eq!((*file, *nth), (0, 999));
    assert!(
        reason.contains("if branches disagree with has_else"),
        "{reason}"
    );
    let rows = |v: &Verdict| -> Vec<_> {
        let each = v.findings.iter();
        each.map(|(f, n, x)| (*f, *n, x.kind, x.seq, x.v, x.seq_end))
            .collect()
    };
    assert_eq!(rows(&dirty), rows(&clean));
    assert!(!rows(&clean).is_empty(), "the python corpus holds findings");
}
