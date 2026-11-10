use super::newer;

/// v2.35 G7 (R3-12): a tag that does not read as a release version is
/// the unknown verdict 2, never "up to date"; a readable pair compares.
/// One row per line: `tag current verdict`.
const CASES: &str = "\
v2.0.0 1.9.2 1
v1.9.2 1.9.2 0
v1.9.1 1.9.2 0
v2.0.0-hotfix 1.9.2 2
v2.0.0 garbage 2";

#[test]
fn an_unreadable_tag_is_unknown_not_current() {
    for line in CASES.lines() {
        let [tag, current, want] = line.split(' ').collect::<Vec<_>>()[..] else {
            panic!("row: {line}")
        };
        assert_eq!(newer(tag, current).to_string(), want, "{line}");
    }
}
