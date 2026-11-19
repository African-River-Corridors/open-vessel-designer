// Web end-to-end for issue #1082 (spec 0017, D12 and D13): plain words in the app, and a sign that it
// is calculating. Run by e2e/run-web.sh with the other web-smoke files.
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { remote } from "webdriverio";
import { serve } from "./web-server.mjs";

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
  const b = await remote({ logLevel: "warn", strictSelectors: false, capabilities: chrome() });
  sessions.push(b);
  return b;
}
const until = (b, what, check, timeout = 30000) =>
  b.waitUntil(check, { timeout, interval: 50, timeoutMsg: `timed out waiting for: ${what}` });

/** Open the app past the first-run notice. */
async function openApp(b) {
  await b.url(`${origin}/ovd/app`);
  await until(b, "the notice", async () => (await b.$("[data-testid=notice]")).isExisting());
  await (await b.$("button=I understand")).click();
}

before(async () => {
  ({ server, origin } = await serve(DIST));
});
after(async () => {
  for (const b of sessions) await b.deleteSession().catch(() => {});
  server?.close();
});

describe("Plain words and the calculating sign (issue #1082)", () => {
  it("tells the designer to choose a Service Area, in the Input panel and the Outputs panel, with no command line", async () => {
    const b = await start();
    await openApp(b);
    // The starter Design has no Service Area (issue #925), so it shows the error in both places.
    const field = '.field[data-input="service_area"] .msg.error';
    await until(b, "the Service Area error", async () => (await b.$(field)).isExisting());
    const expected = "Choose a Service Area: river only, or coastal and river.";
    assert.equal((await (await b.$(field)).getText()).trim(), expected);
    const outputs = () => b.$$(".err").then((els) => Promise.all(els.map((e) => e.getText())));
    await until(b, "the error in the Outputs panel", async () => (await outputs()).some((t) => t.includes("Service Area")));
    assert.ok((await outputs()).some((t) => t.includes(expected)), `the Outputs panel says: ${await outputs()}`);
    // No text on the page names the program with a sub-command.
    const page = await b.execute(() => document.body.innerText);
    assert.doesNotMatch(page, /\bovd\s+[a-z]|simulate/);
  });

  it("shows Calculating… while a slow evaluation runs after an edit to a gas barge, and not before or after", async () => {
    const b = await start();
    await openApp(b);
    await b.execute(() => {
      HTMLInputElement.prototype.click = function () {};
    });
    await (await b.$("button=Open")).click();
    const picker = await b.$("[data-testid=file-picker]");
    await picker.waitForExist({ timeout: 5000 });
    await picker.setValue(GAS_FIXTURE);
    await until(b, "the file name", async () => (await b.$("header")).getText().then((t) => t.includes("gas_igc_2g_self_propelled.vdesign")));
    await until(b, "results", async () => (await b.$("[data-testid=outputs]")).isExisting());

    const sign = b.$("header [data-testid=calculating]");
    assert.equal(await sign.getAttribute("role"), "status", "the sign is a status");
    assert.equal(await sign.getAttribute("aria-live"), null, "role=status already announces politely");
    assert.equal((await sign.getText()).trim(), "", "nothing to announce at rest");

    // Make the evaluation take about a second, so the sign has time to show whatever the machine's speed.
    // The first answer is kept: the second part of the test hands it back at once.
    await b.executeAsync((done) => {
      const p = window.__ovdPlatform;
      const real = p.evaluate.bind(p);
      real().then((kept) => {
        window.__kept = kept;
        window.__slow = true;
        p.evaluate = async () => {
          if (!window.__slow) return window.__kept;
          await new Promise((r) => setTimeout(r, 1200));
          return real();
        };
        done();
      });
    });
    const toggle = b.$('.field[data-input="vent_mast_lowerable"] input[type=checkbox]');
    await toggle.click();
    await until(b, "the Calculating… sign", async () => (await sign.getText()).trim() === "Calculating…");
    await until(b, "the sign to go once the result is drawn", async () => (await sign.getText()).trim() === "");

    // An evaluation that returns at once (under about 200 ms) never shows the sign.
    await b.execute(() => {
      window.__slow = false;
      window.__seen = false;
      new MutationObserver(() => {
        if (document.querySelector("[data-testid=calculating]")?.textContent.trim()) window.__seen = true;
      }).observe(document.body, { subtree: true, childList: true, characterData: true });
    });
    await toggle.click();
    await new Promise((r) => setTimeout(r, 1500));
    assert.equal(await b.execute(() => window.__seen), false, "the sign did not flash");
  });
});
