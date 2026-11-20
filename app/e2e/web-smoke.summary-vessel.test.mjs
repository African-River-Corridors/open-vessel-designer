// Web end-to-end for the Summary's picture of the vessel (spec 0017, D7 follow-up): the side elevation and the
// plan, framed on the hull, at the top of the Summary's right column. Run by e2e/run-web.sh with the other
// web-smoke files.
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { closeBrowsers, openBrowser } from "./browser.mjs";
import { serve } from "./web-server.mjs";

const here = import.meta.dirname;
const DIST = path.resolve(here, "../dist-web-test");

const chrome = () => ({
  browserName: "chrome",
  "wdio:enforceWebDriverClassic": true,
  "goog:chromeOptions": {
    ...(process.env.CHROME_BINARY ? { binary: process.env.CHROME_BINARY } : fs.existsSync("/Applications/Google Chrome.app") ? { binary: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" } : {}),
    args: ["--headless=new", "--no-sandbox", "--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist", "--window-size=1500,950"],
  },
});

let origin, server;
async function start() {
  return openBrowser(chrome());
}
const until = (b, what, check, timeout = 30000) =>
  b.waitUntil(check, { timeout, interval: 100, timeoutMsg: `timed out waiting for: ${what}` });
const testid = (id) => `[data-testid=${id}]`;
const exists = async (b, id) => (await b.$(testid(id))).isExisting();
const click = async (b, id) => (await b.$(testid(id))).click();
const box = (b, id) =>
  b.execute((sel) => {
    const r = document.querySelector(sel).getBoundingClientRect();
    return { top: r.top, bottom: r.bottom, left: r.left, right: r.right, height: r.height, width: r.width };
  }, testid(id));

/** A first start at 1280 × 800, to the wizard's Review of the worked example. */
async function toReview(b) {
  await b.setWindowSize(1280, 800);
  await b.url(`${origin}/ovd/app`);
  await until(b, "the notice", () => exists(b, "notice"));
  await (await b.$("button=I understand")).click();
  await until(b, "the Start page", () => exists(b, "start-page"));
  await click(b, "start-example");
  await until(b, "the example list", () => exists(b, "start-examples"));
  await click(b, "start-example-coastal-lng-barge");
  await until(b, "the review step", () => exists(b, "wizard-create"));
}
async function createExample(b) {
  await toReview(b);
  await click(b, "wizard-create");
  await until(b, "the wizard to close", async () => !(await exists(b, "wizard")));
  await until(b, "the picture of the vessel", () => exists(b, "summary-vessel-side"), 120000);
}

before(async () => {
  ({ server, origin } = await serve(DIST));
});
after(async () => {
  await closeBrowsers();
  server?.close();
});

describe("The Summary shows the vessel", () => {
  it("after Create at 1280 × 800 the verdict, the headline numbers and the picture are all in sight without scrolling", async () => {
    const b = await start();
    await createExample(b);
    const summary = await box(b, "summary");
    assert.equal(await b.execute(() => document.querySelector("[data-testid=summary]").scrollTop), 0, "the Summary has not been scrolled");
    for (const id of ["summary-verdict", "summary-numbers", "summary-vessel-side"]) {
      const r = await box(b, id);
      assert.ok(r.height > 0 && r.top >= summary.top && r.bottom <= summary.bottom + 0.5, `${id} in sight: ${JSON.stringify({ r, summary })}`);
    }
    assert.ok((await box(b, "summary-vessel-side")).height >= 60, "the side elevation is large enough to read");
    // The picture is the hull, framed: not the whole A3 sheet, and with hull geometry drawn.
    const sheet = await b.execute(() => {
      const svg = document.querySelector("[data-testid=summary-vessel-side] svg");
      const shown = [...svg.querySelectorAll('[data-origin="viewport.main"]')].filter((e) => getComputedStyle(e).display !== "none").length;
      const hidden = [...svg.querySelectorAll('[data-layer="TITLEBLOCK"]')].every((e) => getComputedStyle(e).display === "none");
      return { viewBox: svg.getAttribute("viewBox"), shown, hidden };
    });
    assert.notEqual(sheet.viewBox, "0 0 420 297", "framed on the hull");
    assert.ok(sheet.shown > 0, "hull geometry is drawn");
    assert.equal(sheet.hidden, true, "the title block is left out");
    // Its text names the Option and its principal dimensions.
    const label = await (await b.$(testid("summary-vessel-side"))).getAttribute("aria-label");
    assert.match(label, /^Side elevation of LNG 140x29: 140 m long, 29 m beam, 12 m deep$/);
    assert.match(await (await b.$(testid("summary-vessel-plan"))).getAttribute("aria-label"), /^Plan of LNG 140x29: 140 m long, 29 m beam$/);
  });

  it("on a taller window the plan shows under the side elevation; a click opens the side elevation in Drawings, Enter on the plan the plan", async () => {
    const b = await start();
    await createExample(b);
    await b.setWindowSize(1500, 950);
    await until(b, "the plan under the side elevation", async () => (await box(b, "summary-vessel-plan")).height > 0);
    const side = await box(b, "summary-vessel-side");
    const plan = await box(b, "summary-vessel-plan");
    assert.ok(plan.top >= side.bottom, "the plan is under the side elevation");
    await click(b, "summary-vessel-side");
    await until(b, "the side elevation view", async () => (await (await b.$("[data-view]")).getAttribute("data-view")) === "side");
    await click(b, "group-summary");
    await until(b, "the Summary again", () => exists(b, "summary-vessel-plan"));
    await b.execute(() => document.querySelector("[data-testid=summary-vessel-plan]").focus());
    await b.keys("Enter");
    await until(b, "the plan view", async () => (await (await b.$("[data-view]")).getAttribute("data-view")) === "plan");
  });

  it("while the Design is calculated the picture's place says so, with no drawing of another Design", async () => {
    const b = await start();
    await toReview(b);
    // Hold the evaluation without holding the page, so the Summary can be read while it calculates.
    await b.execute(() => {
      const p = window.__ovdPlatform;
      const real = p.evaluate.bind(p);
      p.evaluate = async () => {
        await new Promise((r) => setTimeout(r, 3000));
        return real();
      };
    });
    await click(b, "wizard-create");
    await until(b, "the Summary calculating", () => exists(b, "summary-calculating"), 60000);
    assert.equal(await exists(b, "summary-vessel-pending"), true, "the picture's place says the Design is being calculated");
    assert.equal(await exists(b, "summary-vessel"), false, "no picture yet");
    await until(b, "the picture", () => exists(b, "summary-vessel-side"), 120000);
    assert.equal(await exists(b, "summary-vessel-pending"), false);
  });

  it("a Design with no result shows no picture", async () => {
    const b = await start();
    await b.url(`${origin}/ovd/app`);
    await until(b, "the notice", () => exists(b, "notice"));
    await (await b.$("button=I understand")).click();
    await until(b, "the Start page", () => exists(b, "start-page"));
    await click(b, "start-skip");
    // The starter Design has no Service Area, so it has no result.
    await until(b, "the not-calculated verdict", async () => (await (await b.$(testid("summary-verdict"))).getAttribute("data-kind")) === "not-calculated");
    assert.equal(await exists(b, "summary-vessel"), false);
  });

  it("nothing moves when damage stability ends and the drawing is redrawn", async () => {
    const b = await start();
    await createExample(b);
    await until(b, "damage stability running", () => exists(b, "summary-provisional"), 60000);
    // Measured from the Summary's own top: the message line above the app goes when the run ends, and moves
    // the whole page, which is not the picture's doing.
    const inSummary = async () => {
      const top = (await box(b, "summary")).top;
      const side = await box(b, "summary-vessel-side");
      const numbers = await box(b, "summary-numbers");
      return { side: { top: side.top - top, height: side.height, width: side.width }, numbersTop: numbers.top - top };
    };
    const before = await inSummary();
    await until(b, "damage stability to end", async () => (await (await b.$(testid("summary-damage"))).getAttribute("data-status")) === "run", 400000);
    await until(b, "the line to go", async () => !(await exists(b, "summary-provisional")));
    const later = await inSummary();
    assert.deepEqual(later.side, before.side, "the picture keeps its box");
    assert.equal(later.numbersTop, before.numbersTop, "the numbers do not move");
  });
});
