## Repository search

- Before broad repository discovery, run `build_index`.
- Run `wi --help` if you need search filters, examples, or subcommands.
- Use `wi <term>` before grep/find/ls/Read to locate code.
- For implementation work, prefer `wi pack <term>` to get a compact read set.
- Before editing a symbol or feature area, run `wi impact <term>` to find related tests/docs/callers.
- Read only files returned by `wi` unless the result is insufficient.
- If `wi` returns no useful result, rerun `build_index` once and retry.
- Fall back to grep/find/Read only after that retry fails.

## Local Audit

Run `wi-stats` to inspect repository-local usage events. The audit is advisory and cannot detect external grep/find/ls/Read activity.
