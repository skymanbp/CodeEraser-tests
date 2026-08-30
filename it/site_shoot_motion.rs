//! The shoot is reproducible only because motion is off.
//!
//! Every shot in `scripts/shoot_gui.js` clicks a button, and
//! `gui/ui/style.css` gives buttons a 0.12s background transition. A
//! capture two animation frames after the click therefore landed at an
//! arbitrary point along that interpolation: three runs over ONE saved
//! report set produced three different digests for `gui-tree.png`, and
//! two for `gui-candidates.png`. Non-determinism there is not cosmetic
//! — it destroys the only way to ask whether a committed picture is
//! current, since a re-shoot always differs and the difference never
//! means anything.
//!
//! The remedy is the app's own: it already answers
//! `prefers-reduced-motion`, so the harness declares it and the
//! transition never starts. That makes this a PAIR — a CSS rule and a
//! CDP call, each inert without the other, in two files that no one
//! edits together. This leg is what notices when one half leaves.

use crate::common::repo_root;

/// The media feature both halves name.
const FEATURE: &str = "prefers-reduced-motion";

#[test]
fn the_shoot_turns_off_the_motion_the_app_can_turn_off() {
    let root = repo_root();
    let css = std::fs::read_to_string(root.join("gui/ui/style.css")).expect("style.css");
    let shoot = std::fs::read_to_string(root.join("scripts/shoot_gui.js")).expect("shoot_gui.js");

    let block = css
        .split_once(&format!("@media ({FEATURE}: reduce)"))
        .map(|(_, rest)| rest.split_once('}').expect("an open media block").0)
        .unwrap_or_else(|| {
            panic!("gui/ui/style.css no longer answers {FEATURE}; the shoot's captures go back to racing a 0.12s transition")
        });
    assert!(
        block.contains("button") && block.contains("transition: none"),
        "the {FEATURE} block must still stop the button transition every shot clicks through, not something else: {block:?}"
    );
    assert!(
        shoot.contains("setEmulatedMedia") && shoot.contains(FEATURE),
        "scripts/shoot_gui.js must emulate {FEATURE} before it navigates, or the CSS rule above is never reached and the pictures stop reproducing"
    );
}
