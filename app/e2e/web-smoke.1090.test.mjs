// Web end-to-end for issue #1090 (spec 0017): the Summary lists the failed blocking Rules as a count and a short
// list with links, and a group with one view shows one row of tabs. Run by e2e/run-web.sh with the other
// web-smoke files.
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
  const b = await remote({ logLevel: "warn", strictSelectors: false, capabilities: chrome() });
  sessions.push(b);
  return b;
}
const until = (b, what, check, timeout = 30000) =>
  b.waitUntil(check, { timeout, interval: 100, timeoutMsg: `timed out waiting for: ${what}` });
const testid = (id) => `[data-testid=${id}]`;
const exists = async (b, id) => (await b.$(testid(id))).isExisting();
const click = async (b, id) => (await b.$(testid(id))).click();
const verdictKind = async (b) => (await b.$(testid("summary-verdict"))).getAttribute("data-kind");

/** Past the notice and the Start page, then the gas barge fixture made too shallow (2.5 m): it fails stability. */
async function shallowGasBarge(b) {
  await b.url(`${origin}/ovd/app`);
  await until(b, "the notice", () => exists(b, "notice"));
  await (await b.$("button=I understand")).click();
  await until(b, "the Start page", () => exists(b, "start-page"));
  await click(b, "start-skip");
  await until(b, "the Start page to close", async () => !(await exists(b, "start-page")));
  await b.execute(() => {
    HTMLInputElement.prototype.click = function () {};
  });
  await (await b.$("button=Open")).click();
  const picker = await b.$(testid("file-picker"));
  await picker.waitForExist({ timeout: 5000 });
  await picker.setValue(GAS_FIXTURE);
  await until(b, "the gas barge's verdict", async () => (await verdictKind(b)) === "met", 60000);
  await showAllInputs(b);
  const depth = await b.$('.field[data-input="moulded_depth_m"] input[type=text]');
  await depth.setValue("2.5");
  await until(b, "the verdict to fail", async () => (await verdictKind(b)) === "not-met", 60000);
}

before(async () => {
  ({ server, origin } = await serve(DIST));
});
after(async () => {
  for (const b of sessions) await b.deleteSession().catch(() => {});
  server?.close();
});

describe("The Summary's failed Rules as a short list, and one row of tabs (issue #1090)", () => {
  it("a gas barge that fails stability shows the count, five lines and the link to the rest", async () => {
    const b = await start();
    await shallowGasBarge(b);
    const verdict = (await (await b.$(testid("summary-verdict"))).getText()).trim();
    const count = Number(/(\d+) blocking Rules are not met\./.exec(verdict)?.[1]);
    assert.ok(count > 5, `a count over five in "${verdict}"`);
    const lines = await b.$$(testid("summary-failed-rule"));
    assert.equal(lines.length, 5, "five lines");
    for (const line of lines) assert.ok(await (await line.$(testid("summary-failed-link"))).isExisting(), "each line has its link");
    const more = (await (await b.$(testid("summary-failed-more"))).getText()).trim();
    assert.ok(more.startsWith(`and ${count - 5} more.`), more);
    await click(b, "summary-open-rules");
    await until(b, "the Rule checks view", () => exists(b, "rule-checks"));
  });

  it("each link reaches its row: a check in the Checks panel, a Rule in the Rule checks view", async () => {
    const b = await start();
    await shallowGasBarge(b);
    const kinds = new Set();
    for (let i = 0; i < 5; i++) {
      await click(b, "group-summary");
      await until(b, "the Summary's list", () => exists(b, "summary-failed"));
      const line = (await b.$$(testid("summary-failed-rule")))[i];
      const title = await line.getAttribute("data-rule");
      const link = await line.$(testid("summary-failed-link"));
      const to = await link.getAttribute("data-to");
      kinds.add(to);
      await link.click();
      if (to === "check") {
        await until(b, `the focus on the check row of ${title}`, () => b.execute((t) => document.activeElement?.getAttribute("data-check") === t, title));
        assert.equal(await b.execute(() => !document.querySelector("section.bottom").hidden), true, "the Checks panel shows");
      } else {
        await until(b, `the focus on the Rule row of ${title}`, () =>
          b.execute((t) => document.activeElement?.matches("[data-testid=rule-check-row]") && document.activeElement.getAttribute("data-rule") === t, title),
        );
      }
    }
    assert.ok(kinds.has("check") && kinds.has("rule"), `both kinds of link were followed: ${[...kinds]}`);
  });

  it("the Summary group shows one row of tabs; every other group keeps its row of views", async () => {
    const b = await start();
    await shallowGasBarge(b);
    const rows = () => b.execute(() => document.querySelectorAll("[data-testid=views-panel] [role=tablist]").length);
    await click(b, "group-summary");
    assert.equal(await rows(), 1, "one row on the Summary");
    for (const g of ["drawings", "stability-propulsion", "boq-finance"]) {
      await click(b, `group-${g}`);
      assert.equal(await rows(), 2, `two rows on ${g}`);
    }
  });
});
