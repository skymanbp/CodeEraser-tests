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

/// One core session per process (plan v2.33 W2-text Z1): a link dropped
/// healthy parks its session and the next `open` of that core takes it
/// back — a refusal leaves it whole; an `own` link neither takes nor
/// parks. On a private copy of the core, so no other test of this
/// process shares the slot's key.
#[test]
fn a_dropped_link_parks_its_session_for_the_next_open() {
    let real = std::env::var("CE_CORE_BIN").expect("the unit tests run with CE_CORE_BIN");
    let dir = crate::testutil::scratch("z1-slot");
    let copy = dir.join(if cfg!(windows) {
        "ce-core.exe"
    } else {
        "ce-core"
    });
    std::fs::copy(&real, &copy).expect("copy the core");
    let core = copy.display().to_string();
    let open = || super::Link::open(&core).expect("the copy answers").0;
    let first = open();
    let pid = first.pid();
    drop(first);
    let mut again = open();
    assert_eq!(again.pid(), pid, "the parked session is taken back");
    let refused = again.request(
        "bags",
        serde_json::json!({ "units": [["k", "fn", 2, [], [], [], []]] }),
    );
    assert!(
        refused.is_err_and(|e| e.contains("refused")),
        "a refusal, named"
    );
    drop(again);
    let own = super::Link::own(&core).expect("own").0;
    assert_ne!(own.pid(), pid, "an own link spawns its own core");
    drop(own);
    assert_eq!(
        open().pid(),
        pid,
        "the refusal kept it; the own link never parked"
    );
}
