// A stand-in for a Chrome that still writes into its profile: writes small files into <profile>/Busy as
// fast as it can. It never outlives its test: it exits when its parent process is gone (a killed test
// file), and after <lifetime> ms whatever happens. It names the profile on its command line, as each
// Chrome process of a session does, so the harness's clean-up finds it.
//   node busy-writer.cjs <profile> <parent pid> <lifetime ms> --user-data-dir=<profile>
const fs = require("node:fs");
const [profile, parent, lifetime] = [process.argv[2], Number(process.argv[3]), Number(process.argv[4])];
const end = Date.now() + lifetime;
for (let i = 0; ; i++) {
  if (i % 64 === 0) {
    if (Date.now() > end) process.exit(0);
    try {
      process.kill(parent, 0);
    } catch {
      process.exit(0);
    }
  }
  try {
    fs.mkdirSync(`${profile}/Busy`, { recursive: true });
    fs.writeFileSync(`${profile}/Busy/f${i % 50}`, "x".repeat(4096));
  } catch {}
}
