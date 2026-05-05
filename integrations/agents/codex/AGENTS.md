# AGENTS

## Repository search

- Use `wi <term>` directly before grep/find/ls/Read for repository discovery; `wi` auto-builds or auto-rebuilds a missing/stale index once before searching.
- Use `wi refs <term>` before broad reference searches.
- Use `wi pack <term>` before implementation to get a compact read set.
- Use `wi impact <term>` before edits to find related tests/docs/callers.
- Use `wi --help` for filters, examples, subcommands, and command details.
- Read only files returned by `wi` unless the result is insufficient.
- Run `build_index` manually only when you want an explicit rebuild or when `wi` reports that auto-build failed.
- Fall back to grep/find/Read only after that retry fails.

## Read Budget

- Start with `wi pack <term>` for the narrowest useful read set.
- Expand to files from `wi impact <term>` before editing.
- Use `wi refs <term>` before a broad reference search.
- Treat broad grep/find/ls output as a fallback, not the first discovery step.
- Use `wi-stats` to review local workflow adoption after a session.
