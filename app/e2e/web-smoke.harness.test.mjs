// The web test harness itself (e2e/browser.mjs): a test file ends, whatever happens in it, and leaves no
// browser, no driver and no profile behind. Each case runs a stand-in test file in a fresh root of its own.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, it } from "node:test";

const FILE = path.join(import.meta.dirname, "harness", "one-file.mjs");

/** Run the stand-in file; give how it ended, how long it took and what of it is still there. */
async function runCase(which, env = {}) {
  const root = fs.mkdtempSync(path.join(process.env.OVD_E2E_ROOT || os.tmpdir(), `harness-${which}.`));
  const started = Date.now();
  // Not as a part of this run's own test tree: without NODE_TEST_CONTEXT the stand-in is a run of its own.
  const { NODE_TEST_CONTEXT, ...outer } = process.env;
  const child = spawn(process.execPath, ["--test", FILE], {
    env: { ...outer, HARNESS_CASE: which, OVD_E2E_ROOT: root, ...env },
    stdio: ["ignore", "pipe", "pipe"],
  });
  let out = "";
  child.stdout.on("data", (d) => (out += d));
  child.stderr.on("data", (d) => (out += d));
  const code = await new Promise((resolve) => {
    const guard = setTimeout(() => child.kill("SIGKILL"), 90000);
    child.on("exit", (c, signal) => (clearTimeout(guard), resolve(signal ? `killed by ${signal}` : c)));
  });
  const seconds = (Date.now() - started) / 1000;
  let alive = "";
  try {
    alive = execFileSync("pgrep", ["-f", root], { encoding: "utf8" }).trim();
  } catch {}
  const left = fs.readdirSync(root, { recursive: true }).filter((f) => /profile-\d+$|chromedriver\.log$/.test(f));
  fs.rmSync(root, { recursive: true, force: true });
  return { code, seconds, alive, left, out };
}

describe("the web test harness (e2e/browser.mjs)", () => {
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

  it("a file over its time limit fails, and its browser and driver are stopped", async () => {
    const r = await runCase("hangs", { OVD_E2E_FILE_LIMIT_S: "8" });
    assert.notEqual(r.code, 0, "a hung file fails");
    assert.ok(r.seconds < 30, `the file took ${r.seconds} s to end`);
    assert.match(r.out, /over the file time limit/);
    assert.equal(r.alive, "", `processes left alive: ${r.alive}`);
  });
});
