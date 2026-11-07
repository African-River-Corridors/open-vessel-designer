---
repos: [{identifier: African-River-Corridors/open-vessel-designer, sha: a63ca819db7bb8a6a163350eb256d3767e3c7334}]
---

# #945 research: Settings click lost (cause not proven)

OVD is for concept and preliminary design only, never for approval (ADR 0011).

## Searched / exists / reusing / building because

- Searched: `SettingsDialog.svelte`, `App.svelte` (open flag, header, init), `splash.ts`, `LanguageControl.svelte`, `focus.ts`.
- Exists: the #936 dump on a timeout in `inSettings`.
- Reusing: that dump. Building: nothing in the app, because the cause is not proven.

## Ruled out by reading the code

- No outro or transition: `{#if settingsOpen}` removes the dialog at once. No state toggle: the button only sets `settingsOpen = true`.
- The splash has `pointer-events: none`, and it was gone in the dump.
- The first-run notice was not open in the dump.
- A stale `.veil`: not present in the failing dump (no dialog at all).

## Evidence from a failing run

Run 42099358283 (app-gate, job app-e2e, branch at main d938baec). The failing step is the **second** `inSettings` of "switches the language in Settings...": the page is in Spanish, after `browser.refresh()` (the report names the first one; the dump shows otherwise). The dump and `settings-dialog-missing.png` show: no dialog, header in Spanish with all three `.meta` spans (the first result is in), no `.veil`. Nothing else looks odd; the header's last item `Motor 0.3.0` wraps to a second line.

## Leading hypothesis (not proven)

After a reload the test waits only for the header text `Abrir`. The header buttons draw at mount, but the `.meta` spans draw later, inside `{#if snap}`, once `onMount` has fetched the first snapshot. The header is flex with a `.grow` spacer, so the right-hand group, Settings included, moves left when the spans arrive, and the header can wrap and change height. A click that WebDriver aims at the old position then lands beside the button. The splash code treats `header .meta` as the "app is ready" signal, which supports this reading.

Test-only fix would be: wait for `header .meta` after the reload. App fix (users can hit it too): keep the Settings button where it is while the file label loads, for example by drawing the three spans always (empty until `snap`), or by putting Settings after them.

## What stopped a proof

I added a log to `inSettings` (branch `ticket/945-settings-click`, commit "e2e: log where the Settings button was at the click") that records the button rect and the `header .meta` count at the click, and the same plus `elementFromPoint` at the old centre on a timeout. 12 app-gate dispatches (42105379145 to 42105420385, 42106784081 to 42106838603) all passed, so the race did not fire. One failure in 15 dispatches in all (the 3 before the log: 42099330487, 42099343589 pass; 42099358283 fail).

## Next step

Merge the log commit (it keeps the "diagnostics stay in inSettings" rule), then dispatch until it fires. If `before.meta` is 0 and the rect moved, the cause is proven: fix it in the app and add a test that checks the button rect does not change when the snapshot arrives.
