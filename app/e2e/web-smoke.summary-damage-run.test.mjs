// Web end-to-end for the Summary when damage stability ends (spec 0017, D7 follow-up): everything stays in sight,
// and nothing moves when the run ends and the drawing is redrawn. The damage run is a stand-in that ends when the test
// says, and the evaluation after it gives a drawing of another shape: what is checked is the Summary's layout, so no
// whole damage run (half a minute on a quiet machine, minutes on a busy one) is needed. Run by e2e/run-web.sh with
// the other web-smoke files.
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
const until = (b, what, check, timeout = 30000) =>
  b.waitUntil(check, { timeout, interval: 100, timeoutMsg: `timed out waiting for: ${what}` });
const testid = (id) => `[data-testid=${id}]`;
const exists = async (b, id) => (await b.$(testid(id))).isExisting();
const click = async (b, id) => (await b.$(testid(id))).click();
const box = (b, id) =>
  b.execute((sel) => {
    const r = document.querySelector(sel).getBoundingClientRect();
    return { top: r.top, bottom: r.bottom, height: r.height };
  }, testid(id));

/** A first start at 1280 × 800, then the worked example through the wizard's Create, with the damage run held. */
async function createExample(b) {
  await b.setWindowSize(1280, 800);
  await b.url(`${origin}/ovd/app`);
  await until(b, "the notice", () => exists(b, "notice"));
  await (await b.$("button=I understand")).click();
  await until(b, "the Start page", () => exists(b, "start-page"));
  await click(b, "start-example");
  await until(b, "the example list", () => exists(b, "start-examples"));
  await click(b, "start-example-coastal-lng-barge");
  await until(b, "the review step", () => exists(b, "wizard-create"));
  // The damage run Create starts: it reports its first progress and ends only when the test calls __endDamage.
  await b.execute(() => {
    const p = window.__ovdPlatform;
    p.runDamage = (option, onProgress) =>
      new Promise((resolve) => {
        onProgress({ option, done: 0, total: 106 });
        window.__endDamage = () => resolve({ option, outcome: "done", message: null });
      });
  });
  await click(b, "wizard-create");
  await until(b, "the wizard to close", async () => !(await exists(b, "wizard")));
  await until(b, "the picture of the vessel", () => exists(b, "summary-vessel-side"), 120000);
}
/** The verdict, the damage line, the scope line, the picture and the numbers are all in the Summary's box. */
async function allInSight(b, ids) {
  const summary = await box(b, "summary");
  assert.equal(await b.execute(() => document.querySelector("[data-testid=summary]").scrollTop), 0, "not scrolled");
  for (const id of ids) {
    const r = await box(b, id);
    assert.ok(r.height > 0 && r.top >= summary.top && r.bottom <= summary.bottom + 0.5, `${id} in sight: ${JSON.stringify({ r, summary })}`);
  }
}

before(async () => {
  ({ server, origin } = await serve(DIST));
});
after(async () => {
  await closeBrowsers();
  server?.close();
});

/** Boxes measured from the Summary's own top: the message line above the app goes when the run ends and moves
 * the whole page, which is not the Summary's doing. */
async function inSummary(b) {
  const top = (await box(b, "summary")).top;
  const side = await box(b, "summary-vessel-side");
  const numbers = await box(b, "summary-numbers");
  return { side: { top: side.top - top, height: side.height }, numbersTop: numbers.top - top };
}

describe("The Summary when damage stability ends", () => {
  it("at 1280 × 800 everything stays in sight, and nothing moves when the run ends and the drawing is redrawn", async () => {
    const b = await openBrowser(chrome());
    await createExample(b);
    await until(b, "damage stability running", () => exists(b, "summary-provisional"), 60000);
    await allInSight(b, ["summary-verdict", "summary-provisional", "summary-damage", "summary-scope-line", "summary-vessel-side", "summary-numbers"]);
    const before = await inSummary(b);
    const sheetBefore = await b.execute(() => document.querySelector("[data-testid=summary-vessel-side] svg").getAttribute("viewBox"));
    // The evaluation after the run gives a drawing of another shape (the plan's sheet where the side elevation was),
    // a harder redraw than a real run's; then the run ends.
    await b.execute(() => {
      const p = window.__ovdPlatform;
      const real = p.evaluate.bind(p);
      p.evaluate = async () => {
        const e = await real();
        for (const o of e.options) if (o.outcome.views) o.outcome.views.side = o.outcome.views.plan;
        return e;
      };
      window.__endDamage();
    });
    await until(b, "the line to go", async () => !(await exists(b, "summary-provisional")));
    await until(b, "the drawing to be redrawn", async () =>
      (await b.execute(() => document.querySelector("[data-testid=summary-vessel-side] svg")?.getAttribute("viewBox"))) !== sheetBefore);
    await allInSight(b, ["summary-verdict", "summary-damage", "summary-scope-line", "summary-vessel-side", "summary-numbers"]);
    const later = await inSummary(b);
    assert.deepEqual(later.side, before.side, "the picture keeps its place and size");
    assert.equal(later.numbersTop, before.numbersTop, "the numbers do not move");
  });
});
