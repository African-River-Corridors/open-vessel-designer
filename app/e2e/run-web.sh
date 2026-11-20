#!/usr/bin/env bash
# Run the web smoke tests: build the web app in test mode (it exposes a read-only hook the tests use),
# then drive headless Chrome against it. Needs the WebAssembly package (bash tools/build_web.sh),
# Chrome, and the CLI built at target/debug/ovd (cargo build -p ovd-cli).
#
# Each run has a temp root of its own. The test files put every Chrome profile and driver log there
# (e2e/browser.mjs). On every exit (the end, a failure, the time limit, Ctrl-C) the script stops
# whatever the run started and removes the root. A run first removes the roots of older runs whose
# script is gone. After the tests it checks that no process the run started is alive.
#
# Limits, in seconds: OVD_E2E_TEST_LIMIT_S per test (240), OVD_E2E_FILE_LIMIT_S per file (600),
# OVD_E2E_RUN_LIMIT_S for the whole run (900). OVD_E2E_JOBS test files run at once (3).
set -euo pipefail
cd "$(dirname "$0")/.."
[ -f wasm-pkg/ovd_web_bg.wasm ] || { echo "e2e-web: no app/wasm-pkg (run bash tools/build_web.sh)" >&2; exit 1; }
export OVD_CLI="${OVD_CLI:-$(cd .. && pwd)/target/debug/ovd}"
[ -x "$OVD_CLI" ] || { echo "e2e-web: no CLI at $OVD_CLI (cargo build -p ovd-cli)" >&2; exit 1; }

BASE="${TMPDIR:-/tmp}"
BASE="${BASE%/}"
# Old runs: a root whose owner script is no longer running is left over. Stop what still uses it, remove it.
for old in "$BASE"/ovd-e2e-run.*; do
  [ -d "$old" ] || continue
  owner=$(cat "$old/owner" 2>/dev/null || echo "")
  if [ -n "$owner" ] && kill -0 "$owner" 2>/dev/null; then continue; fi
  pkill -KILL -f "$old" 2>/dev/null || true
  rm -rf "$old"
  echo "e2e-web: removed a left-over temp root, $old"
done

ROOT=$(mktemp -d "$BASE/ovd-e2e-run.XXXXXX")
echo $$ > "$ROOT/owner"
NODE_PID=""

# Every process below $1, deepest first, then $1.
tree() { local c; for c in $(pgrep -P "$1" 2>/dev/null); do tree "$c"; done; echo "$1"; }
stop_all() {
  if [ -n "$NODE_PID" ]; then kill -KILL $(tree "$NODE_PID") 2>/dev/null || true; fi
  pkill -KILL -f "$ROOT" 2>/dev/null || true
}
cleanup() {
  local status=$?
  trap - EXIT INT TERM
  stop_all
  rm -rf "$ROOT"
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

npx vite build --config vite.web.config.ts --mode test --outDir dist-web-test

# The driver cache stays where webdriverio keeps it; the run's own temp files go to the root.
export WEBDRIVER_CACHE_DIR="${WEBDRIVER_CACHE_DIR:-$BASE}"
export OVD_E2E_ROOT="$ROOT" TMPDIR="$ROOT"
export OVD_E2E_FILE_LIMIT_S="${OVD_E2E_FILE_LIMIT_S:-600}"
# The driver is fetched once, here, before the files start in parallel (a cold cache in CI).
node e2e/fetch-driver.mjs "$ROOT/driver.env"
while IFS= read -r line; do export "$line"; done < "$ROOT/driver.env"
[ -x "${CHROMEDRIVER_PATH:-}" ] || { echo "e2e-web: no chromedriver" >&2; exit 1; }
RUN_LIMIT_S="${OVD_E2E_RUN_LIMIT_S:-900}"

start=$(date +%s)
node --test --test-concurrency="${OVD_E2E_JOBS:-3}" --test-timeout="$(( ${OVD_E2E_TEST_LIMIT_S:-240} * 1000 ))" \
  e2e/web-smoke.test.mjs e2e/web-smoke.*.test.mjs &
NODE_PID=$!
timed_out=0
while kill -0 "$NODE_PID" 2>/dev/null; do
  if (( $(date +%s) - start >= RUN_LIMIT_S )); then
    echo "e2e-web: the run passed its time limit of ${RUN_LIMIT_S} s; stopping it" >&2
    timed_out=1
    stop_all
    break
  fi
  sleep 1
done
status=0
wait "$NODE_PID" || status=$?
NODE_PID=""
[ "$timed_out" = 0 ] || status=124
echo "e2e-web: the run took $(( $(date +%s) - start )) s"

# Nothing the run started may outlive it.
left=$(pgrep -f "$ROOT" || true)
if [ -n "$left" ]; then
  echo "e2e-web: $(echo "$left" | wc -l | tr -d ' ') process(es) the run started are still alive:" >&2
  ps -o pid=,command= -p "$(echo $left | tr ' ' ',')" | cut -c1-160 >&2 || true
  [ "$status" != 0 ] || status=1
fi
exit "$status"
