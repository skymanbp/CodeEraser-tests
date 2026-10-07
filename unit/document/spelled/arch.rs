//! The W7 differential's arch leg (plan v2.33 W7; mounted inside the
//! frozen arch face): paths, directories, a directory as an arc end
//! (`slashed`) and the reasons spelled by the core and by the frozen
//! `Names`; the path order (`rankFiles`, `rankDirs`) and the
//! directories' widths the core measures held to the frozen `ranks`,
//! `slashed` and `widths`.

use super::super::binder::ranks;
use super::super::kit::{Leg, leg};
use super::{Names, slashed, widths};
use crate::structure::oracle::tables::Tables;
use serde_json::json;

/// A face's tables: the paths, the directories (the root `""` first).
fn tables(leg: &mut Leg) -> Tables {
    let paths = leg.paths(0, 8);
    let mut dir_paths = vec![String::new()];
    dir_paths.extend(leg.paths(0, 6));
    let (files, dirs, edges, pkg_edges) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    Tables {
        files,
        paths,
        dirs,
        dir_paths,
        edges,
        pkg_edges,
        focus: Vec::new(),
    }
}

/// The tables the request carried as measures of its strings, as the
/// frozen face computed them.
fn measured(t: &Tables) -> serde_json::Value {
    let slashed: Vec<String> = t.dir_paths.iter().map(|d| slashed(d)).collect();
    let rank = ranks(t.paths.iter().chain(&slashed).map(String::as_str));
    let (by_file, by_dir) = rank.split_at(t.paths.len());
    let numbered = |r: &[usize]| {
        r.iter()
            .enumerate()
            .map(|(i, r)| [i, *r])
            .collect::<Vec<_>>()
    };
    json!({"rankFiles": numbered(by_file), "rankDirs": numbered(by_dir), "widths": widths(&t.dir_paths)})
}

leg!(
    arch_spells_and_measures_as_the_frozen_face,
    "arch",
    0x7711,
    |leg| {
        let t = tables(leg);
        let (why, texts) = leg.reasons();
        let sent = json!({"path": t.paths, "dir": t.dir_paths, "why": texts});
        let (nf, nd) = (t.paths.len(), t.dir_paths.len());
        let drawn = [
            ("path", vec![nf]),
            ("dir", vec![nd]),
            ("slashed", vec![nd]),
            ("why", vec![texts.len()]),
        ];
        let refs = leg.references(&drawn, 200);
        let want = measured(&t);
        let frozen = Names { t: &t, why };
        let rows = leg.spell("arch", (&sent, &json!({})), &frozen, refs);
        for table in ["rankFiles", "rankDirs", "widths"] {
            leg.table(&sent, &want[table], &rows[table]);
        }
    }
);
