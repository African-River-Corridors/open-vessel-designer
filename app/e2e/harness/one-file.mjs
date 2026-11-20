// A stand-in test file for web-smoke.harness.test.mjs. HARNESS_CASE picks what it does:
// "opens" opens a browser and checks its profile is in the run's root; "never-opens" asks for a Chrome
// that does not exist, so the session never opens; "hangs" opens a browser and then waits for ever;
// "busy-profile" keeps a process writing files into the profile while the session ends, as Chrome
// can for a moment after it is told to quit.
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
    if (which === "busy-profile") {
      const profile = path.join(process.env.OVD_E2E_ROOT, profiles[0]);
      // A writer that names the profile on its command line, as each Chrome process of the session does.
      const writer = spawn(
        process.execPath,
        ["-e", "const fs=require('fs'),p=process.argv[1];let i=0;for(;;){try{fs.mkdirSync(p+'/Busy',{recursive:true});fs.writeFileSync(p+'/Busy/f'+(i++%50),'x'.repeat(4096))}catch{}}", profile, `--user-data-dir=${profile}`],
        { stdio: "ignore" },
      );
      await new Promise((r) => setTimeout(r, 300));
      await closeBrowsers();
      assert.equal(fs.existsSync(profile), false, "the profile is removed");
      assert.notEqual(writer.exitCode ?? writer.signalCode, null, "the writer was stopped");
    }
  }
});
