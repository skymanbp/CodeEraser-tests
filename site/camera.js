// The viewer's camera, driven under Node (site/viewer.js exports it
// when there is no document). Four properties the eight figures on
// codeeraser.dev rely on and no browser run would state as a number:
// fitting, anchored zoom, the clamp, and the scale bounds. Run by
// cli/tests/it/site_viewer.rs; exit 1 names the property that broke.
"use strict";
const path = require("path");
const { camera, MAX, STEP } = require(path.join(__dirname, "..", "..", "..", "site", "viewer.js"));

let failures = 0;
function check(ok, what) {
  if (!ok) { failures += 1; console.error(`FAIL ${what}`); }
}
const near = (a, b, eps = 1e-6) => Math.abs(a - b) <= eps;

// A wide picture fits the stage's width, a tall one its height, both
// centred on the axis they do not fill — the letterboxed judgment
// diagram in its capped stage is the tall case.
{
  const wide = camera(1440 / 620);
  const v = wide.resize(912, 393);
  check(near(v.w, 912) && near(v.x, 0) && v.s === 1, "wide picture fits the stage width");
  const tall = camera(650 / 760);
  const t = tall.resize(912, 702);
  check(near(t.h, 702) && near(t.x, (912 - t.w) / 2) && near(t.y, 0), "tall picture fits the height, centred");
}

// Zooming about a point keeps that point's picture fraction under it,
// for factors in either direction and points anywhere on the stage,
// until the clamp has to move the picture (the edge cases below).
{
  const cam = camera(1424 / 892);
  cam.resize(912, 571);
  for (const [f, px, py] of [[1.4, 100, 50], [2, 456, 285], [1 / 1.3, 800, 500], [3, 10, 560]]) {
    const before = cam.view();
    const fx = (px - before.x) / before.w, fy = (py - before.y) / before.h;
    const after = cam.zoomAt(f, px, py);
    const gx = (px - after.x) / after.w, gy = (py - after.y) / after.h;
    const clamped = after.x === 0 || after.y === 0 || near(after.x, 912 - after.w) || near(after.y, 571 - after.h);
    check(clamped || (near(fx, gx) && near(fy, gy)), `zoom ×${f} at (${px},${py}) keeps its anchor`);
  }
}

// The clamp: a zoomed picture always covers the stage (no gap on any
// side), whatever pan is asked for; a fitted one is centred.
{
  const cam = camera(1440 / 620);
  cam.resize(912, 393);
  cam.zoomAt(3, 456, 196);
  for (const [dx, dy] of [[5000, 5000], [-5000, -5000], [37, -11], [-5000, 5000]]) {
    const v = cam.pan(dx, dy);
    check(v.x <= 0 && v.y <= 0 && v.x + v.w >= 912 - 1e-6 && v.y + v.h >= 393 - 1e-6, `pan (${dx},${dy}) leaves no gap`);
  }
  const back = cam.reset();
  check(back.s === 1 && near(back.x, 0) && near(back.w, 912), "reset refits");
}

// Scale is bounded to [1, MAX]; n steps in then n steps out land back
// on the start (the steps are geometric), and an empty stage — a tab
// panel not yet shown — yields numbers, never NaN.
{
  const cam = camera(1);
  cam.resize(500, 500);
  check(cam.zoomAt(1000, 250, 250).s === MAX, "scale is capped at MAX");
  check(cam.zoomAt(1 / 1000, 250, 250).s === 1, "scale floors at 1");
  for (let i = 0; i < 5; i += 1) cam.zoomAt(STEP, 100, 100);
  for (let i = 0; i < 5; i += 1) cam.zoomAt(1 / STEP, 100, 100);
  check(near(cam.view().s, 1, 1e-9), "n steps in and out return to 1x");
  const empty = camera(1440 / 620);
  const z = empty.resize(0, 0);
  check([z.x, z.y, z.w, z.h, z.s].every(Number.isFinite), "an empty stage yields finite numbers");
  const shown = empty.resize(912, 393);
  check(near(shown.w, 912) && near(shown.x, 0), "a stage shown later fits on its first resize");
}

if (failures) process.exit(1);
console.log("camera: every property holds");
