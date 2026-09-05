//! The four named exits of the RM14 corpus-generation gate
//! (baseline_bridge.rs): RETIRED (duplication genuinely removed),
//! REKEYED (a path move re-hashed the §7.2 id), REKEYED_SUITE (the
//! same move's second generation, into the test suite's own baseline)
//! and REANCHORED (7.0.0: every member re-hashed on container anchors).
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

/// The fourth exit, wholesale (proto 7.0.0, plan v2.29 step 8): the
/// §7.2 member identity stopped hashing a side's `nth` (its order among
/// same-key units, which a deleted earlier sibling shifted) and hashes
/// its CONTAINER-CHAIN ANCHOR instead (score/anchor.rs; `ce.baseline/2`,
/// a 6.x file refused by name). No duplication moved or vanished — every
/// member of both baselines simply re-hashed, so subset strength is
/// conserved modulo this one named re-hash. Each line is `old new` for
/// the superproject's baseline, derived by a one-shot instrument over
/// the current blocks (both encodings through `member_id`'s own throat,
/// the 6.x one replicated beside it) and asserted equal to the committed
/// discrete table at the establish that introduced it. The gate maps
/// every key it expects through this document and demands the ledger
/// describe the present: old key gone, successor seated (or retired
/// under its new name).
const REANCHORED: &str = "\
58680738777709039 2906470714490555695
220661488074354803 5367228321686517421
260199259066623225 16069206510978510719
383014491731907364 893885578290999606
772956878816100071 12166497122174400483
1349423228576917097 9441803825852099319
1490270073819770218 11100202388371426840
1666479829264484020 6745729160753970166
1864340262165647438 14337367506821926604
2384546146665276654 2247815219895934008
2697479895567114003 17109385265783004251
4432215216987320809 13442659180930832101
4490328886040948111 9801605542138674801
4721657295879003918 5221156939305294370
4806253556737496764 5414355871597516298
5115550998188702487 5910043469992256953
5121323985804442370 2619735925418418798
5339641534576888699 7958468518719001281
5453968854601614421 6631610987271775939
6305307650134482201 16825186465806316591
6931640579431850328 8277309085554555782
7043914318929455545 13445979226203383951
7979142639953595438 7038633364399792900
8722503002056228623 18120040199708299305
10065199023259739283 2642110337326275137
10402323885518070738 14961322951907020810
10644037952343320839 8110698983170028587
10761901687568248150 9206266380669934656
10951126460536114704 11777542260286451750
11585514087456054514 1890265009673430078
11978244492478385021 10172017980086749569
12132610844762202836 10781421503533844466
12760032928841135025 16008063474913422179
12951342303428434221 6298325535801379451
13636634923828280533 5552311341795790881
14677245901967675124 18165864476337164842
15132855933522337343 17892654107272750141
15710209983803503586 10398559894504384704
15711950771825569048 6431664410968615078
15829829114724947655 7648469069779258723
16971071944191990935 4109802609287562483
17344645433235839538 6663525434078246090
17382394238861462870 2746967240018182176
18281263396683763653 15778361983480758011
18402792940064279843 6725743369576513159
";

/// The same re-hash over the suite's own baseline (`cli/tests/`), one
/// line per current suite member — the REKEYED_SUITE keys map through
/// here too.
const REANCHORED_SUITE: &str = "\
445712772482298422 6820148659211860224
483837180733512712 7643732151769404362
1267438155500086114 4627816182577592402
1834470752127472533 10624204810296426211
2251448376289996924 17178382221569181982
2764460389950581398 2384747343415053334
3399331561252789081 13388372138811861843
3907290372153289633 12885443608603086983
3994762648766995931 18145302135429668631
4306278447391791598 8004640118776511968
4436406663102990921 16824025268915345897
5809836232211080205 10116995092941979681
5897628904388091846 14955390874888204888
6790153226082099745 5132720413110981505
7159712613621110196 13045399194712769330
7580513586960168485 8348646900489352633
7741550164311302591 12977226742982971547
8456322743912740697 15830877975425264319
8496712671615963652 15553722416467884234
8726798762914518262 9494331921244478216
9153965647152423328 3619811066347535954
9217265253020060045 3712657412139628083
9492909480328913746 2531709104514685410
10051308176248165342 13469575929976872162
10893335597300576946 17678660281602160708
12421398433142680457 5800321423343094137
13029361875312402170 15001131918392021334
14118089687064050930 14617403661228352020
14854481600624742842 10303298932748484520
15216415892928276502 15352526001381748162
15362123217158421226 6900582664555111166
16247644560556919310 6874473896895705594
16528659519593053534 1389266726151744718
17026778488399984839 16666268405563905379
17653464557457225298 1120348366388737672
18341374642905750366 10408772140774367092
18393983398663711169 9611994909034364333
";

/// The 7.0.0 successor of a 6.x member id, in whichever baseline it
/// lives; None = the migration ledger never saw the key.
pub fn reanchored(old: u64) -> Option<u64> {
    pairs_of(REANCHORED)
        .into_iter()
        .chain(pairs_of(REANCHORED_SUITE))
        .find(|(o, _)| *o == old)
        .map(|(_, n)| n)
}

/// Both REANCHORED documents parsed: (old, new, in_suite).
pub fn reanchored_rows() -> Vec<(u64, u64, bool)> {
    let main = pairs_of(REANCHORED).into_iter().map(|(o, n)| (o, n, false));
    let suite = pairs_of(REANCHORED_SUITE)
        .into_iter()
        .map(|(o, n)| (o, n, true));
    main.chain(suite).collect()
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
