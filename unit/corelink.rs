/// An explicit --core path travels verbatim; the untouched
/// default consults the ONE resolver the daemon and MCP already
/// use — equality against core_bin pins the single-authority
/// property without poking the process environment (the e2e
/// suite owns CE_CORE_BIN; mutating it here would race them).
#[test]
fn explicit_core_wins_and_default_routes_through_the_one_resolver() {
    assert_eq!(super::resolve_core("x/custom/core"), "x/custom/core");
    assert_eq!(
        super::resolve_core("ce-core"),
        crate::daemon::judge::core_bin().expect("resolver always answers")
    );
}
