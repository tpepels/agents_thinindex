# Optional MCP Integration Plan

This is a placeholder plan for local-only tool integration. It is not required for normal thinindex use.

A future MCP or tool wrapper should:

- expose `build_index`, `wi`, `wi refs`, `wi pack`, `wi impact`, and `wi-stats` as local commands;
- preserve `wi --help` as the source of truth for syntax and filters;
- avoid network access, telemetry, hosted state, or proprietary agent APIs;
- record only the existing repository-local usage events;
- surface the same advisory limitation as `wi-stats`: local usage cannot prove that an agent avoided external grep, find, ls, or file reads.

This file is advisory documentation only. No MCP server is bundled by this plan.
