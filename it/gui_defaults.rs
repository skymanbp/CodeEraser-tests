//! The GUI's default history windows, held to the constants that own
//! them: gui/ui/index.html writes the trend window (`trend-commits`) and
//! the churn window (`cand-days`, `hub-days`) as `value=` attributes that
//! cannot read Rust, and the MCP face once drifted on exactly this.

use crate::common::repo_root;

/// The `value=` of the input whose id is `id`.
fn value_of(html: &str, id: &str) -> String {
    let tag = html
        .split(&format!("id=\"{id}\""))
        .nth(1)
        .and_then(|t| t.split('>').next())
        .unwrap_or_else(|| panic!("index.html has no input {id}"));
    let v = tag.split("value=\"").nth(1).expect("a value attribute");
    v.split('"').next().unwrap_or("").to_string()
}

#[test]
fn the_gui_windows_default_to_the_owning_constants() {
    let html = std::fs::read_to_string(repo_root().join("gui/ui/index.html")).expect("index.html");
    let commits = codeeraser::trend::DEFAULT_COMMITS.to_string();
    let days = codeeraser::churn::DEFAULT_DAYS.to_string();
    let got = ["trend-commits", "cand-days", "hub-days"].map(|id| value_of(&html, id));
    assert_eq!(got, [commits, days.clone(), days]);
}
