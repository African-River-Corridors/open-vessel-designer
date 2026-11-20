// Unit tests for src/lib/focusTrap.ts (issue #1080, second rework): which element gets focus next in a dialog.
// Run: npm run test:unit
// Label: property
import test from "node:test";
import assert from "node:assert/strict";
import { wrapTarget } from "../src/lib/focusTrap.ts";

const list = ["a", "b", "c"];

test("Tab from the last element goes to the first", () => assert.equal(wrapTarget(list, "c", false), "a"));
test("Shift+Tab from the first element goes to the last", () => assert.equal(wrapTarget(list, "a", true), "c"));
test("inside the list the browser moves focus", () => {
  assert.equal(wrapTarget(list, "b", false), null);
  assert.equal(wrapTarget(list, "b", true), null);
  assert.equal(wrapTarget(list, "a", false), null);
  assert.equal(wrapTarget(list, "c", true), null);
});
test("with focus off the list (the page, or a removed element) Tab enters at the first and Shift+Tab at the last", () => {
  assert.equal(wrapTarget(list, null, false), "a");
  assert.equal(wrapTarget(list, null, true), "c");
  assert.equal(wrapTarget(list, "gone", false), "a");
  assert.equal(wrapTarget(list, "gone", true), "c");
});
test("one element keeps focus on itself", () => {
  assert.equal(wrapTarget(["a"], "a", false), "a");
  assert.equal(wrapTarget(["a"], "a", true), "a");
});
test("an empty list moves nothing", () => assert.equal(wrapTarget([], null, false), null));
