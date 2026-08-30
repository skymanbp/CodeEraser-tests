//! The derived-fact registry (plan v2.21, ADR-009): every fact a
//! document carries that moves with a release — versions, revisions,
//! schema ids, gate floors, toolchain pins — is resolved here once and
//! rendered everywhere. Ids read `family:key#form`; the form names the
//! value grammar and the token class a chip span is read with.
//!
//! Two tiers. A LINKED fact is read through a typed product path (a
//! `pub const`, the config loader, the manifest parser). A SCRAPED
//! fact is read from a file by a grammar this module owns and carries
//! the debt line naming what promotes it; the scraped count is a
//! ratchet (facts_registry.rs) that only goes down.
//!
//! "Bless writes; bless never decides." A regeneration run rewrites a
//! generated surface from its source; a plain run byte-compares. The
//! switch used to be read at six sites in two spellings, and nothing
//! stopped a CI leg from being handed CE_BLESS=1 and turning every
//! compare into a write. Now the sites call `blessing()`, the switch
//! is exactly "1" (any-value `is_ok()` let CE_BLESS=0 or an empty var
//! bless-and-pass — attack-review finding), and a run that is BOTH
//! blessing and on CI refuses by name. bless_guard.rs holds the census
//! (one reader, and the workflows never spell the switch).

pub mod chip;
pub mod gate;
pub mod report;
pub mod ver;

use std::collections::BTreeSet;
use std::sync::OnceLock;

/// The projection's own schema id (contracts/docs-facts.json).
pub const PROJECTION_SCHEMA: &str = "ce.docs-facts/0.1.0";

/// CE_BLESS=1 — exactly "1". Under CI (the `CI` variable every hosted
/// runner exports) a bless is a category error, not a mode: CI only
/// compares, and a write there would pass a drift by regenerating it.
pub fn blessing() -> bool {
    let on = std::env::var("CE_BLESS").as_deref() == Ok("1");
    if on && std::env::var_os("CI").is_some() {
        panic!(
            "CE_BLESS=1 under CI: a bless writes locally and CI only compares (plan v2.21 gate 1)"
        );
    }
    on
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Linked,
    Scraped,
}

/// One registered fact: id, rendered value, tier, the owning source
/// (`path::NAME` for a const) and — scraped only — the debt line.
#[derive(Clone, Debug)]
pub struct Fact {
    pub id: String,
    pub value: String,
    pub tier: Tier,
    pub source: String,
    pub debt: Option<String>,
}

pub fn linked(id: &str, value: impl std::fmt::Display, source: &str) -> Fact {
    Fact {
        id: id.into(),
        value: value.to_string(),
        tier: Tier::Linked,
        source: source.into(),
        debt: None,
    }
}

pub fn scraped(id: &str, value: impl std::fmt::Display, source: &str, debt: &str) -> Fact {
    Fact {
        id: id.into(),
        value: value.to_string(),
        tier: Tier::Scraped,
        source: source.into(),
        debt: Some(debt.into()),
    }
}

/// The form suffix of an id: `#v` three-part SemVer, `#vminor` its
/// major.minor, `#digits` a bare integer, `#schemaver` a report id
/// `ce.<name>/<ver>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    V,
    VMinor,
    Digits,
    SchemaVer,
}

impl Form {
    pub fn of(id: &str) -> Form {
        match id.rsplit_once('#') {
            Some((_, "v")) => Form::V,
            Some((_, "vminor")) => Form::VMinor,
            Some((_, "digits")) => Form::Digits,
            Some((_, "schemaver")) => Form::SchemaVer,
            _ => panic!("fact id {id:?}: no known #form suffix"),
        }
    }

    /// Is `c` part of a token of this form? A chip span is read as
    /// maximal runs of these characters (chip.rs).
    pub fn token_char(self, c: char) -> bool {
        match self {
            Form::V | Form::VMinor => c.is_ascii_digit() || c == '.',
            Form::Digits => c.is_ascii_digit(),
            Form::SchemaVer => c.is_ascii_alphanumeric() || matches!(c, '.' | '/' | '-'),
        }
    }

    /// Does `value` have this form's shape?
    pub fn admits(self, value: &str) -> bool {
        match self {
            Form::V => dotted(value, 3),
            Form::VMinor => dotted(value, 2),
            Form::Digits => dotted(value, 1),
            Form::SchemaVer => value
                .strip_prefix("ce.")
                .and_then(|rest| rest.split_once('/'))
                .is_some_and(|(name, ver)| {
                    !name.is_empty()
                        && name
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                        && (dotted(ver, 1) || dotted(ver, 3))
                }),
        }
    }
}

/// `parts` non-empty all-digit groups joined by `.`.
fn dotted(value: &str, parts: usize) -> bool {
    let groups: Vec<&str> = value.split('.').collect();
    groups.len() == parts
        && groups
            .iter()
            .all(|g| !g.is_empty() && g.bytes().all(|b| b.is_ascii_digit()))
}

/// Every fact, id-sorted; ids unique, each value admitted by its
/// form, debt present exactly on the scraped tier. Resolved once per
/// process — the scrapers read files.
pub fn registry() -> &'static [Fact] {
    static ALL: OnceLock<Vec<Fact>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut all = ver::facts();
        all.extend(report::facts(&crate::common::repo_root()));
        all.extend(gate::facts());
        all.sort_by(|a, b| a.id.cmp(&b.id));
        let mut seen = BTreeSet::new();
        for f in &all {
            assert!(seen.insert(f.id.clone()), "duplicate fact id {}", f.id);
            assert!(
                Form::of(&f.id).admits(&f.value),
                "fact {} value {:?} is not shaped like its form",
                f.id,
                f.value
            );
            assert_eq!(
                f.tier == Tier::Scraped,
                f.debt.is_some(),
                "fact {}: debt rides the scraped tier only",
                f.id
            );
        }
        all
    })
}

/// The rendered value of one id; an unknown id is a refusal, never
/// an empty string a document could carry.
pub fn resolve(id: &str) -> String {
    registry()
        .iter()
        .find(|f| f.id == id)
        .map(|f| f.value.clone())
        .unwrap_or_else(|| panic!("unknown fact id {id:?}"))
}

pub fn scraped_count() -> usize {
    registry()
        .iter()
        .filter(|f| f.tier == Tier::Scraped)
        .count()
}

/// The projection document (contracts/docs-facts.json): one row per
/// fact plus the scraped count, pretty JSON, LF.
pub fn projection() -> String {
    use serde_json::json;
    let rows: Vec<serde_json::Value> = registry()
        .iter()
        .map(|f| {
            let tier = match f.tier {
                Tier::Linked => "linked",
                Tier::Scraped => "scraped",
            };
            let mut row = json!({"id": f.id, "value": f.value, "tier": tier, "source": f.source});
            if let Some(debt) = &f.debt {
                row["debt"] = json!(debt);
            }
            row
        })
        .collect();
    let doc = json!({"schema": PROJECTION_SCHEMA, "facts": rows, "scraped": scraped_count()});
    serde_json::to_string_pretty(&doc).expect("projection json")
}
