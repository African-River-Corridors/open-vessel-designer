// Unit tests for src/lib/inputsShown.ts (issue #1034 Phase 2). Run: npm run test:unit
// Label: property
import test from "node:test";
import assert from "node:assert/strict";
import { isShown } from "../src/lib/inputsShown.ts";

const number = (name) => ({ name, group: "Limits", kind: "number", required: false, nullable: true });
const held = (v) => ({ value: v, source: "default" });

test("a river-only Option shows both river limits although it holds neither", () => {
  for (const name of ["draught_limit_m", "air_draft_limit_m"]) {
    assert.equal(isShown(number(name), {}, true), true, name);
  }
});

test("a coastal-and-river Option does not show a river limit it does not hold", () => {
  for (const name of ["draught_limit_m", "air_draft_limit_m"]) {
    assert.equal(isShown(number(name), {}, false), false, name);
    assert.equal(isShown(number(name), { [name]: held(null) }, false), true, name);
  }
});

test("river-only draws only the two river limits unasked; other absent inputs stay hidden", () => {
  assert.equal(isShown(number("beam_max_m"), {}, true), false);
});

test("a required choice the Option does not hold is still shown (issue #925)", () => {
  assert.equal(isShown({ name: "service_area", group: "Service", kind: "choice", required: true }, {}, false), true);
});
