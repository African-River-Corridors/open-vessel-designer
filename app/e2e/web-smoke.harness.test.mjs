// The web test harness itself (e2e/browser.mjs): a test file ends, whatever happens in it, and leaves no
// browser, no driver and no profile behind. Each case runs a stand-in test file in a fresh root of its own.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { after, describe, it } from "node:test";

const FILE = path.join(import.meta.dirname, "harness", "one-file.mjs");
// The cases' folders live under the run's root, so run-web.sh's clean-up covers them too. Run alone,
// this file makes a root of its own and removes it at the end.
const OWN = !process.env.OVD_E2E_ROOT;
const ROOT = process.env.OVD_E2E_ROOT || fs.mkdtempSync(path.join(os.tmpdir(), "ovd-e2e-harness."));
const alive = (pid) => {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
};
const stopAll = (root) => {
  try {
    execFileSync("pkill", ["-KILL", "-f", root]);
  } catch {}
};
after(() => {
  stopAll(ROOT);
  if (OWN) fs.rmSync(ROOT, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
});

/** Run the stand-in file; give how it ended, how long it took and what of it is still there. With
 * `killOn`, kill the file (SIGKILL) as soon as its output matches, and give the writer's pid. */
async function runCase(which, env = {}, killOn = null) {
  const root = fs.mkdtempSync(path.join(ROOT, `harness-${which}.`));
  const started = Date.now();
  // Not as a part of this run's own test tree: without NODE_TEST_CONTEXT the stand-in is a run of its own.
  const { NODE_TEST_CONTEXT, ...outer } = process.env;
  const child = spawn(process.execPath, ["--test", FILE], {
    env: { ...outer, HARNESS_CASE: which, OVD_E2E_ROOT: root, ...env },
    stdio: ["ignore", "pipe", "pipe"],
  });
  let out = "";
  child.stdout.on("data", (d) => {
    out += d;
    if (killOn && killOn.test(out)) {
      // The file runs in a process below the runner: kill every process of it but the writer, which
      // must end on its own.
      const writer = Number(out.match(/WRITER (\d+)/)?.[1] ?? 0);
      const tree = (pid) => {
        let kids = [];
        try {
          kids = execFileSync("pgrep", ["-P", String(pid)], { encoding: "utf8" }).split("\n").filter(Boolean).map(Number);
        } catch {}
        return [...kids.flatMap(tree), pid];
      };
      for (const pid of tree(child.pid)) if (pid !== writer) try { process.kill(pid, "SIGKILL"); } catch {}
      killOn = null;
    }
  });
  child.stderr.on("data", (d) => (out += d));
  const code = await new Promise((resolve) => {
    const guard = setTimeout(() => child.kill("SIGKILL"), 90000);
    child.on("exit", (c, signal) => (clearTimeout(guard), resolve(signal ? `killed by ${signal}` : c)));
  });
  const seconds = (Date.now() - started) / 1000;
  const writer = Number(out.match(/WRITER (\d+)/)?.[1] ?? 0);
  // A writer must end on its own when its file is killed: give it a moment, as the guard checks often.
  let writerAlive = false;
  if (writer) {
    for (let i = 0; i < 50 && alive(writer); i++) await new Promise((r) => setTimeout(r, 100));
    writerAlive = alive(writer);
  }
  let left_alive = "";
  try {
    left_alive = execFileSync("pgrep", ["-f", root], { encoding: "utf8" }).trim();
  } catch {}
  stopAll(root); // whatever the case, nothing of it outlives it
  const left = fs.readdirSync(root, { recursive: true }).filter((f) => /profile-\d+$|chromedriver\.log$/.test(f));
  fs.rmSync(root, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
  return { code, seconds, alive: left_alive, left, out, writer, writerAlive };
}

describe("the web test harness (e2e/browser.mjs)", () => {
  it("every web test file opens its browsers through e2e/browser.mjs, never webdriverio's remote() itself", () => {
    const direct = fs
      .readdirSync(import.meta.dirname)
      .filter((f) => /^web-smoke\..*\.mjs$|^web-smoke\.test\.mjs$/.test(f))
      .filter((f) => /from ["']webdriverio["']/.test(fs.readFileSync(path.join(import.meta.dirname, f), "utf8")));
    assert.deepEqual(direct, [], "these files start their own drivers: use openBrowser and closeBrowsers from ./browser.mjs");
  });

  it("a file whose browser never opens ends at once, its driver stopped", async () => {
    const r = await runCase("never-opens");
    assert.equal(r.code, 0, r.out);
    assert.ok(r.seconds < 30, `the file took ${r.seconds} s to end`);
    assert.equal(r.alive, "", `processes left alive: ${r.alive}`);
    assert.deepEqual(r.left, [], "nothing left in the root");
  });

  it("a file that opens a browser keeps its profile in the run's root and removes it at the end", async () => {
    const r = await runCase("opens");
    assert.equal(r.code, 0, r.out);
    assert.equal(r.alive, "", `processes left alive: ${r.alive}`);
    assert.deepEqual(r.left, [], "nothing left in the root");
  });

  it("a profile still being written to while its session ends is removed, and the test passes", async () => {
    const r = await runCase("busy-profile");
    assert.equal(r.code, 0, r.out);
    assert.equal(r.alive, "", `processes left alive: ${r.alive}`);
    assert.deepEqual(r.left, [], "nothing left in the root");
  });

  it("a writer started by a test file ends by itself when the file is killed", async () => {
    const r = await runCase("busy-killed", {}, /WRITER \d+/);
    assert.ok(r.writer > 0, r.out);
    assert.equal(r.writerAlive, false, `the writer ${r.writer} outlived its killed file`);
  });

  it("a file over its time limit fails, and its browser and driver are stopped", async () => {
    const r = await runCase("hangs", { OVD_E2E_FILE_LIMIT_S: "8" });
    assert.notEqual(r.code, 0, "a hung file fails");
    assert.ok(r.seconds < 30, `the file took ${r.seconds} s to end`);
    assert.match(r.out, /over the file time limit/);
    assert.equal(r.alive, "", `processes left alive: ${r.alive}`);
  });
});
