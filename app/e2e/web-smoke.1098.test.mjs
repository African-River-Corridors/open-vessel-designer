// Web end-to-end tests of issue #1098: texts for designers. The wizard shows every Cargo's description (no
// filter hides one), and a required choice has "About this input". Run by e2e/run-web.sh, in headless Chrome,
// against the built web app.
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
/** Wait for `check`. On a time-out, log the page text and the data-testid values, so CI shows what the page held. */
const until = async (b, what, check, timeout = 30000) => {
  try {
    return await b.waitUntil(check, { timeout, interval: 100, timeoutMsg: `timed out waiting for: ${what}` });
  } catch (e) {
    const seen = await b.execute(() => ({ text: document.body.innerText, ids: [...document.querySelectorAll("[data-testid]")].map((n) => n.dataset.testid) })).catch(() => null);
    console.error(`[diagnostic] waiting for ${what}:`, JSON.stringify(seen));
    throw e;
  }
};
const exists = async (b, selector) => (await b.$(selector)).isExisting();
const testid = (id) => `[data-testid=${id}]`;
const click = async (b, selector) => (await b.$(selector)).click();

/** A first start: accept the first-run notice; the Start page opens after it. */
async function firstStart(b) {
  await b.url(`${origin}/ovd/app`);
  await until(b, "the notice", () => exists(b, testid("notice")));
  await (await b.$("button=I understand")).click();
  await until(b, "the notice to close", async () => !(await exists(b, testid("notice"))));
  await until(b, "the Start page", () => exists(b, testid("start-page")));
}

before(async () => {
  ({ server, origin } = await serve(DIST));
});
after(async () => {
  for (const b of sessions) await b.deleteSession().catch(() => {});
  server?.close();
});

describe("OVD web: texts for designers (issue #1098)", () => {
  it("the wizard's Cargo step shows the crude oil and diesel descriptions, written for a designer", async () => {
    const b = await start();
    await firstStart(b);
    await click(b, testid("start-wizard"));
    await until(b, "the wizard", () => exists(b, testid("wizard")));
    const crude = await (await b.$(testid("choice-cargo-crude_oil"))).getText();
    const diesel = await (await b.$(testid("choice-cargo-diesel_en590"))).getText();
    assert.match(crude, /870 kg\/m3 at 15 C/);
    assert.match(diesel, /835 kg\/m3 at 20 C/);
    for (const text of [crude, diesel]) assert.ok(!text.includes("::"), text);
  });

  it("a required choice, the Service Area, has \"About this input\" and its text", async () => {
    const b = await start();
    await firstStart(b);
    await click(b, testid("start-skip"));
    await until(b, "the Start page to close", async () => !(await exists(b, testid("start-page"))));
    const toggle = b.$('.field[data-input="service_area"] [data-testid=about-toggle]');
    await until(b, "the Service Area's About control", () => toggle.isExisting());
    await toggle.click();
    const text = b.$('.field[data-input="service_area"] [data-testid=about-text]');
    await until(b, "the Service Area's help", () => text.isExisting());
    assert.match(await text.getText(), /picks the rules that are checked/);
  });
});
