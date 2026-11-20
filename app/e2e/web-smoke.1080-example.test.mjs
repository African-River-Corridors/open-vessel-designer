// A web end-to-end test of issue #1080 (spec 0017), in a file of its own because it waits for a whole damage
// run: starting from the example. The steps it shares are in e2e/wizard-steps.mjs.
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { WORKED, click, countDamageRuns, damageStatus, exists, firstStart, heading, platformSnapshot, start, testid, typeWorkedExample, typedInputs, until } from "./wizard-steps.mjs";

describe("OVD web: the setup wizard, starting from the example (issue #1080)", () => {
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
});
