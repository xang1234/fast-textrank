# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

# Build (PyO3 + Python 3.14)

This project uses PyO3 to bind Rust to Python. The local `.venv` runs Python 3.14.

## Required environment variable

**Always** prefix cargo and maturin commands with:

```bash
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
```

Without it, PyO3 rejects Python 3.14 with a cryptic ABI version error.

## Running tests — use `cargo test --lib`

```bash
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo test --lib
```

**Do NOT run bare `cargo test`** — it tries to link integration/benchmark test binaries as a Python dylib and fails with hundreds of undefined `_Py*` symbol errors. The `--lib` flag restricts to unit tests inside `src/` and avoids the dylib link.

To filter to a specific module:

```bash
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo test --lib module_name
```

## Known test failures (ignore these)

Two tests in `python::json::tests` always fail:
- `test_json_include_pos_filtering`
- `test_json_include_pos_multiple_tags`

These are pre-existing bugs in the POS-filtering test setup, not caused by your changes. A full `cargo test --lib` run shows `N passed; 2 failed` — the 2 failures are expected.

## Building the Python wheel

```bash
source .venv/bin/activate
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --release
```
