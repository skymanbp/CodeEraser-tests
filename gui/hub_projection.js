// K44 (plan v2.17 L round, sealed criterion §7): the diagnostics
// hub renders a report's row arrays generically, keeping the first
// five scalar columns of row 0 — so a 0.3.0 advisory row of the
// deadcode report must keep its `symbol` (and `name`) through that
// projection, or the symbol-level face shows a file with no symbol.
// serde_json emits the row's keys in BTreeMap order (alphabetical:
// code, line, name, symbol, why — the Rust half of the pin is
// cli/tests/it/deadcode_e2e.rs), which is the order this projection
// sees; the leg drives the REAL gui/ui/reports.js under DOM stubs and
// reads the rendered header cells.
//
// Usage: node cli/tests/gui/hub_projection.js   (exit 1 = a lost column)
"use strict";
const fs = require("fs");
const path = require("path");
const vm = require("vm");

// cli/tests/gui/ → the superproject root (the suite is the cli/tests submodule)
const root = path.join(__dirname, "..", "..", "..");

// Just enough DOM for the module's boot IIFE (a select it fills, two
// listeners) and for a family that registers its own renderer (an
// option appended, the chips and tables written) — one element per id,
// keeping what was written so a custom renderer's output can be read;
// hubTable is called directly.
const els = {};
const stubEl = (id) =>
  (els[id] ??= { addEventListener() {}, appendChild() {}, innerHTML: "", hidden: false, disabled: false, value: "" });
const sandbox = {
  Object,
  Array,
  String,
  Number,
  document: { getElementById: stubEl, createElement: () => ({}) },
  $: stubEl,
  i18nRefreshers: [],
  tr: (k, ...a) => `${k}(${a.join(",")})`,
  esc: (s) => String(s),
  posInt: (v) => v,
  invoke: async () => ({}),
  setStatus() {},
};
vm.createContext(sandbox);
vm.runInContext(fs.readFileSync(path.join(root, "gui/ui/reports.js"), "utf8"), sandbox);
vm.runInContext(fs.readFileSync(path.join(root, "gui/ui/hub_merge.js"), "utf8"), sandbox);

const problems = [];
const say = (ok, what) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) problems.push(what);
};

// One advisory row exactly as ce.deadcode-report/0.3.0 emits it.
const row = {
  code: "public_unmentioned",
  line: 84,
  name: "cli/src/config.rs",
  symbol: "DedupCfg",
  why: "no other file spells this exported name",
};
const headers = (html) => [...html.matchAll(/<th[^>]*>([^<]*)<\/th>/g)].map((m) => m[1]);
const cols = headers(sandbox.hubTable(["unmentioned", [row]]));
say(cols.includes("symbol") && cols.includes("name"), `symbol and name survive the projection (${cols.join(",")})`);
say(cols.join(",") === "code,line,name,symbol,why", "all five advisory columns render, in document order");
const body = sandbox.hubTable(["unmentioned", [row]]);
say(body.includes(">DedupCfg<"), "the symbol cell carries the name");

// Non-vacuity: the projection IS a cut — a sixth scalar column is
// dropped, so the five-column survival above is not a vacuous pass
// on a renderer that keeps everything.
const wide = { ...row, extra: "sixth" };
say(!headers(sandbox.hubTable(["wide", [wide]])).includes("extra"), "the projection really cuts at five columns");

// The similar report's candidate row (ce.similar-report/0.1.0, plan
// v2.29 step 6) was NAMED for this cut: its five leading scalars in
// alphabetical order are where, what, and the two judged numbers —
// `hits` is an array and `widened` / `shape_equal` fall past five.
const candidate = {
  at: "a.rs:2-4",
  key: "fetch_user_row/1",
  nth: 0,
  role: true,
  score: 12,
  hits: [2, 3, 0, 2, 3, 0],
  shape_equal: true,
  widened: false,
};
say(
  headers(sandbox.hubTable(["candidates", [candidate]])).join(",") === "at,key,nth,role,score",
  "a similar candidate projects to at,key,nth,role,score"
);

// A family whose document nests registers its own renderer (plan
// v2.31 step 7, ce.merge-report/0.1.0): the hub dispatches to it, and
// it still leads with the counts as chips — then a card per group
// whose parameter table holds every member's text at the parameter.
const mergeDoc = {
  schema: "ce.merge-report/0.1.0",
  counts: { groups: 1, members: 2, nodes: 10, suggestions: 1, holes: 2, feasible: 1 },
  unsendable: { not_isomorphic: 0, no_slot_table: 0, unbuilt: 0, over_cap: 0 },
  groups: [
    {
      group: 0, family: "t1t2", fragment: false, params: 1, kept: 0, savings: 4, feasible: true, reason: "ok",
      members: [{ path: "a.py", unit: "a.py:f/1#0", lines: [1, 3], run: [1, 3] }, { path: "a.py", unit: null, lines: [5, 8], run: [6, 8] }],
      holes: [{ param: 0, values: [{ member: 0, text: "alpha" }, { member: 1, text: "beta" }] }],
    },
  ],
  degraded: null,
};
say(typeof vm.runInContext("HUB.merge && HUB.merge.render", sandbox) === "function", "the merge family registers its own renderer");
els["hub-family"].value = "merge";
sandbox.mergeDoc = mergeDoc;
vm.runInContext("hubDoc = mergeDoc; renderHub();", sandbox);
say(els["hub-chips"].innerHTML.startsWith("<span>counts.groups <b>1</b>"), "the merge card leads with the counts as chips");
const cells = els["hub-tables"].innerHTML;
say(cells.includes("<code>alpha</code>") && cells.includes("<code>beta</code>"), "a parameter row holds every member's text");
say(cells.includes("6–8") && !cells.includes("1–3 ("), "a trimmed member shows the run it sent, a whole one its lines alone");

process.exit(problems.length === 0 ? 0 : 1);
