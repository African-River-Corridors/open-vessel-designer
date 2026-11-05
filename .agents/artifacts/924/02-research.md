---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
ticket: 924
phase: research
---

# #924 research findings

Read at `a63ca819`. Nothing was built or run. #923 and #921 have no PR yet; #921 is in progress in
a local worktree (uncommitted). PR #944 (#928) is open. Numbers in brackets are the questions.

## A. How the app runs an Option today

**[1] Desktop evaluation.** The state is one `Mutex<AppState>` (`lib.rs:15-22`). The `evaluate`
command is synchronous and holds the lock for the whole run (`lib.rs:137-140`).
`AppState::evaluate_json` calls `DesignSession::evaluate` (`state.rs:200-212`), which checks the file
and calls `run_option` on each Option with no hard error (`session.rs:1061-1095`). So every
Option of the Design runs on each evaluation, with damage, today.

**[2] Save.** `DesignSession::save` runs every Option again to write its hash (`session.rs:1100-1118`).

**[3] Precedent.** `open_from_outside` (`lib.rs:218-246`) runs on `std::thread::spawn`, takes the
lock only to swap the state, and tells the front end with `app.emit("design-opened")`. The front end
listens with `listen` (`tauri.ts:33-34`). A damage run can copy this shape.

**[4] Front-end evaluation.** An edit is sent after a 150 ms pause, then the whole Design is
evaluated (`App.svelte:40`, `:96-101`). Every command and file action evaluates again through
`adopt` (`App.svelte:63-73`). `lastGood` keeps the last results per Option name, shown as out of
date while an Option has an error (`App.svelte:54-59`).

**[5] Web build.** The wasm `Session` runs in the page's main thread (`web.ts:1-4`, `:147`;
`ovd-web/src/lib.rs:1-60`). A long call freezes the page. There is no Web Worker in the web
build (`git grep -n Worker -- app/src` finds none).

## B. The Damage view and the status line

**[6] Damage view.** It reads the `damage`, `damage_by_rule` and `marpol_damage` blocks and the
`rule_trace` of the results; it computes nothing (`Damage.svelte:1-15`, `damage.ts:1-5`). With no
damage set it shows `app-damage-none` and the trace's reasons (`Damage.svelte:47-58`,
`locales/en/app.ftl:450`). The drawings come from `damage_views` (`MainPanel.svelte:97-102`).

**[7] Status bar.** There is none. The app has a header, the one-line `MessageBar` under it
(`App.svelte:228-256`), the left, main and bottom panels (`:259-276`). The MessageBar shows one
message at a time with a count, and a `dismiss` action only (`MessageBar.svelte:1-60`). The bottom
panel's Compare table has a `Status` row (`BottomPanel.svelte:240-246`).

**[8] PR #944 (#928).** Changes `Damage.svelte`, `damage.ts`, `api.ts` and `views.rs` so a
`damage_by_rule` set shows its governing case and drawing. #924's Damage view work must start
from it.

## C. Damage in the engine

**[9] Where damage runs.** Inside `simulate_once`, after the tank fit: the IGC cases at
`simulate.rs:3250-3272`, the ADN and ES-TRIN cases at `:3273-3320`, the liquid cases at
`:1659-1720`. The fit does not read them, so skipping damage should not move sizing. #923's
byte-equal test is the proof.

**[10] Hash.** `damage`, `damage_by_rule`, `marpol_damage`, `oil_outflow` and `rule_trace` are
all in `UNHASHED_KEYS` (`run.rs:362-383`). So a run without damage should store the same results
hash. #923's byte-equal test covers it; #924 relies on it for save.

**[11] #923's promise.** An engine switch, a progress callback (cases done / total) and a way for
it to stop the run; `damage_status` `run` / `not run` / `not applicable`; `ovd run` on by default
with `--no-damage`; skipped rows read `not run`; the verdict line never says "meets every" when
not run. Nothing yet says whether the case **total** is known on a run without damage.

**[12] #921.** In progress: a cargo feature `parallel`, on by default; WebAssembly stays serial
(`damage.rs` `#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]`). Cases run per loading
condition on worker threads and come back in case order (`run_ordered`). So a progress callback
may be called from several threads.

**[13] Timing.** `ovd-format` uses no clock (`git grep Instant -- crates/ovd-format` finds none).
`std::time::Instant::now()` panics on `wasm32-unknown-unknown`. A time per case has to be
measured outside the engine: in the front end (`performance.now()`), which both builds have.

## D. Reports

**[14] No app report.** The app has no report action (`git grep -i report -- app/src app/src-tauri`
finds only the principal-characteristics strip). Spec 0015 says the desktop app may show the
report later, under a separate spec (`docs/specs/0015-concept-report-format.md:68`). `ovd report`
is issue #833, open. So item 3 of #924 ("before a report, a dialog") has no host in the app yet.

## E. Tests

**[15] e2e.** `smoke.test.mjs:400-440` fills a coastal-and-river gas Design with the CLI and checks
the damage cases and drawing on open. The open limit is 18 s (`SLOW_OPEN_MS`, `:411`), set at three
times the measured release open time of about 6 s (#922, merged in `a63ca819`). The e2e job builds an
optimised profile with debug assertions (`app/e2e/run.sh:1-6`).
