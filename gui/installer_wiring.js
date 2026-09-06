// The installer's Claude Code wiring, gated (K step 10; re-cut when
// the wiring became `ce setup`, plan v2.29 step 10 O72). The passive
// lane once died for three days on exactly this class: 9f86d58 moved
// the marketplace manifest to the repo root, the old plugin/-pointing
// registration kept resolving to a cache miss, and no leg noticed —
// the wiring is exercised only when a human runs an installer on a
// machine with Claude Code, which CI never is. This gate pins the
// STATIC half of the chain: the NSIS hook delegates to `ce setup` (and
// `ce setup --unwire`, keyed on the same marker file), and every name
// `ce setup` spells — the marketplace source at its `release` ref, the
// marketplace name, the plugin@marketplace target, the marker — must
// resolve against the repository it ships from, so a rename or move
// reddens the push that made it instead of the install three days
// later.
//
// Usage: node cli/tests/gui/installer_wiring.js   (exit 1 = broken wiring)
"use strict";
const fs = require("fs");
const path = require("path");

// cli/tests/gui/ → the superproject root (the suite is the cli/tests submodule)
const root = path.join(__dirname, "..", "..", "..");
const read = (p) => fs.readFileSync(path.join(root, p), "utf8");

const problems = [];
function say(ok, what) {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) problems.push(what);
}

// 1. The bundler actually includes the hook script it names.
const conf = JSON.parse(read("gui/src-tauri/tauri.conf.json"));
const hooksRel = conf.bundle.windows.nsis.installerHooks;
const hooksPath = path.join(root, "gui/src-tauri", hooksRel);
say(fs.existsSync(hooksPath), `installerHooks resolves (${hooksRel})`);
const nsh = fs.readFileSync(hooksPath, "utf8");

// 2. The names `ce setup` spells, read off its source: the ONE table
// (`NAMES`) they live in now — the hook only prints the exit-code
// legend. Each field is read out of that literal by name.
const rs = read("cli/src/setup/mod.rs");
const names = rs.slice(rs.indexOf("pub const NAMES: Names = Names {"));
const field = (name) => {
  const m = names.match(new RegExp(`^\\s*${name}: "([^"]+)",`, "m"));
  return m ? m[1] : null;
};
const marker = field("marker");
const source = field("source");
const marketName = field("marketplace");
const target = field("plugin");
say(!!(marker && source && marketName && target), "setup spells marker, source, marketplace name and target");

// 3. Delegation + marker symmetry: POSTINSTALL runs `ce setup` (which
// writes the marker on a fresh registration), PREUNINSTALL keys on the
// marker, runs `ce setup --unwire` and deletes it — an asymmetric
// rename leaves either an unremovable marketplace or an uninstall that
// removes a registration this installer never made.
const macro = (name) => {
  const at = nsh.indexOf(`!macro ${name}`);
  const end = nsh.indexOf("!macroend", at);
  return at >= 0 && end > at ? nsh.slice(at, end) : "";
};
const post = macro("NSIS_HOOK_POSTINSTALL");
const pre = macro("NSIS_HOOK_PREUNINSTALL");
say(post.includes(`'"$INSTDIR\\ce.exe" setup'`), "POSTINSTALL runs ce setup");
say(!post.includes("marketplace add "), "POSTINSTALL spells no marketplace command of its own");
say(pre.includes(`IfFileExists "$INSTDIR\\${marker}"`), `PREUNINSTALL keys on the marker (${marker})`);
say(pre.includes(`'"$INSTDIR\\ce.exe" setup --unwire'`), "PREUNINSTALL runs ce setup --unwire");
say(pre.includes(`Delete "$INSTDIR\\${marker}"`), "PREUNINSTALL deletes the marker");

// 4. The marketplace source names THIS repository at its `release`
// ref (O82: verify-publish fast-forwards that branch to every
// published tag, so an install follows releases, not main). WHOLE-TOKEN
// equality, not includes(): a prefix test waved
// "skymanbp/CodeEraser-plugin" through this gate's own counterfactual
// run. Every slug-shaped `marketplace add` mention anywhere in the
// hook (manual-recovery prose included) must be the same source — a
// stale slug in a comment is the same defect as one on the exec line.
const [slug, ref] = (source || "").split("@");
say(slug === "skymanbp/CodeEraser" && ref === "release", `marketplace source is this repository at release (${source})`);
const mentions = [
  ...new Set(
    [...nsh.matchAll(/marketplace add (\S+)/g)]
      .map((x) => x[1].replace(/[,.)]+$/, ""))
      .filter((x) => x.includes("/"))
  ),
];
say(mentions.every((x) => x === source), `every hook mention of the source is ${source} (${mentions.join(", ") || "none"})`);
const mp = JSON.parse(read(".claude-plugin/marketplace.json"));
say(mp.name === marketName, `root manifest is the marketplace (name=${mp.name})`);

// 5. The install target's two halves resolve: plugin@marketplace.
const m = /^([a-z-]+)@([a-z-]+)$/.exec(target || "");
say(!!m, `install target is plugin@marketplace (${target})`);
if (m) {
  say(m[2] === mp.name, `target marketplace ${m[2]} = manifest name`);
  const plugin = (mp.plugins || []).find((p) => p.name === m[1]);
  say(!!plugin, `target plugin ${m[1]} exists in the manifest`);
  if (plugin) {
    const src = path.join(root, plugin.source);
    say(fs.existsSync(src), `plugin source resolves (${plugin.source})`);
  }
}

process.exit(problems.length === 0 ? 0 : 1);
