//! Frozen W7 oracle (plan v2.33 W7; test-only): flow: cli/src/flow_report/face.rs, and unit_at (cli/src/flow_report/mod.rs) it calls, copied byte for
//! byte from 1324c927 (spans cli/src/flow_report/mod.rs 58-61; cli/src/flow_report/face.rs 205-212, 214-218, 220-242) when the core took over spelling the
//! strings. Only this header and the imports above the copy are new. Never
//! edited: the differential (cli/tests/unit/document/spelled/, mounted
//! here so it reads the copied items' private fields) holds the core to it.
#![allow(dead_code)]

use super::binder::{self as document, Resolve, Why};
use crate::flow::lower::{Lowered, Unit};

#[path = "../spelled/flow.rs"]
mod spelled;

/// The unit of `file` at `nth`.
pub fn unit_at(file: &Lowered, nth: usize) -> Option<&Unit> {
    file.units.iter().find(|u| u.nth == nth)
}

/// The flow document's strings: the paths, the unit names (a unit the
/// lowering left out by its own name; one the core refused that is no
/// lowered unit, empty), the variable names, the reasons.
struct Names<'a> {
    paths: &'a [String],
    files: &'a [Lowered],
    why: Why,
}

impl Names<'_> {
    fn file(&self, f: i128) -> Option<&Lowered> {
        usize::try_from(f).ok().and_then(|f| self.files.get(f))
    }
}

impl Resolve for Names<'_> {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        match (class, ints) {
            ("path", _) => document::at(self.paths, ints),
            ("why", _) => self.why.at(ints),
            ("unit", [f, nth]) => {
                let (file, nth) = (self.file(*f)?, usize::try_from(*nth).ok()?);
                let unit = unit_at(file, nth).map(|u| u.name.clone());
                let left = file
                    .unlowered
                    .iter()
                    .find(|u| u.nth == nth)
                    .map(|u| u.name.clone());
                Some(unit.or(left).unwrap_or_default())
            }
            ("var", [f, nth, v]) => {
                let unit = unit_at(self.file(*f)?, usize::try_from(*nth).ok()?)?;
                unit.legend.var_name.get(usize::try_from(*v).ok()?).cloned()
            }
            _ => None,
        }
    }
}
