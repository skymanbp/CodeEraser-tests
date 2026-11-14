// The GUI's copies of the core's console sentences (plan v2.35 5C), as
// one JSON document on stdout: both languages, each sentence that takes
// arguments called with the markers {0} and {1}. This file asserts
// nothing; cli/tests/it/gui_core_words.rs fills the markers and holds
// every copy to the line the core's console prints for a real tree — the
// copies drifted once (5A reworded the core's code-1 reason and the GUI
// kept the old one) because nothing read the two side by side.
//
// Usage: node cli/tests/gui/words.js
"use strict";
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const { root } = require("./gate");

// Load the REAL table under a stub sandbox (the i18n gate's pattern).
const sandbox = { localStorage: { getItem: () => null, setItem() {} }, document: { querySelectorAll: () => [], getElementById: () => null } };
vm.createContext(sandbox);
vm.runInContext(fs.readFileSync(path.join(root, "gui/ui/i18n.js"), "utf8"), sandbox);
const table = vm.runInContext("CE_I18N", sandbox);

const out = {};
for (const lang of ["en", "zh"]) {
  const t = table[lang];
  out[lang] = {
    deadWhy: t.deadWhy,
    stateWords: t.stateWords,
    graphUnder: t.graphUnder,
    graphFalls: t.graphFalls("{0}"),
    gateListed: t.gateListed("{0}"),
    gateNoneBlocked: t.gateNoneBlocked("{0}", "{1}"),
    gateBlocked: t.gateBlocked("{0}", "{1}"),
    gateUnanswered: t.gateUnanswered("{0}"),
    axis4: t.checkAxisNames[4],
    axisOff: t.axisOff("{0}"),
    deadAddedHint: t.deadAddedHint("{0}"),
  };
}
process.stdout.write(JSON.stringify(out));
