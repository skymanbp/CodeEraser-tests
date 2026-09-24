use super::*;

/// A Go receiver prefix and a C++ class qualifier both strip; every
/// other spelling is its own base.
#[test]
fn a_qualified_name_strips_to_its_base() {
    let spelled = [
        "(*T) add",
        "(T) add",
        "add",
        "(anonymous)",
        "K::b",
        "Outer::Inner::m",
        "K::operator==",
    ];
    let bases: Vec<&str> = spelled.iter().map(|n| base_name(n)).collect();
    assert_eq!(
        bases,
        ["add", "add", "add", "(anonymous)", "b", "m", "operator=="]
    );
}
