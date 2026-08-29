use super::*;
use crate::testutil::{scratch, write_tree};

/// Every counter of one language on one small tree, and the
/// collision blindness told apart from a real reference: `twin`
/// is declared in two files and spelled by nothing else — saved,
/// and counted as saved by collision; `used` is declared twice
/// too but a third file references it, so it is not; `lonely` is
/// exported and unmentioned; `hidden` is unmentioned and not
/// exported; `self_fenced` is vetoed by its own doc fence.
#[test]
fn the_census_counts_the_veto_per_language_and_names_the_collisions() {
    let root = scratch("mention-rates");
    write_tree(
        &root,
        &[
            ("Cargo.toml", "[package]\nname = \"fx\"\n"),
            (
                "src/lib.rs",
                "pub fn twin() {}\npub fn used() {}\npub fn lonely() {}\nfn hidden() {}\n\
                     /// ```\n/// self_fenced();\n/// ```\npub fn self_fenced() {}\n",
            ),
            ("src/other.rs", "pub fn twin() {}\npub fn used() {}\n"),
            ("notes.md", "call used() here\n"),
        ],
    );
    let (idx, _db) = crate::dedup::refreshed_index(&root, None).expect("index");
    super::super::refresh(&root, &idx).expect("mention pass");
    let got = census(&root, &idx).expect("census");
    let rust = &got["rust"];
    assert_eq!(
        *rust,
        LangRates {
            declared: Split {
                all: 7,
                exported: 6
            },
            unmentioned: Split {
                all: 2,
                exported: 1
            },
            vetoed: Vetoed {
                other: 4,
                fold: 0,
                self_text: 1,
                collision_saved: 2,
            },
        },
        "{got:?}"
    );
    assert_eq!(got.len(), 1, "markdown declares nothing in the domain");
    std::fs::remove_dir_all(&root).ok();
}
