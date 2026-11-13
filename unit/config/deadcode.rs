//! `[deadcode] block` (plan v2.35, the dead-file amendment, 5B): the
//! declared list read at the load throat, its mask, and its place in the
//! knob fingerprint. One literal table: a `block` value, then ` => `,
//! then the mask (bit 0 orphaned, bit 1 born) or `!` and the words the
//! refusal must carry.

use crate::config::Config;

const CASES: &str = r#"
[] => 0
["orphaned"] => 1
["born"] => 2
["orphaned", "born"] => 3
["born", "orphaned", "born"] => 3
["standing"] => !"standing" cannot be blocked — the inventory never blocks: declare an entry or delete the file
["standing", "born"] => !"standing" cannot be blocked
["later"] => !unknown state "later"
["Born"] => !unknown state "Born"
"#;

#[test]
fn the_block_list_reads_to_its_mask_or_is_refused_by_name() {
    for (list, says) in crate::testutil::case_table(CASES, 9) {
        let toml = format!("[deadcode]\nblock = {list}\n");
        let cfg: Config = toml::from_str(&toml).expect(&toml);
        match (says.strip_prefix('!'), cfg.deadcode.fault()) {
            (Some(words), Some(fault)) => assert!(fault.contains(words), "{list}: {fault}"),
            (Some(words), None) => panic!("{list} accepted; expected: {words}"),
            (None, Some(fault)) => panic!("{list} refused: {fault}"),
            (None, None) => assert_eq!(cfg.deadcode.mask().to_string(), says, "{list}"),
        }
    }
    // an unknown key of the table is refused at parse, by name
    let err = toml::from_str::<Config>("[deadcode]\nblocks = [\"born\"]\n").expect_err("a typo");
    assert!(err.to_string().contains("blocks"), "{err}");
}

/// The default (an empty list) is silence to the knob fingerprint, a
/// declared list moves it by name, and order and repeats move nothing:
/// the canonical form is the sorted, unique set.
#[test]
fn the_block_list_enters_the_knob_fingerprint_as_a_set() {
    let digest = |toml: &str| toml::from_str::<Config>(toml).expect(toml).knobs_digest();
    assert_eq!(digest("[deadcode]\nblock = []\n"), None);
    assert_eq!(digest(""), None);
    let both = digest("[deadcode]\nblock = [\"orphaned\", \"born\"]\n");
    assert!(both.is_some(), "a declared list moves the digest");
    assert_eq!(
        both,
        digest("[deadcode]\nblock = [\"born\", \"orphaned\", \"born\"]\n")
    );
    assert_ne!(both, digest("[deadcode]\nblock = [\"born\"]\n"));
}
