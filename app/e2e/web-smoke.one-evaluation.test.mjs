// Web end-to-end for a follow-up of issue #1096 (spec 0017, D13): after Create the app evaluates the new Design
// once before damage stability runs, and once after it, never twice in a row; and the damage progress says what
// it counts. Run by e2e/run-web.sh with the other web-smoke files.
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
const sessions = [];
async function start() {
  const b = await openBrowser(chrome());
  sessions.push(b);
  return b;
}
const until = (b, what, check, timeout = 60000) =>
  b.waitUntil(check, { timeout, interval: 100, timeoutMsg: `timed out waiting for: ${what}` });
const testid = (id) => `[data-testid=${id}]`;
const exists = async (b, id) => (await b.$(testid(id))).isExisting();
/** A click from the page itself: it needs no drawn frame, so it works while the frames are held. */
const press = (b, id) => b.execute((sel) => document.querySelector(sel).click(), testid(id));
const evaluations = (b) => b.execute(() => window.__evaluations);

before(async () => {
  ({ server, origin } = await serve(DIST));
});
after(async () => {
  await closeBrowsers();
  server?.close();
});

describe("After Create the app evaluates once before damage stability and once after (follow-up of #1096)", () => {
  it("evaluates the new Design once, even when an earlier evaluation still waits for a frame; then once after the damage run", async () => {
    const b = await start();
    await b.url(`${origin}/ovd/app`);
    await until(b, "the notice", () => exists(b, "notice"));
    await (await b.$("button=I understand")).click();
    await until(b, "the Start page", () => exists(b, "start-page"));

    // Count the evaluations, and hold the page's frames: an evaluation asked for now waits to draw the sign.
    await b.execute(() => {
      window.__evaluations = 0;
      const p = window.__ovdPlatform;
      const real = p.evaluate.bind(p);
      p.evaluate = (...a) => {
        window.__evaluations += 1;
        return real(...a);
      };
      window.__heldFrames = [];
      window.__realFrame = window.requestAnimationFrame;
      window.requestAnimationFrame = (f) => window.__heldFrames.push(f);
    });
    // The skip link takes the blank starter Design: its evaluation waits for a frame, and the Start page stays
    // open until it ends. Before that frame comes: the example, and Create.
    await press(b, "start-skip");
    await press(b, "start-example");
    await until(b, "the example list", () => exists(b, "start-examples"));
    await press(b, "start-example-coastal-lng-barge");
    await until(b, "the review step", () => exists(b, "wizard-create"));
    await press(b, "wizard-create");
    await until(b, "the wizard to close", async () => !(await exists(b, "wizard")));
    assert.equal(await evaluations(b), 0, "nothing evaluated while the frames were held");

    // Let the frames come: both evaluations stop waiting, and only the latest one runs.
    await b.execute(() => {
      window.requestAnimationFrame = window.__realFrame;
      for (const f of window.__heldFrames.splice(0)) window.__realFrame(f);
    });
    await until(b, "damage stability to start by itself", () => exists(b, "summary-provisional"), 120000);
    assert.equal(await evaluations(b), 1, "one evaluation of the new Design before the damage run");
    // The progress says what it counts.
    assert.match(await (await b.$(testid("summary-damage"))).getText(), /of about [\d,]+ cases, counting every layout the search tries/);

    await press(b, "summary-damage-cancel");
    await until(b, "the run to end", async () => !(await exists(b, "summary-provisional")), 60000);
    await until(b, "the evaluation after the run", async () => (await evaluations(b)) >= 2);
    await new Promise((r) => setTimeout(r, 1000));
    assert.equal(await evaluations(b), 2, "one evaluation after the damage run, and no more");
  });
});
