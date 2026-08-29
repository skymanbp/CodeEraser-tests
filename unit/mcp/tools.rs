use super::*;
use crate::mcp::adapters::count;

/// The trend window has ONE default (trend::DEFAULT_COMMITS): this
/// face answered 10 while clap and the GUI answered 30, and the
/// tools/list prose said 10 too. Both halves are pinned here.
#[test]
fn trend_window_default_is_the_shared_one() {
    let d = crate::trend::DEFAULT_COMMITS;
    let row = TOOLS
        .iter()
        .find(|t| t.name == "trend")
        .and_then(|t| t.extra.iter().find(|(n, ..)| *n == "commits"))
        .expect("the trend tool declares a commits arg");
    assert!(row.2.contains(&d.to_string()), "tools/list says: {}", row.2);
    assert_eq!(count(&json!({}), "commits", d), d, "absent = the default");
    assert_eq!(
        count(&json!({"commits": 0}), "commits", d),
        d,
        "0 is no window"
    );
}
