//! The five named exits of the RM14 corpus-generation gate
//! (baseline_bridge.rs): RETIRED (duplication genuinely removed),
//! REKEYED (a path move re-hashed the §7.2 id), REKEYED_SUITE (the
//! same move's second generation, into the test suite's own baseline),
//! REANCHORED (7.0.0: every member re-hashed on container anchors) and
//! RELOCATED (a path move after 7.0.0, in the anchored key space) —
//! those two live in baseline_reanchored.rs. Split from the
//! gate at the E01 300-line wall when the third ledger landed, and
//! again at the same wall when the fifteenth RETIRED row did — the
//! ledgers are documents, the gate is the reader.

/// Frozen members whose SOURCE duplication was genuinely REMOVED
/// after the freeze — each entry names its de-duplication batch.
/// The gate below exists for GENERATION immutability (a corpus
/// regeneration dropping members arrives with no entry here); real
/// cleanups are this tool's whole point and retire BY NAME, never
/// silently. A listed id back in the baseline = a stale entry,
/// refused — the ledger can only ever describe the present. A row
/// names the key the member carried on the day it dissolved: the
/// frozen 6.x key before 7.0.0, the anchored (relocated) key after —
/// both readers in baseline_bridge.rs share `seated_or_retired`.
pub const RETIRED: [(u64, &str); 44] = [
    (
        6631610987271775939,
        "v2.32 step 5 lane A: the clone and docdup reports moved into the core (CE.Clone.Document / CE.Docdup.Document), so t3's `print` and docdup's `print` through crate::report::emit were deleted — the dedup/t3/mod.rs <-> docdup/judge/mod.rs member dissolved",
    ),
    (
        6663525434078246090,
        "v2.32 step 5 lane A: the same move — t3's Counts lost its Serialize derive and its serialised-only fields, so its field run no longer rhymes with config/thresholds.rs's Thresholds; that member dissolved",
    ),
    (
        7038633364399792900,
        "v2.32 step 5 lane A: the same move — faces.rs's clone_t3 and docdup read the core's documents (t3::answer / judge::answer), so the two `enveloped` faces rhyming with each other are gone; the faces.rs self-member dissolved",
    ),
    (
        5552311341795790881,
        "v2.30 step 7b (3): Scan.hs's `reply` took an Echo record and its fenced / conditions helpers, so its head no longer rhymes with Audit.hs's `reply` (51 tokens) — the Audit.hs <-> Scan.hs member dissolved",
    ),
    (
        1389266726151744718,
        "v2.30 step 7b (3): it/sonar_whitepaper.rs's six per-language tables retired with the Rust walkers and the file became the whitepaper register's reader, so its head and tail no longer rhyme with it/metrics.rs's top level — that member dissolved",
    ),
    (
        3619811066347535954,
        "v2.30 step 7b (3): the same retirement — the old sonar_whitepaper.rs CASES table rhymed with itself across two of its language tables, and the register is one NDJSON fixture; the self-member dissolved",
    ),
    (
        12885443608603086983,
        "v2.30 step 7b (3): the same retirement — the old sonar_whitepaper.rs CASES table rhymed with it/coc_haskell.rs's CASES; that member dissolved",
    ),
    (
        12977226742982971547,
        "v2.30 step 7b (3): the same retirement — the old sonar_whitepaper.rs head and tail rhymed with it/divergence_stances.rs's top level; that member dissolved",
    ),
    (
        15830877975425264319,
        "v2.30 step 7b (3): the same retirement — the old sonar_whitepaper.rs CASES table rhymed with it/divergence_stances.rs's CASES; that member dissolved",
    ),
    (
        16666268405563905379,
        "v2.30 step 7b (3): the same retirement — the old sonar_whitepaper.rs head rhymed with it/coc_haskell.rs's top level; that member dissolved",
    ),
    (
        17178382221569181982,
        "v2.30 step 7b (3): the same retirement — the old sonar_whitepaper.rs CASES table rhymed with it/metrics.rs's CASES; that member dissolved",
    ),
    (
        6725743369576513159,
        "v2.30 step 7b: Erase.hs took the closure's helpers into its import list and the target battery into its knobless cascade, so its head no longer rhymes with Audit.hs's (94 tokens) — the Audit.hs <-> Erase.hs member dissolved",
    ),
    (
        10781421503533844466,
        "v2.30 step 7b: the structure request record moved to Structure/Request.hs beside the shape road, so Structure.hs no longer rhymes with Verdict/Wire.hs's request record (56 tokens) — the Structure.hs <-> Verdict/Wire.hs member dissolved",
    ),
    (
        16825186465806316591,
        "v2.30 step 7b: EraseProps' (name, probe) battery table became WireHarness.runLegs' two parallel lists when the closure legs joined it, and the EraseProps <-> TrendProps battery member dissolved",
    ),
    (
        9206266380669934656,
        "v2.30 step 7b: StructureProps' (name, probe) battery table became WireHarness.runLegs' two parallel lists when the shape-road legs joined it, and its three battery members — against AuditProps, SplitProps and TrendProps — dissolved together",
    ),
    (
        14337367506821926604,
        "v2.30 step 7b: StructureProps' (name, probe) battery table became WireHarness.runLegs' two parallel lists when the shape-road legs joined it, and its three battery members — against AuditProps, SplitProps and TrendProps — dissolved together",
    ),
    (
        16008063474913422179,
        "v2.30 step 7b: StructureProps' (name, probe) battery table became WireHarness.runLegs' two parallel lists when the shape-road legs joined it, and its three battery members — against AuditProps, SplitProps and TrendProps — dissolved together",
    ),
    (
        13445979226203383951,
        "v2.30 step 5b-9: score::measure and the join's judge_pairs each spelled the same path -> row map and the first lines of the clone-row seating after it; the seating is one function both roads read, score::clone_rows, and the map another, score::row_index, so the join/verdicts.rs <-> score/mod.rs member dissolved",
    ),
    (
        10398559894504384704,
        "v2.30 step 3: the LangSpec schema change (fn_required_fields, if_kinds, call_fields, owner_kinds on every table) broke the GO / HASKELL table rhyme; the shape re-paired as FAMILY / TYPESCRIPT, the new member 12441163793521289489",
    ),
    (
        2746967240018182176,
        "v2.30 step 1: scan rowShape via CE.Wire.rowCheck; trend/2 pair gone",
    ),
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
    (
        18165864476337164842,
        "v2.30 step 5b-7 2026-09-27: the PostToolUse leg restated the PreToolUse hook's prelude and the gate named it twice, so both write hooks now read one gate (guard::write_event) — the audit/guard prelude pair this member rode left with the restatement",
    ),
    (
        6298325535801379451,
        "v2.32 step 2 2026-10-01: docdup/spec.rs's const table of numbers and markers moved into the core (CE.Docdup.Cost, CE.Lang.Common.Prose) and the file kept only its readers, so it no longer rhymes with graph/wire.rs's const table",
    ),
    (
        12441163793521289489,
        "v2.32 step 2 2026-10-01: the scan LangSpec tables moved into the core's CE.Lang modules and scan/spec_{c,launch,lua,r}.rs were deleted - the table-to-table rhyme this member rode left with them",
    ),
    (
        14066198317823549133,
        "v2.32 step 2 2026-10-01: the scan LangSpec tables moved into the core's CE.Lang modules and scan/spec_{c,launch,lua,r}.rs were deleted - the table-to-table rhyme this member rode left with them",
    ),
    (
        15283346326385119323,
        "v2.32 step 2 2026-10-01: the scan LangSpec tables moved into the core's CE.Lang modules and scan/spec_{c,launch,lua,r}.rs were deleted - the table-to-table rhyme this member rode left with them",
    ),
    (
        16409031450442344353,
        "v2.32 step 2 2026-10-01: the scan LangSpec tables moved into the core's CE.Lang modules and scan/spec_{c,launch,lua,r}.rs were deleted - the table-to-table rhyme this member rode left with them",
    ),
    (
        17679470848269995785,
        "v2.32 step 2 2026-10-01: the scan LangSpec tables moved into the core's CE.Lang modules and scan/spec_{c,launch,lua,r}.rs were deleted - the table-to-table rhyme this member rode left with them",
    ),
    (
        18074344538836094835,
        "v2.32 step 2 2026-10-01: the scan LangSpec tables moved into the core's CE.Lang modules and scan/spec_{c,launch,lua,r}.rs were deleted - the table-to-table rhyme this member rode left with them",
    ),
    (
        18120040199708299305,
        "v2.32 step 4B 2026-10-02: the join console moved onto the bound document (report::Bound) and join/report.rs no longer spells the file-row loop trend/report.rs rhymed with - the block left with it (ce.toml dedup budget 42 -> 41)",
    ),
    (
        18190675168993883734,
        "v2.32 step 2 2026-10-01: the scan LangSpec tables moved into the core's CE.Lang modules and scan/spec_{c,launch,lua,r}.rs were deleted - the table-to-table rhyme this member rode left with them",
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

/// One parser for the two-column ledgers, this file's and
/// baseline_reanchored.rs's.
pub fn pairs_of(doc: &str) -> Vec<(u64, u64)> {
    doc.lines()
        .map(|l| {
            let mut w = l.split_whitespace().map(|n| n.parse().expect("member id"));
            (w.next().expect("old"), w.next().expect("new"))
        })
        .collect()
}
