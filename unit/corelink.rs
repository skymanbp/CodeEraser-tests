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
/// parks. On a private core of its own, so no other test of this
/// process shares the slot's key.
#[test]
fn a_dropped_link_parks_its_session_for_the_next_open() {
    let core = private_core().display().to_string();
    let open = || super::Link::open(&core).expect("the core answers").0;
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

/// The real core under a path of its own — the slot key is the canonical
/// path, so a hard link is its own slot. A link and not a copy: Linux
/// refuses to exec an image any process holds open for writing
/// (ETXTBSY), and a child another test thread forks mid-copy inherits
/// the copy's write fd until its own exec. Into the scratch dir, else
/// (temp on another volume) beside the real core; never a copy.
fn private_core() -> std::path::PathBuf {
    let real = std::path::PathBuf::from(
        std::env::var("CE_CORE_BIN").expect("the unit tests run with CE_CORE_BIN"),
    );
    let name = real.file_name().expect("CE_CORE_BIN names a file");
    let link = crate::testutil::scratch("z1-slot").join(name);
    if std::fs::hard_link(&real, &link).is_ok() {
        return link;
    }
    let beside = real
        .parent()
        .expect("the core sits in a directory")
        .join(format!("ce-z1-slot-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&beside);
    std::fs::create_dir_all(&beside).expect("mkdir beside the core");
    let link = beside.join(name);
    std::fs::hard_link(&real, &link).expect("link the core beside itself");
    link
}

/// A line over the core's byte ceiling (plan v2.33 legacy item 1). A
/// whole pass never sends it: it meets the named degradation a family
/// cap gives. Sent anyway, the core refuses it before decoding, so its
/// `too_large` echoes no id — a named refusal, not a desync: the session
/// is not broken and the next request is answered under its own id.
#[test]
fn an_overlong_line_is_refused_by_name_and_the_session_survives() {
    let mut link = own_link();
    let over =
        || serde_json::json!({ "pad": "x".repeat(crate::tables::get().limits.caps.line_bytes) });
    let pass = super::judged::whole_pass::<serde_json::Value>(
        &mut link,
        ("tables/1", "tables"),
        over(),
        ("probe pass", &[]),
    );
    let held = pass.expect_err("over the line").to_string();
    assert!(
        held.starts_with("tables/1 degraded the probe pass (request line "),
        "{held}"
    );
    assert!(held.ends_with("line the core reads; not sent)"), "{held}");
    let sent = link.request("tables", over()).expect_err("over the line");
    assert!(
        sent.starts_with("core refused tables.request: too_large: "),
        "{sent}"
    );
    assert!(!link.is_broken(), "the framing held");
    let next = link.request("tables", serde_json::json!({}));
    assert!(next.is_ok(), "the next request is answered: {next:?}");
}

/// The other side of `is_broken` (plan v2.33 step 11b item 1): a core
/// that is gone mid-session fails the next request the way a refusal
/// does — an `Err` with words — but the link reports it broken, so the
/// daemon's retry budget and the fourclass batch count it, and a dropped
/// link never parks it. An own link on a private core copy: the process
/// killed is this test's own child, by its id.
#[test]
fn a_core_gone_mid_session_breaks_the_link() {
    let mut link = own_link();
    let pid = link.pid().to_string();
    let (kill, args): (&str, &[&str]) = match cfg!(windows) {
        true => ("taskkill", &["/F", "/PID"]),
        false => ("kill", &["-9"]),
    };
    let killed = std::process::Command::new(kill)
        .args(args)
        .arg(&pid)
        .output()
        .expect("the kill command runs");
    assert!(killed.status.success(), "{killed:?}");
    let gone = link.request("tables", serde_json::json!({}));
    assert!(gone.is_err(), "no core answers: {gone:?}");
    assert!(link.is_broken(), "a lost framing breaks the link");
}

/// The session road (resolve/1, bags/1) meets the same line as a whole
/// pass (plan v2.33 step 11b item 6): a body whose line passes the
/// ceiling is named and never sent — the resolve family had sent it and
/// read the core's id-less `too_large` as every core ladder owed.
#[test]
fn the_session_road_names_a_line_over_the_ceiling_unsent() {
    let pad = "x".repeat(crate::tables::get().limits.caps.line_bytes);
    let refused =
        super::judged::session_ask(("tables/1", "tables"), serde_json::json!({ "pad": pad }))
            .expect_err("over the line");
    assert!(
        refused.starts_with("tables/1: request line ")
            && refused.ends_with("line the core reads; not sent"),
        "{refused}"
    );
}

/// An own link on the core the unit tests run with.
fn own_link() -> super::Link {
    let core = std::env::var("CE_CORE_BIN").expect("the unit tests run with CE_CORE_BIN");
    super::Link::own(&core).expect("the core answers").0
}
