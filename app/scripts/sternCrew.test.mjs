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

test("a deckhouse Option shows its tiers, its wheelhouse and the outfit warning", () => {
  const english = "The deckhouse carries no outfit mass.";
  const results = {
    stern_arrangement: { style: "deckhouse", tiers: 3, wheelhouse: "elevating", wheelhouse_basis: "air_draft_limit" },
    warnings: ["another", english],
    text_ids: { [english]: { id: OUTFIT_WARNING_ID } },
    results: { vent_exit_above_keel_m: 15.3, vent_exit_above_deck_m: 10.8, vent_distance_to_accommodation_m: 84.3, vent_mast_distance_met: true },
    stern_block_mass: { gas_fuel_system: { items: [{ label: "gas heater", mass_t: 0.48 }] } },
  };
  const f = sternCrewFigures(results, results.text_ids);
  assert.equal(f.style, "deckhouse");
  assert.equal(f.tiers, 3);
  assert.equal(f.wheelhouse, "elevating");
  assert.equal(f.wheelhouseBasis, "air_draft_limit");
  assert.equal(f.outfitWarning, english);
  assert.equal(f.crew, null, "no crew block: the view says so");
  assert.equal(f.gas.ventExitAboveDeckM, 10.8);
  assert.equal(f.gas.ventDistanceMet, true);
  assert.deepEqual(f.gas.fuelItems, [{ label: "gas heater", massT: 0.48 }]);
});

test("a run with no stern arrangement gives no figures", () => {
  assert.equal(sternCrewFigures(null), null);
  assert.equal(sternCrewFigures({}), null);
});
