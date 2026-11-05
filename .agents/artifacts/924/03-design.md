---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
ticket: 924
phase: design
---

# #924 design: damage on demand in the app

Builds on #923 (engine switch, `damage_status`, progress callback with stop) and #921 (parallel
cases). Findings are cited as `R[n]` (`02-research.md`).

## 1. What runs when

| Action | Damage |
|---|---|
| open, new, edit, option and profile commands | not run (#923 switch off) |
| save | not run; the hash is the same (R10) |
| "Run damage stability" | run, for the chosen Option, off the main thread |

`DesignSession::evaluate` and `save` call #923's no-damage run. `ovd run` and `ovd report` keep
damage on by default (#923 item 2). The engine and the app give the same results for the same
inputs and switch; only the default differs, as the maintainer asked.

## 2. Where damage results live

The session keeps the last damage run per Option: `name → (fingerprint, results)`. The fingerprint
is a hash of the canonical JSON of the Option's inputs and of the Defaults Profile's text. The
profile's name is not enough: a loaded profile can change its content under the same name, and
inputs left as `default` would then take new values. `evaluate_json` uses the kept
results for an Option whose fingerprint still matches, and the no-damage results otherwise. A
rename moves the entry; a delete drops it.

Options: (a) keep in `ovd-shell` `AppState`; (b) keep in the front end.
**Pick (a).** The desktop and web builds share `ovd-shell` (`ovd-web/src/lib.rs:1-5`), so one code
path keeps them. The front end stays a reader.

After an edit, the fingerprint no longer matches, so the status is `not run` again. The old damage
cases are not shown as "out of date": damage verdicts of another hull mislead. The Damage view says
"Damage stability has not been run for these inputs."

## 3. The run, desktop

- A new async command `run_damage(option)` copies the Option out under the lock, releases it, and
  runs on `std::thread::spawn`, as `open_from_outside` does (R3).
- Progress: the engine callback (#923) stores `done` in an `AtomicUsize` and emits
  `damage-progress {option, done, total}`, at most every 100 ms. With #921 the callback runs on
  worker threads (R12), so it must be `Sync` and must only count.
- Stop: a `cancel_damage` command sets an `AtomicBool`; the callback returns "stop" when it is set.
  The status stays `not run`; nothing is kept.
- End: under the lock, keep the results only if the Option's fingerprint is unchanged, then emit
  `damage-done {option}`; the front end evaluates again.

**Decision for the maintainer:** an edit while damage runs.
- (a) The edit cancels the run. The results would be for inputs that are gone.
- (b) Edits are blocked until the run ends or is cancelled.
- Recommendation: **(a)**. The app stays live, which is why damage is off by default.

**Decision for the maintainer:** which Options one click runs.
- (a) The active Option only.
- (b) Every Option of the Design, one after the other, with one bar.
- Recommendation: **(a)**. One Option is what the Damage view shows. A coastal gas Option takes
  seconds (#921: 4.1 s and 7.9 s with 158 cases on a release build); several Options multiply it.

## 4. The run, web build

The wasm session runs in the page's main thread (R5); a damage run there freezes the page and no
progress can paint.

**Decision for the maintainer:** damage in the web build.
- (a) A Web Worker with its own wasm `Session`. The page sends the Design text and the Option name;
  the worker runs damage and posts `progress`, then `done` with the results. Cancel is
  `worker.terminate()`; the status stays `not run`. No engine stop is needed for the web.
- (b) Run on the main thread in steps (a few cases, then yield). Needs a step-wise engine API
  that #923 does not plan.
- (c) No damage in the web build; the view says "Run damage in the desktop app or with `ovd run`."
- Recommendation: **(a)**. It meets the ticket without a new engine API. Costs: one more wasm
  instance in memory while it runs, a Vite worker entry in `vite.web.config.ts`, and the worker
  hands the results back to the page's session (`Session.keep_damage(option, results_json)`).
  WebAssembly stays serial (#921), so the web run is slower than the desktop run; the estimate in
  section 6 uses the time measured in that build.

## 5. Where the action is

There is no status bar (R7). The MessageBar is the app's one status line.

**Decision for the maintainer:** where "Run damage stability" lives besides the Damage view.
- (a) A `note` in the MessageBar for the active Option when its damage is `not run` and its stack
  has a damage Rule Set: "Damage stability has not been run." with a **Run** button. While it runs,
  the same line shows the bar, the count and **Cancel**. The MessageBar gains an optional
  `action: {label, run}` beside `dismiss`.
- (b) A new status strip at the foot of the window.
- Recommendation: **(a)**. No new chrome; the line already exists for status.

In the Damage view, when not run: a short panel with the status, the case count, the estimate and
a **Run damage stability** button; while running, the progress bar and **Cancel** (both
`data-testid`s for e2e: `damage-run`, `damage-progress`, `damage-cancel`, `damage-status`). When
`not applicable`: the existing `app-damage-none` text and the trace's reasons. The Rule Checks
view shows the `not run` rows #923 writes.

## 6. Progress and time estimate

- The bar is `done / total` (cases), with the text "{done} of {total} cases".
- The time is measured in the front end, never in the engine (R13): `performance.now()` at the
  start and at each progress event. Time per case = elapsed / done. Remaining =
  (total − done) × time per case. Shown as "about {s} s left", updated with each event.
- Before a run, the estimate needs the case total and a time per case.

**Decision for the maintainer:** the case total before a run.
- (a) #923's no-damage results carry the total cases the run would assess, in an unhashed key
  (for example `damage_cases_total`). It is cheap: cases are listed before they are assessed.
- (b) The app does not know the total until the run starts; no estimate before it.
- Recommendation: **(a)**. Ask the #923 builder now, before its PR is written. Without it, the
  pre-run estimate and the dialog's "about N s" cannot be shown.

**Decision for the maintainer:** the time per case before the first run on this machine.
- (a) No seconds until a run has been measured: show the case count only ("158 cases"). After a
  run, keep the measured time per case in the app's settings (desktop) or `localStorage` (web) and
  use it next time.
- (b) A fixed seed from #921's release timing.
- Recommendation: **(a)**. A fixed seed is a figure from another machine; it would be wrong by more
  than the factor of 2 the ticket allows on a slow laptop or in the web build.

The e2e test logs the estimate after the first progress event and the actual time, and checks the
factor of 2 (the ticket's criterion).

## 7. Before a report

There is no report in the app (R14). Spec 0015 leaves an app report to a separate spec.

**Decision for the maintainer:** item 3 (the pre-report dialog).
- (a) Move item 3 and its e2e criterion to the ticket that adds a report to the app. #924 fixes the
  words now (below), so that ticket uses them as they are.
- (b) Show the dialog before a drawing export (PDF, DXF). A drawing is not a report; it would ask
  a question nobody needs on most exports.
- Recommendation: **(a)**.

The words (Message Ids in `locales/en/app.ftl`):
- Title: "Damage stability has not been run"
- Body, with a measured time: "Damage stability has not been run (about {seconds} s). Run it now,
  or make the report without it?"
- Body, with no measured time: "Damage stability has not been run ({cases} cases). Run it now, or
  make the report without it?"
- Buttons: **Run it now** (default), **Make the report without it**, **Cancel**.
- "Run it now" runs damage with the progress bar, then makes the report. A cancel during that run
  leaves the report unmade.

`ovd report` itself runs damage by default (#923 item 2), so the CLI needs no dialog.

## 8. Open and edit now that damage is off

- Open is faster: the coastal open measured in #922 (about 6 s on CI) loses the damage share,
  about 55 % on a coastal run (#921's profile). The e2e open limits can come down after measuring.
- The `smoke.test.mjs:431-440` test that expects damage cases on open changes: open, see `not run`,
  click **Run**, then see the cases and drawing.
- Hashes in the Compare table do not move (R10).

## 9. #921 as a blocker

The issue says "Blocked by #923; #921 (so the estimate is worth showing)". This design needs #921
only for speed: nothing here reads its code, except that the callback must be `Sync`.

**Decision for the maintainer:** keep #921 as a blocker.
- (a) Drop it. Build #924 on #923 alone. The estimate is right on a serial engine too; it is only
  longer. When #921 lands, the measured time per case falls by itself.
- (b) Keep it, as the issue says. The feature ships when it is fast.
- Recommendation: **(a)**. The progress bar and Cancel matter most when damage is slow. The e2e
  factor-of-2 check measures whichever engine is built.

## Dependencies

- #923 must give: the switch on `run_option` (or a sibling), the callback with stop, `damage_status`,
  and (decision 6a) the case total on a no-damage run.
- #921: speed only (decision 9). The callback must be `Sync`.
- PR #944 (#928): merge first; #924 edits the same `Damage.svelte`.
