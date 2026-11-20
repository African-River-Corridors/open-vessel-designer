// Unit tests for src/lib/vessel.ts: the Summary's picture of the vessel. Run: npm run test:unit
// Label: example
import test from "node:test";
import assert from "node:assert/strict";
import { frameOf, vesselDims } from "../src/lib/vessel.ts";

test("the frame is the union of the hull's boxes, with a margin on every side", () => {
  const f = frameOf([
    { x: 125, y: 80, width: 140, height: 20 },
    { x: 130, y: 70, width: 20, height: 10 },
  ], 0.05);
  // Union 125..265 x 70..100: 140 by 30; the margin is 5 % of the longer side, 7.
  assert.deepEqual(f, { x: 118, y: 63, width: 154, height: 44 });
});

test("a line has no height but still counts; an empty box does not", () => {
  const f = frameOf([
    { x: 10, y: 50, width: 100, height: 0 },
    { x: 0, y: 0, width: 0, height: 0 },
  ], 0);
  assert.deepEqual(f, { x: 10, y: 50, width: 100, height: 0 });
});

test("no boxes give no frame", () => {
  assert.equal(frameOf([], 0.05), null);
  assert.equal(frameOf([{ x: 1, y: 1, width: 0, height: 0 }], 0.05), null);
});

test("the principal dimensions come from the results, in metres as the strip shows them", () => {
  const results = { dims: { loa_m: 140, beam_m: 29, depth_m: 12.004 } };
  assert.deepEqual(vesselDims(results), { loa: "140", beam: "29", depth: "12" });
  assert.deepEqual(vesselDims({ dims: { loa_m: 85.5, beam_m: 11.45, depth_m: 3.2 } }), { loa: "85.5", beam: "11.45", depth: "3.2" });
});

test("without all three dimensions there are none to name", () => {
  assert.equal(vesselDims(null), null);
  assert.equal(vesselDims({ dims: { loa_m: 140, beam_m: 29 } }), null);
  assert.equal(vesselDims({ dims: { loa_m: 140, beam_m: Number.NaN, depth_m: 12 } }), null);
});
