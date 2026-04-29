# PLAN_12G_PARSER_PERFORMANCE_AND_REGRESSION_GATES.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_12A through PLAN_12F are complete and green.

Goal:
Harden Tree-sitter parser performance, scalability, and regression gates after broad language coverage is implemented.

This pass does not add new languages. It makes the parser framework fast, bounded, measurable, and hard to regress.

Product rule:
Broad parser support is not useful if indexing becomes slow, memory-heavy, noisy, or unstable on real repositories.

Scope:
This pass focuses on:
- parser performance
- query performance
- memory/resource bounds
- parser diagnostics
- regression gates
- real-repo benchmark stability

Do not add:
- new language support unless required to fix a regression from existing declared support
- a second parser architecture
- hand parsers
- line scanners
- ctags
- release packaging
- license enforcement
- payment behavior
- telemetry
- cloud behavior
- new product commands unless required for internal reporting

Hard requirements:
- Use the existing Tree-sitter extraction framework.
- Do not create a second parser architecture.
- Do not use line-oriented or regex-based code-symbol parsing.
- Do not call or reintroduce ctags.
- Do not add GPL or AGPL dependencies.
- No newly built index may emit `source = "ctags"`.
- Do not weaken parser conformance, symbol coverage, or real-repo hardening checks from PLAN_12E/12F.

Performance targets:
Define practical performance budgets for:
- total `build_index` time on fixture repos
- total `build_index` time on real repos under `test_repos/`
- parser time by language
- records emitted by language
- refs emitted by language
- parse error count by language
- maximum per-file parse time warning threshold
- maximum per-file record count warning threshold

Do not use brittle exact timing assertions in normal tests. Use:
- smoke thresholds only where stable
- ignored/manual real-repo performance reports
- regression-friendly summaries

Required implementation:
1. Add parser timing instrumentation internally.
2. Report parser timing by language in benchmark or real-repo hardening output.
3. Report parser record counts by language.
4. Report parse errors by language.
5. Add warnings for unusually slow files.
6. Add warnings for unusually noisy files.
7. Add safeguards against record explosions.
8. Add safeguards against ref explosions.
9. Add deterministic ordering for performance reports.
10. Ensure unsupported/generated/vendor/minified files are ignored or reported clearly.
11. Add or update ignore guidance for generated/vendor/minified paths.

Resource bounds:
Add named constants or config points for:
- max parseable file size if needed
- max records per file warning threshold
- max refs per file warning threshold
- max parse time warning threshold if practical
- minified/generated file detection where practical

Do not silently drop important source files without reporting the reason.

Regression gates:
Add tests/checks that catch:
- record explosion from one file
- duplicate records
- malformed records
- malformed refs
- ctags source records
- parser panic on bad syntax
- comments/strings creating fake symbols
- supported language fixture losing expected symbols
- expected-symbol manifest checks failing

Normal tests:
Normal `cargo test` must not depend on real `test_repos/`.

Add fixture-based tests for:
- parser does not panic on malformed files
- parser handles large-ish but reasonable files
- parser warning/report path works
- record explosion guard works
- deterministic parser report ordering

Ignored/manual tests:
Update ignored real-repo tests to print:
- repos checked
- files parsed
- parse time by language
- record count by language
- ref count by language
- slowest files
- noisiest files
- parse errors
- unsupported extensions
- expected symbol coverage summary

If `wi bench` exists:
- include parser timing/resource metrics in bench output.
- do not pollute usage_events if an internal non-logging query path exists.

Docs:
Update parser/support docs with:
- performance expectations
- generated/vendor/minified ignore guidance
- how to interpret parser coverage/performance reports
- how to add expected-symbol checks for real repos
- known limits around macros/generated code/semantic resolution

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth.
- Keep AGENTS.md and existing CLAUDE.md generation aligned with the canonical Repository search block.

Acceptance:
- parser performance reporting exists
- real-repo hardening reports parser time and record/ref counts by language
- slow/noisy files are surfaced
- record/ref explosion safeguards exist
- broad parser support remains covered by conformance and expected-symbol checks
- normal tests remain deterministic and do not require `test_repos/`
- no second parser architecture is introduced
- no ctags or line-scanner code parser backend is reintroduced
- no GPL/AGPL dependency is introduced
- SQLite, refs, pack, impact, bench, stats, and wi-init remain stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- license audit command if configured
- `grep -R "ctags\\|Ctags\\|CTAGS" src tests docs README.md Cargo.toml install.sh uninstall.sh THIRD_PARTY_NOTICES || true`
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- if `wi bench` exists: `cargo run --bin wi -- bench`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- parser performance metrics added
- resource bounds added
- slow/noisy file handling
- real-repo performance summary
- regression gates added
- verification commands and results
- ignored local/real repo test status
- commit hash
