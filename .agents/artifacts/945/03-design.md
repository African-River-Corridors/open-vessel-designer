---
ticket: 945
phase: design
date: 2026-11-10
status: rule chosen; done-tests committed; for the build seat
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
2. `.status { flex: 1 1 0; min-width: 0; display: flex; align-items: center; justify-content: flex-end;
   gap: var(--sp-2); overflow: hidden; }`. Flex basis `0` is the point: the row-breaking of a wrapping flex
   container uses each item's *hypothetical* size, and this item's is zero, so it never pushes a button and
   never starts a new row, whatever text it holds. It then takes exactly the free space `.grow` took.
3. `.meta { min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }` with
   `.meta.file { flex: 1 1 auto; font-family: var(--font-mono); }` and the other two `flex: 0 1 auto`. The file
   label is as wide as its text when there is room; when there is not, the three shrink inside `.status` and
   show an ellipsis. Each gets `title={text}` so a cut text reads in full on hover.
4. Nothing else moves. `.grow` is deleted. `header` keeps `flex-wrap: wrap`: at widths where the buttons
   alone do not fit, the row wraps exactly as it does today, before and after the texts draw alike.

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

The texts move from the far right (after Settings) to just left of the Input panel button, right-aligned in
the space. The order is unchanged: file, Defaults, Engine. Where the header is wide, the texts sit where the
eye already looks for them; where it is narrow, the file name is the first thing cut, with the tooltip. The
maintainer may prefer the texts left-aligned beside the file buttons (a document-title position); that is a
one-word change (`justify-content: flex-start`) and does not affect the rule or the tests. Logged as a
judgement call.

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

## Done-tests (committed first, on `ticket/945-header`, headed `// DONE-TEST #945 — do not edit in the build`)

`app/e2e/web-smoke.test.mjs`, listed in `.done-test`:

1. **"keeps the Settings button where it is while the header text draws in, in each language (issue #945)"**
   (kept from PR #1061, and made stricter). For en, es, pt, zh-Hans and ak: after a reload the header is one
   row; the Settings button's `x`, `y` and the one-row state are the same after the texts draw, with the three
   texts empty, with a 25-character file name, and with a 68-character name in all three.
2. **"shows a 25-character file name in full in the header (issue #945)"** (new). Opens
   `dry_bulk_box_hold.vdesign`; the first `header .meta` reads the whole name, is not cut
   (`scrollWidth <= clientWidth`) and lies inside the header.

Runs, with the CLI and web bundle built from this branch:

| Code under test | Test 1 (button still) | Test 2 (25-character name) |
|---|---|---|
| main `ab07d313` | fails: `x` 1180.75 with the texts drawn, 1373.09 with them empty (en) | passes |
| PR #1061 head `bebf0eca` | passes | fails: the label is 104 px wide and cut |
| The rule (prototype in the DOM) | `x` equal in all cases | 60 characters shown at 1280 px (en) |

So each test can fail, and fails on the code it is meant to catch.

## Out of scope

- The native language test's 20 dispatches (acceptance criterion 2 of #945): the coordinator dispatches
  `app-gate.yml` on the build PR.
- The web badge and download link are web-only and present from first paint; they are not part of the rule.
- Shrinking the button labels so Portuguese fits one row at 720 px: a separate question, if wanted.
