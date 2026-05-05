# Agent Integration

thinindex provides local guidance and audit surfaces for repository-search behavior. It does not require proprietary agent APIs, network calls, telemetry, or hosted services.

## Canonical Workflow

`wi-init` writes the canonical Repository search block to `AGENTS.md` and normalizes an existing `CLAUDE.md` when present. `wi --help` remains the source of truth for filters, examples, and subcommands.

The workflow is intentionally direct:

1. Use `wi <term>` before grep, find, ls, or file reads; `wi` auto-builds or auto-rebuilds a missing/stale index once before searching.
2. Prefer `wi pack <term>` for implementation work.
3. Run `wi impact <term>` before editing a symbol or feature area.
4. Read only files returned by `wi` unless the result is insufficient.
5. Run `build_index` manually only when you want an explicit rebuild or when `wi` reports that auto-build failed.

## Integration Packs

Optional examples live under `integrations/agents/`:

- `codex/AGENTS.md`: Codex-style repository instructions.
- `claude/CLAUDE.md`: Claude-style repository instructions.
- `generic/README.md`: Manual instructions for agents without a dedicated config file.
- `mcp/README.md`: Local-only MCP/tool integration plan for future wrappers.

These packs are advisory snippets. They do not enforce behavior by themselves and should not claim hard prevention of grep, find, ls, or file reads.

## Local Audit

`wi` records local usage events in `.dev_index/index.sqlite`. The log stays in the repository-local index and is rebuilt with the disposable local cache.

`wi-stats` reports:

- search hit and miss windows;
- command categories for `search`, `refs`, `pack`, and `impact`;
- whether context commands appear in recent local usage;
- a clear scope note that local `wi` usage cannot detect external grep, find, ls, or file-read activity.

This audit is useful for coaching and debugging workflow adoption. It is not telemetry and is not a hard enforcement mechanism.
