// Fetch chromedriver (and Chrome for Testing, where no Chrome is installed) once, before the test files
// start in parallel. Writes, to the file named as its argument, the lines run-web.sh exports:
// CHROMEDRIVER_PATH and CHROME_BINARY.
// Without this, files started at the same moment on a cold cache each download and unpack the driver
// into the same folder, and one of them fails (`spawn …/chromedriver EACCES`, `ECONNREFUSED`).
import fs from "node:fs";
import { startWebDriver } from "@wdio/utils";

const binary = process.env.CHROME_BINARY || (fs.existsSync("/Applications/Google Chrome.app") ? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" : undefined);
const options = {
  capabilities: { browserName: "chrome", "goog:chromeOptions": { ...(binary ? { binary } : {}), args: ["--headless=new"] } },
};
const started = Date.now();
const driver = await startWebDriver(options);
driver.kill("SIGKILL");
const chrome = options.capabilities["goog:chromeOptions"].binary;
console.error(`e2e-web: chromedriver ready in ${((Date.now() - started) / 1000).toFixed(1)} s: ${driver.spawnfile}`);
fs.writeFileSync(process.argv[2], `CHROMEDRIVER_PATH=${driver.spawnfile}\n` + (chrome ? `CHROME_BINARY=${chrome}\n` : ""));
