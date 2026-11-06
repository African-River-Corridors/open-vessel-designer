// Unit tests for src/lib/damage.ts (issue #886). Run: npm run test:unit
import test from "node:test";
import assert from "node:assert/strict";
import { isDamageRule, conditionsOf, damageNotes, damageSets, governingCase, objectOf, passes } from "../src/lib/damage.ts";

const c = (condition, name, ok, extra = {}) => ({
  condition, name, flooded: ["M2C"], floats: ok ? "yes" : "no equilibrium", heel_deg: 0, trim_deg: 0, draught_mid_m: 2.5,
  max_gz_m: 0.8, range_end_deg: 20, area_m_rad: 0.14, ok, ...extra,
});
const row = (object, outcome, over = {}) => ({ rule: "igc/2.7/damage-survival", rule_sets: ["igc-2g@3.1.0"], title: "Survival", object, outcome, gap: null, ...over });

// Two conditions: the first has a failing case after a passing one, the second passes everywhere.
const cases = [
  c("Full load", "side 1", true), c("Full load", "side 2", false), c("Full load", "side 3", false),
  c("Ballast", "side 1", true), c("Ballast", "side 2", true),
];
const gas = {
  damage: { rule_set: "igc-2g@3.1.0", method: "lost buoyancy", cases, governing: [
    { condition: "Full load", case: "side 2" }, { condition: "Ballast", case: "side 2" },
  ] },
  rule_trace: { rows: [
    ...cases.map((k) => row(objectOf(k), k.ok ? "pass" : "fail")),
    row(null, "not applicable", { rule: "es-trin/28.03(4)/damage-stability", rule_sets: ["es-trin-2025@2.0.0"], title: "ES-TRIN damage" }),
    row(null, "not evaluated", { rule: "igc/2.7.1.1/openings-above-damage-waterline", title: "Openings", gap: "no openings list" }),
  ] },
};

test("damageSets reads the object of the gas path", () => {
  const [s] = damageSets(gas);
  assert.equal(s.ruleSet, "igc-2g@3.1.0");
  assert.equal(s.cases.length, 5);
  assert.deepEqual(conditionsOf(s), ["Full load", "Ballast"]);
});

test("damageSets reads the list of the liquid path, the by-rule blocks and MARPOL", () => {
  const list = [{ name: "side 1", flooded: [], floats: "yes", ok: true }];
  const sets = damageSets({ damage: list, marpol_damage: list, damage_by_rule: { "es-trin-2025@2.0.0": { rule_set: "es-trin-2025@2.0.0", method: "m", cases: list } } });
  assert.deepEqual(sets.map((s) => s.id), ["damage", "by-rule:es-trin-2025@2.0.0", "marpol"]);
  assert.deepEqual(conditionsOf(sets[0]), [""]);
});

test("damageSets is empty where the results hold no damage cases", () => {
  assert.deepEqual(damageSets(null), []);
  assert.deepEqual(damageSets({}), []);
  assert.deepEqual(damageSets({ damage: [] }), []);
});

test("the governing case is the engine's, not the first failing case", () => {
  const [s] = damageSets(gas);
  assert.equal(governingCase(s, "Full load", gas.rule_trace).name, "side 2");
  assert.equal(governingCase(s, "Ballast", gas.rule_trace).name, "side 2");
  // Two failing cases, the second with the lower maximum GZ: the engine names the second.
  const two = [c("X", "a", false, { max_gz_m: 0.5 }), c("X", "b", false, { max_gz_m: 0.1 })];
  const trace = { rows: two.map((k) => row(objectOf(k), "fail")) };
  const [t2] = damageSets({ damage: { rule_set: "igc-2g@3.1.0", cases: two, governing: [{ condition: "X", case: "b" }] } });
  assert.equal(governingCase(t2, "X", trace).name, "b");
});

test("a list with no engine governing case falls back to its first failing case", () => {
  const list = [{ name: "a", flooded: [], floats: "yes", ok: true }, { name: "b", flooded: [], floats: "yes", ok: false }];
  const [s] = damageSets({ damage: list });
  assert.equal(governingCase(s, "", { rows: [] }).name, "b");
});

test("NORMAM's damage Rule is a damage Rule", () => {
  assert.ok(isDamageRule("normam/5.23.8/standard-damage-cases"));
  assert.ok(isDamageRule("igc/2.7/damage-survival"));
  assert.ok(!isDamageRule("igc/2.7.1.2/heel-any-stage"));
});

test("the governing case the view shows is the one the engine names and the trace holds a row for", () => {
  const [s] = damageSets(gas);
  for (const g of gas.damage.governing) {
    const shown = governingCase(s, g.condition, gas.rule_trace);
    assert.equal(shown.name, g.case);
    assert.ok(gas.rule_trace.rows.some((r) => r.object === `${g.condition}: ${g.case}`));
  }
});

test("the verdict is the trace's, and the case's own where the trace has no row", () => {
  const [s] = damageSets(gas);
  assert.equal(passes(s, cases[1], gas.rule_trace), false);
  assert.equal(passes(s, cases[0], gas.rule_trace), true);
  // The trace says fail where the case says ok: the trace wins.
  const t2 = { rows: [row(objectOf(cases[0]), "fail")] };
  assert.equal(passes(s, cases[0], t2), false);
  assert.equal(passes(s, cases[0], { rows: [] }), true);
});

test("a liquid hull with no failing case and no engine pick has no governing case", () => {
  const list = [{ name: "a", flooded: [], floats: "yes", ok: true }, { name: "b", flooded: [], floats: "yes", ok: true }];
  const [s] = damageSets({ damage: list });
  assert.equal(governingCase(s, "", { rows: [] }), null);
});

test("the rows of another Rule Set with the same object name are not read", () => {
  const [s] = damageSets(gas);
  const other = { rows: [row(objectOf(cases[1]), "pass", { rule: "adn/9.3.1.15.1/damage-stability", rule_sets: ["adn-gas@1.0.0"] })] };
  assert.equal(passes(s, cases[1], other), false, "no row of this set: the case's own verdict");
});

test("damageNotes lists the damage Rules not evaluated or not applicable, with the engine's reason", () => {
  const notes = damageNotes(gas.rule_trace);
  assert.deepEqual(notes.map((n) => [n.outcome, n.gap]), [["not applicable", null], ["not evaluated", "no openings list"]]);
  assert.deepEqual(damageNotes(null), []);
});

test("damageNotes lists a damage Rule that was not run, with the engine's reason (issue #923)", () => {
  const trace = { rows: [row(null, "not run", { gap: "damage stability was not run: the run skipped it" })] };
  assert.deepEqual(damageNotes(trace).map((n) => [n.outcome, n.gap]), [["not run", "damage stability was not run: the run skipped it"]]);
});

test("two cases with one name take their own trace rows, in order", () => {
  const twin = [c("A", "bottom", true), c("A", "bottom", false)];
  const trace = { rows: [row("A: bottom", "pass"), row("A: bottom", "fail")] };
  const [s] = damageSets({ damage: { rule_set: "igc-2g@3.1.0", cases: twin, governing: [] } });
  assert.equal(passes(s, twin[0], trace), true);
  assert.equal(passes(s, twin[1], trace), false);
  assert.equal(governingCase(s, "A", trace), twin[1]);
});

test("a damage_by_rule set marks the engine's governing case of every condition (issue #928)", () => {
  // A river-only gas file: no `damage` block, the ADN set names the governing case, and every case passes.
  const adn = [c("Full load", "a", true), c("Full load", "b", true), c("Ballast", "a", true), c("Ballast", "b", true)];
  const rule = (k) => ({ ...row(objectOf(k), "pass"), rule: "adn/9.3.1.15.1/damage-stability", rule_sets: ["adn-gas@2.0.0"] });
  const results = {
    damage_by_rule: { "adn-gas": { rule_set: "adn-gas@2.0.0", method: "m", cases: adn, governing: [{ condition: "Full load", case: "b" }, { condition: "Ballast", case: "a" }] } },
    rule_trace: { rows: adn.map(rule) },
  };
  const [s] = damageSets(results);
  assert.deepEqual(s.engineGoverning, [{ condition: "Full load", case: "b" }, { condition: "Ballast", case: "a" }]);
  assert.equal(governingCase(s, "Full load", results.rule_trace).name, "b");
  assert.equal(governingCase(s, "Ballast", results.rule_trace).name, "a");
});

// ---- Damage on demand (issue #924) ----
import { readFileSync } from "node:fs";
import { REPORT_DAMAGE_PROMPT, damageStatus, secondsBefore, secondsLeft, secondsPerCase, wholeSeconds } from "../src/lib/damage.ts";

test("damageStatus reads the engine's status and case count, and null for results with none", () => {
  assert.deepEqual(damageStatus({ damage_status: "not run", damage_cases: 158 }), { status: "not run", cases: 158 });
  assert.deepEqual(damageStatus({ damage_status: "run", damage_cases: 0 }), { status: "run", cases: 0 });
  assert.deepEqual(damageStatus({ damage_status: "not applicable" }), { status: "not applicable", cases: null });
  assert.deepEqual(damageStatus({ damage_status: "something new", damage_cases: "x" }), { status: null, cases: null });
  assert.deepEqual(damageStatus(null), { status: null, cases: null });
});

test("secondsLeft: no total, no time, first event, last event", () => {
  assert.equal(secondsLeft(0, 0, 0), null, "no total");
  assert.equal(secondsLeft(5, 0, 1000), null, "no total, some done");
  assert.equal(secondsLeft(0, 100, 500), null, "nothing done yet: no time per case");
  assert.equal(secondsLeft(3, 100, 0), null, "no time has passed");
  assert.equal(secondsLeft(NaN, 100, 1000), null);
  // First event: 10 of 100 done in 2 s, so 0.2 s a case and 90 cases left: 18 s.
  assert.equal(secondsLeft(10, 100, 2000), 18);
  // Last event: nothing left.
  assert.equal(secondsLeft(100, 100, 20000), 0);
  assert.equal(secondsLeft(101, 100, 20000), 0);
});

test("the estimate before a run needs a measured time per case, and the case count", () => {
  assert.equal(secondsBefore(158, null), null, "no measured time: the case count only");
  assert.equal(secondsBefore(null, 0.05), null);
  assert.equal(secondsBefore(0, 0.05), null);
  assert.equal(secondsBefore(158, 0.05), 7.9);
  assert.equal(secondsPerCase(100, 5000), 0.05);
  assert.equal(secondsPerCase(0, 5000), null);
  assert.equal(secondsPerCase(100, 0), null);
});

test("seconds show whole, and never as 0", () => {
  assert.equal(wholeSeconds(0.2), 1);
  assert.equal(wholeSeconds(7.6), 8);
  assert.equal(wholeSeconds(90), 90);
});

test("the report question's words are messages of the English catalogue, as the design fixes them", () => {
  const ftl = readFileSync(new URL("../../locales/en/app.ftl", import.meta.url), "utf8");
  for (const id of Object.values(REPORT_DAMAGE_PROMPT)) assert.match(ftl, new RegExp(`^${id} = `, "m"), id);
  assert.match(ftl, /^app-report-damage-title = Damage stability has not been run$/m);
  assert.match(ftl, /^app-report-damage-body-time = Damage stability has not been run \(about \{ \$seconds \} s\)\. Run it now, or make the report without it\?$/m);
  assert.match(ftl, /^app-report-damage-body-cases = Damage stability has not been run \(\{ \$cases \} cases\)\. Run it now, or make the report without it\?$/m);
  assert.match(ftl, /^app-report-damage-run = Run it now$/m);
  assert.match(ftl, /^app-report-damage-skip = Make the report without it$/m);
});
