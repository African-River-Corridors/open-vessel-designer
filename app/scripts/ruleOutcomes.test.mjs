// Unit tests for src/lib/ruleOutcomes.ts (issue #923). Run: npm run test:unit
import test from "node:test";
import assert from "node:assert/strict";
import { OUTCOMES, outcomeRank, outcomeTag } from "../src/lib/ruleOutcomes.ts";
import { readFileSync } from "node:fs";

test("every outcome the engine writes has a rank and a tag, `not run` apart from `not evaluated`", () => {
  const engine = ["pass", "fail", "warn", "note", "not applicable", "not evaluated", "not run"];
  assert.deepEqual(Object.keys(OUTCOMES).sort(), [...engine].sort());
  const ranks = engine.map(outcomeRank);
  assert.equal(new Set(ranks).size, engine.length, "a rank is shared");
  assert.notEqual(OUTCOMES["not run"].label, OUTCOMES["not evaluated"].label);
  assert.ok(outcomeRank("not run") > outcomeRank("not evaluated") && outcomeRank("not run") < outcomeRank("not applicable"));
});

test("an outcome this build does not know sorts last and does not throw", () => {
  assert.equal(outcomeRank("something new"), 99);
  assert.deepEqual(outcomeTag("something new"), { label: null, tone: "plain" });
  assert.doesNotThrow(() => [{ outcome: "not run" }, { outcome: "x" }, { outcome: "fail" }].sort((a, b) => outcomeRank(a.outcome) - outcomeRank(b.outcome)));
});

test("every tag label is a message of the English catalogue", () => {
  const ftl = readFileSync(new URL("../../locales/en/app.ftl", import.meta.url), "utf8");
  for (const [outcome, o] of Object.entries(OUTCOMES)) assert.match(ftl, new RegExp(`^${o.label} = `, "m"), outcome);
});
