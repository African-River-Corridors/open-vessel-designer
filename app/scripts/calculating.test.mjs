// Unit tests for src/lib/calculating.ts and src/lib/sayOrder.ts (issue #1082). Run: npm run test:unit
// Label: property
import test from "node:test";
import assert from "node:assert/strict";
import { Calculating } from "../src/lib/calculating.ts";
import { wordingOf } from "../src/lib/sayOrder.ts";

const wait = (ms) => new Promise((r) => setTimeout(r, ms));
const sign = (afterMs = 20) => {
  const log = [];
  return { log, c: new Calculating(afterMs, (on) => log.push(on)) };
};

test("a quick run never shows the sign", async () => {
  const { c, log } = sign();
  await c.during(async () => 1);
  await wait(40);
  assert.deepEqual(log, [false]);
});

test("a slow run shows the sign, then hides it", async () => {
  const { c, log } = sign();
  await c.during(() => wait(60));
  assert.deepEqual(log, [true, false]);
});

test("when two runs overlap, the first to finish leaves the sign on", async () => {
  const { c, log } = sign();
  const first = c.during(() => wait(40));
  await wait(5);
  const second = c.during(() => wait(120));
  await first;
  assert.equal(c.running, 1);
  assert.equal(log.at(-1), true, "still on while the second run goes");
  await second;
  assert.equal(c.running, 0);
  assert.equal(log.at(-1), false);
});

test("an error does not leave the sign on, and the error goes on", async () => {
  const { c, log } = sign();
  await assert.rejects(c.during(async () => { await wait(40); throw new Error("boom"); }), /boom/);
  assert.equal(log.at(-1), false);
  assert.equal(c.running, 0);
});

// Issue #1096: QA found a gap in the run counter; two quick runs that overlap turn the sign off once, after the last.
test("two quick overlapping runs turn the sign off once, after the last run ends", async () => {
  const { c, log } = sign(30);
  const first = c.during(() => wait(10));
  const second = c.during(() => wait(20));
  await first;
  assert.deepEqual(log, [], "nothing yet: the second run still goes");
  await second;
  assert.deepEqual(log, [false], "off once, never on: both were quicker than the wait");
});

// Issue #1096: on the web an evaluation blocks the page, so the timer cannot fire while it runs. A run said to be
// slow shows the sign at once and lets the page paint it before the work starts.
test("a run said to be slow shows the sign at once and lets the page paint before the work starts", async () => {
  const order = [];
  const c = new Calculating(1000, (on) => order.push(on ? "on" : "off"), async () => order.push("painted"));
  await c.during(async () => order.push("work"), true);
  assert.deepEqual(order, ["on", "painted", "work", "off"]);
});

test("a slow run inside a run that already shows the sign does not paint again", async () => {
  const order = [];
  const c = new Calculating(1000, (on) => order.push(on ? "on" : "off"), async () => order.push("painted"));
  await c.during(() => c.during(async () => order.push("inner"), true), true);
  assert.deepEqual(order, ["on", "painted", "inner", "off"]);
});

test("a slow run inside a run whose timer has not fired yet shows the sign at once", async () => {
  const order = [];
  const c = new Calculating(1000, (on) => order.push(on ? "on" : "off"), async () => order.push("painted"));
  await c.during(() => c.during(async () => order.push("inner"), true));
  assert.deepEqual(order, ["on", "painted", "inner", "off"]);
});

test("a slow run inside a run whose timer has already turned the sign on still waits for the page to draw it", async () => {
  const order = [];
  const c = new Calculating(10, (on) => order.push(on ? "on" : "off"), async () => order.push("painted"));
  await c.during(async () => {
    await wait(30); // the timer fires: the sign is on in the page, but not yet drawn
    await c.during(async () => order.push("inner"), true);
  });
  assert.deepEqual(order, ["on", "painted", "inner", "off"]);
});

test("after redraw, the next slow run waits for one more frame even when the sign was already drawn", async () => {
  const order = [];
  const c = new Calculating(1000, (on) => order.push(on ? "on" : "off"), async () => order.push("painted"));
  await c.during(async () => {
    await c.during(async () => order.push("first"), true);
    c.redraw();
    await c.during(async () => order.push("second"), true);
  }, true);
  assert.deepEqual(order, ["on", "painted", "first", "painted", "second", "off"]);
});

const finder = (table) => (id) => table[id] ?? null;

test("the app wording in the Display Language wins over everything", () => {
  const out = wordingOf("engine", "m", finder({ "app-plain-m": "spanish plain", m: "spanish engine" }), finder({ "app-plain-m": "english plain" }), false);
  assert.equal(out, "spanish plain");
});

test("without it in the Display Language, the English app wording wins over the engine text and its translation", () => {
  const out = wordingOf("engine", "m", finder({ m: "spanish engine" }), finder({ "app-plain-m": "english plain" }), false);
  assert.equal(out, "english plain");
  assert.equal(wordingOf("engine", "m", finder({}), finder({ "app-plain-m": "english plain" }), true), "english plain");
});

test("without app wording, English shows the engine text as it is", () => {
  assert.equal(wordingOf("engine", "m", finder({ m: "ignored" }), finder({}), true), "engine");
});

test("without app wording, another language shows its translation, else the engine text", () => {
  assert.equal(wordingOf("engine", "m", finder({ m: "spanish engine" }), finder({}), false), "spanish engine");
  assert.equal(wordingOf("engine", "m", finder({}), finder({}), false), "engine");
});
