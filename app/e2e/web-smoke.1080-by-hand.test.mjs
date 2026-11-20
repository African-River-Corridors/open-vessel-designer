// A web end-to-end test of issue #1080 (spec 0017), in a file of its own because it waits for a whole damage
// run: the worked example typed by hand. The steps it shares are in e2e/wizard-steps.mjs.
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { WORKED, box, cargoVolumeOnPage, click, countDamageRuns, damageStatus, exists, firstStart, nativeCargoVolume, platformSnapshot, start, testid, typeWorkedExample, typedInputs, until } from "./wizard-steps.mjs";

describe("OVD web: the setup wizard, the worked example typed by hand (issue #1080)", () => {
  it("makes the worked example by hand: 11 typed inputs and no other, damage runs once by itself, the cargo volume equals the command line's", async () => {
    const b = await start();
    await firstStart(b);
    await typeWorkedExample(b);
    // The review lists the 11 typed choices, the made Option name and the count left to the Defaults Profile.
    assert.equal((await b.$$(`${testid("wizard-choices")} li`)).length, 11);
    assert.equal(await (await b.$(box("option-name"))).getValue(), "LNG 140x29");
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
});
