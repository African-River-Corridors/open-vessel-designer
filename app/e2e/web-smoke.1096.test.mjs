// Web end-to-end for issue #1096 (spec 0017, D13): the "Calculating…" sign beside the Input panel, shown for
// every evaluation, and the Summary while a new Design is calculated. Run by e2e/run-web.sh with the other
// web-smoke files.
//
// On the web the engine runs on the page's own thread: while it calculates, no timer fires and nothing is
// drawn. These tests hold the page the same way (a busy wait before the real evaluation), and they record the
// sign with an animation-frame observer, so they see only what the page really drew.
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { remote } from "webdriverio";
import { serve } from "./web-server.mjs";
import { showAllInputs } from "./inputs-mode.mjs";

const here = import.meta.dirname;
const GAS_FIXTURE = path.resolve(here, "../../crates/ovd-format/tests/fixtures/gas_igc_2g_self_propelled.vdesign");
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
  // One browser at a time: Chrome may hold back the animation frames of a window that is not in front, and
  // these tests read what was drawn frame by frame.
  while (sessions.length > 0) await sessions.pop().deleteSession().catch(() => {});
  const b = await remote({ logLevel: "warn", strictSelectors: false, capabilities: chrome() });
  sessions.push(b);
  return b;
}
const until = (b, what, check, timeout = 30000) =>
  b.waitUntil(check, { timeout, interval: 100, timeoutMsg: `timed out waiting for: ${what}` });
const testid = (id) => `[data-testid=${id}]`;
const exists = async (b, id) => (await b.$(testid(id))).isExisting();
const click = async (b, id) => (await b.$(testid(id))).click();

/** Accept the first-run notice; the Start page opens after it. */
async function firstStart(b) {
  await b.url(`${origin}/ovd/app`);
  await until(b, "the notice", () => exists(b, "notice"));
  await (await b.$("button=I understand")).click();
  await until(b, "the Start page", () => exists(b, "start-page"));
}
/** Past the notice and the Start page, on the starter Design. */
async function openApp(b) {
  await firstStart(b);
  await click(b, "start-skip");
  await until(b, "the Start page to close", async () => !(await exists(b, "start-page")));
}
/** Open the gas barge fixture through the web file picker. */
async function openGasBarge(b) {
  await b.execute(() => {
    HTMLInputElement.prototype.click = function () {};
  });
  await (await b.$("button=Open")).click();
  const picker = await b.$(testid("file-picker"));
  await picker.waitForExist({ timeout: 5000 });
  await picker.setValue(GAS_FIXTURE);
}

/**
 * Record, frame by frame, where the sign was drawn: "input" (the Input panel), "views" (the views' tab row),
 * "other", or "off". An animation-frame callback runs just before the frame is painted, so each entry is what
 * the page drew. Also notes whether the Summary's "being calculated" line was ever drawn.
 */
const record = (b) =>
  b.execute(() => {
    window.__seen = [];
    window.__summaryCalculating = false;
    window.__trace = [];
    window.__t0 = performance.now();
    if (window.__recording) return;
    window.__recording = true;
    const tick = () => {
      const el = document.querySelector("[data-testid=calculating]");
      const where = !el || !el.textContent.trim() ? "off" : el.closest("section.left") ? "input" : el.closest("[data-testid=views-panel]") ? "views" : "other";
      if (window.__seen.at(-1) !== where) window.__seen.push(where);
      const calc = !!document.querySelector("[data-testid=summary-calculating]");
      if (calc) window.__summaryCalculating = true;
      const line = `frame ${where}${calc ? " summary-calculating" : ""}`;
      if (window.__trace.at(-1)?.[1] !== line) window.__trace.push([Math.round(performance.now() - window.__t0), line]);
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
const seen = (b) => b.execute(() => window.__seen);
/** What the page drew and ran since `record`, in time order, for a failure message. */
const trace = async (b) => JSON.stringify(await b.execute(() => window.__trace));
/** Wait until the sign has been drawn and has gone again: one evaluation began and ended. */
async function signCameAndWent(b, what) {
  try {
    await until(b, `the sign to come and go: ${what}`, async () => {
      const s = await seen(b);
      return s.some((x) => x !== "off") && s.at(-1) === "off";
    });
  } catch (e) {
    throw new Error(`${e.message}; trace: ${await trace(b)}`);
  }
}

/** Make every evaluation hold the page for `ms` first, as the engine does on the web for a large Design. */
const holdEvaluations = (b, ms) =>
  b.execute((ms) => {
    const p = window.__ovdPlatform;
    window.__realEvaluate ??= p.evaluate.bind(p);
    p.evaluate = (...args) => {
      window.__trace?.push([Math.round(performance.now() - (window.__t0 ?? 0)), `evaluation held (${document.visibilityState}, sign ${document.querySelector("[data-testid=calculating]")?.textContent.trim() || "off"}, summary-calculating ${!!document.querySelector("[data-testid=summary-calculating]")})`]);
      const end = performance.now() + ms;
      while (performance.now() < end) {
        /* the page is held */
      }
      return window.__realEvaluate(...args);
    };
  }, ms);

const rectOf = (b, selector) =>
  b.execute((sel) => {
    const r = document.querySelector(sel).getBoundingClientRect();
    return { left: r.left, right: r.right, top: r.top, bottom: r.bottom, width: r.width, height: r.height };
  }, selector);
/**
 * The boxes of every control the designer can see, but the sign: drawn, and on top at its own centre (a control
 * scrolled under the Input panel's fixed top, or out of its panel, is not in sight).
 */
const controls = (b) =>
  b.execute(() =>
    [...document.querySelectorAll("button, input, select, summary, a, [role=tab]")]
      .filter((e) => {
        const r = e.getBoundingClientRect();
        if (r.width === 0 || e.closest("[data-testid=calculating]")) return false;
        const top = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
        return top !== null && (e === top || e.contains(top) || top.contains(e));
      })
      .map((e) => {
        const r = e.getBoundingClientRect();
        return { what: e.outerHTML.slice(0, 80), left: r.left, right: r.right, top: r.top, bottom: r.bottom };
      }),
  );
const apart = (a, c) => a.right <= c.left + 0.5 || a.left >= c.right - 0.5 || a.bottom <= c.top + 0.5 || a.top >= c.bottom - 0.5;

before(async () => {
  ({ server, origin } = await serve(DIST));
});
after(async () => {
  for (const b of sessions) await b.deleteSession().catch(() => {});
  server?.close();
});

describe("The Calculating sign beside the Input panel, for every evaluation (issue #1096)", () => {
  it("after Create shows the sign and a Summary that says the Design is being calculated, then the result; while damage stability runs by itself the Summary says the answer may change", async () => {
    const b = await start();
    await b.setWindowSize(1280, 800); // a laptop screen, as the maintainer tried it
    await firstStart(b);
    await click(b, "start-example");
    await until(b, "the example list", () => exists(b, "start-examples"));
    await click(b, "start-example-coastal-lng-barge");
    await until(b, "the review step", () => exists(b, "wizard-create"));
    await holdEvaluations(b, 1500);
    await record(b);
    await click(b, "wizard-create");
    await until(b, "the wizard to close", async () => !(await exists(b, "wizard")));
    await until(b, "the Summary's verdict", () => exists(b, "summary-verdict"));

    assert.equal(await b.execute(() => window.__summaryCalculating), true, `the Summary said the Design was being calculated: ${await trace(b)}`);
    assert.ok((await seen(b)).includes("input"), `the sign was drawn in the Input panel: ${await seen(b)}`);
    assert.equal(await exists(b, "summary-calculating"), false, "the line goes once the result is drawn");

    // Damage stability then runs by itself: the Summary says the answer may change, with the run's progress.
    await until(b, "the automatic damage run", () => exists(b, "summary-provisional"));
    assert.equal((await (await b.$(testid("summary-provisional"))).getText()).trim(), "The answer may still change until damage stability ends.");
    // In sight in the Summary's own box, without scrolling: right under the verdict.
    const inSight = await b.execute(() => {
      const line = document.querySelector("[data-testid=summary-provisional]").getBoundingClientRect();
      const box = document.querySelector("[data-testid=summary]").getBoundingClientRect();
      return line.top >= box.top && line.bottom <= box.bottom;
    });
    assert.equal(inSight, true, "the line is in sight without scrolling the Summary");
    assert.equal(await (await b.$(testid("summary-damage"))).getAttribute("data-status"), "running");
    assert.match(await (await b.$(testid("summary-damage"))).getText(), /\d+ of about [\d,]+ cases, counting every layout the search tries/);
    // When the run ends (here, cancelled), the line goes.
    await click(b, "summary-damage-cancel");
    await until(b, "the run to end", async () => !(await exists(b, "summary-provisional")), 60000);
  });

  it("shows the sign for every evaluation that holds the page: opening a Design, adding and switching an Option, an edit", async () => {
    const b = await start();
    await openApp(b);
    await until(b, "the Summary", () => exists(b, "summary-verdict"));
    await holdEvaluations(b, 1000);

    await record(b);
    await openGasBarge(b);
    await signCameAndWent(b, "opening a Design");
    assert.equal(await b.execute(() => window.__summaryCalculating), true, `the Summary said the opened Design was being calculated: ${await trace(b)}`);
    assert.ok((await seen(b)).includes("input"), "drawn in the Input panel");

    await record(b);
    await click(b, "option-add");
    await signCameAndWent(b, "adding an Option");
    assert.equal(await b.execute(() => window.__summaryCalculating), false, "an Option is not a new Design");

    await record(b);
    await (await b.$(testid("option-picker"))).selectByIndex(0);
    await signCameAndWent(b, "switching the Option");

    await showAllInputs(b); // Essentials hides this input (#1084)
    await record(b);
    await (await b.$('.field[data-input="vent_mast_lowerable"] input[type=checkbox]')).click();
    await signCameAndWent(b, "an edit");
    assert.ok((await seen(b)).includes("input"), "drawn in the Input panel");
  });

  for (const width of [1280, 1000]) {
    it(`at ${width} px the sign shows inside the Input panel clear of every control, moves to the views' tab row when the panel is hidden, and goes when the result is drawn`, async () => {
      const b = await start();
      await b.setWindowSize(width, 950);
      await openApp(b);
      await openGasBarge(b);
      await until(b, "results", () => exists(b, "outputs"));
      // Hold the evaluation without holding the page, so the boxes can be read while the sign shows.
      await b.execute(() => {
        const p = window.__ovdPlatform;
        const real = p.evaluate.bind(p);
        p.evaluate = async () => {
          await new Promise((r) => setTimeout(r, 4000));
          return real();
        };
      });
      await showAllInputs(b); // Essentials hides this input (#1084)
      await record(b);
      await (await b.$('.field[data-input="vent_mast_lowerable"] input[type=checkbox]')).click();
      await until(b, "the sign in the Input panel", async () => (await seen(b)).at(-1) === "input");

      const sign = await rectOf(b, testid("calculating"));
      const panel = await rectOf(b, "section.left");
      const view = await b.execute(() => ({ width: document.documentElement.clientWidth, height: document.documentElement.clientHeight }));
      assert.ok(sign.width > 0 && sign.height > 0, "the sign has a box");
      assert.ok(sign.left >= panel.left && sign.right <= panel.right && sign.top >= panel.top && sign.bottom <= panel.bottom, `inside the Input panel: ${JSON.stringify({ sign, panel })}`);
      assert.ok(sign.right <= view.width && sign.bottom <= view.height, "inside the window");
      for (const c of await controls(b)) assert.ok(apart(sign, c), `the sign covers a control: ${c.what}`);

      // Hide the Input panel while the evaluation runs: the sign shows in the views' tab row.
      await click(b, "toggle-input-panel");
      await until(b, "the sign in the views' tab row", async () => (await seen(b)).at(-1) === "views");
      const moved = await rectOf(b, testid("calculating"));
      const tabs = await rectOf(b, "[data-testid=views-panel] > header");
      assert.ok(moved.top >= tabs.top && moved.bottom <= tabs.bottom && moved.right <= view.width, `in the tab row: ${JSON.stringify({ moved, tabs })}`);
      for (const c of await controls(b)) assert.ok(apart(moved, c), `the sign covers a control: ${c.what}`);

      await until(b, "the sign to go once the result is drawn", async () => (await seen(b)).at(-1) === "off", 15000);
      assert.equal((await (await b.$(testid("calculating"))).getText()).trim(), "", "nothing to announce at rest");
    });
  }
});
