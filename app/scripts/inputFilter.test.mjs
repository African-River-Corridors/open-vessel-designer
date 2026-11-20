// Unit tests for src/lib/inputFilter.ts (issue #1084, spec 0017 D9 and D10). Run: npm run test:unit
// Label: property
import test from "node:test";
import assert from "node:assert/strict";
import {
  DEFAULT_MODE, MODE_KEY, essentialsShown, filterMatches, hiddenByEssentials, loadMode, saveMode, shownInputs,
} from "../src/lib/inputFilter.ts";

const spec = (name, group, extra = {}) => ({ name, label: name.replace(/_/g, " "), group, kind: "number", required: false, essential: false, ...extra });
const held = (value, source = "default") => ({ value, source });

// A trimmed schema: the six required inputs, two essential inputs, and inputs that are neither.
const SPECS = [
  spec("cargo", "Required", { kind: "cargo", required: true, essential: true, label: "Cargo" }),
  spec("formation", "Required", { kind: "formation", required: true, essential: true, label: "Formation" }),
  spec("length_m", "Required", { required: true, essential: true, label: "Length" }),
  spec("beam_m", "Required", { required: true, essential: true, label: "Beam" }),
  spec("hull_form", "Required", { kind: "choice", required: true, essential: true, label: "Hull Form" }),
  spec("service_area", "Required", { kind: "choice", required: true, essential: true, label: "Service Area" }),
  spec("design_speed_kn", "Speed", { essential: true, label: "Design service speed" }),
  spec("tank_pressure_bar", "Cargo Containment", { essential: true, applies_to: ["type_c"], label: "Tank design pressure" }),
  spec("freeboard_m", "Hull", { label: "Freeboard" }),
  spec("block_coefficient", "Hull", { label: "Block coefficient" }),
  spec("steel_price", "Cost", { label: "Steel price" }),
];
const common = {
  cargo: held("lng", "typed"), formation: held({ kind: "self_propelled" }, "typed"), length_m: held(110, "typed"),
  beam_m: held(16, "typed"), hull_form: held("shipd", "typed"), service_area: held("coastal", "typed"),
  design_speed_kn: held(12), freeboard_m: held(1.2), block_coefficient: held(0.8), steel_price: held(900),
};
const names = (list) => list.map((s) => s.name);
const shownIn = (inputs, riverOnly = false) => names(SPECS.filter((s) => essentialsShown(s, inputs, riverOnly)));

test("Essentials for an LNG self-propelled coastal Option: the required inputs, the essential ones that apply, nothing else", () => {
  const inputs = { ...common, tank_pressure_bar: held(4) };
  assert.deepEqual(shownIn(inputs), ["cargo", "formation", "length_m", "beam_m", "hull_form", "service_area", "design_speed_kn", "tank_pressure_bar"]);
});

test("Essentials for a bauxite pushed barge: an essential input that does not apply to it is not shown", () => {
  // Rust sends no tank pressure for a dry-bulk Option, so the Option holds none.
  const inputs = { ...common, cargo: held("bauxite", "typed"), formation: held({ kind: "pushed_convoy" }, "typed") };
  const shown = shownIn(inputs);
  assert.ok(!shown.includes("tank_pressure_bar"));
  assert.ok(shown.includes("design_speed_kn"));
  assert.deepEqual(shown.slice(0, 6), ["cargo", "formation", "length_m", "beam_m", "hull_form", "service_area"]);
});

test("a typed input that is not essential is shown; so is an input from the Vessel Class", () => {
  const typed = { ...common, freeboard_m: held(1.5, "typed"), block_coefficient: held(0.7, "class") };
  const shown = shownIn(typed);
  assert.ok(shown.includes("freeboard_m"));
  assert.ok(shown.includes("block_coefficient"));
  assert.ok(!shown.includes("steel_price"));
});

test("an input the Option does not hold stays hidden, typed or not", () => {
  const spec1 = SPECS.find((s) => s.name === "freeboard_m");
  assert.equal(essentialsShown(spec1, {}, false), false);
});

test("a required choice not yet answered is shown in Essentials; so are the river limits of a river-only Option", () => {
  const inputs = { ...common };
  delete inputs.service_area;
  assert.ok(shownIn(inputs).includes("service_area"));
  const limit = spec("draught_limit_m", "Limits");
  assert.equal(essentialsShown(limit, {}, true), true);
  assert.equal(essentialsShown(limit, {}, false), false);
});

test("hiddenByEssentials counts the inputs All inputs draws and Essentials does not", () => {
  const inputs = { ...common, tank_pressure_bar: held(4) };
  assert.equal(hiddenByEssentials(SPECS, inputs, false), 3);
  assert.equal(hiddenByEssentials(SPECS, { ...inputs, freeboard_m: held(1, "typed") }, false), 2);
});

test("shownInputs: All inputs draws every held input; Essentials draws fewer", () => {
  const inputs = { ...common, tank_pressure_bar: held(4) };
  assert.equal(shownInputs(SPECS, inputs, false, "all", "", (s) => s.label).length, SPECS.length);
  assert.equal(shownInputs(SPECS, inputs, false, "essentials", "", (s) => s.label).length, SPECS.length - 3);
});

test("the filter matches part of a label without regard to case", () => {
  assert.equal(filterMatches("Freeboard", "free"), true);
  assert.equal(filterMatches("Freeboard", "BOARD"), true);
  assert.equal(filterMatches("Freeboard", "fReE"), true);
  assert.equal(filterMatches("Freeboard", "beam"), false);
  assert.equal(filterMatches("Freeboard", "  "), true, "blank text matches all");
});

test("the filter finds inputs that Essentials hides, in both modes, whatever group they are in", () => {
  const inputs = { ...common, tank_pressure_bar: held(4) };
  for (const mode of ["essentials", "all"]) {
    const found = shownInputs(SPECS, inputs, false, mode, "FREEBOARD", (s) => s.label);
    assert.deepEqual(names(found), ["freeboard_m"], mode);
  }
  const several = shownInputs(SPECS, inputs, false, "essentials", "e", (s) => s.label);
  assert.ok(new Set(several.map((s) => s.group)).size > 1, "inputs of more than one group");
});

test("the filter matches the label as shown in the Display Language, not the English one", () => {
  const french = (s) => (s.name === "freeboard_m" ? "Franc-bord" : s.label);
  assert.deepEqual(names(shownInputs(SPECS, common, false, "all", "franc", french)), ["freeboard_m"]);
  assert.deepEqual(names(shownInputs(SPECS, common, false, "all", "freeboard", french)), []);
});

test("the filter does not find an input the Option does not hold", () => {
  assert.deepEqual(names(shownInputs(SPECS, common, false, "all", "pressure", (s) => s.label)), []);
});

test("the mode is kept: the first start opens on Essentials, a saved choice is read back", () => {
  const store = new Map();
  const storage = { getItem: (k) => store.get(k) ?? null, setItem: (k, v) => store.set(k, v) };
  assert.equal(DEFAULT_MODE, "essentials");
  assert.equal(loadMode(storage), "essentials");
  saveMode("all", storage);
  assert.equal(store.get(MODE_KEY), "all");
  assert.equal(loadMode(storage), "all");
  store.set(MODE_KEY, "nonsense");
  assert.equal(loadMode(storage), "essentials");
});

test("a storage that throws leaves Essentials", () => {
  const broken = { getItem() { throw new Error("blocked"); }, setItem() { throw new Error("blocked"); } };
  assert.equal(loadMode(broken), "essentials");
  assert.doesNotThrow(() => saveMode("all", broken));
});

test("blocked site data: reading the global localStorage throws, and the mode is still Essentials", () => {
  Object.defineProperty(globalThis, "localStorage", { configurable: true, get() { throw new Error("SecurityError"); } });
  try {
    assert.equal(loadMode(), "essentials");
    assert.doesNotThrow(() => saveMode("all"));
  } finally {
    delete globalThis.localStorage;
  }
});
