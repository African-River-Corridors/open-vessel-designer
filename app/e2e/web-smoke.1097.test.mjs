// Web end-to-end for issue #1097 (spec 0017, D7): the Summary takes the full height of the main area. Run by
// e2e/run-web.sh with the other web-smoke files.
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { remote } from "webdriverio";
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
const sessions = [];
async function start() {
  const b = await remote({ logLevel: "warn", strictSelectors: false, capabilities: chrome() });
  sessions.push(b);
  return b;
}
const until = (b, what, check, timeout = 30000) =>
  b.waitUntil(check, { timeout, interval: 100, timeoutMsg: `timed out waiting for: ${what}` });
const testid = (id) => `[data-testid=${id}]`;
const exists = async (b, id) => (await b.$(testid(id))).isExisting();
const click = async (b, id) => (await b.$(testid(id))).click();
const outputsOpen = (b) => b.execute(() => !document.querySelector("section.bottom").hidden);
const pressed = async (b, id) => (await (await b.$(testid(id))).getAttribute("aria-pressed")) === "true";

/** A first start at 1280 × 800, then the example through the wizard's Create. */
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
  await until(b, "the Summary's numbers", () => exists(b, "summary-numbers"), 60000);
}

before(async () => {
  ({ server, origin } = await serve(DIST));
});
after(async () => {
  for (const b of sessions) await b.deleteSession().catch(() => {});
  server?.close();
});

describe("The Summary takes the full height of the main area (issue #1097)", () => {
  it("after Create at 1280 × 800 shows the three headings and the headline numbers without scrolling, with the Outputs panel closed; a drawing view shows the Outputs panel", async () => {
    const b = await start();
    await createExample(b);
    assert.equal(await outputsOpen(b), false, "the Outputs panel is closed on the Summary");
    assert.equal(await pressed(b, "toggle-outputs-panel"), false, "the button says so");

    const boxes = await b.execute(() => {
      const box = (el) => {
        const r = el.getBoundingClientRect();
        return { top: r.top, bottom: r.bottom };
      };
      const summary = document.querySelector("[data-testid=summary]");
      return {
        summary: box(summary),
        scrollTop: summary.scrollTop,
        headings: [...summary.querySelectorAll("h4")].map(box),
        numbers: box(document.querySelector("[data-testid=summary-numbers]")),
        main: box(document.querySelector("section.main")),
        window: window.innerHeight,
      };
    });
    assert.equal(boxes.headings.length, 3, "three headings");
    assert.equal(boxes.scrollTop, 0, "the view has not been scrolled");
    for (const h of [...boxes.headings, boxes.numbers]) {
      assert.ok(h.bottom <= boxes.summary.bottom + 0.5, `in sight without scrolling: ${JSON.stringify({ h, summary: boxes.summary })}`);
    }
    assert.ok(boxes.main.bottom >= boxes.window - 2, `the main area runs to the foot of the window: ${JSON.stringify(boxes)}`);

    // A drawing view shows the Outputs panel, as before.
    await click(b, "group-drawings");
    await until(b, "the Outputs panel on a drawing", () => outputsOpen(b));
    assert.equal(await pressed(b, "toggle-outputs-panel"), true);
    // Back on the Summary it closes again.
    await click(b, "group-summary");
    await until(b, "the Outputs panel to close on the Summary", async () => !(await outputsOpen(b)));
  });

  it("keeps the designer's own choice to open the Outputs panel on the Summary for the session, and Show in Checks opens it", async () => {
    const b = await start();
    await createExample(b);
    await click(b, "toggle-outputs-panel");
    await until(b, "the Outputs panel on the Summary", () => outputsOpen(b));
    await click(b, "group-drawings");
    await click(b, "group-summary");
    assert.equal(await outputsOpen(b), true, "still open on the Summary");
    // Closed again by the designer, a Summary link to a check opens it.
    await click(b, "toggle-outputs-panel");
    await until(b, "the Outputs panel to close", async () => !(await outputsOpen(b)));
    const link = await b.$(testid("summary-show-check"));
    if (await link.isExisting()) {
      await link.click();
      await until(b, "Show in Checks to open the Outputs panel", () => outputsOpen(b));
    }
  });
});
