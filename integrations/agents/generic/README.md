# Generic Agent Instructions

Use this pack when an agent accepts plain repository instructions but has no dedicated config format.

1. Run `build_index` before broad repository discovery.
2. Run `wi --help` when filters, examples, or subcommands are needed.
3. Run `wi <term>` before grep, find, ls, or file reads.
4. For implementation work, run `wi pack <term>` first.
5. Before editing, run `wi impact <term>` for related tests, docs, and callers.
6. Read only files returned by `wi` unless the result is insufficient.
7. If `wi` misses, rerun `build_index` once and retry before falling back.

`wi-stats` provides a local-only audit of recorded `wi` usage. It is an advisory report, not hard enforcement.
