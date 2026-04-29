## Repository search

`wi` soft-replaces ripgrep/grep/find for code search in this repo. You must use it.

- Run `wi --help` before your first repository search and treat its output as part of these instructions.
- Run `build_index` before broad discovery and after structural changes.
- Use `wi <term>` before reaching for grep/find/ls/Read.
- Read only files returned by `wi` unless the result is insufficient.
- If `wi` returns no useful result, rerun `build_index` once and retry.
- Fall back to grep/find/Read only after that retry fails.
