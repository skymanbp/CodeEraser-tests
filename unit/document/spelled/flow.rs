//! The W7 differential's flow leg (plan v2.33 W7; mounted inside the
//! frozen flow face): the paths, the reasons, a unit by its file and
//! place (the first lowered unit there, else the first one left out,
//! else empty) and a unit's variable spelled by the core and by the
//! frozen `Names` / `unit_at`. Places repeat within a file now and
//! then, so "the first" is tested.

use super::super::kit::{Leg, leg};
use super::Names;
use crate::flow::lower::{Legend, Lowered, Unit, Unlowered};
use crate::scan::lang::Lang;
use serde_json::{Value, json};

fn unit(leg: &mut Leg) -> Unit {
    let var_name = leg.words(4);
    let legend = Legend {
        stmt_at: vec![],
        stmt_end: vec![],
        stmt_text: vec![],
        var_name,
        var_at: vec![],
    };
    let (stmts, vars, uses) = (Vec::new(), Vec::new(), Vec::new());
    let (start_line, end_line, params, dynamic) = (1, 1, 0, false);
    let (nth, name) = (leg.rng.below(5), leg.word());
    Unit {
        nth,
        name,
        start_line,
        end_line,
        params,
        dynamic,
        stmts,
        vars,
        uses,
        legend,
    }
}

fn file(leg: &mut Leg) -> Lowered {
    let units = (0..leg.rng.below(4)).map(|_| unit(leg)).collect();
    let left = |leg: &mut Leg| Unlowered {
        nth: leg.rng.below(5),
        name: leg.word(),
        start_line: 1,
        reason: String::new(),
    };
    let unlowered = (0..leg.rng.below(3)).map(|_| left(leg)).collect();
    Lowered {
        lang: Lang::Python,
        units,
        unlowered,
    }
}

/// Per file `[nth, …]` lists, as the face sends them.
fn per_file(files: &[Lowered]) -> [Value; 3] {
    let each = |pick: &dyn Fn(&Lowered) -> Vec<Value>| {
        files
            .iter()
            .map(|f| Value::Array(pick(f)))
            .collect::<Value>()
    };
    [
        each(&|f| f.units.iter().map(|u| json!([u.nth, u.name])).collect()),
        each(&|f| f.unlowered.iter().map(|u| json!([u.nth, u.name])).collect()),
        each(&|f| {
            (f.units.iter())
                .map(|u| json!([u.nth, u.legend.var_name]))
                .collect()
        }),
    ]
}

leg!(flow_spells_as_the_frozen_face, "flow", 0x7719, |leg| {
    let files = leg.many(4, file);
    let paths: Vec<String> = files.iter().map(|_| leg.path()).collect();
    let (why, texts) = leg.reasons();
    let [units, unlowered, vars] = per_file(&files);
    let sent =
        json!({"path": paths, "why": texts, "units": units, "unlowered": unlowered, "vars": vars});
    let n = files.len();
    let drawn = [
        ("path", vec![n]),
        ("why", vec![texts.len()]),
        ("unit", vec![n, 5]),
        ("var", vec![n, 5, 4]),
    ];
    let frozen = Names {
        paths: &paths,
        files: &files,
        why,
    };
    leg.spell_drawn("flow", &sent, &frozen, &drawn);
});
