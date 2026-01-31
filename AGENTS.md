Agent Workflow Guidelines
=========================

This repository uses Codex CLI for agent-assisted development. Follow these rules when making changes:

- Git staging
  - Do NOT use `git add -A` or `git add .`.
  - Only stage files you intentionally modified, scoped to the directory you worked in (e.g., `git add doc-parser/src/parser.rs`).
  - If a change spans multiple crates or subdirectories, create separate commits and stage each path explicitly per commit.

- Plans and progress
  - Keep plans concise and update status as steps complete.
  - Group related actions into a single preamble message before running commands.

- Code changes
  - Prefer small, focused patches that address the stated task.
  - Maintain existing code style and structure.
  - Update or add tests alongside code changes where practical.

- Validation
  - Run only the necessary tests for the code you changed.
  - Avoid introducing unrelated formatting or refactors unless explicitly requested.

These guidelines apply across the entire repository unless a deeper `AGENTS.md` overrides them.

## E2E Demo Report

Run the e2e demo regularly to verify the full sync workflow works correctly with Anki. This generates an HTML report with screenshots showing card creation, updates, and deletion.

**To run:**
```bash
./e2e/run-demo.sh
```

**Output:** `e2e/demo-output/demo_report.html`

Run this:
- After significant changes to sync logic, card generation, or Anki integration
- Before releases to verify end-to-end functionality
- When debugging sync issues (the report shows exact CLI output and resulting cards)

The demo runs in a container with a headless Anki instance, so no local Anki installation is required.

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
