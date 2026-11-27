// Web end-to-end test of the Summary view (issue #1081, spec 0017 D7): the built web app in headless Chrome.
// Run by e2e/run-web.sh. For a gas barge: every number on the Summary equals the same number in the Outputs
// panel, the verdict line agrees with the Checks panel, and the damage button starts the run and the Summary
// then says "run" with the count of cases.
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { closeBrowsers, openBrowser } from "./browser.mjs";
import { serve } from "./web-server.mjs";

const here = import.meta.dirname;
const DRY_FIXTURE = path.resolve(here, "../../crates/ovd-format/tests/fixtures/dry_bulk_box_hold.vdesign");
const DIST = path.resolve(here, "../dist-web-test");
const CLI = process.env.OVD_CLI;
assert.ok(CLI, "OVD_CLI must be set (run e2e/run-web.sh)");

const chrome = () => ({
  browserName: "chrome",
  "wdio:enforceWebDriverClassic": true,
  "goog:chromeOptions": {
    ...(process.env.CHROME_BINARY ? { binary: process.env.CHROME_BINARY } : fs.existsSync("/Applications/Google Chrome.app") ? { binary: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" } : {}),
    args: ["--headless=new", "--no-sandbox", "--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist", "--window-size=1280,900"],
  },
});

let origin, server;
const sessions = [];
const until = (b, what, check, timeout = 30000) => b.waitUntil(check, { timeout, interval: 100, timeoutMsg: `timed out waiting for: ${what}` });
const exists = async (b, sel) => (await b.$(sel)).isExisting();

/** A coastal gas barge with damage cases, filled by the native CLI (as the damage test of web-smoke.test.mjs does). */
function gasBarge() {
  const inputs = {
    cargo: { value: "LNG", source: "typed" },
    hull_form: { value: "raked_box", source: "typed" },
    formation: { value: { kind: "self_propelled" }, source: "typed" },
    length_overall_m: { value: 110.0, source: "typed" },
    beam_m: { value: 11.4, source: "typed" },
    moulded_depth_m: { value: 6.0, source: "typed" },
    loading_conditions: { value: "is_code_2008", source: "typed" },
    service_area: { value: "coastal-and-river", source: "typed" },
  };
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "ovd-web-summary-"));
  const bare = path.join(dir, "bare.vdesign");
  const filled = path.join(dir, "gas-summary.vdesign");
  fs.writeFileSync(bare, JSON.stringify({ format: "vdesign", format_version: 3, engine_version: "0.2.4", units: "SI", options: [{ name: "Option 1", inputs }] }));
  execFileSync(CLI, ["fill", "--as-new", bare, filled], { stdio: "ignore" });
  return filled;
}

async function start() {
  const b = await openBrowser(chrome());
  sessions.push(b);
  await b.url(`${origin}/ovd/app`);
  await until(b, "the notice", async () => exists(b, "[data-testid=notice]"));
  await (await b.$("button=I understand")).click();
  // A first start opens the Start page after the notice (spec 0017): take its skip link.
  await until(b, "the Start page", async () => (await b.$("[data-testid=start-page]")).isExisting());
  await (await b.$("[data-testid=start-skip]")).click();
  await until(b, "the Start page to close", async () => !(await b.$("[data-testid=start-page]").isExisting()));
  await until(b, "the app", async () => exists(b, "header"));
  return b;
}
/** Open a Design file through the picker. */
async function openDesign(b, file) {
  await b.execute(() => {
    HTMLInputElement.prototype.click = function () {};
    window.confirm = () => true;
  });
  await (await b.$("button=Open")).click();
  const picker = await b.$("[data-testid=file-picker]");
  await picker.waitForExist({ timeout: 5000 });
  await picker.setValue(file);
  await until(b, "the file name", async () => (await b.$("header")).getText().then((t) => t.includes(path.basename(file))));
  await until(b, "the checks", async () => (await b.$$("[data-testid=checks] [data-testid=check]")).length > 0);
}
const summary = (b) => b.$("[data-testid=summary]");
/** The Summary's numbers: label and value text. */
const summaryNumbers = (b) =>
  b.execute(() =>
    [...document.querySelectorAll("[data-testid=summary] [data-testid=summary-number]")].map((r) => [
      r.querySelector("th").textContent.trim(),
      r.querySelector("td").textContent.replace(/\s+/g, " ").trim(),
    ]),
  );
const outputNumbers = (b) =>
  b.execute(() =>
    Object.fromEntries(
      [...document.querySelectorAll("[data-testid=outputs] tr")].map((r) => [
        r.querySelector("th")?.textContent.replace(/[\s†]+/g, " ").trim(),
        r.querySelector("td")?.textContent.replace(/\s+/g, " ").trim(),
      ]),
    ),
  );

before(async () => {
  ({ server, origin } = await serve(DIST));
});
after(async () => {
  await closeBrowsers();
  server?.close();
});

describe("Summary view (issue #1081)", () => {
  it("is the view shown after a Design is opened, and the other views are still reachable", async () => {
    const b = await start();
    await openDesign(b, gasBarge());
    await until(b, "the Summary", async () => (await summary(b)).isExisting());
    assert.equal(await (await b.$("[data-testid=view-title]")).getText(), "SUMMARY");
    const tops = await b.$$("[data-testid=views-panel] [role=tablist]:not(.sub) button");
    assert.equal(await tops[0].getText(), "Summary", "Summary is the first tab");
    assert.equal(await tops[0].getAttribute("aria-selected"), "true");
    // The drawings are one tab away, unchanged.
    await (await b.$('//button[@role="tab" and normalize-space()="Drawings"]')).click();
    await until(b, "the side elevation", async () => (await b.$$("[data-testid=slot-side] svg")).length === 1);
    // Opening another Design brings the Summary back.
    await openDesign(b, DRY_FIXTURE);
    await until(b, "the Summary again", async () => (await summary(b)).isExisting());
  });

  it("shows every number as the Outputs panel does, and a verdict that agrees with the Checks panel (gas barge)", async () => {
    const b = await start();
    await openDesign(b, gasBarge());
    await until(b, "the Summary", async () => (await summary(b)).isExisting());
    const numbers = await summaryNumbers(b);
    const outputs = await outputNumbers(b);
    assert.ok(numbers.length >= 12, `${numbers.length} numbers`);
    for (const [label, value] of numbers) {
      assert.ok(label in outputs, `the Outputs panel has a row "${label}"`);
      assert.equal(value, outputs[label], label);
    }
    for (const label of ["Cargo volume", "Cargo", "Draught", "Freeboard", "Lightship", "Displacement", "Tanks across", "Tanks along"]) {
      assert.ok(numbers.some(([l]) => l.startsWith(label)), `a "${label}" line`);
    }
    // The verdict agrees with the Checks panel: met exactly when no check fails.
    const failing = (await b.$$("[data-testid=check][data-tone=fail]")).length;
    const verdict = await b.$("[data-testid=summary-verdict]");
    assert.equal(await verdict.getAttribute("data-kind"), failing === 0 ? "met" : "not-met");
    // The scope sentence is the Outputs panel's, word for word.
    // The sentence sits behind the scope's one line: open it first.
    await (await b.$("[data-testid=summary-scope-line]")).click();
    assert.equal((await (await b.$("[data-testid=summary-scope]")).getText()).trim(), outputs["Scope of the check"]);
    // The warning checks not met are the Checks panel's, in its words.
    const panelWarn = await b.execute(() => [...document.querySelectorAll("[data-testid=check][data-tone=warn] strong")].map((e) => e.textContent.trim()));
    const summaryWarn = await b.execute(() => [...document.querySelectorAll("[data-testid=summary-warning] strong")].map((e) => e.textContent.trim()));
    assert.deepEqual(summaryWarn, panelWarn);
  });

  it("the damage button starts the run; the Summary then says run, with the count of cases", async () => {
    const b = await start();
    await openDesign(b, gasBarge());
    await until(b, "the Summary", async () => (await summary(b)).isExisting());
    const status = () => b.$("[data-testid=summary-damage]").then((e) => e.getAttribute("data-status"));
    assert.equal(await status(), "not run");
    assert.match(await (await b.$("[data-testid=summary-damage]")).getText(), /not run/i);
    await (await b.$("[data-testid=summary-damage-run]")).click();
    await until(b, "damage to have run", async () => (await status()) === "run", 240000);
    const text = await (await b.$("[data-testid=summary-damage]")).getText();
    assert.match(text, /\brun\b/i);
    const cases = Number(/(\d+)\s+cases/i.exec(text)?.[1]);
    assert.ok(cases >= 10, `the count of cases in "${text}"`);
    assert.equal(await exists(b, "[data-testid=summary-damage-run]"), false, "no button once it has run");
    const engine = await b.execute(async () => (await window.__ovdPlatform.evaluate()).options[0].outcome.results.damage_cases);
    assert.equal(cases, engine, "the count is the engine's");
  });

  it("a Design that is not calculated says so and links to the input to fix", async () => {
    const b = await start();
    // The starter Design has no Service Area, so it has no results until one is chosen.
    await until(b, "the Summary", async () => (await summary(b)).isExisting());
    const verdict = await b.$("[data-testid=summary-verdict]");
    assert.equal(await verdict.getAttribute("data-kind"), "not-calculated");
    const link = await b.$("[data-testid=summary-verdict] a");
    assert.ok(await link.isExisting(), "a link to the input");
    assert.match(await link.getText(), /Service Area/i);
    await link.click();
    const focused = await b.execute(() => document.activeElement?.closest("[data-input]")?.getAttribute("data-input"));
    assert.equal(focused, "service_area");
    assert.equal((await b.$$("[data-testid=summary-number]")).length, 0, "no numbers without results");
  });

  it("a dry-bulk Option has no tank lines; the Input panel link opens the panel; the view does not scroll sideways at 1280 px, panel shown or hidden", async () => {
    const b = await start();
    await openDesign(b, DRY_FIXTURE);
    await until(b, "the Summary", async () => (await summary(b)).isExisting());
    const labels = (await summaryNumbers(b)).map(([l]) => l);
    assert.ok(!labels.some((l) => l.startsWith("Tanks ")), `no tank lines: ${labels}`);
    assert.ok(labels.length > 0);
    // The counts of typed and default inputs, with a link to the Input panel.
    const counts = await (await b.$("[data-testid=summary-input-counts]")).getText();
    assert.match(counts, /\d+ typed/);
    assert.match(counts, /\d+ default/);
    const sideways = () =>
      b.execute(() => {
        const s = document.querySelector("[data-testid=summary]");
        return s.scrollWidth - s.clientWidth;
      });
    await b.setWindowSize(1280, 900);
    assert.ok((await sideways()) <= 1, "1280 px, Input panel shown");
    await (await b.$('button[aria-pressed][aria-label="Hide the Input panel"]')).click();
    assert.equal(await (await b.$("section.left")).isDisplayed(), false, "Input panel hidden");
    assert.ok((await sideways()) <= 1, "1280 px, Input panel hidden");
    await (await b.$("[data-testid=summary-open-input]")).click();
    await until(b, "the Input panel", async () => (await b.$("section.left")).isDisplayed());
  });

  it("the headings run in order, there is one landmark, and the links and button are reached by keyboard", async () => {
    const b = await start();
    await openDesign(b, gasBarge());
    await until(b, "the Summary", async () => (await summary(b)).isExisting());
    const levels = await b.execute(() =>
      [...document.querySelectorAll("[data-testid=views-panel] h2, [data-testid=views-panel] h3, [data-testid=views-panel] h4")].map((h) => Number(h.tagName[1])),
    );
    assert.deepEqual(levels.slice(0, 2), [2, 3]);
    assert.ok(levels.length >= 5 && levels.slice(2).every((l) => l === 4), `heading levels ${levels}`);
    const landmarks = await b.execute(() => {
      const root = document.querySelector("[data-testid=summary]");
      const sel = "main, nav, aside, header, footer, section, form, [role=region], [role=main], [role=navigation], [role=banner]";
      return root.querySelectorAll(sel).length + (root.matches(sel) ? 1 : 0);
    });
    assert.equal(landmarks, 1, "one landmark in the Summary");
    const reachable = await b.execute(() => [...document.querySelectorAll("[data-testid=summary] a, [data-testid=summary] button")].every((e) => e.tabIndex >= 0 && !e.disabled));
    assert.ok(reachable);
  });
});
