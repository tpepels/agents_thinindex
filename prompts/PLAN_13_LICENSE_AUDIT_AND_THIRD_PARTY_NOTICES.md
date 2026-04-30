# PLAN_13_LICENSE_AUDIT_AND_THIRD_PARTY_NOTICES.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_12G are complete and green.

Goal:
Add a repeatable dependency/license audit process and third-party notice generation so thinindex can move toward proprietary Windows/macOS/Linux packaging without hidden GPL/AGPL/copyleft risk.

This pass does not add new parser behavior, search semantics, storage changes, ML prediction, pricing gates, payment behavior, telemetry, or release installers.

Product rule:
No bundled dependency may be assumed commercially safe without an explicit license audit record.

Hard requirements:
- Do not add GPL or AGPL dependencies.
- Do not reintroduce Universal Ctags.
- Confirm rewritten PLAN_11A through PLAN_11C removed Universal Ctags from active code/tests/install requirements and added the audited Tree-sitter framework/query pack.
- Do not reintroduce JSONL storage.
- Do not reintroduce `WI.md`.
- Do not add license enforcement or payment code.
- Do not claim commercial packaging is ready unless the audit supports it.

Scope:
Create or update:

- dependency license audit tooling
- `THIRD_PARTY_NOTICES` or equivalent
- docs for bundled dependency policy
- tests/checks that fail on forbidden licenses where practical

Preferred tooling:
Use a Rust dependency license audit tool if practical.

Acceptable tools:
- `cargo-deny`
- `cargo-about`
- both, if useful and not excessive

Keep the setup simple and maintainable.

Required license policy:
Allow permissive licenses such as:
- MIT
- Apache-2.0
- BSD-2-Clause
- BSD-3-Clause
- ISC
- Zlib
- Unicode-3.0 where applicable
- CC0/Public Domain where applicable

Deny or require explicit review for:
- GPL
- AGPL
- LGPL unless deliberately accepted after review
- MPL unless deliberately accepted after review
- EPL/CDDL unless deliberately accepted after review
- unknown/no license
- custom licenses
- non-commercial licenses

If a dependency has a complex or dual license, document the chosen interpretation and do not hide uncertainty.

Parser/grammar audit:
Audit every bundled parser and grammar dependency introduced by PLAN_11.

For each bundled parser/grammar, record:
- crate/package name
- upstream repository if available
- license
- why it is acceptable
- whether source/notice text is included or referenced

If any bundled parser/grammar license is unclear, remove that dependency or mark packaging blocked.

Third-party notices:
Add or update `THIRD_PARTY_NOTICES` with:
- direct runtime dependencies that must be noticed
- bundled parser/grammar dependencies
- SQLite/rusqlite/libsqlite3-sys status if bundled
- any generated parser code if applicable
- explicit statement that Universal Ctags is not bundled and not used

Do not overclaim legal conclusions. The notice file should record facts and obligations, not give legal advice.

Docs:
Update docs as needed:
- README.md
- docs/PRODUCT_BOUNDARY.md
- docs/ROADMAP.md
- install/release docs if present

Required doc content:
- bundled dependencies are audited before release packaging
- Universal Ctags is not bundled and not used
- only permissively licensed parser dependencies are allowed
- proprietary packaging remains blocked if audit finds unknown/copyleft dependencies
- `THIRD_PARTY_NOTICES` is part of release artifacts

Cargo metadata:
Add configuration files if using tools, for example:
- `deny.toml`
- `about.toml`
- `about.hbs`

Do not add noisy/generated notice output to the repo unless it is intended to be committed as `THIRD_PARTY_NOTICES`.

Tests/checks:
Add focused tests or scripts if practical.

Required checks:
- license audit command is documented
- forbidden ctags references do not return in code/docs except historical/explicit “not bundled/not used” statements
- `THIRD_PARTY_NOTICES` exists
- docs do not claim Universal Ctags is bundled
- docs do not claim GPL dependencies are commercially safe
- parser dependency license entries exist in notices/audit docs

If `cargo-deny` is added:
- add a verification command for `cargo deny check licenses`
- configure allowed/denied licenses explicitly

If `cargo-about` is added:
- add a verification command that generates notices
- document whether generated notices should be committed or checked manually

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth for command syntax, filters, examples, and subcommands.
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical `## Repository search` block.
- AGENTS.md should be created if absent.
- CLAUDE.md should be normalized only if present; do not create CLAUDE.md.
- Repeated `wi-init` runs must not duplicate `## Repository search`.
- Remove/normalize legacy markers: `@WI.md`, `See WI.md for repository search/index usage.`, `See `WI.md` for repository search/index usage.`, and old paragraph-style Repository search blocks.
- Update tests whenever help text or canonical Repository search text changes.

Acceptance:
- dependency/license audit process exists and is documented
- forbidden/copyleft license policy is explicit
- bundled parser/grammar dependencies are audited
- `THIRD_PARTY_NOTICES` exists and is accurate enough for current bundled dependencies
- Universal Ctags is not bundled, not used, and not required
- no GPL/AGPL dependency is introduced
- docs reflect packaging/license reality
- existing CLI behavior remains stable
- no release installer/archive behavior is added in this plan

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- license audit command if added, for example `cargo deny check licenses`
- notice generation command if added, for example `cargo about generate about.hbs`
- `grep -R "ctags\\|Ctags\\|CTAGS" src tests docs README.md Cargo.toml install.sh uninstall.sh THIRD_PARTY_NOTICES || true`
- `cargo run --bin wi -- --help`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- audit tooling added
- allowed/denied license policy
- parser/grammar license audit summary
- THIRD_PARTY_NOTICES summary
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- remaining legal/licensing caveats
- commit hash

## Implementation tracking

- [x] Confirm PLAN_00 through PLAN_12G are complete and the Tree-sitter parser stack is active.
- [x] Add repeatable dependency license audit tooling.
- [x] Configure explicit permissive license policy and private workspace handling.
- [x] Update `THIRD_PARTY_NOTICES` for runtime dependencies, parser grammars, generated parser sources, SQLite bundling, and Universal Ctags removal.
- [x] Update docs for audit command, packaging blockers, parser dependency policy, and release notice requirements.
- [x] Add focused tests/checks for audit policy, notices, parser license entries, and forbidden documentation claims.
- [x] Run required verification.
- [x] Commit with `Add license audit and third-party notices`.
