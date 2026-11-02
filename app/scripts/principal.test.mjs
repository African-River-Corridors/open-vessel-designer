// Unit tests for src/lib/principal.ts (issue #889). Run: npm run test:unit
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import { PRINCIPAL_ROWS, principalItems } from "../src/lib/principal.ts";

const FILE = new URL("../../catalogue/report/principal_characteristics.json", import.meta.url);
const file = JSON.parse(fs.readFileSync(FILE, "utf8"));
const RUN = JSON.parse(fs.readFileSync(new URL("../../examples/gas-140x29-coastal-lng/results.json", import.meta.url), "utf8"));
const fmt = (v, d) => v.toLocaleString("en-GB", { minimumFractionDigits: d, maximumFractionDigits: d });
const NE = "not evaluated";
const at = (obj, path) => path.reduce((v, k) => (v == null ? undefined : Array.isArray(v) ? v.find((x) => x.name === k.slice(5)) : v[k]), obj);

test("the strip has the rows of the data file, in its order, with its labels, units and decimals", () => {
  assert.deepEqual(PRINCIPAL_ROWS, file.rows);
  const items = principalItems(RUN.options[0].results, fmt, NE);
  assert.deepEqual(items.map((i) => i.id), file.rows.map((r) => r.id));
  assert.deepEqual(items.map((i) => i.labelId), file.rows.map((r) => r.label_id));
  assert.deepEqual(items.map((i) => i.unit), file.rows.map((r) => r.unit));
});

test("every value is read from the run and rounded to the file's decimals", () => {
  const results = RUN.options[0].results;
  for (const [i, item] of principalItems(results, fmt, NE).entries()) {
    const row = file.rows[i];
    const v = at(results, row.path);
    if (typeof v === "number") assert.equal(item.text, fmt(v, row.decimals), row.id);
    else assert.equal(item.text, NE, row.id);
  }
});

test("a run without a crew block shows not evaluated for crew and berths, and numbers elsewhere", () => {
  const results = structuredClone(RUN.options[0].results);
  delete results.crew;
  const by = Object.fromEntries(principalItems(results, fmt, NE).map((i) => [i.id, i]));
  assert.equal(by["crew-minimum"].text, NE);
  assert.equal(by["crew-berths"].text, NE);
  assert.equal(by["crew-berths"].evaluated, false);
  assert.equal(by["loa"].text, "140.00");
  results.crew = { minimum: 7, berths: 11 };
  const again = Object.fromEntries(principalItems(results, fmt, NE).map((i) => [i.id, i]));
  assert.equal(again["crew-minimum"].text, "7");
  assert.equal(again["crew-berths"].text, "11");
});

test("no results at all leaves every row not evaluated", () => {
  assert.ok(principalItems(null, fmt, NE).every((i) => i.text === NE && !i.evaluated));
});
