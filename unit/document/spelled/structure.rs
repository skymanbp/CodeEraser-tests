//! The W7 differential's structure leg (plan v2.33 W7; mounted inside
//! the frozen structure document): the directory names and — when the
//! split advisory rode — its files' paths and units spelled by the core
//! and by the frozen `Names`.

use super::super::kit::{Leg, leg};
use super::Names;
use crate::structure::seams::SeamFacts;
use serde_json::{Value, json};

/// The advisory's files and units, or none.
fn seams(leg: &mut Leg) -> Option<SeamFacts> {
    leg.rng.chance(70).then(|| {
        let mut sf = SeamFacts::default();
        for _ in 0..leg.rng.below(5) {
            sf.files.push((leg.path(), 0));
            let units = (0..leg.rng.below(4)).map(|_| (leg.word(), 0)).collect();
            sf.unit_names.push(units);
        }
        sf
    })
}

/// The strings as the face sends them, and the reference classes with
/// their list sizes (a path or a unit is drawn even without the
/// advisory, where both sides spell none).
fn face(dirs: &[String], sf: Option<&SeamFacts>) -> (Value, Vec<(&'static str, Vec<usize>)>) {
    let mut sent = json!({"dir": dirs});
    let (files, widest) = sf.map_or((2, 2), |sf| {
        let names = |u: &Vec<(String, u64)>| u.iter().map(|x| json!(x.0)).collect::<Value>();
        sent["path"] = sf.files.iter().map(|f| json!(f.0)).collect();
        sent["unit"] = sf.unit_names.iter().map(names).collect();
        let widest = sf.unit_names.iter().map(Vec::len).max().unwrap_or(0);
        (sf.files.len(), widest)
    });
    let drawn = vec![
        ("unit", vec![files, widest]),
        ("dir", vec![dirs.len()]),
        ("path", vec![files]),
    ];
    (sent, drawn)
}

leg!(
    structure_spells_as_the_frozen_document,
    "structure",
    0x7717,
    |leg| {
        let dirs = leg.paths(1, 6);
        let sf = seams(leg);
        let (sent, drawn) = face(&dirs, sf.as_ref());
        let frozen = Names {
            dirs,
            seams: sf.as_ref(),
        };
        leg.spell_drawn("structure", &sent, &frozen, &drawn);
    }
);
