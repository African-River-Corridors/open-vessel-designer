// Web end-to-end tests of issue #1084 (spec 0017 D9, D10, D11): Essentials and All inputs, the filter box, and
// help on the page. The built web app in headless Chrome, as in web-smoke.test.mjs. This file does not switch to
// All inputs at the start: the Input panel opens on Essentials, as a first start does.
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { closeBrowsers, openBrowser } from "./browser.mjs";
import { serve } from "./web-server.mjs";

const here = import.meta.dirname;
const DIST = path.resolve(here, "../dist-web-test");

// An input that holds a default and is not marked essential: Essentials hides it until it is typed.
const DEFAULT_INPUT = "bow_rake_length_to_depth_ratio";

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
const until = (b, what, check, timeout = 30000) =>
  b.waitUntil(check, { timeout, interval: 100, timeoutMsg: `timed out waiting for: ${what}` });
const field = (b, name) => b.$(`.field[data-input="${name}"]`);
const shown = async (b, name) => (await field(b, name)).isExisting();
const fieldNames = (b) => b.execute(() => [...document.querySelectorAll(".field[data-input]")].map((e) => e.getAttribute("data-input")));
const pressed = async (b, id) => (await (await b.$(`[data-testid=${id}]`)).getAttribute("aria-pressed")) === "true";
const groupOpen = (b, name) => b.execute((n) => document.querySelector(`.field[data-input="${n}"]`).closest("details").open, name);

// The page learns of a toggle from the details element's own event, which comes a moment after the click. Wait for it,
// or text typed at once in the filter box reaches the page first and the page never learns that the group closed.
const toggleGroup = async (b, name) => {
  await b.execute((n) => document.querySelector(`.field[data-input="${n}"]`).closest("details").querySelector("summary").click(), name);
  await b.pause(100);
};

/** Open the app and answer the Service Area, as the starter Design asks, so the panel and the results are there. */
async function ready(b) {
  await b.url(`${origin}/ovd/app`);
  const area = () => b.$('.field[data-input="service_area"] select');
  await until(b, "the Service Area", async () => (await area()).isExisting());
  // The first-run notice covers the page until it is accepted; it is drawn with the page.
  const notice = b.$("button=I understand");
  if (await notice.isExisting()) await notice.click();
  await until(b, "the notice to close", async () => !(await b.$("[data-testid=notice]").isExisting()));
  // A first start opens the Start page after the notice (spec 0017): take its skip link.
  // After a reload the Design is restored and there is no Start page: skip only when it is there.
  const skip = await b.$("[data-testid=start-skip]");
  if (await skip.isExisting()) {
    await skip.click();
    await until(b, "the Start page to close", async () => !(await b.$("[data-testid=start-page]").isExisting()));
  }
  const select = await area();
  if (await b.execute((el) => el.value === "__unset__", select)) {
    await select.selectByAttribute("value", "river-only");
    for (const [name, text] of [["draught_limit_m", "12"], ["air_draft_limit_m", "40"]]) {
      const box = b.$(`.field[data-input="${name}"] input[type=text]`);
      await box.waitForExist({ timeout: 15000, timeoutMsg: `the ${name} box` });
      await box.setValue(text);
    }
  }
  await until(b, "results", async () => (await b.$("[data-testid=outputs]")).isExisting());
}

before(async () => {
  ({ server, origin } = await serve(DIST));
});
after(async () => {
  await closeBrowsers();
  server?.close();
});

describe("Essentials, All inputs, the filter box and help on the page (issue #1084)", () => {
  it("opens on Essentials; a default input that is not essential is hidden until it is typed in All inputs", async () => {
    const b = await start();
    await ready(b);
    assert.equal(await pressed(b, "mode-essentials"), true, "the first start opens on Essentials");
    assert.equal(await shown(b, DEFAULT_INPUT), false, "a default input that is not essential is hidden");
    assert.ok((await b.$("[data-testid=inputs-more]").getText()).length > 0, "the line that counts the inputs left out");

    await (await b.$("[data-testid=mode-all]")).click();
    await until(b, "the input in All inputs", () => shown(b, DEFAULT_INPUT));
    assert.equal(await (await b.$("[data-testid=inputs-more]")).isExisting(), false, "no count line in All inputs");
    await (await b.$(`.field[data-input="${DEFAULT_INPUT}"] input[type=text]`)).setValue("3");
    await until(b, "the input to be typed", async () => (await (await b.$(`.field[data-input="${DEFAULT_INPUT}"] .badge`)).getText()).toLowerCase().includes("typed"));

    await (await b.$("[data-testid=mode-essentials]")).click();
    await until(b, "Essentials to draw", async () => (await b.$("[data-testid=inputs-more]")).isExisting());
    assert.equal(await shown(b, DEFAULT_INPUT), true, "a typed input shows in Essentials");
  });

  it("the Show all inputs link in Essentials switches to All inputs", async () => {
    const b = await start();
    await ready(b);
    await (await b.$("[data-testid=inputs-more] button")).click();
    await until(b, "All inputs", () => pressed(b, "mode-all"));
    assert.equal(await shown(b, DEFAULT_INPUT), true);
  });

  it("keeps the choice of mode after a reload", async () => {
    const b = await start();
    await ready(b);
    await (await b.$("[data-testid=mode-all]")).click();
    await until(b, "All inputs", () => pressed(b, "mode-all"));
    await ready(b);
    assert.equal(await pressed(b, "mode-all"), true, "All inputs after the reload");
    assert.equal(await shown(b, DEFAULT_INPUT), true);
    await (await b.$("[data-testid=mode-essentials]")).click();
    await ready(b);
    assert.equal(await pressed(b, "mode-essentials"), true, "Essentials after the reload");
  });

  it("the filter finds \"freeboard\" in either mode, opens closed groups, and clearing it restores the panel", async () => {
    const b = await start();
    await ready(b);
    for (const mode of ["essentials", "all"]) {
      await (await b.$(`[data-testid=mode-${mode}]`)).click();
      await until(b, mode, () => pressed(b, `mode-${mode}`));
      const before = await fieldNames(b);
      // Close the group of the input, so the filter must open it.
      await toggleGroup(b, "min_freeboard_m");
      await until(b, `the group to close (${mode})`, async () => !(await groupOpen(b, "min_freeboard_m")));

      await (await b.$("[data-testid=input-filter]")).setValue("FREEBOARD");
      await until(b, "the filter result", async () => {
        const names = await fieldNames(b);
        return names.length > 0 && names.every((n) => n.includes("freeboard"));
      });
      assert.ok((await fieldNames(b)).includes("min_freeboard_m"));
      assert.equal(await groupOpen(b, "min_freeboard_m"), true, "the group is open");
      assert.match(await (await b.$("[data-testid=input-filter-count]")).getText(), /^\d+ inputs? match/);

      await (await b.$("[data-testid=input-filter-clear]")).click();
      await until(b, "the panel restored", async () => (await fieldNames(b)).length === before.length);
      assert.deepEqual(await fieldNames(b), before, `${mode}: the same inputs as before`);
      assert.equal(await groupOpen(b, "min_freeboard_m"), false, "the group is closed again");
      await toggleGroup(b, "min_freeboard_m"); // open it again for the next round
    }
  });

  it("the \"fix the error under\" link reaches an input Essentials hides, by switching to All inputs", async () => {
    const b = await start();
    await ready(b);
    // A channel shallower than its under-keel clearance is an error under both inputs. The clearance holds its
    // default, so Essentials hides it.
    const clearance = "channel_under_keel_clearance_m";
    await (await b.$("[data-testid=mode-all]")).click();
    await until(b, "the channel depth", () => shown(b, "channel_depth_m"));
    const none = b.$('.field[data-input="channel_depth_m"] .none input');
    if ((await none.isExisting()) && (await none.isSelected())) await none.click();
    const box = b.$('.field[data-input="channel_depth_m"] input[type=text]');
    await box.waitForExist({ timeout: 15000, timeoutMsg: "the channel depth box" });
    await box.setValue("0.1");
    await until(b, "the error link", () => b.$(`[data-testid=stale-why] a[href="#${clearance}"]`).isExisting());
    await (await b.$("[data-testid=mode-essentials]")).click();
    await until(b, "Essentials", () => pressed(b, "mode-essentials"));
    assert.equal(await shown(b, clearance), false, "Essentials hides the clearance");
    await (await b.$(`[data-testid=stale-why] a[href="#${clearance}"]`)).click();
    await until(b, "All inputs", () => pressed(b, "mode-all"));
    await until(b, "the clearance to show", () => shown(b, clearance));
    await until(b, "the clearance box to have focus", () => b.execute((n) => document.activeElement?.closest(`.field[data-input="${n}"]`) !== null, clearance));
  });

  it("\"About this input\" opens the help under the field and closes it again", async () => {
    const b = await start();
    await ready(b);
    const toggle = b.$(`.field[data-input="min_freeboard_m"] [data-testid=about-toggle]`);
    assert.equal(await toggle.getAttribute("aria-expanded"), "false");
    assert.match(await toggle.getText(), /About this input/);
    await toggle.click();
    const text = b.$(`.field[data-input="min_freeboard_m"] [data-testid=about-text]`);
    await until(b, "the help text", () => text.isExisting());
    assert.match(await text.getText(), /Default:|Basis:|range/);
    assert.equal(await toggle.getAttribute("aria-expanded"), "true");
    await toggle.click();
    await until(b, "the help to close", async () => !(await text.isExisting()));
    assert.equal(await toggle.getAttribute("aria-expanded"), "false");
  });

  it("works from the keyboard: the switch, the filter box and the help control take focus and Enter or Space", async () => {
    const b = await start();
    await ready(b);
    await b.execute(() => document.querySelector("[data-testid=mode-all]").focus());
    await b.keys(["Enter"]);
    await until(b, "All inputs by keyboard", () => pressed(b, "mode-all"));
    await b.execute(() => document.querySelector("[data-testid=input-filter]").focus());
    await b.keys("beam");
    await until(b, "the filter by keyboard", async () => (await fieldNames(b)).includes("beam_m"));
    await b.keys(["Escape"]);
    await until(b, "Escape clears", async () => (await fieldNames(b)).length > 5);
    await b.execute(() => document.querySelector('.field[data-input="min_freeboard_m"] [data-testid=about-toggle]').focus());
    await b.keys([" "]);
    await until(b, "help by keyboard", async () => (await b.$(`.field[data-input="min_freeboard_m"] [data-testid=about-text]`)).isExisting());
    const names = await b.execute(() => [
      document.querySelector("[data-testid=input-filter]").getAttribute("aria-label"),
      document.querySelector("[data-testid=mode-all]").closest("[role=group]").getAttribute("aria-label"),
      document.querySelector('.field[data-input="min_freeboard_m"] [data-testid=about-toggle]').getAttribute("aria-label"),
    ]);
    assert.ok(names.every((n) => n && n.length > 0), `accessible names: ${names}`);
  });
});
