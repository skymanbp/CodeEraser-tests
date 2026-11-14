use super::*;

/// The spellings a face may receive for one file all come out as the
/// index's: relative, `./`-headed (once or with a doubled slash),
/// backslashed, and absolute under the root.
#[test]
fn the_spellings_of_one_file_name_it_once() {
    let root = std::env::temp_dir().join("ce-typed-root");
    let abs = root.join("src").join("a.rs").display().to_string();
    for spelt in [
        "src/a.rs",
        "./src/a.rs",
        ".//src/a.rs",
        "src\\a.rs",
        "src/./a.rs",
        abs.as_str(),
    ] {
        assert_eq!(typed(&root, spelt), "src/a.rs", "{spelt}");
    }
}

/// A directory's own forms: a trailing slash drops, `.` and `./` are the
/// root (empty); a parent step and a path outside the root stay as typed,
/// slashed.
#[test]
fn the_root_the_parent_and_the_outside_stay_what_they_are() {
    let root = std::env::temp_dir().join("ce-typed-root");
    assert_eq!(typed(&root, "lib/"), "lib");
    assert_eq!(
        (typed(&root, "."), typed(&root, "./")),
        (String::new(), String::new())
    );
    assert_eq!(typed(&root, "../x.rs"), "../x.rs");
    let outside = std::env::temp_dir().join("elsewhere").join("x.rs");
    let want = outside.display().to_string().replace('\\', "/");
    assert_eq!(typed(&root, &outside.display().to_string()), want);
}
