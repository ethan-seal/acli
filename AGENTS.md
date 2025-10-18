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
