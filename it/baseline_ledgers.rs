//! The three named exits of the RM14 corpus-generation gate
//! (baseline_bridge.rs): RETIRED (duplication genuinely removed),
//! REKEYED (a path move re-hashed the §7.2 id) and REKEYED_SUITE (the
//! same move's second generation, into the test suite's own baseline).
//! Split from the gate at the E01 300-line wall when the third ledger
//! landed — the ledgers are documents, the gate is the reader.

/// Frozen members whose SOURCE duplication was genuinely REMOVED
/// after the freeze — each entry names its de-duplication batch.
/// The gate below exists for GENERATION immutability (a corpus
/// regeneration dropping members arrives with no entry here); real
/// cleanups are this tool's whole point and retire BY NAME, never
/// silently. A listed id back in the baseline = a stale entry,
/// refused — the ledger can only ever describe the present.
pub const RETIRED: [(u64, &str); 14] = [
    (
        384679663923384372,
        "v2.23 step 4: ast.rs's `children` and `named_children` differed          in nothing but which accessor pair they called; both sides of          this member were that file's top level, and one `kids` walk          dissolved the pair",
    ),
    (
        17681371623117319386,
        "v2.23 step 3: LangSpec spelled `&'static [&'static str]` once per \n         field, so windows of its declarations rhymed with each other; \n         naming the type (`pub type Kinds`) dissolved the run this member \n         belonged to",
    ),
    (
        15941172437324464441,
        "v0.6 P3: budget_breach's scope+size stanza folded into the \
         shared sized_write throat — the zone observer would have been \
         its second copy, which is exactly what the ratchet refuses",
    ),
    (
        12069581799026901328,
        "v0.5.0 cleanup: the docdup/t3 audit assembly legs retired whole \
         (one-shot instruments, user ruling 2026-08-20) — their twin \
         stanzas went with the files",
    ),
    (
        5157928330096415643,
        "ADR-008 P3 tenth bite: probe_gate.rs Target/probe stanzas table-driven",
    ),
    (
        13860957365059798074,
        "ADR-008 P3 tenth bite: probe_gate.rs Target/probe stanzas table-driven",
    ),
    (
        17525617435279245638,
        "ADR-008 P3 tenth bite: probe_gate.rs Target/probe stanzas table-driven",
    ),
    (
        9291417281997523150,
        "M7.5 deep-thin: dormant generator/replay halves excised (EVAL-SET amendment)",
    ),
    (
        11389896668359803242,
        "M7.5 deep-thin: dormant generator/replay halves excised (EVAL-SET amendment)",
    ),
    (
        11980760446779025474,
        "M7.5 deep-thin: dormant generator/replay halves excised (EVAL-SET amendment)",
    ),
    (
        14078978657527709474,
        "headroom sprint 2026-08-24: the guard hook-envelope moved to its own leaf, dissolving the audit/guard/health face chains this member rode",
    ),
    (
        14752821476017908148,
        "headroom sprint 2026-08-24: the guard hook-envelope moved to its own leaf, dissolving the audit/guard/health face chains this member rode",
    ),
    (
        18322300311120329557,
        "headroom sprint 2026-08-24: the guard hook-envelope moved to its own leaf, dissolving the audit/guard/health face chains this member rode",
    ),
    (
        1045130446377401539,
        "v2.18 subtraction batch 2026-08-28: the clone/docdup parse_result zip-and-shape tails moved INTO lockstep::parse_scores as its row shaper — the two families' last clone pair, one member",
    ),
];

/// §7.2 members whose id was RE-KEYED, not removed: the 2026-08-26
/// tests merge moved every integration root from `cli/tests/` to
/// `cli/tests/it/`, and a member id hashes its sides' PATHS — so the
/// duplication survived under a new key. Each line is `old new`,
/// derived by re-hashing every current block's member with the it/
/// prefix stripped back to the pre-merge path (one-shot instrument,
/// same `member_id("clone", ..)` throat) — all 22 landed in the same
/// establish, so the gate can demand BOTH exits: the old key gone
/// AND its successor seated. A rename is neither a cleanup (RETIRED)
/// nor a rewrite; it gets its own ledger so RETIRED's "duplication
/// genuinely removed" claim stays true of every RETIRED row. One
/// document, not a tuple table: the pair table is this repo's
/// most-rhyming token shape, and this ledger's first draft duly
/// cloned against the tree.rs vocabulary probe.
const REKEYED: &str = "\
403099628869665561 5334522415661725441
544715961310330730 12869198444950852490
2882419735887471358 14233146052196931664
3617975609329458940 10800140972768024034
4594855599954596366 17942699572433341952
4742601464868157263 10488150594258799447
5315089852001355175 9604165174791687559
5649599479936906696 14886130437521744082
6110245542527137987 1789320524636541571
7831405124474784806 13432724845369755580
8423437723322751843 1761923728217702749
9365579024641649012 16776566333703835756
10471417808950161288 11934305135765209544
12304973788363298482 686744464717091746
13294590378060918370 5084876662118533682
14215491476022457249 16099042295628905391
14377273919435821282 5091239511442668826
14768101773271409225 1813728321700676971
15454680224575117074 2831076054381254698
16465453656258218267 13321394588352495875
16603034293491020654 3504532828427293052
18388344836232998712 5687268460904012680
";

/// The REKEYED document parsed: (old, new) per line.
pub fn rekeyed_pairs() -> Vec<(u64, u64)> {
    pairs_of(REKEYED)
}

/// The second generation of the same move (plan v2.18 step #12): the
/// suite became a READER of this tree, so every member both of whose
/// sides live under `cli/tests/` left the superproject's baseline and
/// sits in the suite's own (`cli/tests/ce-baseline.json`) under its own
/// root spelling — `it/x.rs`, not `cli/tests/it/x.rs` — which the §7.2
/// id hashes. Each line is `successor suite`: the REKEYED successor key
/// and the same member re-hashed at the suite's root (the same
/// one-shot instrument, `member_id("clone", ..)` over the suite's own
/// blocks). All 22 REKEYED successors moved, none stayed: the gate
/// demands the successor GONE from the superproject's baseline and its
/// suite key SEATED in the suite's, so subset strength is conserved
/// across the two ledgers modulo the named move.
const REKEYED_SUITE: &str = "\
5334522415661725441 7580513586960168485
12869198444950852490 8496712671615963652
14233146052196931664 10051308176248165342
10800140972768024034 5897628904388091846
17942699572433341952 15216415892928276502
10488150594258799447 18393983398663711169
9604165174791687559 3994762648766995931
14886130437521744082 2764460389950581398
1789320524636541571 9217265253020060045
13432724845369755580 16528659519593053534
1761923728217702749 3399331561252789081
16776566333703835756 2251448376289996924
11934305135765209544 483837180733512712
686744464717091746 9153965647152423328
5084876662118533682 9492909480328913746
16099042295628905391 8456322743912740697
5091239511442668826 10893335597300576946
1813728321700676971 6790153226082099745
2831076054381254698 13029361875312402170
13321394588352495875 7741550164311302591
3504532828427293052 8726798762914518262
5687268460904012680 4306278447391791598
";

/// The REKEYED_SUITE document parsed: (successor, suite) per line.
pub fn suite_pairs() -> Vec<(u64, u64)> {
    pairs_of(REKEYED_SUITE)
}

/// One parser for the two-column ledgers.
fn pairs_of(doc: &str) -> Vec<(u64, u64)> {
    doc.lines()
        .map(|l| {
            let mut w = l.split_whitespace().map(|n| n.parse().expect("member id"));
            (w.next().expect("old"), w.next().expect("new"))
        })
        .collect()
}
