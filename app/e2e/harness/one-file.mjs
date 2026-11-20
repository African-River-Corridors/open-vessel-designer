// A stand-in test file for web-smoke.harness.test.mjs. HARNESS_CASE picks what it does:
// "opens" opens a browser and checks its profile is in the run's root; "never-opens" asks for a Chrome
// that does not exist, so the session never opens; "hangs" opens a browser and then waits for ever.
import assert from "node:assert/strict";
import fs from "node:fs";
import { it } from "node:test";
import { openBrowser } from "../browser.mjs";

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
  }
});
