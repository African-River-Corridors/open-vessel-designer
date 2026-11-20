// A stand-in test file for web-smoke.harness.test.mjs. HARNESS_CASE picks what it does:
// "opens" opens a browser and checks its profile is in the run's root; "never-opens" asks for a Chrome
// that does not exist, so the session never opens; "hangs" opens a browser and then waits for ever;
// "busy-profile" keeps a process writing files into the profile while the session ends, as Chrome
// can for a moment after it is told to quit (harness/busy-writer.cjs); "busy-killed" starts that writer
// and then waits for the harness to kill this file.
import assert from "node:assert/strict";
import fs from "node:fs";
import { spawn } from "node:child_process";
import path from "node:path";
import { it } from "node:test";
import { closeBrowsers, openBrowser } from "../browser.mjs";

const chrome = (binary) => ({
  browserName: "chrome",
  "wdio:enforceWebDriverClassic": true,
  "goog:chromeOptions": {
    ...(binary ? { binary } : fs.existsSync("/Applications/Google Chrome.app") ? { binary: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" } : {}),
    args: ["--headless=new", "--no-sandbox"],
  },
});

const which = process.env.HARNESS_CASE;
it(which, async () => {
  if (which === "never-opens") {
    await assert.rejects(openBrowser(chrome("/no/such/chrome")));
  } else {
    const b = await openBrowser(chrome(process.env.CHROME_BINARY));
    assert.ok(b.sessionId, "a session is open");
    const profiles = fs.readdirSync(process.env.OVD_E2E_ROOT, { recursive: true }).filter((f) => /profile-\d+$/.test(f));
    assert.equal(profiles.length, 1, `one profile in the run's root, found ${profiles}`);
    if (which === "hangs") await new Promise(() => setInterval(() => {}, 1000));
    if (which === "busy-profile" || which === "busy-killed") {
      const profile = path.join(process.env.OVD_E2E_ROOT, profiles[0]);
      const writer = spawn(
        process.execPath,
        [path.join(import.meta.dirname, "busy-writer.cjs"), profile, String(process.pid), "30000", `--user-data-dir=${profile}`],
        { stdio: "ignore" },
      );
      console.log(`WRITER ${writer.pid}`);
      try {
        await new Promise((r) => setTimeout(r, 300));
        // "busy-killed": the harness kills this file now, with the writer running.
        if (which === "busy-killed") await new Promise(() => setInterval(() => {}, 1000));
        await closeBrowsers();
        assert.equal(fs.existsSync(profile), false, "the profile is removed");
        assert.notEqual(writer.exitCode ?? writer.signalCode, null, "the writer was stopped");
      } finally {
        writer.kill("SIGKILL");
      }
    }
  }
});
