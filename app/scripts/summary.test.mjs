// Unit tests for src/lib/summary.ts (issue #1081, spec 0017 D7). Run: npm run test:unit
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import { summarise, summaryRows } from "../src/lib/summary.ts";

const read = (u) => JSON.parse(fs.readFileSync(new URL(u, import.meta.url), "utf8"));
const GAS = read("../../examples/gas-140x29-coastal-lng/results.json").options[0].results;
const DRY = read("./fixtures/dry-bulk-run.json").options[0].results;
const at = (obj, path) => path.reduce((v, k) => (v == null ? undefined : Array.isArray(v) ? v.find((x) => x.name === k.slice(5)) : v[k]), obj);
const INPUTS = { hull: { source: "typed" }, cargo: { source: "typed" }, beam_m: { source: "default" }, depth_m: { source: "class" }, speed: { source: "default" } };
const input = (results, extra = {}) => ({ results, message: null, blockers: [], inputs: INPUTS, ...extra });
const label = (s, id) => s.rows.find((r) => r.label === id);

test("a feasible gas barge: verdict met, the scope sentence, and every headline number read from the results", () => {
  const s = summarise(input(GAS));
  assert.deepEqual(s.verdict, { kind: "met" });
  assert.equal(s.scope, GAS.rule_trace.scope);
  for (const r of s.rows) assert.deepEqual(r.value, at(GAS, r.path), r.label);
  assert.deepEqual(
    s.rows.map((r) => r.label),
    [
      "app-metric-cargo-volume", "app-metric-cargo", "app-metric-governs",
      "app-metric-tanks-across", "app-metric-tanks-along", "app-metric-tank-od",
      "app-metric-draught", "app-metric-draught-max", "app-metric-freeboard", "app-metric-lightship", "app-metric-displacement",
      "app-metric-installed-power", "app-metric-service-speed-deep", "app-metric-service-speed-channel",
      "app-metric-air-draft-loaded", "app-metric-air-draft-light",
    ],
  );
  assert.equal(label(s, "app-metric-cargo-volume").value, GAS.results.cargo_volume_m3);
  assert.equal(label(s, "app-metric-tanks-across").value, GAS.arrangement.n_wide);
  assert.equal(label(s, "app-metric-tank-od").value, GAS.arrangement.tank_od_m);
  assert.deepEqual(s.inputs, { typed: 2, defaults: 3 });
});

test("the warning checks that are not met come with the Checks panel's name and message", () => {
  const results = structuredClone(GAS);
  results.checks.push({ name: "slender hull", ok: false, message: "L/B is over 8", severity: "warning" });
  results.checks.push({ name: "other", ok: true, message: "OK", severity: "warning" });
  const s = summarise(input(results));
  assert.deepEqual(s.warnings, [{ name: "slender hull", message: "L/B is over 8" }]);
  assert.deepEqual(s.verdict, { kind: "met" }, "a warning check does not fail the verdict");
  assert.deepEqual(summarise(input(GAS)).warnings, []);
});

test("an Option with a blocking Rule not met: the verdict names it by title", () => {
  const results = structuredClone(GAS);
  const failing = results.rule_trace.rows.find((r) => r.consequence === "block" && r.outcome === "pass");
  failing.outcome = "fail";
  const twin = { ...failing };
  results.rule_trace.rows.push(twin);
  results.checks.push({ name: "stable", ok: false, message: "GM too small", severity: "error" });
  const s = summarise(input(results));
  assert.deepEqual(s.verdict, { kind: "not-met", titles: [failing.title, "stable"] });
  // A failed Rule that only warns does not count.
  const soft = structuredClone(GAS);
  const warn = soft.rule_trace.rows.find((r) => r.consequence !== "block");
  if (warn) warn.outcome = "fail";
  assert.deepEqual(summarise(input(soft)).verdict, { kind: "met" });
});

test("an Option that is not calculated (no Service Area): the verdict says so and names the input", () => {
  const blockers = [{ name: "service_area", label: "Service Area" }];
  const s = summarise(input(null, { blockers }));
  assert.deepEqual(s.verdict, { kind: "not-calculated", inputs: blockers, message: null });
  assert.deepEqual(s.rows, []);
  assert.equal(s.scope, null);
  assert.deepEqual(s.warnings, []);
  assert.deepEqual(s.inputs, { typed: 2, defaults: 3 }, "the input counts do not need results");
  const gone = summarise(input(null, { message: "No results for this Option" }));
  assert.deepEqual(gone.verdict, { kind: "not-calculated", inputs: [], message: "No results for this Option" });
});

test("damage run gives the count of cases; damage not run gives no count, and the other states are kept", () => {
  const run = summarise(input({ ...GAS, damage_status: "run", damage_cases: 48 }));
  assert.deepEqual(run.damage, { status: "run", cases: 48 });
  const notRun = summarise(input({ ...GAS, damage_status: "not run", damage_cases: 48 }));
  assert.equal(notRun.damage.status, "not run");
  assert.equal(summarise(input({ ...GAS, damage_status: "not applicable" })).damage.status, "not applicable");
  assert.equal(summarise(input(null)).damage.status, null);
});

test("a dry-bulk Option has no tank lines, and no power or speed lines where it has no powering block", () => {
  const s = summarise(input(DRY));
  const ids = s.rows.map((r) => r.label);
  for (const id of ["app-metric-tanks-across", "app-metric-tanks-along", "app-metric-tank-od"]) assert.ok(!ids.includes(id), id);
  assert.ok(ids.includes("app-metric-cargo-volume"));
  assert.ok(ids.includes("app-metric-draught"));
  for (const r of s.rows) assert.deepEqual(r.value, at(DRY, r.path), r.label);
  assert.equal(ids.includes("app-metric-installed-power"), at(DRY, ["powering"]) != null);
});

test("a missing number is undefined in its row, never a made-up one", () => {
  const results = structuredClone(GAS);
  delete results.results.freeboard_m;
  assert.equal(label(summarise(input(results)), "app-metric-freeboard").value, undefined);
  assert.equal(summaryRows(null).every((r) => r.value === undefined), true);
});
