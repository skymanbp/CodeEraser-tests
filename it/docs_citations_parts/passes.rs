//! The passes that resolve one page's citations against the ledger,
//! in page order, each ledger entry claimable once:
//!   1. by line — an unclaimed entry for the same target whose
//!      ledgered line equals the page's `#L`: the page was rendered
//!      from it. Its window still at that line → current; elsewhere
//!      once → moved; several times → ambiguous, the occurrence
//!      nearest the ledgered line taken and named; nowhere →
//!      vanished, the one hard red, red under bless too. Line first,
//!      window second: after an insertion above two adjacent cited
//!      lines, the second page pointer shows the FIRST entry's text,
//!      and pairing by window would hand it the wrong identity;
//!   2. by window — an unclaimed entry whose window sits at the
//!      page's `#L`: a pointer a human moved by hand to where the
//!      text now is (the ledger's line follows under bless);
//!   3. seed — no entry: a new window (`needs_human` when none forms;
//!      a blank or past-EOF anchor is refused here, where the pointer
//!      is the human's fresh statement), or a rename when an orphaned
//!      entry of a missing target carries the same text (named
//!      through CE_ALLOW_RENAME=1);
//!   4. orphans — entries no citation claimed: extra when their text
//!      still exists (a bless drops them), vanished when it does not
//!      (dropped only by name: CE_DROP_VANISHED=<hash16,…>).

use super::anchor::{self, Window};
use super::ledger::{Resolution, Resolved, first_line, hash16, shifted_end, stated_end};
use super::{Citation, Ledger, LedgerEntry, lines, target_path};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) struct Pass<'a> {
    root: &'a Path,
    ledger: &'a Ledger,
    claimed: BTreeSet<String>,
    targets: BTreeMap<String, Vec<String>>,
}

impl<'a> Pass<'a> {
    pub(super) fn new(root: &'a Path, ledger: &'a Ledger) -> Self {
        Pass {
            root,
            ledger,
            claimed: BTreeSet::new(),
            targets: BTreeMap::new(),
        }
    }

    fn target(&mut self, c: &Citation) -> Vec<String> {
        let path = target_path(self.root, &c.citing, &c.target);
        self.targets
            .entry(path.to_string_lossy().into_owned())
            .or_insert_with(|| lines(&path))
            .clone()
    }

    /// The unclaimed entries under a key prefix (`citing|` for a page,
    /// `citing|target|` for one of its targets), key order.
    fn unclaimed(&self, prefix: String) -> impl Iterator<Item = (&'a String, &'a LedgerEntry)> {
        self.ledger
            .range(prefix.clone()..)
            .take_while(move |(k, _)| k.starts_with(&prefix))
            .filter(|(k, _)| !self.claimed.contains(*k))
    }

    /// The first unclaimed entry of the citation's (citing, target)
    /// satisfying `pick`, claimed.
    fn claim(
        &mut self,
        c: &Citation,
        pick: impl Fn(&LedgerEntry) -> bool,
    ) -> Option<&'a LedgerEntry> {
        let (key, entry) = self
            .unclaimed(format!("{}|{}|", c.citing, c.target))
            .find(|(_, e)| pick(e))?;
        self.claimed.insert(key.clone());
        Some(entry)
    }

    pub(super) fn resolve_one(&mut self, c: &Citation, out: &mut Resolution) {
        if let Some(entry) = self.claim(c, |e| e.line == c.line) {
            self.by_line(c, entry, out);
        } else if let Some(r) = self.by_window(c) {
            out.resolved.push(Some(r));
        } else {
            self.seed(c, out);
        }
    }

    fn by_line(&mut self, c: &Citation, entry: &LedgerEntry, out: &mut Resolution) {
        let target = self.target(c);
        let found = anchor::occurrences(&target, &entry.text);
        let where_ = format!("{}:{}", c.citing, c.citing_line);
        if found.is_empty() {
            out.hard.push(format!(
                "{where_}: vanished: `{}` is no longer in {} — re-aim the citation, then name the drop: CE_DROP_VANISHED={}",
                first_line(&entry.text),
                c.target,
                hash16(&entry.text)
            ));
            out.resolved.push(None);
            return;
        }
        let unique = found.len() == 1;
        let (line, end) = if anchor::at(&target, &entry.text, entry.head, c.line) {
            (c.line, stated_end(c, c.line).or(shifted_end(entry, c.line)))
        } else {
            let (line, note) = relocate(entry, &found);
            out.soft.push(format!(
                "{where_}: {note}: L{} -> L{line} (CE_BLESS=1 re-renders)",
                entry.line
            ));
            (line, shifted_end(entry, line))
        };
        out.resolved.push(Some(Resolved {
            line,
            end,
            window: window_of(&target, entry, line, unique),
        }));
    }

    fn by_window(&mut self, c: &Citation) -> Option<Resolved> {
        let target = self.target(c);
        let entry = self.claim(c, |e| anchor::at(&target, &e.text, e.head, c.line))?;
        let unique = anchor::occurrences(&target, &entry.text).len() == 1;
        Some(Resolved {
            line: c.line,
            end: stated_end(c, c.line).or(shifted_end(entry, c.line)),
            window: window_of(&target, entry, c.line, unique),
        })
    }

    fn seed(&mut self, c: &Citation, out: &mut Resolution) {
        let target = self.target(c);
        let where_ = format!("{}:{}", c.citing, c.citing_line);
        let window = match target.get(c.line.wrapping_sub(1)) {
            None => Err(format!("L{} is past EOF (target has {} lines)", c.line, target.len())),
            Some(t) if t.trim().is_empty() => Err(format!("anchor L{} is blank — cite the item, not the gap", c.line)),
            Some(_) => anchor::seed(&target, c.line).ok_or_else(|| format!(
                "needs_human: no window around L{} weighs {} non-space characters and occurs once",
                c.line,
                anchor::MIN_CHARS
            )),
        };
        let window = match window {
            Ok(w) => w,
            Err(why) => {
                out.hard.push(format!("{where_}: {}: {why}", c.target));
                out.resolved.push(None);
                return;
            }
        };
        match self.renamed_from(c, &window.text) {
            Some((key, old)) if std::env::var_os("CE_ALLOW_RENAME").is_some() => {
                self.claimed.insert(key);
                out.soft.push(format!(
                    "{where_}: renamed: {old} -> {} (accepted by name)",
                    c.target
                ));
            }
            Some((_, old)) => {
                out.hard.push(format!(
                    "{where_}: renamed: {old} -> {} — name it: CE_BLESS=1 CE_ALLOW_RENAME=1",
                    c.target
                ));
                out.resolved.push(None);
                return;
            }
            None => out.soft.push(format!(
                "{where_}: missing ledger entry (CE_BLESS=1 seeds it)"
            )),
        }
        out.resolved.push(Some(Resolved {
            line: c.line,
            end: stated_end(c, c.line),
            window,
        }));
    }

    /// An orphaned entry of THIS page whose target file is gone and
    /// whose text is the seed's: the file was renamed under the link.
    fn renamed_from(&self, c: &Citation, text: &str) -> Option<(String, String)> {
        self.unclaimed(format!("{}|", c.citing))
            .filter(|(_, e)| e.text == text)
            .find(|(_, e)| !target_path(self.root, &c.citing, &e.target).is_file())
            .map(|(k, e)| (k.clone(), e.target.clone()))
    }

    pub(super) fn orphans(&mut self, out: &mut Resolution) {
        let drops = named_drops();
        for (key, entry) in self.ledger {
            if self.claimed.contains(key) {
                continue;
            }
            let citing = key.split('|').next().unwrap_or_default();
            let path = target_path(self.root, citing, &entry.target);
            let alive =
                path.is_file() && !anchor::occurrences(&lines(&path), &entry.text).is_empty();
            let hash = hash16(&entry.text);
            if alive {
                out.soft.push(format!(
                    "{key}: extra ledger entry, no citation claims it (CE_BLESS=1 drops it)"
                ));
            } else if drops.contains(&hash) {
                out.soft.push(format!("{key}: vanished, dropped by name"));
            } else {
                out.hard.push(format!(
                    "{key}: vanished: `{}` is no longer in {} — re-aim its citation, then name the drop: CE_DROP_VANISHED={hash}",
                    first_line(&entry.text),
                    entry.target
                ));
            }
        }
    }
}

/// Moved (one occurrence) or ambiguous (several: the one nearest the
/// ledgered line, named as such) — the cited line and the note.
fn relocate(entry: &LedgerEntry, found: &[usize]) -> (usize, String) {
    if let [one] = found {
        return (one + entry.head, "moved".to_string());
    }
    let nearest = found
        .iter()
        .map(|l| l + entry.head)
        .min_by_key(|l| l.abs_diff(entry.line))
        .expect("non-empty");
    let note = format!(
        "ambiguous ({} occurrences, took the one nearest the ledgered line)",
        found.len()
    );
    (nearest, note)
}

/// The entry's window while it still occurs once; a fresh minimal
/// seed at the resolved line once it does not (falling back to the
/// entry's when even that cannot form).
fn window_of(target: &[String], entry: &LedgerEntry, line: usize, unique: bool) -> Window {
    let kept = Window {
        text: entry.text.clone(),
        head: entry.head,
    };
    if unique {
        return kept;
    }
    anchor::seed(target, line).unwrap_or(kept)
}

fn named_drops() -> BTreeSet<String> {
    std::env::var("CE_DROP_VANISHED")
        .map(|v| v.split(',').map(str::trim).map(String::from).collect())
        .unwrap_or_default()
}
