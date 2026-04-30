# Agent Integration

thinindex provides local guidance and audit surfaces for repository-search behavior. It does not require proprietary agent APIs, network calls, telemetry, or hosted services.

## Canonical Workflow

`wi-init` writes the canonical Repository search block to `AGENTS.md` and normalizes an existing `CLAUDE.md` when present. `wi --help` remains the source of truth for filters, examples, and subcommands.

The workflow is intentionally direct:

1. Run `build_index` before broad repository discovery.
2. Use `wi <term>` before grep, find, ls, or file reads.
3. Prefer `wi pack <term>` for implementation work.
4. Run `wi impact <term>` before editing a symbol or feature area.
5. Read only files returned by `wi` unless the result is insufficient.
6. If results are missing or stale, rerun `build_index` once before falling back.

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
