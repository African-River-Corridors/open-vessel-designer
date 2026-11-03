// Unit tests for src/lib/sternCrew.ts (issue #887): every figure of the Stern & crew view is a key of the run.
// Run: npm run test:unit
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { OUTFIT_WARNING_ID, sternCrewFigures } from "../src/lib/sternCrew.ts";

const run = (file) => JSON.parse(readFileSync(new URL(`../../${file}`, import.meta.url), "utf8")).options[0].results;
const container = run("examples/gas-140x29-coastal-lng/results.json");

test("the crew figures are the keys of the run's crew block", () => {
  const f = sternCrewFigures(container, container.text_ids);
  const c = container.crew;
  assert.equal(f.style, "container_block");
  assert.equal(f.tiers, null);
  assert.equal(f.wheelhouse, container.stern_arrangement.wheelhouse);
  assert.equal(f.wheelhouseBasis, container.stern_arrangement.wheelhouse_basis);
  assert.deepEqual(f.crew.ruleSet, { id: c.rule_set.id, version: c.rule_set.version, status: c.rule_set.status });
  assert.equal(f.crew.crew, c.crew);
  assert.equal(f.crew.minimum, c.minimum);
  assert.equal(f.crew.berths, c.berths);
  assert.equal(f.crew.crossCheck.ruleSet.id, c.cross_check.rule_set.id);
  assert.equal(f.crew.crossCheck.flagged, c.cross_check.flagged);
  assert.equal(f.crew.crossCheck.difference, c.cross_check.difference);
  assert.equal(f.outfitWarning, null, "a container block has no outfit warning");
  assert.equal(f.rooms, null, "a container block has no rooms table");
});

test("the gas figures are the keys of the run and the gas fuel system items", () => {
  const f = sternCrewFigures(container, container.text_ids);
  const r = container.results;
  if (r.vent_exit_above_keel_m === undefined) {
    assert.equal(f.gas, null);
    return;
  }
  assert.equal(f.gas.ventExitAboveDeckM, r.vent_exit_above_deck_m);
  assert.equal(f.gas.ventDistanceToAccommodationM, r.vent_distance_to_accommodation_m);
  assert.deepEqual(f.gas.fuelItems.map((i) => [i.label, i.massT]), container.stern_block_mass.gas_fuel_system.items.map((i) => [i.label, i.mass_t]));
});

test("a real deckhouse run gives its tiers, wheelhouse, crew, rooms, gas figures and the outfit warning", () => {
  // `deckhouse-run.json` is a run of app/e2e/fixtures/gas_deckhouse_self_propelled.vdesign; a Rust test keeps it current.
  const run = JSON.parse(readFileSync(new URL("./fixtures/deckhouse-run.json", import.meta.url), "utf8"));
  const f = sternCrewFigures(run, run.text_ids);
  assert.equal(f.style, "deckhouse");
  assert.equal(f.tiers, run.stern_arrangement.tiers);
  assert.ok(f.tiers >= 1);
  assert.equal(f.wheelhouse, run.stern_arrangement.wheelhouse);
  assert.equal(f.wheelhouseBasis, run.stern_arrangement.wheelhouse_basis);
  const outfit = run.warnings.find((w) => run.text_ids[w]?.id === OUTFIT_WARNING_ID);
  assert.ok(outfit, "the run carries the outfit warning");
  assert.equal(f.outfitWarning, outfit);
  assert.equal(f.crew.crew, run.crew.crew);
  assert.equal(f.crew.minimum, run.crew.minimum);
  assert.equal(f.crew.berths, run.crew.berths);
  assert.equal(f.crew.ruleSet.id, run.crew.rule_set.id);
  assert.equal(f.gas.ventExitAboveDeckM, run.results.vent_exit_above_deck_m);
  assert.deepEqual(f.gas.fuelItems.map((i) => [i.label, i.massT]), run.stern_block_mass.gas_fuel_system.items.map((i) => [i.label, i.mass_t]));
  assert.equal(f.rooms.totalM2, run.rooms.total_m2);
  assert.equal(f.rooms.rows.length, run.rooms.rows.length);
  assert.deepEqual(f.rooms.rows.map((r) => r.areaM2), run.rooms.rows.map((r) => r.area_m2));
});

test("the outfit warning is found by its Message Id, so a run without that id gives none", () => {
  const run = JSON.parse(readFileSync(new URL("./fixtures/deckhouse-run.json", import.meta.url), "utf8"));
  assert.equal(sternCrewFigures(run, {}).outfitWarning, null);
});

test("a run with no stern arrangement gives no figures", () => {
  assert.equal(sternCrewFigures(null), null);
  assert.equal(sternCrewFigures({}), null);
});
