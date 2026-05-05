# Generic Agent Instructions

Use this pack when an agent accepts plain repository instructions but has no dedicated config format.

1. Run `wi <term>` before grep, find, ls, or file reads; `wi` auto-builds or auto-rebuilds a missing/stale index once before searching.
2. Run `wi --help` when filters, examples, or subcommands are needed.
3. For implementation work, run `wi pack <term>` first.
4. Before editing, run `wi impact <term>` for related tests, docs, and callers.
5. Read only files returned by `wi` unless the result is insufficient.
6. Run `build_index` manually only when you want an explicit rebuild or when `wi` reports that auto-build failed.

`wi-stats` provides a local-only audit of recorded `wi` usage. It is an advisory report, not hard enforcement.
