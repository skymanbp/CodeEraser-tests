//! The Markdown rungs' string readings (plan v2.33 W2-text stage G),
//! asked of the core directly: Rust's lowercase over every scalar value
//! (each character alone, and a capital sigma after it with and without a
//! cased letter before — the two contexts Final_Sigma reads), the
//! reference label fold, and a link target's scheme test with its
//! percent-decoding — each against the frozen 3b7234eb reading
//! (ladder::frozen) or the standard library it called.

use super::text::ranges;
use super::{agree, check, link, questions};
use crate::graph::ladder::frozen;
use crate::graph::ladder::frozen::percent_decode;
use serde_json::{Value, json};

/// Whether a capital sigma after `lead` and a character lowers final.
fn sigma(lead: &str, c: char) -> bool {
    format!("{lead}{c}\u{3a3}")
        .to_lowercase()
        .ends_with('\u{3c2}')
}

#[test]
#[ignore = "needs a core: the differential gate"]
fn lowercase_agrees_on_every_scalar_value() {
    let map: Vec<(u32, String)> = (0..=0x10FFFF_u32)
        .filter_map(char::from_u32)
        .filter_map(|c| {
            let l: String = c.to_lowercase().collect();
            (l != c.to_string()).then_some((c as u32, l))
        })
        .collect();
    let want = json!({
        "map": map,
        "final": ranges(|c| sigma("", c)),
        "after": ranges(|c| sigma("A", c)),
    });
    let body = json!({ "inspect": { "lower": true } });
    let reply = crate::corelink::judged::ask(&mut link(), "resolve/1", "9.0.0", "resolve", body)
        .expect("core");
    agree(
        "lower",
        &[json!("every scalar value")],
        &[want],
        std::slice::from_ref(&reply["inspected"]["lower"]),
    );
}

/// A reader leg over drawn texts: `count()` of them, up to `max`
/// characters of `alphabet` each, asked of the core under `key` and of
/// the frozen reading.
fn texts_agree(seed: u64, (key, alphabet, max): (&str, &str, usize), oracle: fn(&str) -> Value) {
    let alphabet: Vec<char> = alphabet.chars().collect();
    let qs = questions(seed, |d, _| json!(d.text(&alphabet, max)));
    check(key, &qs, |q| oracle(q.as_str().expect("text")));
}

/// What a label is drawn from: capital, small and final sigmas, case-
/// ignorable marks (apostrophe, period, colon, combining acute, soft
/// hyphen, zero-width space), cased letters with odd mappings (dotted
/// capital I, a titlecase digraph, sharp s and its capital, Cherokee), and
/// spaces of several kinds.
const LABEL: &str = "ΣΣσςΑαAa'.:\u{301}\u{ad}\u{200b}İǅßẞᏣꭰ \t\u{a0}\u{3000}\u{85}x1ΟΔ";

#[test]
#[ignore = "needs a core: the differential gate"]
fn label_fold_agrees() {
    texts_agree(5, ("fold", LABEL, 12), |t| json!(frozen::fold(t)));
}

/// What a target is drawn from: escapes good and ill-formed (a `+`
/// sign, non-hexadecimal, multi-byte), scheme characters, separators.
const TARGET: &str = "%%%0123456789aAfFgG+-.:/#/éx\u{1f389}C";

#[test]
#[ignore = "needs a core: the differential gate"]
fn target_readings_agree() {
    texts_agree(6, ("url", TARGET, 14), |t| {
        json!([frozen::is_scheme(t), percent_decode(t)])
    });
}
