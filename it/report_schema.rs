//! Report JSON contracts: goldens pin the exact printed form of
//! `ce scan --format json` and `ce dedup --format json` (plan §7.1 /
//! M2 review R10 — any shape change must bump the schema id and
//! deliberately regenerate with
//! `CE_BLESS=1 cargo test --test it -- report_schema::`). Since plan
//! v2.32 step 5 both documents are the core's (document/1), printed in
//! the form the catalogue states (`pretty`).

use crate::common;
use crate::common::{assert_matches_golden, golden_path};

/// One file, one function that trips fn-naming — every document field
/// including a finding, over the real judged pipeline.
const SAMPLE: &str = "# sample\nimport os\n\n\ndef loadConfig(path, mode):\n    # read it\n    if mode:\n        return os.path.join(path, mode)\n    return path\n";

#[test]
fn scan_report_json_matches_golden() {
    let dir = common::tmp("scan-golden");
    std::fs::create_dir_all(dir.join("src")).expect("src");
    std::fs::write(dir.join("src/sample.py"), SAMPLE).expect("sample");
    let doc = codeeraser::scan::judged(&dir, &common::core_bin())
        .expect("judged")
        .document;
    let json = codeeraser::document::rendered("scan", &doc).expect("render");
    assert_matches_golden(&json, &golden_path("scan-report/report.golden.json"));
}

/// Real pipeline on a deterministic seed pair: pins block spans,
/// token/distinct counts, groups, and every summary field of the
/// 0.5.x shape.
#[test]
fn dedup_report_json_matches_golden() {
    let dir = common::tmp("dedup-golden");
    common::seed_clone_pair(&dir);
    let (found, summary) = codeeraser::dedup::analyze(&dir, None, None, None).expect("analyze");
    let value = codeeraser::dedup::answer(&common::core_bin(), &found, &summary, None)
        .expect("report json")
        .document;
    let json = codeeraser::document::rendered("dedup", &value).expect("render");
    assert_matches_golden(&json, &golden_path("dedup-report/report.golden.json"));
}
