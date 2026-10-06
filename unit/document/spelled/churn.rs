//! The W7 differential's churn leg (plan v2.33 W7; mounted inside the
//! frozen churn face): the pairs' paths and the submodules spelled by
//! the core and by the frozen `Names`.

use super::super::kit::leg;
use super::Names;
use serde_json::json;

leg!(churn_spells_as_the_frozen_face, "churn", 0x7712, |leg| {
    let (paths, submodules) = (leg.paths(0, 8), leg.words(4));
    let sent = json!({"path": paths, "submodule": submodules});
    let drawn = [
        ("path", vec![paths.len()]),
        ("submodule", vec![submodules.len()]),
    ];
    let frozen = Names {
        paths,
        submodules: &submodules,
    };
    leg.spell_drawn("churn", &sent, &frozen, &drawn);
});
