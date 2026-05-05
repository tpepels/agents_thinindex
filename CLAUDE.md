## Repository search

- Use `wi <term>` before grep/find/ls/Read to locate code; `wi` auto-builds or auto-rebuilds a missing/stale index once before searching.
- Run `wi --help` if you need search filters, examples, or subcommands.
- For implementation work, prefer `wi pack <term>` to get a compact read set.
- Before editing a symbol or feature area, run `wi impact <term>` to find related tests/docs/callers.
- Read only files returned by `wi` unless the result is insufficient.
- Run `build_index` manually only when you want an explicit rebuild or when `wi` reports that auto-build failed.
- Fall back to grep/find/Read only after that retry fails.
