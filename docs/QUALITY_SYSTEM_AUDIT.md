# Parser Quality System Audit

This document records the final parser-quality audit state for the current thinindex quality track. It connects the parser framework, support claims, quality plugin, real-repo workflow, ctags boundary, license audit, and release checks so future changes have one coherence checklist.

Universal Ctags is optional, external, not bundled, not required, and not used by production indexing.

## Audit Summary

Current status: coherent with no major known drift.

- Code-symbol extraction is Tree-sitter-backed through the registry/query/capture framework.
- CSS, HTML, Markdown, JSON, TOML, and YAML are extras-backed deterministic landmarks, not Tree-sitter code-symbol support.
- Support claims come from `src/support.rs` and are mirrored by `README.md`, `docs/PARSER_SUPPORT.md`, and generated `docs/LANGUAGE_SUPPORT.md`.
- Quality reports and comparator output stay under `.dev_index/quality/` and do not write production SQLite `records` or `refs`.
- Real-repo checks remain ignored/manual and local to `test_repos/`.
- Release and CI gates use fixture-backed deterministic checks by default.

## Parser Framework

Production code-symbol extraction lives in `src/tree_sitter_extraction.rs`.

- `LanguageRegistry::default()` registers all active grammar adapters.
- Each adapter includes extension mapping, query spec, grammar loader, and license metadata.
- Query captures are validated before mapping records.
- Allowed captures and normalized record kinds are documented in `docs/PARSER_MAINTENANCE.md`.
- `IndexRecord.source` for parser-backed code symbols is `tree_sitter`.

No external parser command is required for production indexing. Do not add line scanners, broad regex parsers, hand parsers, or another code-symbol parser path.

## Language Support Claims

Source of truth:

- `src/support.rs`
- `README.md`
- `docs/PARSER_SUPPORT.md`
- `docs/LANGUAGE_SUPPORT.md`

Guardrails:

- supported and experimental Tree-sitter entries require grammar/query/fixture/license/docs coverage appropriate to their level
- blocked entries stay visible and claim no backend
- extras-backed formats stay marked as `extras`
- tests in `tests/support_levels.rs`, `tests/parser_conformance.rs`, and `tests/format_conformance.rs` check claim consistency

Languages and formats not listed in the support matrix are unsupported.

## Quality Plugin Isolation

Quality code lives under `src/quality/` and tests under `tests/quality*`.

Normal deterministic checks cover:

- parser conformance fixtures
- support-level claim checks
- quality report/export fixtures
- expected-symbol, expected-pattern, expected-absent, threshold, and integrity gates
- no production DB pollution from quality reports

Manual checks cover:

- optional comparator report generation
- real-repo quality gates
- quality improvement cycle
- real-repo benchmarks

Quality output is local report data only. It must not become indexed source data.

## Real Repos

Real repositories are local-only under ignored `test_repos/`.

- `docs/REAL_REPO_MANIFEST.md` documents the manifest schema and curation rules.
- `cargo test --test real_repos -- --ignored` validates local repos when present and skips clearly otherwise.
- Manifest entries should prefer expected symbols, expected patterns, expected absent symbols, and coarse thresholds over brittle total record counts.
- Third-party repository contents must not be committed.

## Ctags Boundary

The structural boundary is documented in `docs/QUALITY_CTAG_BOUNDARY.md`.

Allowed surfaces:

- optional quality comparator code under `src/quality/`
- quality tests under `tests/quality*`
- quality/comparator documentation with the explicit external/optional boundary

Forbidden surfaces:

- production parser/indexer/store/search/refs/pack/impact code
- install, uninstall, release, and package paths
- generated agent instructions
- release artifacts
- production SQLite `records` or `refs` with `source = "ctags"`

Run `cargo test --test quality_ctags_allowlist` to check the structural gate directly.

## License Audit

License policy is documented in `docs/LICENSE_AUDIT.md` and `THIRD_PARTY_NOTICES`.

Run:

```sh
cargo deny check licenses
```

Parser and grammar dependencies must remain permissively licensed and listed in notices. GPL, AGPL, LGPL-only, MPL-only, EPL, CDDL, unknown, custom, or non-commercial dependency terms block release packaging unless a future plan records an explicit review exception.

## Docs And Release Readiness

Docs must agree on these shipped facts:

- `.dev_index/index.sqlite` is canonical storage
- pre-alpha JSONL cache files are disposable rebuild input only
- `AGENTS.md` is the generated instruction surface, with existing `CLAUDE.md` normalized when present
- `WI.md` is not generated
- `wi --help` is the source of truth for CLI syntax, filters, examples, and subcommands
- ctags is optional quality-comparator tooling only
- support levels are explicit and evidence-backed

Release checks should include `scripts/check-ci`, `scripts/check-release`, archive content checks, notices, license audit, and manual smoke commands.

## Remaining Caveats

- Tree-sitter extraction is syntactic, not semantic or LSP-level analysis.
- Experimental languages remain experimental until real-repo coverage and documented gaps are stronger.
- Optional comparator output is triage evidence, not ground truth.
- Real-repo quality depends on local `test_repos/` contents and remains ignored/manual.
- Future packaging, signing, payment, telemetry, cloud, and hosted behavior are not implemented by the parser-quality track.
