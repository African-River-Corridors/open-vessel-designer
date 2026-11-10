---
ticket: 945
phase: build brief
date: 2026-11-10
status: for the build seat
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 8b967e84
---

# Build brief — #945: keep the Settings button still without hiding the file name

Issue #945 (lost Settings click). Branch `ticket/945-header` (from `main` at `ab07d313`), two commits by the
design lane: the done-tests (`3765eb27`) and the design note (`8b967e84`, `03-design.md` beside this file).
Build on that branch, in your own worktree. PR #1061 (fixed widths) was rejected: do not reuse its CSS.

## The rule (design note, section "The layout rule")

The header's three late-drawing texts (file label, Defaults Profile, engine version) move **left of the panel
buttons, inside the spacer**, and the spacer has no size of its own.

In `app/src/App.svelte`:

1. Replace `<span class="grow"></span>` by `<span class="status">` holding the existing `{#if snap}` block with
   the three `.meta` spans, in the same order: file, profile, engine. Delete the three spans from after the
   Settings button. Keep `class="meta"` on the three spans only (the native test's `fileLabel` reads the
   first `header .meta`; `app/src/lib/splash.ts:19` treats `header .meta` as the ready signal). Keep
   `data-testid="engine-version"` on the third.
2. `.status { flex: 1 1 0; min-width: 0; display: flex; align-items: center; justify-content: flex-end;
   gap: var(--sp-2); overflow: hidden; }`. The basis `0` is the whole fix: a wrapping flex row breaks lines on
   each item's hypothetical size, and this one's is zero, so the texts can never push a button or add a row.
3. `.meta { min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }` (drop
   `margin-left`, the gap does it); `.meta.file { flex: 1 1 auto; font-family: var(--font-mono); }`; the other
   two `flex: 0 1 auto`. Give each span `title={its text}` so a cut text reads in full on hover.
4. Delete `.grow`. Leave `header { flex-wrap: wrap }` as it is. No fixed widths anywhere.

Expected: the Settings button's `x` is identical with empty, real, 25-character and long texts in en, es, pt,
zh-Hans and ak (the design note's prototype table). The header is one row wherever the buttons alone fit one
row. At 1280 px the file label shows 60 characters in en, zh-Hans and ak, 50 in pt, 43 in es.

## Files

- Owned: `app/src/App.svelte` (the header markup and its `<style>` block only).
- Do not edit: `app/e2e/web-smoke.test.mjs`, `.done-test` (the done-tests), `app/e2e/smoke.test.mjs`,
  `locales/*`. If a done-test looks wrong, say so in the PR; do not change it.

## Tests to turn green, unedited

Web (`app/e2e/web-smoke.test.mjs`, both headed `// DONE-TEST #945 — do not edit in the build`):

1. "keeps the Settings button where it is while the header text draws in, in each language (issue #945)".
   Fails on main (the button moves 192 px in English).
2. "shows a 25-character file name in full in the header (issue #945)". Passes on main, fails on PR #1061.

Run them locally. Build the CLI (`cargo build -p ovd-cli`) and the web package (`bash tools/build_web.sh`)
through the lane's build slot, with `PATH` and `RUSTC` pointed at the pinned 1.99.0 toolchain and the cargo
bin folder on `PATH` for `wasm-bindgen`. Then in `app/`: `npm ci`, the test-mode web bundle
(`npx vite build --config vite.web.config.ts --mode test --outDir dist-web-test`), and the two tests with
`OVD_CLI` set to the ovd binary in the slot's debug folder:
`node --test --test-name-pattern "issue #945" e2e/web-smoke.test.mjs`. Then the whole web suite
(`bash e2e/run-web.sh`).

Native (`app/e2e/smoke.test.mjs`): the coordinator dispatches `app-gate.yml` on the PR. Its `app-e2e` job
must pass whole: it opens files by waiting for the name in the first `header .meta` (`smoke.test.mjs:32`,
`:79`), which is what PR #1061 broke (49 failures). Acceptance criterion 2 of the issue is the **native**
language test ("switches the language in Settings, keeps the choice on reload, and back to English") passing
in 20 consecutive dispatches, or a local loop of 50. The web test does not count toward it.

## Also

- The `inSettings` diagnostics in `smoke.test.mjs` stay (acceptance criterion 4). No click retry (criterion 3).
- Screenshots: set `OVD_WEB_SHOTS` when running test 1 and attach `945-header-<lang>.png` for the five
  languages, with a real file open if you can, so the review sees the new text position.
- PR body: the one-line research line (searched / exists / reusing / building because), the design note's
  rule in two sentences, the test table, "Design impact: none" (no engine change; confirm
  `check_fixture_hashes` is unchanged).
- Commit on the branch; the coordinator opens the PR and dispatches the gate.
- Never put any person's name in a commit or file: write "the maintainer".
