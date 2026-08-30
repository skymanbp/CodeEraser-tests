//! The marked-block channel: `<!-- name:begin -->` … `<!-- name:end -->`
//! around a GENERATED region of an authored page — the bench
//! dashboard, the parity table, the site's bench strip. ONE splicer:
//! the three writers each carried their own split-compare-or-write
//! stanza (plan v2.21 S5), and a fourth would have been a fourth
//! copy. The page outside the block is authored and never touched;
//! the block is byte-compared on a plain run and rewritten under
//! CE_BLESS=1 through the one reader (facts::blessing).

use super::blessing;
use crate::common::repo_root;

/// Splice `rendered` into the `marker` block of the repo-relative
/// `rel`. Returns the drift note when the block is behind and no
/// bless ran; `None` when it is current or was just rewritten. The
/// page is CRLF-normalized for the compare (the site pages carry no
/// EOL attribute until S9), and a missing marker is a refusal.
pub fn splice(rel: &str, marker: &str, rendered: &str) -> Option<String> {
    let path = repo_root().join(rel);
    let page = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{rel}: {e}"))
        .replace("\r\n", "\n");
    let (begin, end) = (
        format!("<!-- {marker}:begin -->"),
        format!("<!-- {marker}:end -->"),
    );
    let (head, rest) = page
        .split_once(&begin)
        .unwrap_or_else(|| panic!("{rel}: no {begin}"));
    let (block, tail) = rest
        .split_once(&end)
        .unwrap_or_else(|| panic!("{rel}: no {end}"));
    let want = format!("\n{rendered}");
    if block == want {
        return None;
    }
    if blessing() {
        std::fs::write(&path, format!("{head}{begin}{want}{end}{tail}")).expect("bless block");
        return None;
    }
    Some(format!(
        "{rel}: {marker} block is behind its rendering (CE_BLESS=1 regenerates)"
    ))
}

/// `splice`, asserting.
pub fn assert_current(rel: &str, marker: &str, rendered: &str) {
    if let Some(note) = splice(rel, marker, rendered) {
        panic!("{note}");
    }
}
