use super::*;

/// The window against trend/2's row ceiling (plan v2.33 step 11b item
/// 5): rows and the declared knob rows count together, so the room is
/// the cap less the knobs; at the room the window is sent, one past it
/// is refused by name with the room — before a commit is measured.
#[test]
fn a_window_past_the_cap_less_its_knobs_is_refused_by_name() {
    window_within(4094, 2, 4096).expect("at the room");
    window_within(4096, 0, 4096).expect("no knobs: the whole cap");
    let why = window_within(4095, 2, 4096).expect_err("one past the room");
    assert_eq!(
        why.to_string(),
        "trend: --commits 4095 over the core's trend_rows cap 4096 (2 declared knob row(s) ride \
         with the rows); ask for at most 4094"
    );
}
