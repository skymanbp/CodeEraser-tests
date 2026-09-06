//! The other half of the language gate: what the RENDERERS write.
//!
//! `docs_lang` cuts every generated block before it judges a page —
//! a block's language belongs to its renderer — and that leaves a hole
//! exactly the size of a renderer never given a language. The
//! benchmark renderers were three of them: the frozen evaluation
//! points went out as `17/17 scoped (100%)`,
//! `0/600 flagged (gate <= 1%)` and `overall gate >= 0.90 held` on
//! every Chinese surface, release after release, with every byte gate
//! green — each of those compares a page with the one generator that
//! wrote it, and a generator that never learned the language has
//! nothing to disagree with.
//!
//! So this leg reads the cut half. It is deliberately NOT a general
//! English detector: these blocks print identifiers, paths, corpus
//! and metric names that are English-shaped and correct in both
//! languages. It is a deny-list of the words the defect spelled.

use crate::common::repo_root;
use crate::docs_lang::SURFACES;

/// The words the benchmark renderers published into the Chinese
/// pages' generated blocks, plus the column head that went with them
/// and the dirty-tree suffix `measured` would have printed on the
/// first dirty row. A renderer reaching for one of these on a Chinese
/// page is refused by name.
///
/// A word here is a maximal run of `[A-Za-z0-9_]`, so an identifier
/// the tables legitimately print — the metric `guard_fpr_per500`, the
/// column `check_warm` — is one token and never a hit. Nothing is cut
/// first, deliberately: two of the three defect sites rendered inside
/// `<code>`, which `docs_lang`'s prose cut removes.
const DENY: [&str; 10] = [
    "flagged",
    "answered",
    "scoped",
    "held",
    "wrong",
    "attributed",
    "raw",
    "per",
    "percentile",
    "dirty",
];

/// Every generated block's body, concatenated — the complement of
/// `docs_lang::cut_generated`, and the half no language gate had read.
fn generated(text: &str) -> String {
    let (mut out, mut rest) = (String::new(), text);
    while let Some(b) = rest.find(":begin -->") {
        let body = &rest[b + ":begin -->".len()..];
        let Some(e) = body.find("<!-- ") else {
            break;
        };
        out.push_str(&body[..e]);
        out.push('\n');
        rest = &body[e..];
    }
    out
}

/// The deny-listed words a page's generated blocks print.
fn generated_english(rel: &str, text: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for (i, line) in generated(text).lines().enumerate() {
        for token in line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            if DENY.contains(&token.to_ascii_lowercase().as_str()) {
                errors.push(format!(
                    "{rel}: generated line {}: English prose in a generated \
                     Chinese block: `{token}`",
                    i + 1
                ));
            }
        }
    }
    errors
}

#[test]
fn no_generated_block_puts_english_on_a_chinese_page() {
    let root = repo_root();
    let errors: Vec<String> = SURFACES
        .iter()
        .filter(|(_, zh)| *zh)
        .flat_map(|(rel, _)| generated_english(rel, &crate::facts::read(&root, rel)))
        .collect();
    assert!(
        errors.is_empty(),
        "a generator speaking the wrong language:\n{}",
        errors.join("\n")
    );
}

/// Both halves, on the rendering that actually shipped: the block the
/// Chinese stack page carried is refused by name, the one that
/// replaced it passes, an identifier is one token, and the same words
/// OUTSIDE a generated block belong to `docs_lang`, not to this leg.
#[test]
fn the_generated_gate_refuses_the_english_that_shipped() {
    let shipped = "<!-- bench-stack:begin -->\n<div class=\"card\"><p>判定层：\
                   <code>0/600 flagged (gate &lt;= 1%)</code>；</p></div>\n\
                   <!-- bench-stack:end -->";
    let refused = generated_english("p.html", shipped);
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert!(refused[0].contains("flagged"), "{}", refused[0]);
    let fixed = shipped.replace(
        "0/600 flagged (gate &lt;= 1%)",
        "600 个样本命中 0 个（门 ≤ 1%）",
    );
    assert!(generated_english("p.html", &fixed).is_empty());
    assert!(generated_english("p.html", "<p>0/600 flagged</p>").is_empty());
    assert!(
        generated_english(
            "p.html",
            "<!-- x:begin -->\n<code>guard_fpr_per500</code>\n<!-- x:end -->"
        )
        .is_empty()
    );
}
