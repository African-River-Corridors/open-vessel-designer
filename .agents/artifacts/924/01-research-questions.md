---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
ticket: 924
phase: research-questions
---

# #924 research questions

Ticket: Damage on demand in the app: run it with progress and a time estimate; ask before a
report. Read at `a63ca819`.

## A. How the app runs an Option today

1. How does a desktop evaluation reach the engine, and does it hold the state lock?
   `app/src-tauri/src/lib.rs:15-28` (`Shared`, `lock`), `:137-140` (`evaluate`),
   `crates/ovd-shell/src/state.rs:200-212`, `crates/ovd-format/src/session.rs:1061-1095`.
2. Does save run the Options again? `session.rs:1100-1118`.
3. Is there a precedent for long work off the main thread with events to the front end?
   `app/src-tauri/src/lib.rs:218-246` (`open_from_outside`), `app/src/lib/platform/tauri.ts:33-34`.
4. When does the front end evaluate, and what keeps the last good results?
   `app/src/App.svelte:40-110`.
5. How does the web build evaluate? `app/src/lib/platform/web.ts:147`, `crates/ovd-web/src/lib.rs:1-60`.

## B. The Damage view and the status line

6. What does the Damage view show, and from which result keys? `app/src/lib/views/Damage.svelte:1-60`,
   `app/src/lib/damage.ts:1-40`, `app/src/lib/MainPanel.svelte:97-102`, `:213-221`.
7. Is there a status bar? `app/src/App.svelte:228-276`, `app/src/lib/MessageBar.svelte:1-60`,
   `app/src/lib/BottomPanel.svelte:240-246`.
8. Which open PR changes the Damage view? PR #944 (#928).

## C. Damage in the engine

9. Where does damage run inside one Option run, and after which sizing step?
   `crates/ovd-format/src/simulate.rs:3250-3410` (gas), `:1659-1720` (liquid).
10. Are the damage blocks inside the results hash? `crates/ovd-format/src/run.rs:362-383`
    (`UNHASHED_KEYS`).
11. What does #923 promise: switch, `damage_status`, callback, stop? Issue #923 items 1 to 5 and
    its comment.
12. What does #921 change for threads and WebAssembly? Local branch `ticket/921-damage-parallel`,
    `crates/ovd-format/src/damage.rs` (`damage_workers`, `run_ordered`), `Cargo.toml` feature
    `parallel`.
13. Can the engine time a case in the browser? `std::time::Instant` on `wasm32-unknown-unknown`;
    `git grep Instant -- crates/ovd-format`.

## D. Reports

14. Does the app make a report today? `git grep -i report -- app/src app/src-tauri`;
    `docs/specs/0015-concept-report-format.md:68`; issue #833.

## E. Tests

15. Which e2e test opens a coastal gas file with damage, and what are its time limits?
    `app/e2e/smoke.test.mjs:400-440`, `:410-411`.
