//! The RETIRED exit of the RM14 corpus-generation gate
//! (baseline_bridge.rs): frozen members whose duplication was
//! genuinely removed, each row naming its batch. Moved out of
//! baseline_ledgers.rs at the E01 300-line wall when the forty-fifth
//! row landed (plan v2.32 step 5 Rust half part 2).

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
pub const RETIRED: [(u64, &str); 50] = [
    (
        5414355871597516298,
        "v2.32 step 6 2026-10-03: 8.0.0 retired structure/1's `patterns` request key, so CE.Structure imports the named refusal (CE.Wire.Retired) inside the import run it shared with CE.Docdup and StructureProps no longer refuses the pattern table twice - the two blocks this member's pair rode (Docdup.hs:38-43 / Structure.hs:32-37 imports, StructureProps.hs:124-134 / 134-137 refusals) left with the road (ce.toml dedup budget 37 -> 35)",
    ),
    (
        7648469069779258723,
        "v2.32 step 5 Rust half part 2: the structure console moved into the core (CE.Structure.Lines), so structure/report.rs kept only its typed reader and its two console stanzas, which rhymed with each other (55 tokens), were deleted; that member dissolved",
    ),
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
        11100202388371426840,
        "v2.32 step 6 2026-10-03: 8.0.0 retired structure/1's `patterns` request key, so CE.Structure imports the named refusal (CE.Wire.Retired) inside the import run it shared with CE.Docdup and StructureProps no longer refuses the pattern table twice - the two blocks this member's pair rode (Docdup.hs:38-43 / Structure.hs:32-37 imports, StructureProps.hs:124-134 / 134-137 refusals) left with the road (ce.toml dedup budget 37 -> 35)",
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
    (
        1890265009673430078,
        "v2.33 W3 2026-10-04: docdup/1 gained the `seqs` request shape (an optional field and its `.:?` read) and each scored row its measured run (the `runs` column before the counts), so CE.Docdup's request record / FromJSON / respondWith head and its reply head no longer rhyme with CE.Clone's - the two blocks these members rode left with them (ce.toml dedup budget 35 -> 33)",
    ),
    (
        6431664410968615078,
        "v2.33 W3 2026-10-04: docdup/1 gained the `seqs` request shape (an optional field and its `.:?` read) and each scored row its measured run (the `runs` column before the counts), so CE.Docdup's request record / FromJSON / respondWith head and its reply head no longer rhyme with CE.Clone's - the two blocks these members rode left with them (ce.toml dedup budget 35 -> 33)",
    ),
    (
        10624204810296426211,
        "v2.33 W3 2026-10-04: the clone and docdup verdict mirrors retired into the core (clone/1 `decide`, docdup/1), and unit/dedup/t3.rs and unit/docdup/judge.rs dropped their verdict-boundary tests - the two file heads this member rode (lines 1-9 of each) left with them (the suite's dedup budget 91 -> 90)",
    ),
];
