---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
ticket: 924
phase: outline
---

# #924 outline

Starts after #923 and PR #944 are on `main`. The picks follow the recommendations in
`03-design.md`; a different answer at the gate changes the phase it names.

## Phase 1. The session runs without damage, and keeps damage runs

- **Result:** open, edit, evaluate and save run no damage; a kept damage run is used while its
  Option's inputs are unchanged.
- **Files:** `crates/ovd-format/src/session.rs` (evaluate and save call #923's no-damage run),
  `crates/ovd-shell/src/state.rs` (the kept runs, fingerprint, rename and delete),
  `crates/ovd-shell/src/views.rs` (damage views from the kept run).
- **Checks:** Rust tests in `state.rs`: a coastal gas fixture evaluates with `damage_status`
  `not run`; after a kept run it is `run`; an edit makes it `not run`; a rename keeps it; save
  writes the same hash with and without a kept run.

## Phase 2. Desktop command, progress, cancel

- **Result:** `run_damage(option)` and `cancel_damage()`; events `damage-progress` and
  `damage-done`.
- **Files:** `app/src-tauri/src/lib.rs`, `crates/ovd-shell/src/state.rs` (a method that runs damage
  on a copied Option with a callback and a stop flag), `app/src/lib/platform/types.ts`,
  `app/src/lib/platform/tauri.ts`.
- **Does:** the lock is held only to copy the Option in and to keep the results; an edit cancels
  (design 3a); progress events at most every 100 ms.
- **Checks:** a Rust test in `ovd-shell` that the callback sees `done` reach `total` and that a stop
  leaves the Option `not run`; `cargo clippy` on the app crate.

## Phase 3. Web Worker

- **Result:** the web build runs damage in a worker with the same progress and cancel.
- **Files:** `crates/ovd-web/src/lib.rs` (`run_damage(option, progress_fn)`,
  `keep_damage(option, results)`), `app/src/lib/platform/damage.worker.ts` (new),
  `app/src/lib/platform/web.ts`, `app/vite.web.config.ts`.
- **Checks:** `app/e2e/web-smoke.test.mjs`: run, see progress, cancel, see `not run`.

## Phase 4. Damage view, MessageBar action, estimate

- **Result:** the user sees the status, the case count, the estimate, the bar and Cancel.
- **Files:** `app/src/lib/views/Damage.svelte`, `app/src/lib/MessageBar.svelte` (optional
  `action`), `app/src/App.svelte` (the bar message, the run state), `app/src/lib/damage.ts` (the
  estimate: elapsed / done, remaining), `locales/en/app.ftl` (new Message Ids; other locales
  follow the project's locale rule), settings storage for the time per case.
- **Checks:** `app/scripts/damage.test.mjs` unit tests for the estimate (no total, no time, first
  event, last event); `npm run check` (svelte-check).

## Phase 5. e2e

- **Result:** the ticket's criteria run in CI.
- **Files:** `app/e2e/smoke.test.mjs`.
- **Checks:**
  - open the coastal gas Design: `damage-status` reads `not run`; click **Run**; `damage-progress`
    moves; the governing case and drawing appear;
  - cancel mid-run: status stays `not run`;
  - the estimate after the first event and the actual time are logged; assert within a factor of 2;
  - the old "damage on open" test becomes "damage after Run".
  The pre-report e2e moves with item 3 (design 7a).

## Phase 6. Docs and fragment

- **Files:** `changes/ticket-924-damage-on-demand.md`; the app section of the docs that describes
  the Damage view, if one exists at build time.
- **Checks:** `python3 tools/changelog_collect.py --check`; `bash tools/gates.sh`.

## Out of this ticket

The pre-report dialog (design 7a; its words are fixed in `03-design.md`); a handle that keeps the
first stage across calls; running damage for every Option at once.
