/// The flag must not cost the pipes: a child built here still
/// answers over captured stdio — the transport every consumer
/// (corelink NDJSON, the git runners) rides on. git is already a
/// hard dependency of the product and of the churn battery.
#[test]
fn command_still_pipes_child_output() {
    let out = super::command("git")
        .arg("--version")
        .output()
        .expect("spawn git through the throat");
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("git version"));
}
