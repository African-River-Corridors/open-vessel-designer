---
ticket: 945
phase: design
date: 2026-11-10
status: rule amended after QA's rethink of PR #1063 (the native app must read the file name); three done-tests committed; for the build seat, second build
repos:
  - identifier: African-River-Corridors/open-vessel-designer
    sha: ab07d313fcbeab7ae92468b8b5a0038ef46448f8
---

# Design: keep the Settings button still without hiding the file name

OVD is for concept and preliminary design only, never for approval (ADR 0011).

## Summary of the request

Issue #945 (lost Settings click). The cause is proven: the header's three texts (the file label, the Defaults
Profile and the engine version) draw in after first paint, inside `{#if snap}` in `app/src/App.svelte`, to
the right of the Settings button. When they arrive the right-hand button group moves left, and a click aimed
before they drew lands on empty header.

PR #1061 gave the three texts fixed widths. The button stopped moving, but the file label cut the file name at
about 17 characters: 49 native e2e tests time out waiting for the name, and the user has to hover to read it.
QA rejected it as a rethink.

PR #1063 built the first version of the rule below (texts in a zero-basis spacer, each cut with an ellipsis).
The button never moved, in any language, width or text length, but the native `app-e2e` job failed 48 of 54
tests in 6 of 6 runs, every one at the wait for the file name, while the screenshots showed the name drawn
whole. QA rejected it as a rethink. The cause and the amended rule are in the section "Second rethink".

## Current state

```
[mark] OVD [New] [Open] [Save] [Save as] [About] (web: badge, link) <.grow> [Input panel] [Outputs panel] [Settings] {#if snap} file · Defaults · Engine {/if}
```

`header` is `display: flex; flex-wrap: wrap`. The `.grow` spacer takes the free space, so the button group is
pinned to the right edge by whatever follows it. The texts follow it: when they draw, Settings moves left by
their width. When they are long, the row wraps and the header grows a line.

## Desired end state

- The Settings button's position depends only on the window width and on the button group's own labels. It
  is the same at first paint, after the texts draw, with empty texts and with long texts, in every language.
- The file name shows whole whenever the row has the room. At the default window (1280 px) it shows at least
  60 characters in every language. Only when the row is short of room is it cut, with an ellipsis and the full
  name in a `title` tooltip.
- The header stays one row wherever the buttons alone fit one row. The texts never add a row.
- The native test's `fileLabel` (the first `header .meta`), `[data-testid=engine-version]` and the splash's
  ready signal (`header .meta`, `app/src/lib/splash.ts:19`) keep working unchanged.

## The layout rule

**The texts live inside the spacer, to the left of the buttons. The spacer has no size of its own.**

```
[mark] OVD [New] [Open] [Save] [Save as] [About] (web: badge, link) <.status: {#if snap} file · Defaults · Engine {/if}> [Input panel] [Outputs panel] [Settings]
```

1. Replace `<span class="grow"></span>` by a `<span class="status">` that holds the `{#if snap}` block with
   the three `.meta` spans, in the same order (file, profile, engine). The `.meta` class stays on the three
   spans only, so `smoke.test.mjs` `fileLabel` and the splash signal read the same elements.
2. `.status { flex: 1 1 0; min-width: 0; display: flex; flex-wrap: wrap; align-items: center;
   align-content: flex-start; justify-content: flex-start; gap: var(--sp-2); height: 1.5em; line-height: 1.5em;
   overflow: clip; }`. Flex basis `0` is the point: the row-breaking of a wrapping flex container uses each
   item's *hypothetical* size, and this item's is zero, so it never pushes a button and never starts a new
   row, whatever text it holds. It then takes exactly the free space `.grow` took. The span is one text line
   high and clips what lies below: a text that does not fit on the line wraps under the clip and is gone
   whole. (Amended in the second rethink; the first version had no wrap, no height and `overflow: hidden`.)
3. `.meta { flex: 0 0 auto; white-space: nowrap; }` and `.meta.file { flex: 0 1 auto; min-width: 0;
   overflow: clip; text-overflow: ellipsis; font-family: var(--font-mono); }`. The profile and engine texts
   never shrink: whole or gone. The file name is the only text that shrinks, and it ends in an ellipsis. Each
   gets `title={text}` so a cut text reads in full on hover. No `overflow: hidden` on a text-only element,
   ever: see "Second rethink". (Amended; the first version let all three shrink with `overflow: hidden`.)
4. Nothing else moves. `.grow` is deleted. `header` keeps `flex-wrap: wrap`: at widths where the buttons
   alone do not fit, the row wraps exactly as it does today, before and after the texts draw alike.
5. Two draft translations get shorter, so the 25-character name is whole on the web at 1280 px in Spanish
   and Portuguese: `app-download-desktop` becomes `Descargar para escritorio` (es) and `Baixar para desktop`
   (pt). See "Second rethink", "The web at 1280 px in es and pt".

### Why this and not the alternatives

- **A. Fixed widths per text (PR #1061).** Stops the move, but the widths are tuned per language and cut the
  file name. Any new language or longer profile name needs retuning. Rejected by QA.
- **B. Reserve `min-width` for the two short texts and let the file label take the `.grow` space (QA's
  second suggestion).** Works, but the reserved widths are still per-language constants, and the texts stay
  to the right of Settings, so an unexpected width (a long profile name) moves the button again.
- **C. Texts left of the buttons inside a zero-basis spacer (chosen; QA's first suggestion, made exact).**
  No constants. The button group's position is independent of the texts by construction, in every language,
  including ones with no translations yet (ak falls back to English). The file label gets all the free space.
- D. `flex-wrap: nowrap` on the header. Would also stop a wrap from moving the button, but at narrow widths
  the buttons would clip instead of wrapping. Not needed: with C the texts cannot cause a wrap, and the
  buttons wrap the same before and after the texts draw.

### The visual change

The texts move from the far right (after Settings) to the space between the file buttons and the panel
buttons, left-aligned, so the file name sits beside Open and Save as a document title. The order is
unchanged: file, Defaults, Engine. Where the row has the room, all three show whole. Where it has not, the
engine version is gone first, then the Defaults Profile, each whole or not at all; last, the file name is
cut with an ellipsis and the tooltip has it whole. No text is ever a stub ("Mo…"). (This paragraph was
corrected in the second rethink: the first version said "right-aligned" and "the file name is the first
thing cut", and neither matched its CSS, as QA found.)

## Measurements (headless Chrome, the web build in test mode, body font per `app/src/app.css`)

Today's header, per language. "Buttons" is the mark, the brand and the eight buttons; the native app has no web
badge or link. Gaps are 8 px, padding 12 px each side, so the buttons alone need the sum plus about 112 px.

| Language | Buttons (px) | Buttons + gaps (px) | Fits one row at 720 px (native)? | Free for texts at 1280 px (native) |
|---|---|---|---|---|
| en | 565 | 677 | yes (43 px over) | 603 |
| es | 710 | 822 | no, wraps today too | 458 |
| pt | 682 | 794 | no, wraps today too | 486 |
| zh-Hans | 536 | 648 | yes (72 px over) | 632 |
| ak | 569 | 681 | yes (39 px over) | 599 |

The file label is 12 px mono: 7.2 px a character. "Untitled" is 58 px; a 60-character name is about 433 px.

The rule, applied to the live DOM of today's build (a scratch prototype, not the fix), with the Settings
button's `x` measured with the three texts empty, with the real texts, with a 60-character file name and
with a 68-character text in all three:

| Language | Build | Width | Settings `x` today: empty / real / long | Settings `x` under the rule: empty / real / 60 / long | One row under the rule | File characters shown |
|---|---|---|---|---|---|---|
| en | native | 1280 | 1153 / 961 / 695 (wraps) | 1201 / 1201 / 1201 / 1201 | yes | 60 |
| en | native | 720 | 609 / 641 (wraps) | 641 all four | yes | 2 |
| es | native | 1280 | 1158 / 920 / 1206 (wraps) | 1206 all four | yes | 43 |
| es | native | 720 | wraps already | 148 all four | no (buttons alone wrap) | 8 |
| pt | native | 1280 | 1120 / 933 / 1168 (wraps) | 1168 all four | yes | 50 |
| pt | native | 720 | wraps already | 12 all four | no (buttons alone wrap) | 1 |
| zh-Hans | native | 1280 | 1169 / 980 / 712 (wraps) | 1217 all four | yes | 59 |
| zh-Hans | native | 720 | 609 / 584 (wraps) | 657 all four | yes | 6 |
| ak | native | 1280 | 1153 / 961 / 695 (wraps) | 1201 all four | yes | 60 |
| ak | native | 720 | 609 / 641 (wraps) | 641 all four | yes | 2 |
| en | web | 1280 | 1153 / 961 / 1201 (wraps) | 1201 all four | yes | 37 |
| pt | web | 1280 | 1120 / 1001 (wraps) | 1168 all four | yes | 22 |

Reading: today the real texts move the button in every language at 1280 px (by 192 px in English), and a long
text wraps the header. Under the rule the button never moves and the header never gains a row from the texts,
in all 20 cases measured (five languages, two builds, two widths). The whole 60-character name shows at
1280 px in en, zh-Hans and ak; Portuguese shows 50 and Spanish 43, because their buttons are wider.

**The minimum width (720 px).** The brief asked for the whole name on one row at 720 px. No header rule
gives that: in English the buttons alone take 677 of 720 px, and in Spanish and Portuguese they wrap today
before any text draws. The rule keeps what can be kept there: the button is still, the texts add no row, and
the file label shows an ellipsis with the full name in its tooltip. One row with the whole name at 720 px
would need smaller or icon buttons, which is another ticket. Logged as a judgement call.

## Second rethink (2026-11-10): the native app must read the file name

### What QA found on PR #1063

`app-e2e` (Linux, WebKitGTK 2.52.6 with `webkit2gtk-driver`, through `tauri-driver`) failed 48 of 54 tests in
6 of 6 runs, every one at `smoke.test.mjs:79`: `timed out waiting for: <file>.vdesign to open`. That wait
reads `fileLabel()`, the first `header .meta`, through WebDriver's Get Element Text. The job's screenshot
showed the name drawn whole at 1280 px, with Settings at `x` 1201 as the test's own DIAG line said. The web
job passed 71 of 71, and the native language test passed 6 of 6. PR #1061 (fixed widths, with the same
`overflow: hidden; text-overflow: ellipsis` on `.meta`) had failed the same way with 49; `main`, which has
no overflow on `.meta`, passes.

QA's minors: the profile and engine texts shrink to stubs on a short row; es and pt cut the 25-character
name at 1280 px on the web; the "visual change" paragraph did not match its CSS.

### The cause

WebKit's WebDriver does not use `innerText` or Selenium's atom for Get Element Text. `Session::getElementText`
(`Source/WebDriver/Session.cpp`) evaluates WebKit's own atom, `ElementText.js`, which is one line:
`return utils.getText(element)`. Both live in `Source/WebKit/UIProcess/Automation/atoms/`; `utils.js` is
byte-identical on `main` and on the `webkitgtk-2.52.6` tag (the version the gate installs).

`utils.getText` collects an element's text only while `utils.isShown(element)` is true, and `isShown` ends
with `isElementSubtreeHiddenByOverflow(element)`:

- `elementOverflowsContainer(element)` is true whenever the element's **own** computed `overflow` is
  `hidden`. Its comment says: "FIXME: this needs to take into account the scroll position of the element,
  the display modes of it and its ancestors, and the container it overflows."
- Then every child node is tested, and the base case reads: "Returns true if the child node is overflowed or
  otherwise hidden. Base case: not an element ...". A **text node is not an element, so it counts as
  hidden.**

So an element with `overflow: hidden` on itself and only text inside is "not shown", and its text is the
empty string, whether it is drawn whole or not. Every `.meta` in PR #1063 and PR #1061 is exactly that.
The screenshot and the DIAG line agree with the test: the name was there, and the driver read nothing.
Ancestors are not checked, so `.status { overflow: hidden }` on its own did no harm; the `.meta` rule did.

What the atom accepts, from its code: a computed `overflow` of anything but `hidden`, or an element child
with a non-zero box. `overflow: clip` computes to `clip`; `overflow-x: hidden` alone computes to
`hidden auto` (the other axis turns `auto`, which makes a scroll container). `clip` clips without scrolling
and draws `text-overflow: ellipsis` in WebKit and in Chrome.

### Reproduction

WebKitGTK is not on this Mac and Safari's "Allow remote automation" is off, so the atom source
(`utils.js` at the 2.52.6 tag) was run as a page script in a real WebKit, Playwright's build (WebKit 27.2),
and in headless Chrome beside it (the atom is plain JavaScript over computed styles; both engines agree in
every case). Two pages: a 60-line prototype of the header, and PR #1063's actual web build with CSS injected
over it. `getText` of the first `header .meta` at 1280 px:

| CSS on `.meta` | WebKit atom `isShown` | WebKit atom text | Chromedriver's own `getText` |
|---|---|---|---|
| none (main) | true | the name | the name |
| `overflow: hidden; text-overflow: ellipsis` (PR #1063, PR #1061) | **false** | **""** | the name |
| `overflow: clip; text-overflow: ellipsis` (amended rule) | true | the name | the name |
| `overflow-x: hidden` (computes `hidden auto`) | true | the name | the name |
| `overflow: hidden` with the text in an inner `<span>` | true | the name | the name |

On PR #1063's real web build the atom returned "" for all three texts in all five languages, web and native
emulated; with the amended rule injected it returned each text in every case. Chromedriver reads the name on
PR #1063 because Selenium's atom checks clipping properly, which is why the web job stayed green while the
native job failed.

### The amended rule, and what it does with little room

Items 2, 3 and 5 of "The layout rule" above. `.status` wraps, is one line high (`height: 1.5em;
line-height: 1.5em`, 21 px at the header's 14 px font) and clips with `overflow: clip`; its row gap puts the
second row at 29 px, under the clip. `align-content: flex-start` keeps the first row at its own height (the
default `stretch` would share the 21 px between rows). The profile and engine texts have `flex: 0 0 auto`,
so each is whole or wraps away whole. The file name has `flex: 0 1 auto; min-width: 0; overflow: clip;
text-overflow: ellipsis`: it is first in the DOM, so it is the last to go, and it is the only text that is
cut.

Why not QA's literal minor ("`flex-shrink: 0` on profile and engine, so only the file name gives way")? It
was measured: in Spanish on the web at 1280 px the texts' box is 172 px and the profile and engine alone are
129 + 52 px, so the file label is **0 px** wide, which contradicts QA's third minor (the 25-character name
whole in es and pt at 1280 px). The three minors cannot all hold with the texts shrinking; they hold with
whole-or-gone, where the name is kept last because it is the text the user, the splash and the native test
need. Logged as a judgement call.

Measured on PR #1063's web build with the amended rule injected, in headless Chrome and in WebKit 27.2, the
25-character name open. The two engines agree in every case both measured; the es and pt web boxes are
Chrome's (the WebKit run kept the English link text there by a script slip, and so measured 223 and 258 px).
"Box" is the room the texts have. "whole / whole / whole" lists the file, profile and engine texts:

| Language | Build | Width | Settings `x` (same as PR #1063) | Box (px) | File / profile / engine |
|---|---|---|---|---|---|
| en | web | 1280 | 1201.09 | 369 | whole / whole / whole |
| en | native | 1280 | 1201.09 | 604 | whole / whole / whole |
| es | web | 1280 | 1206.25 | 172 (228 with the shorter link) | cut by 8 px / gone / gone (whole / gone / gone with the shorter link) |
| es | native | 1280 | 1206.25 | 459 | whole / whole / whole |
| pt | web | 1280 | 1167.59 | 225 (286 with the shorter link) | whole / gone / gone (whole / whole / gone with the shorter link) |
| pt | native | 1280 | 1167.59 | 484 | whole / whole / whole |
| zh-Hans | web | 1280 | 1218.2 | 473 | whole / whole / whole |
| zh-Hans | native | 1280 | 1218.2 | 642 | whole / whole / whole |
| ak | web | 1280 | 1201.09 | 396 | whole / whole / whole |
| ak | native | 1280 | 1201.09 | 599 | whole / whole / whole |
| en | native | 720 | 641.09 | 44 | cut (ellipsis) / gone / gone |
| es | native | 720 | 147.81 (buttons wrap) | 104 | cut / gone / gone |
| pt | native | 720 | 12 (buttons wrap) | 33 | cut / gone / gone |
| zh-Hans | native | 720 | 658.2 | 82 | cut / gone / gone |
| ak | native | 720 | 641.09 | 39 | cut / gone / gone |

With the 68-character name in all three texts at 1280 px: one header row in every language, Settings `x`
unchanged, the file label cut with an ellipsis across the whole box, the two other texts gone. On PR #1063
the same case gave three stubs ("a_very_long_… / a_very… / a_v…").

### The web at 1280 px in es and pt

The web header has two items the native app lacks, the "Web version" badge and the download link. In Spanish
the link reads "Descargar la aplicación de escritorio" and is the widest item in the header; it leaves the
texts 172 px, 8 px short of the 25-character name. A zero-basis spacer can never squeeze the link (it has
no size to give up), so the link's width is the only lever. Shortening the two draft translations to
"Descargar para escritorio" (es) and "Baixar para desktop" (pt) gives 228 and 286 px, and the name shows
whole. Any wording of about 25 characters or fewer does it; the words are a translation decision and the
maintainer may prefer others. Logged as a judgement call.

### Judgement calls in this rethink

1. Whole-or-gone instead of "the file name gives way first" (above).
2. The es and pt download-link wording (above).
3. Reproducing in Playwright's WebKit build, installed into `~/Library/Caches/ms-playwright` from a scratch
   folder, since the alternative was to reason without a reproduction. It stays for QA to reuse.

## Done-tests (committed first, on `ticket/945-header`, headed `// DONE-TEST #945 — do not edit in the build`)

`app/e2e/web-smoke.test.mjs`, listed in `.done-test`:

1. **"keeps the Settings button where it is while the header text draws in, in each language (issue #945)"**
   (kept from PR #1061, and made stricter). For en, es, pt, zh-Hans and ak: after a reload the header is one
   row; the Settings button's `x`, `y` and the one-row state are the same after the texts draw, with the three
   texts empty, with a 25-character file name, and with a 68-character name in all three. Unchanged in the
   second rethink.
2. **"shows a 25-character file name in full at 1280 px in each language, readable by WebKit's WebDriver, and
   never a cut profile or engine text (issue #945)"** (the second rethink's version of the 25-character
   test). Opens `dry_bulk_box_hold.vdesign`, sets the window to 1280 px and, in en, es, pt, zh-Hans and ak:
   the first `header .meta` is the whole name, not cut, inside the header; every `header .meta` passes
   WebKit's rule (its computed `overflow` is not `hidden` while its children are only text); the profile and
   engine texts are never cut and never partly in the texts' box. Then, back in English, chromedriver's
   `getText` reads the whole name.
3. **"cuts a long file name with an ellipsis and keeps the profile and engine texts whole or drops them
   (issue #945)"** (new). At 1280 px in English, with a 68-character name set on the file label: the label is
   cut, lies in the texts' box and takes at least 90 % of it; `getText` still gives the whole name; the two
   other texts are never cut and never partly shown.

Runs. The first two rows are from the first design pass (the CLI and web bundle built from this branch).
The last two are from the second rethink: the done-tests run in headless Chrome against PR #1063's web
build (`dd9c6ff6`, QA's build in the `lane-q945` worktree), as built and with the amended CSS and the two
shorter translations patched into the built files.

| Code under test | Test 1 (button still) | Test 2 (25-character name, WebKit-readable, no stubs) | Test 3 (long name) |
|---|---|---|---|
| main `ab07d313` | fails: `x` 1180.75 with the texts drawn, 1373.09 with them empty (en) | the first version passed | not run |
| PR #1061 head `bebf0eca` | passes | the first version failed: the label is 104 px wide and cut | not run |
| PR #1063 head `dd9c6ff6`, as built | passes (42 s) | fails: `WebKit's WebDriver would read no text from this header text (en)` | fails: `the file label does not take the room there is (277 of 369 px)` |
| PR #1063's build with the amended rule patched in | passes | passes | passes |

So each test can fail, and fails on the code it is meant to catch. (In one run of the three together on
PR #1063's build, test 1 timed out at its reload wait under load; alone it passed in 42 s, and QA's 6 of 6
gate runs passed it.)

## Out of scope

- The native language test's 20 dispatches (acceptance criterion 2 of #945): the coordinator dispatches
  `app-gate.yml` on the build PR.
- The web badge and download link are web-only and present from first paint; they are not part of the rule
  (the second rethink shortens two of the link's translations, nothing else).
- Shrinking the button labels so Portuguese fits one row at 720 px: a separate question, if wanted.
- WebKit's atom itself (the `FIXME` in `isElementSubtreeHiddenByOverflow`) is upstream's; OVD works around
  it by never putting `overflow: hidden` on a text-only element the tests read.
