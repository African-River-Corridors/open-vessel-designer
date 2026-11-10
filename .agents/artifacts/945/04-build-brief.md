---
ticket: 945
phase: build brief (second build)
date: 2026-11-10
status: for the build seat; replaces the first brief (79bb4cad), whose build was PR #1063, rejected
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: 79bb4cad
---

# Build brief 2 — #945: the header texts must read under WebKit's WebDriver

Issue #945 (lost Settings click). This replaces the first build brief (this file at `79bb4cad`). PR #1063
(`agent/lane-f945b`, head `dd9c6ff6`) built the first rule and QA rejected it as a rethink: the native
`app-e2e` job fails 48 of 54 tests because WebKit's WebDriver returns no text for the file label. The layout
rule itself held (the Settings button never moved).

Build on `ticket/945-header` (the design lane's commits: done-tests, design note, this brief, and the new
done-test commit) in your own worktree. Do not merge `agent/lane-f945b` into it: take its markup by hand (it
is small) and change the CSS as below. PR #1063 is superseded; the coordinator closes it.

## The cause (proven, design note section "Second rethink")

WebKit's WebDriver implements Get Element Text with its own atom, `utils.getText` in
`Source/WebKit/UIProcess/Automation/atoms/utils.js` (identical on `main` and the `webkitgtk-2.52.6` tag; the
gate installs 2.52.6). Before it collects an element's text it calls `utils.isShown(element)`, and that has
a shortcut: if the element's own computed `overflow` is `hidden`, every child that is **a text node** counts
as hidden by overflow, so an element with only text inside and `overflow: hidden` on itself is "not shown"
and its text is the empty string. Drawn whole or not. `.meta { overflow: hidden }` in PR #1063 (and the
same rule in PR #1061) hits it; `main` has no overflow on `.meta` and passes.

Verified in a real WebKit (Playwright's build, WebKit 27.2) on PR #1063's web build: the atom returns ""
for all three header texts in all five languages; with `overflow: clip` it returns the text. `overflow:
clip` clips the same and draws the `text-overflow: ellipsis`, in WebKit and in Chrome.

## The amended rule

Markup as in PR #1063: `<span class="status">` replaces `<span class="grow">` and holds the `{#if snap}` block
with the three `.meta` spans in the order file, profile, engine, each with `title={its text}`; the three
spans after the Settings button are deleted; `data-testid="engine-version"` stays on the third;
`class="meta"` is on the three spans only.

CSS, in `app/src/App.svelte`'s style block, replacing `.grow`, `.meta` and `.meta.file`:

```css
/* Basis 0: a wrapping row breaks lines on each item's hypothetical size, so the texts never push a button
   or add a row. The span is one text line high and clips the rest: a text that does not fit on the line
   wraps below it and is gone whole, never cut to a stub. The file name comes first, so it goes last. */
.status {
  flex: 1 1 0; min-width: 0; display: flex; flex-wrap: wrap; align-items: center; align-content: flex-start;
  justify-content: flex-start; gap: var(--sp-2); height: 1.5em; line-height: 1.5em; overflow: clip;
}
/* No `overflow: hidden` on a text-only element: WebKit's WebDriver reads its text as empty (see the design note). */
.meta { color: var(--ink-3); font-size: var(--fs-12); flex: 0 0 auto; white-space: nowrap; }
.meta.file { flex: 0 1 auto; min-width: 0; overflow: clip; text-overflow: ellipsis; font-family: var(--font-mono); }
```

What each part does:

- `flex: 1 1 0; min-width: 0` on `.status`: unchanged from the first rule; the button never moves.
- `flex-wrap: wrap; height: 1.5em; line-height: 1.5em; align-content: flex-start; overflow: clip`: one line
  of texts; a text that does not fit wraps to a second line that lies below the clip. The gap also applies
  between rows, so the second row starts at 29 px, under the 21 px clip. `align-content: flex-start` keeps
  the first row at its own height (the default `stretch` would split the height between the rows).
- `.meta { flex: 0 0 auto }`: the profile and engine texts never shrink. They are whole, or they are gone.
- `.meta.file { flex: 0 1 auto; min-width: 0; overflow: clip; text-overflow: ellipsis }`: the file name is
  the only text that shrinks; when it does, it ends in an ellipsis and the `title` has the whole name.
  `clip`, not `hidden`, is the whole fix for the native tests.
- `justify-content: flex-start`: the file name sits beside the file buttons, as a document title. QA found
  the first build already read that way; the first design note's "right-aligned" wording was wrong.
- Nothing uses `overflow: hidden` on an element whose only children are text. `.status` holds elements, so
  its clip could be `hidden` too; `clip` is used because nothing here needs a scroll container.

Two translations, both `# status: draft`, so the 25-character name fits on the web at 1280 px in Spanish
and Portuguese (the download link is the widest header item there, and a zero-basis spacer can never
squeeze it):

- `locales/es/app.ftl`: `app-download-desktop = Descargar para escritorio` (was `Descargar la aplicación de
  escritorio`).
- `locales/pt/app.ftl`: `app-download-desktop = Baixar para desktop` (was `Baixar o aplicativo para desktop`).

Keep the `# status: draft` line above each. Any wording of about 25 characters or fewer does the job; the
maintainer may prefer other words (logged as a judgement call). Run `python3 tools/i18n_check.py` (the Text
Library check, spec 0005; exit 1 on any problem) before the commit.

Expected, measured in headless Chrome and WebKit 27.2 on PR #1063's build with this CSS injected
(`03-design.md`, "Second rethink"): the Settings button `x` is unchanged from the first build in every
language and width (en 1201.09, es 1206.25, pt 1167.59, zh-Hans 1218.2, ak 1201.09 at 1280 px); the
25-character name is whole at 1280 px in all five languages, web and native; the profile and engine texts
are whole in en, zh-Hans and ak on the web and in all five natively, and absent in es and pt on the web;
a 68-character name is cut with an ellipsis and the two other texts are absent.

## Files

- Owned: `app/src/App.svelte` (the header markup and its `<style>` block only), `locales/es/app.ftl` and
  `locales/pt/app.ftl` (the one message each, above).
- Do not edit: `app/e2e/web-smoke.test.mjs`, `.done-test` (the done-tests), `app/e2e/smoke.test.mjs`
  (QA's rule: fix the CSS, not the test), other locales. If a done-test looks wrong, say so in the PR body;
  do not change it.

## Tests to turn green, unedited

`app/e2e/web-smoke.test.mjs`, three tests headed `// DONE-TEST #945 — do not edit in the build`:

1. "keeps the Settings button where it is while the header text draws in, in each language (issue #945)".
   Unchanged. Fails on main; passed on PR #1063.
2. "shows a 25-character file name in full at 1280 px in each language, readable by WebKit's WebDriver, and
   never a cut profile or engine text (issue #945)". New version. Fails on PR #1063's build at its first
   assertion in English: `WebKit's WebDriver would read no text from this header text`. Also fails on it in
   es and pt (the name is cut) and on PR #1061 (the name is cut).
3. "cuts a long file name with an ellipsis and keeps the profile and engine texts whole or drops them
   (issue #945)". New. Fails on PR #1063's build: `the file label does not take the room there is (277 of
   369 px)`, because the three texts shrink together.

All three pass on PR #1063's build with the CSS above and the two translations patched in (the design
lane's proof, run in headless Chrome on 2026-11-10).

Run them as the first brief said: build the CLI (`cargo build -p ovd-cli`) and the web package
(`bash tools/build_web.sh`) through the lane's build slot (`~/Dev/_cargo-target/ovd-x-run <lane> …`), then
in `app/`: `npm ci`, `npx vite build --config vite.web.config.ts --mode test --outDir dist-web-test`, and
`OVD_CLI=<the ovd binary> node --test --test-name-pattern "issue #945" e2e/web-smoke.test.mjs`. Then the
whole web suite (`bash e2e/run-web.sh`) and `npm run check`.

## The native gate, and how QA proves the cause is gone

The native smoke tests cannot run on a Mac. The coordinator dispatches `app-gate.yml` on the PR branch.
The `app-e2e` job must pass whole (54 of 54). Three tests in it are the direct proof:

- "opens an example file and shows its outputs": waits on `fileLabel()` (`smoke.test.mjs:32`, the first
  `header .meta`, read through Get Element Text) and then reads `[data-testid=engine-version]` the same way.
  On PR #1063 it timed out at `smoke.test.mjs:79` in 6 of 6 runs. It passing is the cause gone.
- Every other test that opens a file (47 of them) waits on the same read.
- "switches the language in Settings, keeps the choice on reload, and back to English": acceptance
  criterion 2 of #945 is 20 consecutive passes of it (it passed 6 of 6 on PR #1063; the count restarts on
  the new PR).

If the job fails again at `smoke.test.mjs:79`, the first thing to check in the PR's built CSS is
`getComputedStyle(document.querySelector("header .meta")).overflow`: it must not be `hidden`. Done-test 2
asserts exactly that in Chrome, so a green web job with a red native job would mean a second cause, not
this one.

## Also

- The `inSettings` diagnostics in `smoke.test.mjs` stay (acceptance criterion 4). No click retry
  (criterion 3).
- Screenshots: set `OVD_WEB_SHOTS` when running test 1 and attach `945-header-<lang>.png` for the five
  languages, with a real file open if you can.
- PR body: the one-line research line, the cause in two sentences (the WebKit atom's `overflow: hidden`
  shortcut), the rule in two sentences, the test table with the three done-tests and the whole web suite,
  "Design impact: none" (no engine change; confirm `check_fixture_hashes` is unchanged). Say that PR #1063
  is superseded.
- Commit on the branch; the coordinator opens the PR and dispatches the gate.
- Never put any person's name in a commit or file: write "the maintainer".
