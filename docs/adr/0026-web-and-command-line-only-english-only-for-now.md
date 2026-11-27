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
   CI are removed (issue #1094). The Python API stays. What went:
   - the desktop crate `app/src-tauri/` (the `ovd-app` workspace member, its icons, its Tauri
     config) and its entries in `Cargo.lock`, with the `e2e` build profile only its tests used;
   - the Tauri adapter `app/src/lib/platform/tauri.ts`, the desktop Vite config, the Tauri packages
     and the `tauri` and `e2e` scripts of `app/package.json`;
   - the desktop-only front-end paths: the "open with" listener (`onOpenedFromOutside`), the native
     dialogs and the first-run marker file of the adapter, the "web version" label and the
     "Download the desktop app" link;
   - the desktop end-to-end tests (`app/e2e/smoke.test.mjs`, `run.sh`);
   - the desktop CI: the `app-e2e` job, `app-checks.yml` (daily Linux and Windows builds, the issue it
     opened on a failure) and the installer job of `app-build.yml`. A release now holds the `ovd`
     binaries and the Python wheels that the release gate built and tested;
   - the desktop download from the README and the website.
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

**The desktop app.** The last commit that holds the desktop code and CI is the parent of the commit
that removed it (issue #1094); `3271b020` on `main` holds them too. A return:

- restores `app/src-tauri/`, `platform/tauri.ts`, the desktop Vite config, the Tauri packages, the
  desktop end-to-end tests and the CI jobs from there, and adds the crate back to the workspace;
- implements the `Platform` interface (`app/src/lib/platform/types.ts`) again, widening `kind` to the
  desktop's name, and brings back the "open with" listener and the native dialogs;
- carries over every command the web adapter gained since (the setup wizard's, say) as a Tauri
  command over `crates/ovd-shell`;
- makes the desktop end-to-end job pass, on a pinned runner image, before any pull request waits on it.

## Earlier records

- ADR 0008 (translations with Fluent) and ADR 0015 (message ids, Fluent at the edge): the mechanism
  stands; only the shipped languages are paused.
- Spec 0005 (Languages): its language list and review process are paused with the languages.
