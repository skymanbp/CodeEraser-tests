// The GUI's zh table gets the gate the CLI's has had since G3b:
// both-directions completeness. tr() resolves an unknown key to
// CE_I18N.en[key] → undefined, and applyStaticI18n assigns that to
// textContent — the literal string "undefined" on screen, silently.
// main_lang.rs reddens on a missing OR dead key; this leg asserts the
// same two properties over the real gui/ui files, harvesting usage
// from index.html (data-i18n / data-i18n-ph / data-i18n-title) and every screen's
// tr("...") calls.
//
// Usage: node cli/tests/gui/i18n_gate.js   (exit 1 = a broken table)
"use strict";
const fs = require("fs");
const path = require("path");
const vm = require("vm");

// cli/tests/gui/ → the superproject root (the suite is the cli/tests submodule)
const root = path.join(__dirname, "..", "..", "..");
const ui = (f) => fs.readFileSync(path.join(root, "gui/ui", f), "utf8");

// Load the REAL table under a stub sandbox (the lens gate's pattern).
const sandbox = { localStorage: { getItem: () => null, setItem() {} }, document: { querySelectorAll: () => [], getElementById: () => null } };
vm.createContext(sandbox);
vm.runInContext(ui("i18n.js"), sandbox);
const table = vm.runInContext("CE_I18N", sandbox);

// Harvest every key the UI actually asks for.
const used = new Set();
const html = ui("index.html");
for (const m of html.matchAll(/data-i18n(?:-ph|-title)?="([^"]+)"/g)) used.add(m[1]);
// i18n.js included: axisName() consumes "axisNames" from inside the
// table's own file, and a harvest that skips it calls that key dead
// The list is DISCOVERED, not written down: a hand-maintained list
// of screens is one someone forgets to extend, and it fails in the
// dangerous direction — a screen the harvest cannot see may use a
// key that exists in en and not in zh, and this gate would still
// call the table complete. Adding doctor.js is what surfaced it.
for (const f of fs.readdirSync(path.join(root, "gui/ui")).filter((f) => f.endsWith(".js"))) {
  for (const m of ui(f).matchAll(/\btr\(\s*"([^"]+)"/g)) used.add(m[1]);
}

const problems = [];
const say = (ok, what) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) problems.push(what);
};

// Direction 1: every used key exists in BOTH languages.
for (const lang of ["en", "zh"]) {
  const missing = [...used].filter((k) => !(k in table[lang]));
  say(missing.length === 0, `every used key exists in ${lang} (missing: ${missing.join(", ") || "none"})`);
}

// Direction 2: no dead keys — an entry nothing asks for is drift.
for (const lang of ["en", "zh"]) {
  const dead = Object.keys(table[lang]).filter((k) => !used.has(k));
  say(dead.length === 0, `no dead ${lang} keys (dead: ${dead.join(", ") || "none"})`);
}

// The two languages carry the same key set and the same value SHAPES
// (a function in one and a string in the other would explode only at
// call time, in one language).
const enKeys = Object.keys(table.en).sort().join(",");
const zhKeys = Object.keys(table.zh).sort().join(",");
say(enKeys === zhKeys, "en and zh carry the same key set");
const shapeSkew = Object.keys(table.en).filter((k) => typeof table.en[k] !== typeof table.zh[k]);
say(shapeSkew.length === 0, `value shapes agree (skew: ${shapeSkew.join(", ") || "none"})`);

// No key written twice in one language table. The evaluated object
// cannot show it — a later duplicate silently replaces the earlier one,
// so every check above passes while the first definition is dead text
// (hubPaths / hubPathsPlaceholder sat twice in both tables until plan
// v2.33 step 11a). So this reads the SOURCE: every top-level property
// name of each language's object literal, with strings, template
// literals (and the code inside their `${}`), comments and nested
// brackets skipped.
const src = ui("i18n.js");
const OPEN = "{[(";
const CLOSE = "}])";
// Index just past a string / template literal starting at i.
function skipQuoted(i) {
  const q = src[i];
  for (let j = i + 1; j < src.length; j++) {
    if (src[j] === "\\") j++;
    else if (src[j] === q) return j + 1;
    else if (q === "`" && src[j] === "$" && src[j + 1] === "{") j = scanCode(j + 2, null) - 1;
  }
  throw new Error(`unterminated ${q} at ${i}`);
}
// Index just past the bracket that closes the one opened before i;
// `keys` (when given) collects the property names at this level.
function scanCode(i, keys) {
  let expectKey = keys !== null;
  for (let j = i; j < src.length; ) {
    const c = src[j];
    if (c === "/" && src[j + 1] === "/") j = src.indexOf("\n", j);
    else if (c === "/" && src[j + 1] === "*") j = src.indexOf("*/", j) + 2;
    else if (c === '"' || c === "'" || c === "`") {
      const end = skipQuoted(j);
      if (expectKey && /^\s*:/.test(src.slice(end))) keys.push(src.slice(j + 1, end - 1));
      expectKey = false;
      j = end;
    } else if (OPEN.includes(c)) {
      j = scanCode(j + 1, null);
      expectKey = false;
    } else if (CLOSE.includes(c)) return j + 1;
    else if (/[A-Za-z_$]/.test(c)) {
      const word = /^[A-Za-z_$][\w$]*/.exec(src.slice(j))[0];
      if (expectKey && /^\s*:/.test(src.slice(j + word.length))) keys.push(word);
      expectKey = false;
      j += word.length;
    } else {
      if (c === ",") expectKey = keys !== null;
      j++;
    }
  }
  throw new Error(`unclosed bracket before ${i}`);
}
for (const lang of ["en", "zh"]) {
  const at = new RegExp(`\\n  ${lang}: \\{`).exec(src);
  const keys = [];
  if (at) scanCode(at.index + at[0].length, keys);
  const twice = [...new Set(keys.filter((k, i) => keys.indexOf(k) !== i))];
  // non-vacuity: the names read off the source are the table's names
  const read = at !== null && new Set(keys).size === Object.keys(table[lang]).length;
  say(read, `the ${lang} source table was read (${keys.length} keys written)`);
  say(twice.length === 0, `no ${lang} key written twice (twice: ${twice.join(", ") || "none"})`);
}

// Non-vacuity: the harvest actually found the UI's keys.
say(used.size >= 30, `the harvest is real (${used.size} keys found)`);

process.exit(problems.length === 0 ? 0 : 1);
