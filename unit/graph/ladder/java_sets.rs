//! java_sets.rs: the standard layout's source root and set, and the
//! one visibility rule read from it — a main set sees main sets only.

use super::{own_root_dir, source_root, visible};

/// `root <path> <root> <set>` rows ask the layout (`- -` outside it);
/// `sees <from> <candidate> yes|no` rows ask the visibility rule —
/// main sees another module's main but no test code anywhere; a test
/// set sees main and test alike; a file outside the layout sees
/// everything and is seen by everything.
const CASES: &str = "
root src/main/java/a/B.java src/main/java main
root src/test/java/a/BTest.java src/test/java test
root lib/src/integrationTest/java/a/C.java lib/src/integrationTest/java integrationTest
root a/b/src/main/java/C.java a/b/src/main/java main
root src/main/java/a/src/x/java/D.java src/main/java main
root src/a/B.java - -
root src/main/java - -
root src/main/kotlin/a/B.java - -
sees src/main/java/a/A.java src/main/java/b/B.java yes
sees src/main/java/a/A.java lib/src/main/java/b/B.java yes
sees src/main/java/a/A.java src/test/java/a/T.java no
sees src/main/java/a/A.java lib/src/test/java/a/T.java no
sees src/main/java/a/A.java src/testFixtures/java/a/F.java no
sees src/test/java/a/T.java src/main/java/a/A.java yes
sees src/test/java/a/T.java src/testFixtures/java/a/F.java yes
sees gen/a/G.java src/test/java/a/T.java yes
sees src/main/java/a/A.java gen/a/G.java yes
";

#[test]
fn the_layout_names_root_and_set_and_main_sees_main_only() {
    for line in CASES.trim().lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        match w[0] {
            "root" => assert_eq!(
                source_root(w[1]).unwrap_or(("-", "-")),
                (w[2], w[3]),
                "{line}"
            ),
            _ => assert_eq!(visible(w[1], w[2]), w[3] == "yes", "{line}"),
        }
    }
}

/// A split package answers the importing file's own part, only when
/// exactly one directory lies in its source root.
#[test]
fn the_own_root_part_of_a_split_package() {
    let dirs: Vec<String> = ["src/main/java/a", "src/test/java/a"]
        .map(String::from)
        .to_vec();
    let from_test = own_root_dir("src/test/java/b/T.java", dirs.iter());
    assert_eq!(from_test.as_deref(), Some("src/test/java/a"));
    assert_eq!(own_root_dir("gen/b/G.java", dirs.iter()), None);
    let both = ["src/test/java/a", "src/test/java/x/a"].map(String::from);
    assert_eq!(own_root_dir("src/test/java/b/T.java", both.iter()), None);
}
