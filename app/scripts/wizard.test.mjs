// Unit tests for src/lib/wizard/wizard.ts (spec 0017, issue #1080): the steps and inputs the wizard shows,
// its limits, the Option name and the choices it sends. The schema is a trimmed copy of `ovd schema`.
// Run: npm run test:unit
// Label: property
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import {
  STEPS,
  answersFromInputs,
  canCreate,
  checkNumber,
  choicesSent,
  emptyAnswers,
  inputsOnStep,
  leftToDefaults,
  optionName,
  problemsOnStep,
} from "../src/lib/wizard/wizard.ts";

const schema = JSON.parse(fs.readFileSync(new URL("./fixtures/wizard-schema.json", import.meta.url), "utf8"));
const spec = (name) => schema.inputs.find((i) => i.name === name);

/** The worked example of spec 0017, section 2, as the wizard holds it after a person types it. */
function workedExample() {
  const a = emptyAnswers();
  a.cargo = "LNG";
  a.serviceArea = "coastal-and-river";
  a.formation = { kind: "self_propelled", abreast: null, inLine: null };
  a.hullForm = "raked_box";
  a.length = 140;
  a.beam = 29;
  a.own = { moulded_depth_m: 12, max_speed_kn: 12, max_tank_od_m: 12, tanks_in_line: 2, min_freeboard_m: 1.81 };
  return a;
}

test("the wizard has five steps and a review, in the order of the spec", () => {
  assert.deepEqual(STEPS, ["cargo", "area", "formation", "hull", "limits", "review"]);
});

test("LNG, coastal and river: no river limits; tank inputs and minimum freeboard are on step 5", () => {
  const a = workedExample();
  assert.deepEqual(inputsOnStep(schema, a, "cargo"), ["cargo"]);
  assert.deepEqual(inputsOnStep(schema, a, "area"), ["service_area"]);
  assert.deepEqual(inputsOnStep(schema, a, "formation"), ["formation"]);
  assert.deepEqual(inputsOnStep(schema, a, "hull"), ["hull_form", "length_overall_m", "beam_m"]);
  assert.deepEqual(inputsOnStep(schema, a, "limits"), ["moulded_depth_m", "max_speed_kn", "max_tank_od_m", "tanks_in_line", "min_freeboard_m"]);
});

test("LNG, river only: both river limits join step 2 and minimum freeboard leaves step 5", () => {
  const a = workedExample();
  a.serviceArea = "river-only";
  assert.deepEqual(inputsOnStep(schema, a, "area"), ["service_area", "draught_limit_m", "air_draft_limit_m"]);
  assert.deepEqual(inputsOnStep(schema, a, "limits"), ["moulded_depth_m", "max_speed_kn", "max_tank_od_m", "tanks_in_line"]);
});

test("bauxite pushed barge: no tank inputs, whatever the Service Area", () => {
  const a = emptyAnswers();
  a.cargo = "bauxite";
  a.serviceArea = "river-only";
  a.formation = { kind: "pushed_barge", abreast: null, inLine: null };
  assert.deepEqual(inputsOnStep(schema, a, "limits"), ["moulded_depth_m", "max_speed_kn"]);
  a.serviceArea = "coastal-and-river";
  assert.deepEqual(inputsOnStep(schema, a, "limits"), ["moulded_depth_m", "max_speed_kn", "min_freeboard_m"]);
});

test("before a Cargo is chosen, step 5 offers only what every Design has", () => {
  assert.deepEqual(inputsOnStep(schema, emptyAnswers(), "limits"), ["moulded_depth_m", "max_speed_kn"]);
});

test("a Vessel Class takes the place of length and beam on the Hull step", () => {
  const a = workedExample();
  a.hullMode = "class";
  assert.deepEqual(inputsOnStep(schema, a, "hull"), ["hull_form", "vessel_scheme", "vessel_class"]);
});

test("a value outside a hard limit blocks and names the limit; the limits are the schema's", () => {
  const r = checkNumber(spec("beam_m"), 0.5);
  assert.equal(r.block, true);
  assert.deepEqual(r.hard, spec("beam_m").hard);
  assert.equal(checkNumber(spec("beam_m"), 1000).block, true);
  assert.equal(checkNumber(spec("tanks_in_line"), 0).block, true);
  assert.equal(checkNumber(spec("max_tank_od_m"), 0).block, true, "a limit with gt, not ge");
  assert.equal(checkNumber(spec("beam_m"), Number.NaN).block, true, "text that is not a number");
});

test("a value outside the usual range warns and does not block", () => {
  const r = checkNumber(spec("beam_m"), 36);
  assert.equal(r.block, false);
  assert.deepEqual(r.soft, spec("beam_m").soft);
  const inside = checkNumber(spec("beam_m"), 29);
  assert.equal(inside.block, false);
  assert.equal(inside.soft, null);
});

test("Next is blocked on a hard limit and not on a usual range", () => {
  const a = workedExample();
  a.beam = 0.5;
  assert.ok(problemsOnStep(schema, a, "hull").some((p) => p.input === "beam_m" && p.kind === "hard"));
  a.beam = 36;
  const soft = problemsOnStep(schema, a, "hull");
  assert.ok(soft.some((p) => p.input === "beam_m" && p.kind === "soft"));
  assert.ok(!soft.some((p) => p.block));
});

test("an empty optional limit is not a problem; one that is not a number is", () => {
  const a = workedExample();
  a.own = {};
  assert.deepEqual(problemsOnStep(schema, a, "limits"), []);
  a.own = { moulded_depth_m: Number.NaN };
  assert.ok(problemsOnStep(schema, a, "limits").some((p) => p.block && p.input === "moulded_depth_m"));
});

test("a required answer that is missing blocks its step", () => {
  const a = emptyAnswers();
  for (const [step, input] of [["cargo", "cargo"], ["area", "service_area"], ["formation", "formation"]]) {
    assert.ok(problemsOnStep(schema, a, step).some((p) => p.block && p.kind === "required" && p.input === input), step);
  }
  assert.ok(problemsOnStep(schema, a, "hull").some((p) => p.block && p.input === "length_overall_m"));
});

test("a river-only path cannot create without both limits or a Vessel Class that gives them", () => {
  const a = workedExample();
  a.serviceArea = "river-only";
  assert.equal(canCreate(schema, a), false);
  a.draughtLimit = 3;
  assert.equal(canCreate(schema, a), false, "one limit is not enough");
  a.airDraftLimit = 12;
  assert.equal(canCreate(schema, a), true);
  // A class that gives both limits stands in for them.
  const b = workedExample();
  b.serviceArea = "river-only";
  b.hullMode = "class";
  b.scheme = "CEMT-1992";
  b.vesselClass = "CEMT-I-W";
  assert.equal(canCreate(schema, b), true);
  // A class that gives neither does not.
  b.scheme = "CN-GB38030";
  b.vesselClass = "CN-GB38030-CG-110-H1R";
  assert.equal(canCreate(schema, b), false);
  b.draughtLimit = 2;
  b.airDraftLimit = 9;
  assert.equal(canCreate(schema, b), true);
});

test("the worked example can be created and its limits give no problem", () => {
  const a = workedExample();
  assert.equal(canCreate(schema, a), true);
  for (const step of STEPS) assert.deepEqual(problemsOnStep(schema, a, step).filter((p) => p.block), [], step);
});

test("the Option name is made from the Cargo and the size", () => {
  assert.equal(optionName(schema, workedExample()), "LNG 140x29");
  const a = workedExample();
  a.length = 76.5;
  a.beam = 11.4;
  a.cargo = "bauxite";
  assert.equal(optionName(schema, a), "Bauxite 76.5x11.4");
  a.optionName = "My barge";
  assert.equal(optionName(schema, a), "My barge");
  a.optionName = "   ";
  assert.equal(optionName(schema, a), "Bauxite 76.5x11.4", "a blank name falls back to the made one");
});

test("the Option name of a Vessel Class names the class", () => {
  const a = workedExample();
  a.hullMode = "class";
  a.scheme = "CEMT-1992";
  a.vesselClass = "CEMT-I-W";
  assert.equal(optionName(schema, a), "LNG CEMT-I-W");
});

test("the choices sent for the worked example are the 11 inputs of the spec and nothing else", () => {
  assert.deepEqual(choicesSent(schema, workedExample()), {
    cargo: "LNG",
    hull_form: "raked_box",
    formation: { kind: "self_propelled" },
    length_overall_m: 140,
    beam_m: 29,
    service_area: "coastal-and-river",
    moulded_depth_m: 12,
    max_speed_kn: 12,
    max_tank_od_m: 12,
    tanks_in_line: 2,
    min_freeboard_m: 1.81,
  });
  assert.equal(Object.keys(choicesSent(schema, workedExample())).length, 11);
});

test("an empty optional limit is left out, not sent as null", () => {
  const a = workedExample();
  a.own = { moulded_depth_m: null, max_speed_kn: 12 };
  const sent = choicesSent(schema, a);
  assert.equal("moulded_depth_m" in sent, false);
  assert.equal(sent.max_speed_kn, 12);
  assert.equal("max_tank_od_m" in sent, false);
});

test("an answer given for an input that no longer applies is not sent", () => {
  // The person typed a freeboard, then went back and chose river only.
  const a = workedExample();
  a.serviceArea = "river-only";
  a.draughtLimit = 3;
  a.airDraftLimit = 12;
  const sent = choicesSent(schema, a);
  assert.equal("min_freeboard_m" in sent, false);
  assert.equal(sent.draught_limit_m, 3);
  assert.equal(sent.air_draft_limit_m, 12);
  // Bauxite holds no tanks.
  const b = workedExample();
  b.cargo = "bauxite";
  assert.equal("max_tank_od_m" in choicesSent(schema, b), false);
  assert.equal("tanks_in_line" in choicesSent(schema, b), false);
});

test("a Vessel Class sends the scheme and class and no length or beam", () => {
  const a = workedExample();
  a.hullMode = "class";
  a.scheme = "CEMT-1992";
  a.vesselClass = "CEMT-I-W";
  const sent = choicesSent(schema, a);
  assert.equal(sent.vessel_scheme, "CEMT-1992");
  assert.equal(sent.vessel_class, "CEMT-I-W");
  assert.equal("length_overall_m" in sent, false);
  assert.equal("beam_m" in sent, false);
});

test("a pushed convoy sends its barges abreast and in line", () => {
  const a = workedExample();
  a.cargo = "bauxite";
  a.formation = { kind: "pushed_convoy", abreast: 2, inLine: 3 };
  assert.deepEqual(choicesSent(schema, a).formation, { kind: "pushed_convoy", abreast: 2, in_line: 3 });
});

test("the example's typed inputs fill the wizard and are sent back unchanged", () => {
  const inputs = choicesSent(schema, workedExample());
  const a = answersFromInputs(inputs);
  assert.equal(a.cargo, "LNG");
  assert.equal(a.length, 140);
  assert.deepEqual(a.own, { moulded_depth_m: 12, max_speed_kn: 12, max_tank_od_m: 12, tanks_in_line: 2, min_freeboard_m: 1.81 });
  assert.deepEqual(choicesSent(schema, a), inputs);
  assert.equal(canCreate(schema, a), true);
});

test("the count left to the Defaults Profile is what applies to the Design and was not typed", () => {
  const a = workedExample();
  const n = leftToDefaults(schema, a);
  assert.ok(Number.isInteger(n) && n > 0);
  // Typing one more input takes one off the count.
  const b = workedExample();
  b.own = { ...b.own };
  delete b.own.min_freeboard_m;
  assert.equal(leftToDefaults(schema, b), n + 1);
});
