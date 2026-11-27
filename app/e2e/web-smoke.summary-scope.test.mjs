// Web end-to-end for the Summary's scope line (spec 0017, D7 follow-up): the long scope sentence behind a one-line
// summary with its two counts, and the room it frees given to the picture of the vessel. Run by e2e/run-web.sh
// with the other web-smoke files.
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
// A closed details element hides its body with content-visibility, and a layout query can still give the hidden
// body a size: checkVisibility says whether it is drawn.
const sentenceShown = (b) => b.execute(() => document.querySelector("[data-testid=summary-scope]").checkVisibility({ contentVisibilityAuto: true }));
/** What the result says, from the engine: the Rule Sets and the blocking Rules not evaluated. */
const engineCounts = (b) =>
  b.execute(async () => {
    const trace = (await window.__ovdPlatform.evaluate()).options[0].outcome.results.rule_trace;
    return { sets: trace.rule_sets.length, notEvaluated: trace.verdict.blocking_not_evaluated.length };
  });

/** A first start at 1280 × 800, then the worked example through the wizard's Create. */
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

describe("The Summary's scope line", () => {
  it("at 1280 × 800, while damage stability runs, the verdict, the damage line, the scope line, a larger picture and the numbers are in sight", async () => {
    const b = await openBrowser(chrome());
    await createExample(b);
    await until(b, "damage stability running", () => exists(b, "summary-provisional"), 60000);
    await allInSight(b, ["summary-verdict", "summary-provisional", "summary-damage", "summary-scope-line", "summary-vessel-side", "summary-numbers"]);
    assert.ok((await box(b, "summary-vessel-side")).height >= 110, "the side elevation is about twice its old 64 px");
  });

  it("the line gives the two counts from the result; the sentence opens and closes, and stays as left for the session", async () => {
    const b = await openBrowser(chrome());
    await createExample(b);
    const counts = await engineCounts(b);
    const line = (await (await b.$(testid("summary-scope-line"))).getText()).trim();
    assert.ok(line.startsWith(`${counts.sets} Rule Sets checked; ${counts.notEvaluated} blocking Rules were not evaluated.`), line);
    assert.ok(line.endsWith(" Show the scope"), line);
    assert.doesNotMatch(line, /\.\S/, "a space after every full stop");
    assert.equal(await sentenceShown(b), false, "the sentence is folded at first");

    // Each read waits for what the click changes: the page records the open state on the details element's toggle
    // event, a moment after the sentence is drawn, and only then does the label change.
    const lineText = async () => (await (await b.$(testid("summary-scope-line"))).getText()).trim();
    await click(b, "summary-scope-line");
    await until(b, "the sentence to open", () => sentenceShown(b));
    await until(b, "the label to say Hide the scope", async () => (await lineText()).endsWith("Hide the scope"));
    assert.match((await (await b.$(testid("summary-scope"))).getText()).trim(), /^Service Area /);

    // Another view and back: the Summary keeps it open. Wait for each view to change, not only to exist.
    await click(b, "group-drawings");
    await until(b, "the Summary to go", async () => !(await exists(b, "summary-scope-line")));
    await click(b, "group-summary");
    await until(b, "the Summary again", () => exists(b, "summary-scope-line"));
    await until(b, "the sentence still open", () => sentenceShown(b), 5000);
    assert.ok((await lineText()).endsWith("Hide the scope"), "the label says it is open");

    await click(b, "summary-scope-line");
    await until(b, "the sentence to close", async () => !(await sentenceShown(b)));
    await until(b, "the label to say Show the scope", async () => (await lineText()).endsWith("Show the scope"));
    // Closed, the line still says how many blocking Rules were not evaluated.
    assert.match(await lineText(), /\d+ blocking Rules were not evaluated\./);
  });
});
