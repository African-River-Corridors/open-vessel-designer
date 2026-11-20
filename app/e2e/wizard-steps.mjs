// The shared steps of the web tests of issue #1080 (spec 0017): the Start page and the setup wizard. Its test
// files import these; importing this module also serves the built app for the file and ends its browsers after.
import { after, before } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { closeBrowsers, openBrowser } from "./browser.mjs";
import { serve } from "./web-server.mjs";

export const here = import.meta.dirname;
export const DIST = path.resolve(here, "../dist-web-test");
export const CLI = process.env.OVD_CLI;
assert.ok(CLI, "OVD_CLI must be set (run e2e/run-web.sh)");

export const chrome = () => ({
  browserName: "chrome",
  "wdio:enforceWebDriverClassic": true,
  "goog:chromeOptions": {
    ...(process.env.CHROME_BINARY ? { binary: process.env.CHROME_BINARY } : fs.existsSync("/Applications/Google Chrome.app") ? { binary: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" } : {}),
    args: ["--headless=new", "--no-sandbox", "--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist", "--window-size=1500,950"],
  },
});

let origin, server;
const sessions = [];
export async function start() {
  const b = await openBrowser(chrome());
  sessions.push(b);
  return b;
}
/** Wait for `check`. On a time-out, log the page text and the data-testid values, so CI shows what the page held. */
export const until = async (b, what, check, timeout = 30000) => {
  try {
    return await b.waitUntil(check, { timeout, interval: 100, timeoutMsg: `timed out waiting for: ${what}` });
  } catch (e) {
    const seen = await b.execute(() => ({ text: document.body.innerText, ids: [...document.querySelectorAll("[data-testid]")].map((n) => n.dataset.testid) })).catch(() => null);
    console.error(`[diagnostic] waiting for ${what}:`, JSON.stringify(seen));
    throw e;
  }
};
export const exists = async (b, selector) => (await b.$(selector)).isExisting();
export const testid = (id) => `[data-testid=${id}]`;

/** The worked example of spec 0017, section 2: 11 typed inputs. */
export const WORKED = {
  cargo: "LNG",
  hull_form: "raked_box",
  formation: { kind: "self_propelled" },
  length_overall_m: 140,
  beam_m: 29,
  service_area: "coastal-and-river",
  moulded_depth_m: 12,
  max_speed_kn: 12,
  max_tank_od_m: 12,
  tanks_in_line: 2,
  min_freeboard_m: 1.81,
};

/** A first start: accept the first-run notice; the Start page opens after it. */
export async function firstStart(b) {
  await b.url(`${origin}/ovd/app`);
  await until(b, "the notice", () => exists(b, testid("notice")));
  await (await b.$("button=I understand")).click();
  await until(b, "the notice to close", async () => !(await exists(b, testid("notice"))));
  await until(b, "the Start page", () => exists(b, testid("start-page")));
}
export const click = async (b, selector) => (await b.$(selector)).click();
export const heading = async (b) => (await b.$(testid("wizard-heading"))).getText();
/** Choose a radio card by the schema's input and value (not by its label or class). */
export async function choose(b, input, value) {
  await (await b.$(testid(`choice-${input}-${value}`))).click();
}
export const box = (id) => testid(`input-${id}`);
export async function typeIn(b, id, text) {
  await (await b.$(box(id))).setValue(text);
}
export async function nextTo(b, name) {
  await click(b, testid("wizard-next"));
  await until(b, `the ${name} step`, async () => (await exists(b, testid("wizard-heading"))) && (await heading(b)) === name);
}
/** Walk the wizard from its first step to the review, typing the worked example by hand. */
export async function typeWorkedExample(b) {
  await click(b, testid("start-wizard"));
  await until(b, "the wizard", () => exists(b, testid("wizard")));
  assert.equal(await heading(b), "Cargo");
  await choose(b, "cargo", "LNG");
  await nextTo(b, "Service Area");
  await choose(b, "service_area", "coastal-and-river");
  await nextTo(b, "Formation");
  await choose(b, "formation", "self_propelled");
  await nextTo(b, "Hull");
  await choose(b, "hull_form", "raked_box");
  await typeIn(b, "length_overall_m", "140");
  await typeIn(b, "beam_m", "29");
  await nextTo(b, "Your own limits");
  for (const [id, text] of [["moulded_depth_m", "12"], ["max_speed_kn", "12"], ["max_tank_od_m", "12"], ["tanks_in_line", "2"], ["min_freeboard_m", "1.81"]]) await typeIn(b, id, text);
  await nextTo(b, "Review");
}
export const platformSnapshot = (b) => b.execute(() => window.__ovdPlatform.snapshot());
export const typedInputs = (snap) =>
  Object.fromEntries(Object.entries(snap.design.options[0].inputs).filter(([, v]) => v.source === "typed").map(([k, v]) => [k, v.value]));
/** Arm a count of damage runs, before Create. */
export const countDamageRuns = (b) =>
  b.execute(() => {
    window.__runs = 0;
    const platform = window.__ovdPlatform;
    const original = platform.runDamage;
    platform.runDamage = (...args) => {
      window.__runs += 1;
      return original(...args);
    };
  });
export const damageStatus = (b) =>
  b.execute(async () => (await window.__ovdPlatform.evaluate()).options[0].outcome.results?.damage_status ?? null);
export const cargoVolumeOnPage = async (b) => {
  const cell = await b.$('//table[@data-testid="outputs"]//tr[th[contains(normalize-space(), "Cargo volume")]]/td');
  return parseFloat(await cell.getAttribute("title"));
};
/** The Design the app holds, as the native command line sees it: the stored text of the last Design. */
export async function nativeCargoVolume(b) {
  const stored = JSON.parse(await b.execute(() => localStorage.getItem("ovd.lastDesign")));
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "ovd-web-1080-"));
  const file = path.join(dir, "made.vdesign");
  fs.writeFileSync(file, stored.text);
  const run = JSON.parse(execFileSync(CLI, ["run", "--json", file], { encoding: "utf8", maxBuffer: 1 << 28, stdio: ["ignore", "pipe", "ignore"] }));
  return run.options[0].results.results.cargo_volume_m3;
}

before(async () => {
  ({ server, origin } = await serve(DIST));
});
after(async () => {
  await closeBrowsers();
  server?.close();
});
