// Web end-to-end tests of issue #1080 (spec 0017): the Start page, the setup wizard, and the one automatic
// damage run after Create. Run by e2e/run-web.sh, in headless Chrome, against the built web app.
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { remote } from "webdriverio";
import { serve } from "./web-server.mjs";

const here = import.meta.dirname;
const DIST = path.resolve(here, "../dist-web-test");
const CLI = process.env.OVD_CLI;
assert.ok(CLI, "OVD_CLI must be set (run e2e/run-web.sh)");

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
const exists = async (b, selector) => (await b.$(selector)).isExisting();
const testid = (id) => `[data-testid=${id}]`;

/** The worked example of spec 0017, section 2: 11 typed inputs. */
const WORKED = {
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
async function firstStart(b) {
  await b.url(`${origin}/ovd/app`);
  await until(b, "the notice", () => exists(b, testid("notice")));
  await (await b.$("button=I understand")).click();
  await until(b, "the notice to close", async () => !(await exists(b, testid("notice"))));
  await until(b, "the Start page", () => exists(b, testid("start-page")));
}
const click = async (b, selector) => (await b.$(selector)).click();
const heading = async (b) => (await b.$(testid("wizard-heading"))).getText();
/** Choose a radio card by its visible name. */
async function choose(b, name) {
  await (await b.$(`//label[contains(@class,"choice")][.//span[@class="name"][normalize-space()="${name}"]]`)).click();
}
async function typeIn(b, id, text) {
  await (await b.$(`#wiz-${id}`)).setValue(text);
}
async function nextTo(b, name) {
  await click(b, testid("wizard-next"));
  await until(b, `the ${name} step`, async () => (await exists(b, testid("wizard-heading"))) && (await heading(b)) === name);
}
/** Walk the wizard from its first step to the review, typing the worked example by hand. */
async function typeWorkedExample(b) {
  await click(b, testid("start-wizard"));
  await until(b, "the wizard", () => exists(b, testid("wizard")));
  assert.equal(await heading(b), "Cargo");
  await choose(b, "LNG");
  await nextTo(b, "Service Area");
  await choose(b, "Coastal and river");
  await nextTo(b, "Formation");
  await choose(b, "Self-propelled");
  await nextTo(b, "Hull");
  await choose(b, "Raked box");
  await typeIn(b, "length_overall_m", "140");
  await typeIn(b, "beam_m", "29");
  await nextTo(b, "Your own limits");
  for (const [id, text] of [["moulded_depth_m", "12"], ["max_speed_kn", "12"], ["max_tank_od_m", "12"], ["tanks_in_line", "2"], ["min_freeboard_m", "1.81"]]) await typeIn(b, id, text);
  await nextTo(b, "Review");
}
const platformSnapshot = (b) => b.execute(() => window.__ovdPlatform.snapshot());
const typedInputs = (snap) =>
  Object.fromEntries(Object.entries(snap.design.options[0].inputs).filter(([, v]) => v.source === "typed").map(([k, v]) => [k, v.value]));
/** Arm a count of damage runs, before Create. */
const countDamageRuns = (b) =>
  b.execute(() => {
    window.__runs = 0;
    const platform = window.__ovdPlatform;
    const original = platform.runDamage;
    platform.runDamage = (...args) => {
      window.__runs += 1;
      return original(...args);
    };
  });
const damageStatus = (b) =>
  b.execute(async () => (await window.__ovdPlatform.evaluate()).options[0].outcome.results?.damage_status ?? null);
const cargoVolumeOnPage = async (b) => {
  const cell = await b.$('//table[@data-testid="outputs"]//tr[th[contains(normalize-space(), "Cargo volume")]]/td');
  return parseFloat(await cell.getAttribute("title"));
};
/** The Design the app holds, as the native command line sees it: the stored text of the last Design. */
async function nativeCargoVolume(b) {
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
  for (const b of sessions) await b.deleteSession().catch(() => {});
  server?.close();
});

describe("OVD web: the Start page and the setup wizard (issue #1080)", () => {
  it("opens the Start page after the notice; the skip link gives today's starter Design", async () => {
    const b = await start();
    await firstStart(b);
    const page = await (await b.$(testid("start-page"))).getText();
    for (const text of ["New Design, step by step", "Start from an example", "Open a Design file", "Skip: open the blank starter Design"]) assert.ok(page.includes(text), text);
    await click(b, testid("start-skip"));
    await until(b, "the Start page to close", async () => !(await exists(b, testid("start-page"))));
    // The starter Design: a bauxite pushed barge, 76.5 m long, with no Service Area chosen yet (issue #925).
    const area = await b.$('.field[data-input="service_area"] select');
    await area.waitForExist({ timeout: 15000 });
    assert.equal(await b.execute((el) => el.value, area), "__unset__");
    const snap = await platformSnapshot(b);
    assert.equal(snap.design.options[0].inputs.cargo.value, "bauxite");
    assert.equal(snap.design.options[0].inputs.length_overall_m.value, 76.5);
    assert.equal(snap.file_name, "Untitled");
  });

  it("makes the worked example by hand: 11 typed inputs and no other, damage runs once by itself, the cargo volume equals the command line's", async () => {
    const b = await start();
    await firstStart(b);
    await typeWorkedExample(b);
    // The review lists the 11 typed choices, the made Option name and the count left to the Defaults Profile.
    assert.equal((await b.$$(`${testid("wizard-choices")} li`)).length, 11);
    assert.equal(await (await b.$("#wiz-option-name")).getValue(), "LNG 140x29");
    assert.match(await (await b.$(testid("wizard-defaults"))).getText(), /^\d+ other inputs come from the Defaults Profile/);
    await countDamageRuns(b);
    await click(b, testid("wizard-create"));
    await until(b, "the wizard to close", async () => !(await exists(b, testid("wizard"))));
    await until(b, "the results", () => exists(b, testid("outputs")));
    await until(b, "damage stability to finish", async () => (await damageStatus(b)) === "run", 240000);
    const snap = await platformSnapshot(b);
    assert.deepEqual(typedInputs(snap), WORKED, "the 11 typed inputs and no other typed input");
    assert.equal(snap.design.options[0].name, "LNG 140x29");
    assert.equal(snap.active_option, "LNG 140x29");
    // Once: the run started by itself and nothing starts another.
    await new Promise((r) => setTimeout(r, 1500));
    assert.equal(await b.execute(() => window.__runs), 1, "one automatic damage run");
    assert.equal(await damageStatus(b), "run");
    assert.equal(await cargoVolumeOnPage(b), await nativeCargoVolume(b), "the cargo volume equals `ovd run --json`");
  });

  it("starts from the example: the review holds the 11 inputs, and Create gives the same Design as the hand-typed path", async () => {
    const b = await start();
    await firstStart(b);
    await click(b, testid("start-example"));
    await until(b, "the example list", () => exists(b, testid("start-examples")));
    await click(b, testid("start-example-coastal-lng-barge"));
    await until(b, "the review step", async () => (await exists(b, testid("wizard-heading"))) && (await heading(b)) === "Review");
    assert.equal((await b.$$(`${testid("wizard-choices")} li`)).length, 11);
    await countDamageRuns(b);
    await click(b, testid("wizard-create"));
    await until(b, "the wizard to close", async () => !(await exists(b, testid("wizard"))));
    await until(b, "damage stability to finish", async () => (await damageStatus(b)) === "run", 240000);
    const fromExample = await platformSnapshot(b);
    assert.deepEqual(typedInputs(fromExample), WORKED);
    assert.equal(await b.execute(() => window.__runs), 1);

    // The hand-typed path in a second browser gives the same Design, input for input and source for source.
    const c = await start();
    await firstStart(c);
    await typeWorkedExample(c);
    await click(c, testid("wizard-create"));
    await until(c, "the wizard to close", async () => !(await exists(c, testid("wizard"))));
    await until(c, "the results", () => exists(c, testid("outputs")));
    const byHand = await platformSnapshot(c);
    assert.deepEqual(byHand.design.options[0], fromExample.design.options[0]);
  });

  it("with unsaved edits, New opens the Start page and Cancel keeps the Design; a path asks before it discards", async () => {
    const b = await start();
    await firstStart(b);
    await click(b, testid("start-skip"));
    const area = await b.$('.field[data-input="service_area"] select');
    await area.waitForExist({ timeout: 15000 });
    await area.selectByAttribute("value", "coastal-and-river");
    await until(b, "the edit", async () => (await (await b.$("header")).getText()).includes("(unsaved)"));
    await b.execute(() => {
      window.__asked = [];
      window.__answer = false;
      window.confirm = (message) => {
        window.__asked.push(message);
        return window.__answer;
      };
    });
    const asked = () => b.execute(() => window.__asked.length);
    // New opens the page and asks nothing.
    await click(b, "button=New");
    await until(b, "the Start page", () => exists(b, testid("start-page")));
    assert.equal(await asked(), 0);
    // Cancel leaves the Design and its edit as they were.
    await click(b, testid("start-cancel"));
    await until(b, "the Start page to close", async () => !(await exists(b, testid("start-page"))));
    assert.match(await (await b.$("header")).getText(), /\(unsaved\)/);
    assert.equal(await b.execute((el) => el.value, await b.$('.field[data-input="service_area"] select')), "coastal-and-river");
    assert.equal(await asked(), 0);
    // The skip link asks; a refusal leaves the page open and the Design alone.
    await click(b, "button=New");
    await until(b, "the Start page", () => exists(b, testid("start-page")));
    await click(b, testid("start-skip"));
    await until(b, "the question", async () => (await asked()) === 1);
    assert.match(await b.execute(() => window.__asked[0]), /unsaved changes/);
    assert.equal(await exists(b, testid("start-page")), true, "a refusal leaves the Start page open");
    // Create asks too, and a refusal leaves the wizard open.
    await click(b, testid("start-example"));
    await until(b, "the example list", () => exists(b, testid("start-examples")));
    await click(b, testid("start-example-coastal-lng-barge"));
    await until(b, "the review step", async () => (await exists(b, testid("wizard-heading"))) && (await heading(b)) === "Review");
    await click(b, testid("wizard-create"));
    await until(b, "the question", async () => (await asked()) === 2);
    assert.equal(await exists(b, testid("wizard")), true, "a refusal leaves the wizard open");
    assert.equal((await platformSnapshot(b)).design.options[0].inputs.cargo.value, "bauxite", "the Design is untouched");
    // Agreeing discards the edit and makes the new Design.
    await b.execute(() => {
      window.__answer = true;
    });
    await click(b, testid("wizard-create"));
    await until(b, "the wizard to close", async () => !(await exists(b, testid("wizard"))));
    await until(b, "the new Design", async () => (await platformSnapshot(b)).design.options[0].inputs.cargo.value === "LNG");
    assert.equal(await asked(), 3);
  });

  it("blocks Next on a hard limit and names it; warns on a value outside the usual range and goes on", async () => {
    const b = await start();
    await firstStart(b);
    await click(b, testid("start-wizard"));
    await until(b, "the wizard", () => exists(b, testid("wizard")));
    // Nothing chosen: Next stays on the step and says what is needed.
    await click(b, testid("wizard-next"));
    assert.equal(await heading(b), "Cargo");
    assert.match(await (await b.$(`${testid("wizard")} .main`)).getText(), /Cargo is needed/);
    await choose(b, "LNG");
    await nextTo(b, "Service Area");
    await choose(b, "Coastal and river");
    await nextTo(b, "Formation");
    await choose(b, "Self-propelled");
    await nextTo(b, "Hull");
    await choose(b, "Raked box");
    await typeIn(b, "length_overall_m", "140");
    await typeIn(b, "beam_m", "0.5");
    await until(b, "the limit", async () => /at least 1/.test(await (await b.$('.field[data-input="beam_m"]')).getText()));
    assert.equal(await (await b.$(testid("wizard-next"))).getAttribute("aria-disabled"), "true");
    await click(b, testid("wizard-next"));
    assert.equal(await heading(b), "Hull", "a hard limit blocks Next");
    await typeIn(b, "beam_m", "36");
    await until(b, "the warning", async () => /usual range/.test(await (await b.$('.field[data-input="beam_m"] [data-severity=warning]')).getText()));
    assert.equal(await (await b.$(testid("wizard-next"))).getAttribute("aria-disabled"), "false");
    await nextTo(b, "Your own limits");
    // Back keeps every answer.
    await click(b, testid("wizard-back"));
    await until(b, "the Hull step", async () => (await heading(b)) === "Hull");
    assert.equal(await (await b.$("#wiz-beam_m")).getValue(), "36");
  });

  it("does not let a river-only path reach Create without both limits", async () => {
    const b = await start();
    await firstStart(b);
    await click(b, testid("start-wizard"));
    await until(b, "the wizard", () => exists(b, testid("wizard")));
    await choose(b, "LNG");
    await nextTo(b, "Service Area");
    await choose(b, "River only");
    await until(b, "the river limits", () => exists(b, "#wiz-draught_limit_m"));
    assert.ok(await exists(b, "#wiz-air_draft_limit_m"));
    await nextTo(b, "Formation");
    await choose(b, "Self-propelled");
    await nextTo(b, "Hull");
    await choose(b, "Raked box");
    await typeIn(b, "length_overall_m", "100");
    await typeIn(b, "beam_m", "11");
    await nextTo(b, "Your own limits");
    assert.equal(await exists(b, "#wiz-min_freeboard_m"), false, "minimum freeboard is for coastal and river");
    await nextTo(b, "Review");
    assert.equal(await (await b.$(testid("wizard-create"))).getAttribute("aria-disabled"), "true");
    assert.equal((await b.$$(`${testid("wizard-stops")} li`)).length, 2, "both limits are missing");
    await click(b, testid("wizard-create"));
    assert.equal(await exists(b, testid("wizard")), true, "Create did nothing");
    // The link goes to the step; both limits typed, Create is ready.
    await click(b, `${testid("wizard-stops")} li button`);
    await until(b, "the Service Area step", async () => (await heading(b)) === "Service Area");
    await typeIn(b, "draught_limit_m", "3");
    await typeIn(b, "air_draft_limit_m", "12");
    await nextTo(b, "Formation");
    await nextTo(b, "Hull");
    await nextTo(b, "Your own limits");
    await nextTo(b, "Review");
    assert.equal(await (await b.$(testid("wizard-create"))).getAttribute("aria-disabled"), "false");
  });

  it("works with the keyboard: focus goes to the step heading on Next and Back, and Escape cancels", async () => {
    const b = await start();
    await firstStart(b);
    const active = () => b.execute(() => document.activeElement?.id || document.activeElement?.getAttribute("data-testid") || "");
    assert.equal(await active(), "start-title", "the Start page opens with its heading focused");
    await b.keys(["Tab"]);
    assert.equal(await active(), "start-wizard");
    await b.keys(["Enter"]);
    await until(b, "the wizard", () => exists(b, testid("wizard")));
    assert.equal(await active(), "wizard-title", "the wizard opens with its heading focused");
    // One Tab reaches the choices, a second leaves them for Cancel, a third reaches Next.
    await b.keys(["Tab"]);
    await b.keys([" "]);
    await until(b, "a Cargo chosen", async () => (await b.$("input[name=wiz-cargo]:checked")).isExisting());
    await b.keys(["Tab"]);
    assert.equal(await active(), "wizard-cancel");
    await b.keys(["Tab"]);
    assert.equal(await active(), "wizard-next");
    await b.keys(["Enter"]);
    await until(b, "the Service Area step", async () => (await heading(b)) === "Service Area");
    assert.equal(await active(), "wizard-title", "Next moves focus to the heading");
    // Back: from the heading, Tab to the choices, Cancel, then Back.
    await b.keys(["Tab", "Tab", "Tab"]);
    assert.equal(await active(), "wizard-back");
    await b.keys(["Enter"]);
    await until(b, "the Cargo step", async () => (await heading(b)) === "Cargo");
    assert.equal(await active(), "wizard-title", "Back moves focus to the heading");
    await b.keys(["Escape"]);
    await until(b, "the wizard to close", async () => !(await exists(b, testid("wizard"))));
    assert.equal(await (await platformSnapshot(b)).design.options[0].inputs.cargo.value, "bauxite", "Escape left the Design as it was");
  });
});
