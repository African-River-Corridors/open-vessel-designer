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

test("two cases with one name take their own trace rows, in order", () => {
  const twin = [c("A", "bottom", true), c("A", "bottom", false)];
  const trace = { rows: [row("A: bottom", "pass"), row("A: bottom", "fail")] };
  const [s] = damageSets({ damage: { rule_set: "igc-2g@3.1.0", cases: twin, governing: [] } });
  assert.equal(passes(s, twin[0], trace), true);
  assert.equal(passes(s, twin[1], trace), false);
  assert.equal(governingCase(s, "A", trace), twin[1]);
});
