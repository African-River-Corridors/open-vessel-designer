# Web app and command line only, English only, for now

Status: **accepted**, 2026-11-20. Issues #1095 (English only) and #1094 (no desktop app). The maintainer
decided on 2026-11-20.

## Context

OVD is in a fast build phase. The engine, the command line and the web app change every day. Two
things cost more than they give while that lasts:

- **The desktop app.** It needs its own CI (a Linux and a Windows build, a desktop end-to-end job, the
  installers) and its own platform code. Its end-to-end job broke on a runner image change, and no
  desktop test ran on several pull requests in a row. The web app runs the same engine (the
  WebAssembly session) and the same front end.
- **The other Display Languages.** Spanish, Portuguese (Brazil), Simplified Chinese and Twi were
  drafts, none reviewed by a native speaker. Each new English message added a draft to four files,
  and the web end-to-end tests ran their header and results checks once per language.

## Decision

1. **The web app and the command line are the product for now.** The desktop app, its tests and its
   CI are removed (issue #1094). The Python API stays.
2. **English is the one Display Language for now.** `locales/` holds `en/` only. Settings has no
   language control. A language kept by an earlier version (`ovd.language` in the browser's storage)
   is ignored, without an error. `tools/i18n_check.py` fails on any other folder or file under
   `locales/`, so a language cannot come back half done.
3. **What stays, so each can return.**
   - Languages: every text still goes through the Text Library. The app formats with `t()` and
     `say()`; the engine gives a Message Id beside its English (ADR 0015). The checks that guard the
     English stay: an id the code uses and English lacks, an English id the code does not use, the
     engine's, drawings' and inputs' English against the code.
   - Desktop: the platform seam (`app/src/lib/platform/types.ts`) and the shell crate both platforms
     shared (`crates/ovd-shell`).
4. **Nothing in the engine changes.** No results hash moves. The language was never in a result or a
   saved file.

## What a return needs

**A language.** Restore its folder from the history: `locales/<tag>/` (`app.ftl`, `engine.ftl`,
`inputs.ftl`), with `locales/REVIEWING.md` and `locales/terms.toml`. The last commit on `main` that has
them is the parent of the commit that merged issue #1095 (for example `ef36c07e`). Bring back with them:
the language's line in `LANGUAGES` (`app/src/lib/i18n.svelte.ts`), the language control in Settings
(`LanguageControl.svelte`), the start-up choice from the kept choice and the system language, the
checks of translated files in `tools/i18n_check.py` (orphans, placeholders, review status, the Term
List) and their tests, and the per-language runs of the web end-to-end tests (issue #945). Every
message added since then is a new draft. Allow `locales/<tag>` in the English-only check.

**The desktop app.** The pull request of issue #1094 names the last commit that holds the desktop code
and CI. A return restores them from there, on the platform seam, and makes the desktop end-to-end job
pass before any pull request waits on it.

## Earlier records

- ADR 0008 (translations with Fluent) and ADR 0015 (message ids, Fluent at the edge): the mechanism
  stands; only the shipped languages are paused.
- Spec 0005 (Languages): its language list and review process are paused with the languages.
