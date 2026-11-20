// Web end-to-end tests of issue #1080 (spec 0017): the Start page, the setup wizard, and the one automatic
// damage run after Create. Run by e2e/run-web.sh, in headless Chrome, against the built web app. The two tests
// that wait for a whole damage run have files of their own, so the run starts them at the same time as this one.
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { box, choose, click, countDamageRuns, damageStatus, exists, firstStart, heading, nextTo, platformSnapshot, start, testid, typeIn, until } from "./wizard-steps.mjs";

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
    const msg = await b.$(testid("message-cargo"));
    await msg.waitForExist({ timeout: 5000 });
    assert.equal(await msg.getAttribute("role"), "alert");
    assert.match(await msg.getText(), /Cargo is needed/);
    await choose(b, "cargo", "LNG");
    await nextTo(b, "Service Area");
    await choose(b, "service_area", "coastal-and-river");
    await nextTo(b, "Formation");
    await choose(b, "formation", "self_propelled");
    await nextTo(b, "Hull");
    await choose(b, "hull_form", "raked_box");
    await typeIn(b, "length_overall_m", "140");
    await typeIn(b, "beam_m", "0.5");
    await until(b, "the limit", async () => /at least 1/.test(await (await b.$(testid("field-beam_m"))).getText()));
    assert.equal(await (await b.$(testid("wizard-next"))).getAttribute("aria-disabled"), "true");
    await click(b, testid("wizard-next"));
    assert.equal(await heading(b), "Hull", "a hard limit blocks Next");
    await typeIn(b, "beam_m", "36");
    await until(b, "the warning", async () => /usual range/.test(await (await b.$(`${testid("field-beam_m")} [data-severity=warning]`)).getText()));
    assert.equal(await (await b.$(testid("wizard-next"))).getAttribute("aria-disabled"), "false");
    await nextTo(b, "Your own limits");
    // Back keeps every answer.
    await click(b, testid("wizard-back"));
    await until(b, "the Hull step", async () => (await heading(b)) === "Hull");
    assert.equal(await (await b.$(box("beam_m"))).getValue(), "36");
  });

  it("does not let a river-only path reach Create without both limits", async () => {
    const b = await start();
    await firstStart(b);
    await click(b, testid("start-wizard"));
    await until(b, "the wizard", () => exists(b, testid("wizard")));
    await choose(b, "cargo", "LNG");
    await nextTo(b, "Service Area");
    await choose(b, "service_area", "river-only");
    await until(b, "the river limits", () => exists(b, box("draught_limit_m")));
    assert.ok(await exists(b, box("air_draft_limit_m")));
    await nextTo(b, "Formation");
    await choose(b, "formation", "self_propelled");
    await nextTo(b, "Hull");
    await choose(b, "hull_form", "raked_box");
    await typeIn(b, "length_overall_m", "100");
    await typeIn(b, "beam_m", "11");
    await nextTo(b, "Your own limits");
    assert.equal(await exists(b, box("min_freeboard_m")), false, "minimum freeboard is for coastal and river");
    await nextTo(b, "Review");
    assert.equal(await (await b.$(testid("wizard-create"))).getAttribute("aria-disabled"), "true");
    assert.equal((await b.$$(`${testid("wizard-stops")} li`)).length, 2, "both limits are missing");
    await click(b, testid("wizard-create"));
    assert.equal(await exists(b, testid("wizard")), true, "Create did nothing");
    // The link goes to the step; both limits typed, Create is ready.
    await click(b, testid("stop-go-draught_limit_m"));
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
    await until(b, "a Cargo chosen", async () => (await b.$(`${testid("choices-cargo")} input:checked`)).isExisting());
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

  it("after Create the Design is unsaved: an edit starts no second damage run, a second Create asks, Ctrl+N and Ctrl+O wait while the wizard is open, Tab stays inside", async () => {
    const b = await start();
    await firstStart(b);
    await click(b, testid("start-example"));
    await until(b, "the example list", () => exists(b, testid("start-examples")));
    await click(b, testid("start-example-coastal-lng-barge"));
    await until(b, "the review step", async () => (await exists(b, testid("wizard-heading"))) && (await heading(b)) === "Review");
    await countDamageRuns(b);
    await click(b, testid("wizard-create"));
    await until(b, "the wizard to close", async () => !(await exists(b, testid("wizard"))));
    await until(b, "damage stability to finish", async () => (await damageStatus(b)) === "run", 240000);
    // A Design made by Create has not been saved.
    assert.match(await (await b.$("header")).getText(), /\(unsaved\)/);
    assert.equal((await platformSnapshot(b)).dirty, true);
    // An edit after Create starts no second damage run.
    const area = await b.$('.field[data-input="service_area"] select');
    await area.waitForExist({ timeout: 15000 });
    await area.selectByAttribute("value", "river-only");
    await until(b, "the edit", async () => (await b.execute((el) => el.value, await b.$('.field[data-input="service_area"] select'))) === "river-only");
    await new Promise((r) => setTimeout(r, 2000));
    assert.equal(await b.execute(() => window.__runs), 1, "an edit after Create starts no second damage run");
    // A second Create asks before it discards the first.
    await b.execute(() => {
      window.__asked = [];
      window.__answer = false;
      window.confirm = (message) => {
        window.__asked.push(message);
        return window.__answer;
      };
    });
    await click(b, "button=New");
    await until(b, "the Start page", () => exists(b, testid("start-page")));
    await click(b, testid("start-example"));
    await until(b, "the example list", () => exists(b, testid("start-examples")));
    await click(b, testid("start-example-coastal-lng-barge"));
    await until(b, "the review step", async () => (await exists(b, testid("wizard-heading"))) && (await heading(b)) === "Review");
    await click(b, testid("wizard-create"));
    await until(b, "the question", async () => (await b.execute(() => window.__asked.length)) === 1);
    assert.equal(await exists(b, testid("wizard")), true, "a refusal leaves the wizard open");
    // Ctrl+N and Ctrl+O do nothing while the wizard is open.
    await b.keys(["Control", "n"]);
    await b.keys(["Control", "o"]);
    await new Promise((r) => setTimeout(r, 500));
    assert.equal(await exists(b, testid("start-page")), false, "Ctrl+N opens no Start page under the wizard");
    // What the designer sees after Ctrl+O: the wizard is still open, with no Start page under it.
    assert.equal(await exists(b, testid("wizard")), true, "the wizard is still open after Ctrl+O");
    assert.equal(await exists(b, testid("start-page")), false, "no Start page under the wizard after Ctrl+O");
    // Tab and Shift+Tab never leave the wizard.
    const insideWizard = () => b.execute(() => !!document.activeElement?.closest("[data-testid=wizard]"));
    for (let i = 0; i < 14; i += 1) {
      await b.keys(["Tab"]);
      assert.equal(await insideWizard(), true, `Tab ${i + 1} stays inside the wizard`);
    }
    for (let i = 0; i < 14; i += 1) {
      await b.keys(["Shift", "Tab"]);
      assert.equal(await insideWizard(), true, `Shift+Tab ${i + 1} stays inside the wizard`);
    }
    await b.keys(["Escape"]);
    await until(b, "the wizard to close", async () => !(await exists(b, testid("wizard"))));
    assert.equal(await exists(b, testid("start-page")), false, "Escape closed the wizard and nothing under it");
  });

  it("Tab and Shift+Tab wrap at the first and last element of the wizard and of the Start page", async () => {
    const b = await start();
    await firstStart(b);
    const active = () => b.execute(() => document.activeElement?.getAttribute("data-testid") || document.activeElement?.id || "");
    const inside = (id) => b.execute((t) => !!document.activeElement?.closest(`[data-testid=${t}]`), id);
    // The Start page: its heading is focused; the buttons are the tab stops.
    const stops = await b.execute(() => [...document.querySelectorAll("[data-testid=start-page] button")].map((n) => n.getAttribute("data-testid")));
    const first = stops[0];
    const last = stops[stops.length - 1];
    await b.execute((id) => document.querySelector(`[data-testid=${id}]`).focus(), last);
    await b.keys(["Tab"]);
    assert.equal(await active(), first, "Tab from the last element of the Start page goes to the first");
    await b.keys(["Shift", "Tab"]);
    assert.equal(await active(), last, "Shift+Tab from the first element of the Start page goes to the last");
    assert.equal(await inside("start-page"), true);
    // The wizard, on its first step with nothing chosen: the first stop is the Cargo choices, the last is Next.
    await click(b, testid("start-wizard"));
    await until(b, "the wizard", () => exists(b, testid("wizard")));
    const inChoices = () => b.execute(() => !!document.activeElement?.closest("[data-testid=choices-cargo]"));
    await b.keys(["Tab"]);
    assert.equal(await inChoices(), true, "the first Tab from the heading reaches the Cargo choices");
    await b.keys(["Shift", "Tab"]);
    assert.equal(await inside("wizard"), true, "Shift+Tab from the first element stays inside the wizard");
    assert.equal(await active(), "wizard-next", "Shift+Tab from the first stop goes to the last");
    await b.keys(["Tab"]);
    assert.equal(await inside("wizard"), true, "Tab from the last element stays inside the wizard");
    assert.equal(await inChoices(), true, "Tab from the last stop goes to the first");
  });
});
