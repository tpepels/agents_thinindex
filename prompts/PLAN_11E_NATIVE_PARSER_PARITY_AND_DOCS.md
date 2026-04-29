# PLAN_11E_NATIVE_PARSER_PARITY_AND_DOCS.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_11A through PLAN_11D are complete and green.

Goal:
Finish the ctags-removal migration by validating native parser parity, removing stale ctags references, and updating docs/license notes so packaging work can proceed honestly.

This pass does not add new language families unless needed to close obvious gaps from previous 11-series plans. Do not add license enforcement, payment behavior, telemetry, cloud behavior, or release packaging behavior.

Product rule:
The ctags blocker is resolved only if ctags is absent from code/test/install requirements and native parser dependencies are permissively licensed and documented.

Required audit:
Search for all active ctags references.

Allowed references after this plan:
- docs saying Universal Ctags was removed
- docs saying Universal Ctags is not bundled and not used
- historical roadmap notes if clearly marked as historical

Forbidden references:
- code calling ctags
- tests requiring ctags
- install docs requiring ctags
- help text requiring ctags
- product docs saying ctags can be bundled
- package/release docs including ctags

Parser parity checks:
Validate native parser behavior against the supported language set:
- Rust
- Python
- JavaScript/JSX
- TypeScript/TSX if implemented
- CSS/HTML/Markdown via extras or parser support

Parity does not mean perfect ctags equivalence. It means the product has useful deterministic extraction for common user projects.

Required tests:
- no records with `source = "ctags"` in fixture/local indexes
- Rust/Python/JS/TS fixture symbols searchable with `wi`
- CSS/HTML/Markdown extras remain searchable
- refs/pack/impact still produce useful output on fixture data
- real-repo ignored test passes or reports clear extraction gaps
- benchmark output still runs
- duplicate/integrity checks pass

Docs:
Update:
- README.md
- docs/ROADMAP.md
- docs/PRODUCT_BOUNDARY.md
- THIRD_PARTY_NOTICES or equivalent
- docs/RELEASING.md / install docs if present

Required docs state:
- Universal Ctags is removed.
- thinindex uses bundled permissively licensed native parser dependencies.
- parser support is limited to documented languages.
- native parser gaps are documented.
- proprietary packaging is no longer blocked by Universal Ctags if license audit passes.
- all bundled parser dependencies/grammars are listed in notices/audit docs.

Third-party/license notes:
For every parser/grammar dependency:
- record name
- license
- source/upstream if available
- why accepted

Do not hide uncertainty. If a license is unclear, mark packaging blocked.

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth for command syntax, filters, examples, and subcommands.
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical `## Repository search` block.
- AGENTS.md should be created if absent.
- CLAUDE.md should be normalized only if present; do not create CLAUDE.md.
- Repeated `wi-init` runs must not duplicate `## Repository search`.
- Remove/normalize legacy markers: `@WI.md`, weak WI.md references, and old paragraph-style Repository search blocks.
- Update tests whenever help text or canonical Repository search text changes.

Acceptance:
- ctags is fully removed from active code/tests/install requirements.
- no newly built index records use `source = "ctags"`.
- native parser support is documented honestly.
- bundled parser/grammar licenses are documented.
- parser-related packaging blocker is either documented as resolved or explicitly still blocked with reasons.
- SQLite, refs, pack, impact, bench, stats, and wi-init behavior remain stable.
- no JSONL storage is reintroduced.
- no license/payment/network behavior is added.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `grep -R "ctags\\|Ctags\\|CTAGS" src tests docs README.md Cargo.toml install.sh uninstall.sh THIRD_PARTY_NOTICES || true`
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- remaining ctags references and why allowed, if any
- parser support matrix
- parser/grammar license summary
- known extraction gaps
- packaging blocker status
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- commit hash

Phase tracking:
- [x] Audit active ctags references and parser/license documentation surfaces.
- [x] Update docs/notices to document native parser support, gaps, and packaging blocker status.
- [x] Add or adjust parity tests for supported parser/extras behavior.
- [x] Run required verification.
- [x] Commit scoped PLAN_11E changes.
