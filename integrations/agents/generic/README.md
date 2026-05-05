# Generic Agent Instructions

Use this pack when an agent accepts plain repository instructions but has no dedicated config format.

1. Run `wi <term>` directly before grep, find, ls, or file reads; `wi` auto-builds or auto-rebuilds a missing/stale index once before searching.
2. Run `wi refs <term>` before broad reference searches.
3. Run `wi pack <term>` before implementation work.
4. Run `wi impact <term>` before edits for related tests, docs, and callers.
5. Run `wi --help` when filters, examples, subcommands, or command details are needed.
6. Read only files returned by `wi` unless the result is insufficient.
7. Run `build_index` manually only when you want an explicit rebuild or when `wi` reports that auto-build failed.

`wi-stats` provides a local-only audit of recorded `wi` usage. It is an advisory report, not hard enforcement.
