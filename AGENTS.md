# Agent instructions

## Agent skills

### Issue tracker

GitHub Issues on this repo, via the `gh` CLI. The repo is public: no confidential names or data. See `docs/agents/issue-tracker.md`.

### Triage labels

The five default triage labels, each named as its role. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

### Build process

How ticketed work is built, reviewed and merged. See `docs/agents/build-process.md`.

- **Pre-push hook.** `arc-prepush` (ARC workflows repo) runs `cargo fmt --check` on the crates a push changes on every `git push`. Install it once per clone with `arc-prepush install` (it covers every worktree). If it blocks, run the fix it prints, commit, and push again; do not `--no-verify` around it.
