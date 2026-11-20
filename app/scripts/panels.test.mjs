// Unit tests for the Outputs panel rule of src/lib/panels.ts (issue #1097). Run: npm run test:unit
// Label: example
import test from "node:test";
import assert from "node:assert/strict";
import { outputsShown, toggleOutputs } from "../src/lib/panels.ts";

test("on the Summary the Outputs panel is closed, whatever the saved choice", () => {
  assert.equal(outputsShown({ view: "summary", outputs: true, onSummary: false }), false);
  assert.equal(outputsShown({ view: "summary", outputs: false, onSummary: false }), false);
});

test("on the Summary the Outputs panel shows once the designer opened it there", () => {
  assert.equal(outputsShown({ view: "summary", outputs: false, onSummary: true }), true);
});

test("on any other view the saved choice decides, as before", () => {
  for (const view of ["side", "stability", "rules", "capital"]) {
    assert.equal(outputsShown({ view, outputs: true, onSummary: false }), true);
    assert.equal(outputsShown({ view, outputs: false, onSummary: true }), false);
  }
});

test("the Outputs panel button on the Summary changes the Summary's choice only", () => {
  const state = { view: "summary", outputs: false, onSummary: false };
  const opened = toggleOutputs(state);
  assert.deepEqual(opened, { view: "summary", outputs: false, onSummary: true });
  assert.equal(outputsShown(opened), true);
  assert.deepEqual(toggleOutputs(opened), state);
});

test("the Outputs panel button on another view changes the saved choice only", () => {
  const state = { view: "side", outputs: true, onSummary: true };
  assert.deepEqual(toggleOutputs(state), { view: "side", outputs: false, onSummary: true });
});

test("the designer's choice on the Summary is kept when they go to a drawing and back", () => {
  let state = toggleOutputs({ view: "summary", outputs: true, onSummary: false });
  state = { ...state, view: "side" };
  assert.equal(outputsShown(state), true);
  state = { ...state, view: "summary" };
  assert.equal(outputsShown(state), true);
});
