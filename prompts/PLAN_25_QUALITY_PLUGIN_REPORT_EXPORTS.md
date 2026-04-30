# PLAN_25_QUALITY_PLUGIN_REPORT_EXPORTS.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_24_SINGLE_CYCLE_QUALITY_IMPROVEMENT_RUNNER.md is complete and green.

Progress:
- [x] Inspect existing quality report, gate, gap, cycle, and support-matrix data.
- [x] Add deterministic Markdown, JSON, and JSONL export helpers under `src/quality/`.
- [x] Add export tests for determinism, required sections, JSON parsing, size control, DB isolation, and no ctags requirement.
- [x] Update quality docs and dependency notices.
- [x] Run required verification.
- [x] Commit with completed plan checkboxes.

Goal:
Add stable quality report exports for parser/index quality results.

This pass only improves isolated quality-plugin reporting. Do not add parser architecture, new languages, release packaging, license enforcement, payment behavior, telemetry, cloud behavior, or unrelated product commands.

Product rule:
Quality reports must be reproducible, readable by humans, and usable by agents without reading huge raw JSON blobs.

Isolation:
Keep report generation under the quality plugin layer:
- `src/quality/`
- `tests/quality_*.rs`
- `docs/QUALITY.md`
- local reports under `.dev_index/quality/`

Do not write report data into production `records` or `refs`.

Required outputs:
Support compact report formats:
- Markdown summary
- JSON summary
- optional JSONL detail file if useful

Report contents:
- run timestamp or explicit deterministic mode
- repo list
- language support matrix
- records by language
- refs by language
- expected symbols checked/missing
- expected patterns checked/failing
- comparator-only symbols
- thinindex-only symbols
- parser errors
- unsupported extensions
- slow/noisy files if available
- gap summary
- cycle-plan summary if available

Determinism:
- Sort all repos, paths, languages, symbols, and gaps deterministically.
- Normal tests must use deterministic timestamps or omit timestamps.
- Do not include machine-specific absolute paths unless explicitly in local-only reports.

Size control:
- Keep Markdown summary compact.
- Large details should be separate.
- Do not dump huge raw comparator output into summaries.
- Reports must be safe for agents to inspect without multi-MB context dumps.

Tests:
- report export is deterministic
- Markdown contains required sections
- JSON parses
- large detail data is not embedded in summary
- production DB is not modified
- no ctags required

Docs:
Update quality docs:
- how to generate reports
- report file locations
- how to read summary vs details
- what each section means
- what should be committed and what stays local

Acceptance:
- quality reports can be exported in Markdown and JSON
- exports are deterministic in tests
- reports are compact and agent-readable
- no production index pollution
- existing quality gates remain stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- ctags allowlist gate
- license audit command if configured
- quality report command/script/test if added
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- report formats added
- report paths
- deterministic-output behavior
- sample report summary
- verification commands and results
- commit hash
