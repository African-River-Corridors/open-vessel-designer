// The browsers of one web test file. One chromedriver serves the whole file; each test gets a new Chrome session
// with a new profile folder, ended when the test ends. Every exit path ends the sessions, stops the driver (and the
// Chrome it started) and removes the profile folders: a normal end, a failed test, a browser that never opened, the
// file's time limit, or a signal. Before this, every session started its own chromedriver, all alive until the file
// ended, and nothing had a time limit: one run waited for hours on a file whose browser had not opened.
//
// The run script (`e2e/run-web.sh`) gives each run a temp root, `OVD_E2E_ROOT`, and removes it afterwards. Chrome on
// macOS ignores TMPDIR, so each profile is put there by name (`--user-data-dir`).
import { execFileSync, spawn } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { after, afterEach } from "node:test";
import { remote } from "webdriverio";

const ROOT = process.env.OVD_E2E_ROOT || fs.mkdtempSync(path.join(os.tmpdir(), "ovd-e2e-file."));
const OWN_ROOT = !process.env.OVD_E2E_ROOT;
const FILE = path.basename(process.argv[1] ?? "test", ".mjs");
const DIR = fs.mkdtempSync(path.join(ROOT, `${FILE}.`));
/** The whole file must end within this time (seconds), or it fails and everything it started is stopped. */
const FILE_LIMIT_S = Number(process.env.OVD_E2E_FILE_LIMIT_S || 600);

let driver = null; // { child, port }
let profiles = 0;
const open = new Map(); // browser -> its profile folder

/** With OVD_E2E_COUNT set, one line per driver or session started goes to that file, for measuring a run. */
const count = (what) => process.env.OVD_E2E_COUNT && fs.appendFileSync(process.env.OVD_E2E_COUNT, `${what} ${FILE}\n`);

/** The chromedriver webdriverio already downloaded for this Chrome, or one that a first session downloads. */
async function driverBinary() {
  if (process.env.CHROMEDRIVER_PATH) return process.env.CHROMEDRIVER_PATH;
  const cache = path.join(process.env.WEBDRIVER_CACHE_DIR || os.tmpdir(), "chromedriver");
  const found = () =>
    fs.existsSync(cache)
      ? fs.readdirSync(cache).flatMap((v) => {
          const dir = path.join(cache, v);
          return fs.statSync(dir).isDirectory() ? fs.readdirSync(dir).map((d) => path.join(dir, d, process.platform === "win32" ? "chromedriver.exe" : "chromedriver")) : [];
        }).filter((f) => fs.existsSync(f))
      : [];
  return found().sort().at(-1) ?? null;
}

const freePort = () =>
  new Promise((resolve, reject) => {
    const s = net.createServer().once("error", reject).listen(0, "127.0.0.1", () => {
      const { port } = s.address();
      s.close(() => resolve(port));
    });
  });

async function startDriver() {
  if (driver) return driver;
  const bin = await driverBinary();
  if (!bin) return null; // none downloaded yet: the first session lets webdriverio fetch one
  const port = await freePort();
  // A process group of its own, so stopping the driver also stops every Chrome it started. Unref'd, so the driver
  // never keeps this file's process alive.
  const child = spawn(bin, [`--port=${port}`, `--log-path=${path.join(DIR, "chromedriver.log")}`], { stdio: "ignore", detached: true });
  child.unref();
  const until = Date.now() + 15000;
  for (;;) {
    try {
      if ((await fetch(`http://127.0.0.1:${port}/status`)).ok) break;
    } catch {}
    if (child.exitCode !== null || Date.now() > until) {
      stopDriver();
      throw new Error(`chromedriver did not start (${bin})`);
    }
    await new Promise((r) => setTimeout(r, 50));
  }
  driver = { child, port };
  count("driver");
  return driver;
}

function stopDriver() {
  if (!driver) return;
  try {
    process.kill(-driver.child.pid, "SIGKILL");
  } catch {}
  driver = null;
}

/** Open a Chrome session with a new profile in the run's temp root. `capabilities` as for webdriverio's `remote`. */
export async function openBrowser(capabilities) {
  const profile = path.join(DIR, `profile-${++profiles}`);
  const chrome = capabilities["goog:chromeOptions"] ?? {};
  const caps = {
    ...capabilities,
    "goog:chromeOptions": { ...chrome, args: [...(chrome.args ?? []), `--user-data-dir=${profile}`, "--disable-features=MacAppCodeSignClone"] },
  };
  const d = await startDriver();
  const where = d ? { hostname: "127.0.0.1", port: d.port } : {};
  const b = await remote({ ...where, logLevel: "warn", strictSelectors: false, capabilities: caps });
  open.set(b, profile);
  count("session");
  return b;
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms).unref());

/** The processes that name this profile on their command line: the session's Chrome. */
function usingProfile(profile) {
  try {
    return execFileSync("pgrep", ["-f", "--", `--user-data-dir=${profile}`], { encoding: "utf8" })
      .split("\n")
      .filter(Boolean)
      .map(Number)
      .filter((pid) => pid !== process.pid);
  } catch {
    return []; // pgrep exits 1 when nothing matches
  }
}

/** Wait until no process uses the profile: Chrome can still write to it for a moment after the session
 * ends. After `ms`, stop what is left. */
async function profileReleased(profile, ms = 5000) {
  const until = Date.now() + ms;
  while (usingProfile(profile).length > 0) {
    if (Date.now() > until) {
      for (const pid of usingProfile(profile)) {
        try {
          process.kill(pid, "SIGKILL");
        } catch {}
      }
      await sleep(200);
      return;
    }
    await sleep(100);
  }
}

/** Remove a profile once nothing writes to it, with retries as a second line. A removal that still fails
 * does not fail the test: it is reported, and the run script removes the run's root, and fails the run if
 * any process of it is alive, at the end. */
async function removeProfile(profile) {
  await profileReleased(profile);
  try {
    fs.rmSync(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
  } catch (e) {
    console.error(`# ${FILE}: could not remove ${path.basename(profile)} (${e.code}); the run's clean-up removes it`);
  }
}

/** End every session this file opened and remove their profiles. */
export async function closeBrowsers() {
  const all = [...open];
  open.clear();
  await Promise.all(
    all.map(async ([b, profile]) => {
      await Promise.race([b.deleteSession().catch(() => {}), sleep(10000)]);
      await removeProfile(profile);
    }),
  );
}

function finish() {
  stopDriver();
  try {
    fs.rmSync(OWN_ROOT ? ROOT : DIR, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
  } catch {}
}

afterEach(closeBrowsers);
after(async () => {
  await closeBrowsers();
  finish();
});
process.on("exit", finish);
for (const signal of ["SIGINT", "SIGTERM"]) process.on(signal, () => process.exit(1));
setTimeout(() => {
  console.error(`# ${FILE}: over the file time limit of ${FILE_LIMIT_S} s; stopping its browsers`);
  process.exit(1);
}, FILE_LIMIT_S * 1000).unref();
