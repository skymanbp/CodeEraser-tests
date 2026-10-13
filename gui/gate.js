// The node gates' shared plumbing (audit C row 36): the superproject
// root, the `say` reporter with its list of problems, and the exit
// that reads that list. hub_projection.js, i18n_gate.js,
// lens_invariant.js and installer_wiring.js each spelled these once.
"use strict";
const path = require("path");

// cli/tests/gui/ → the superproject root (the suite is the cli/tests submodule)
const root = path.join(__dirname, "..", "..", "..");

const problems = [];
const say = (ok, what) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) problems.push(what);
};

// Exit 0 when every `say` held, 1 otherwise.
const done = () => process.exit(problems.length === 0 ? 0 : 1);

module.exports = { root, problems, say, done };
