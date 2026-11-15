//! What one leg saw (common.rs drives the legs): per language the sites
//! compared and the outcome classes, the mismatches, and — for a leg with
//! a named divergence between the core and the frozen rungs — the sites
//! that divergence's mark covers, whose mismatches are allowed.

use crate::graph::ladder::{Scope, Site};
use std::collections::BTreeMap;

/// A named divergence's mark: true on a site where the core and the
/// frozen rungs may disagree by a ruled change (computed without the core).
pub(super) type Mark = fn(&Site, &Scope) -> bool;

/// What one leg saw: per language, the sites compared, the outcome
/// classes both sides gave, and the mismatches (first ones kept whole).
#[derive(Default)]
pub(super) struct Tally {
    /// A named divergence's mark, when the leg has one.
    pub(super) mark: Option<Mark>,
    /// Sites the mark covers, and those of them that mismatched.
    pub(super) marked: (usize, usize),
    sites: BTreeMap<&'static str, usize>,
    classes: BTreeMap<(&'static str, String), usize>,
    pub(super) forced: (usize, usize),
    pub(super) packages: (usize, usize),
    pub(super) mains: (usize, usize),
    pub(super) crates: (usize, usize),
    pub(super) private: (usize, usize),
    mismatches: usize,
    shown: Vec<String>,
}

impl Tally {
    pub(super) fn report(&self, leg: &str) {
        println!("== ladder_diff {leg} ==");
        for (lang, n) in &self.sites {
            println!("{lang}: sites {n}");
        }
        for ((lang, class), n) in &self.classes {
            println!("  {lang} {class}: {n}");
        }
        let (trees, arcs) = self.forced;
        println!("forced arcs: trees {trees} arcs {arcs}");
        let (trees, packages) = self.packages;
        println!("R package code: trees {trees} packages {packages}");
        let (trees, mains) = self.mains;
        println!("cabal mains: trees {trees} mains {mains}");
        let (trees, crates) = self.crates;
        println!("cargo crate roots: trees {trees} roots {crates}");
        let (files, private) = self.private;
        println!("cabal and cargo privacy: files {files} private {private}");
        if self.mark.is_some() {
            let (marked, missed) = self.marked;
            println!(
                "marked {marked} mismatched {missed} unmarked-mismatched {}",
                self.mismatches
            );
        }
        println!("mismatches: {}", self.mismatches);
        self.shown.iter().for_each(|m| println!("MISMATCH {m}"));
        assert_eq!(self.mismatches, 0, "{leg}: core and frozen oracle disagree");
    }

    /// One compared site of `lang`, with the frozen side's class.
    pub(super) fn count(&mut self, lang: &'static str, class: String) {
        *self.sites.entry(lang).or_default() += 1;
        *self.classes.entry((lang, class)).or_default() += 1;
    }

    /// Whether `site`'s outcome (`differs`: the two sides disagree) is a
    /// mismatch to report: a disagreement on a site the leg's named
    /// divergence's mark covers is counted apart and not reported.
    pub(super) fn unexplained(&mut self, site: &Site, oracle: &Scope, differs: bool) -> bool {
        let marked = self.mark.is_some_and(|m| m(site, oracle));
        self.marked.0 += usize::from(marked);
        self.marked.1 += usize::from(marked && differs);
        differs && !marked
    }

    pub(super) fn sites(&self) -> usize {
        self.sites.values().sum()
    }

    pub(super) fn miss(&mut self, what: String) {
        self.mismatches += 1;
        if self.shown.len() < 25 {
            self.shown.push(what);
        }
    }

    /// One comparison: a disagreement is spelled with both sides.
    pub(super) fn agree<T: PartialEq + std::fmt::Debug>(
        &mut self,
        what: &str,
        got: &T,
        want: &T,
        ctx: &dyn Fn() -> String,
    ) {
        if got != want {
            self.miss(format!("{what}: core {got:?} oracle {want:?}\n{}", ctx()));
        }
    }
}
