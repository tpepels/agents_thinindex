# PLAN_10_PRICING_BOUNDARY_AND_PRO_EDITION.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_09 are complete and green.

Goal:
Define the product boundary between free/local thinindex and a future Pro edition, without prematurely adding licensing, payments, cloud services, artificial restrictions, or proprietary packaging claims before license audit, release hardening, signing, and installer work are complete.

This pass is product/design/documentation work with light code only if needed to expose current edition/status cleanly. Do not add payment integration, license checks, network calls, account systems, telemetry, or feature gating unless explicitly required by this plan.

Product rule:
Do not make the current tool worse to create a paid tier. The free/local core must remain useful.

Core positioning:
thinindex is a local agent-navigation layer that reduces blind repository discovery.

Free/local core should include:
- `wi-init`
- `build_index`
- local index storage
- `wi <term>`
- basic filters and ranking
- `wi --help`
- AGENTS.md and existing CLAUDE.md setup
- local `.dev_index/index.sqlite`
- basic `wi-stats`
- index integrity checks
- local-only operation

Possible Pro candidates:
- `wi pack <term>` advanced context packs
- `wi impact <term>` advanced impact analysis
- `wi bench` reports
- real-repo benchmark manifests/reports
- advanced reference graph ranking
- CI/reporting integrations
- richer agent integration packs
- team/shared policy packs
- release-quality Tree-sitter parser hardening, if it materially improves bundled parser quality
- exported reports for audits or team review
- signed installers and managed update channel, after licensing, release-hardening, and signing/notarization blockers are resolved

Do not decide final pricing in code. Define the boundary and product rationale.

Commercial packaging blocker:
- Universal Ctags must not be bundled into proprietary release artifacts.
- Do not claim ctags can be bundled.
- Do not claim proprietary Windows/macOS/Linux packaging is ready if production indexing requires ctags.
- The Tree-sitter parser stack from PLAN_11A through PLAN_11C removes the old production ctags blocker; current packaging readiness still depends on license audit, release hardening, signing/notarization scaffolding, and installer status.
- Universal Ctags may only be documented as optional external quality-comparator tooling.
- This plan must not add release artifacts, installers, or license gates.

Required docs:
Add or update a product/pricing boundary doc, for example:

- `docs/PRODUCT_BOUNDARY.md`
- or a clearly named section in `docs/ROADMAP.md`

Preferred new doc:
`docs/PRODUCT_BOUNDARY.md`

Required sections:
1. Product principle
2. Free/local edition
3. Candidate Pro features
4. What must never be paywalled
5. Packaging/licensing blockers
6. What is not being built yet
7. Licensing/payment deferred decisions
8. Evidence needed before charging
9. Open questions

Product principle:
State:
- free thinindex must remain a useful local agent-navigation tool
- paid value should come from proof, hardening, integrations, packaging convenience, and advanced workflows
- do not charge for basic local search or basic repo indexing

What must never be paywalled:
- local indexing
- `wi <term>`
- basic filters
- `wi --help`
- `wi-init`
- AGENTS.md/existing CLAUDE.md setup
- ability to remove `.dev_index`
- local cache rebuilds
- no-network local operation

Candidate Pro features:
Describe as candidates, not implemented gates:
- advanced context packs
- impact analysis
- benchmark reports
- CI integration
- curated real-repo benchmark reports
- team policy/instruction packs
- exported agent-readiness reports
- advanced reference graph quality features
- signed installers and managed update channel after licensing, release-hardening, and signing/notarization blockers are removed

Packaging/licensing blockers:
Document clearly:
- Universal Ctags is not suitable for bundling in proprietary release artifacts.
- The permissively licensed Tree-sitter production parser stack must remain in place and audited before proprietary same-binary Pro packaging.
- All bundled parser dependencies and grammar dependencies must be audited before packaging.
- `THIRD_PARTY_NOTICES` and a dependency license audit are required before release packaging.
- Packaging plans must not proceed until license audit, release hardening, signing/notarization, and installer requirements are satisfied.

What is not being built yet:
Explicitly say this plan does not add:
- license enforcement
- payments
- account login
- cloud sync
- telemetry
- remote indexing
- feature lockouts
- network calls
- release installers
- ctags bundling
- Tree-sitter parser hardening

Evidence needed before charging:
Document required proof:
- benchmark output showing reduced discovery waste
- real agent tasks showing fewer broad reads/grep calls
- improved read-set precision from `wi pack`
- useful affected-file recall from `wi impact`
- real-repo benchmark stability
- user workflow examples

Optional code:
Add a harmless edition/status command only if it fits cleanly, for example:
- `wi --version` already exists and may be enough

Do not add:
- `wi pro`
- license files
- license validation
- config gates
- paid feature checks
- payment calls

README:
Update README only if needed to mention:
- thinindex is currently local/free
- Pro features are roadmap/candidates, not current restrictions
- no cloud/license/payment system exists yet
- proprietary packaging is blocked if ctags is required by production indexing instead of the permissively licensed Tree-sitter parser stack

ROADMAP:
Update docs/ROADMAP.md if present:
- add or update monetization section to match `PRODUCT_BOUNDARY.md`
- remove stale claims if any say licensing/payment is currently implemented
- keep future decisions clearly separate from shipped behavior
- add production ctags removal as a blocker before commercial packaging only if ctags returns to the production parser path

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth for command syntax, filters, examples, and subcommands.
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical `## Repository search` block.
- AGENTS.md should be created if absent.
- CLAUDE.md should be normalized only if present; do not create CLAUDE.md.
- Repeated `wi-init` runs must not duplicate `## Repository search`.
- Remove/normalize legacy markers: `@WI.md`, `See WI.md for repository search/index usage.`, `See `WI.md` for repository search/index usage.`, and old paragraph-style Repository search blocks.
- Update tests whenever help text or canonical Repository search text changes.

Tests:
If docs governance exists, add focused tests:
- `docs/PRODUCT_BOUNDARY.md` exists
- doc states local indexing and `wi <term>` are free/local core
- doc states payments/licensing are not implemented yet
- doc does not claim current license enforcement
- doc states ctags cannot be bundled into proprietary release artifacts
- doc states the permissively licensed Tree-sitter production parser stack must remain in place and audited for proprietary packaging
- README does not claim paid features are currently gated unless code implements it
- README does not claim proprietary packaging is ready if ctags is required by production indexing

Do not add brittle long-prose tests.

Acceptance:
- product boundary is documented clearly
- free/local core is protected in docs
- Pro candidates are framed as candidates, not active gates
- packaging/licensing blockers are documented clearly
- no license/payment/network/telemetry behavior is added
- README/ROADMAP do not misrepresent current behavior
- no proprietary packaging readiness is claimed if ctags is required by production indexing
- existing code behavior remains unchanged unless a small docs/status adjustment is required
- no JSONL storage is reintroduced
- no `WI.md` dependency is reintroduced

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin wi -- --help`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- product boundary summary
- what is explicitly free/local
- what remains candidate Pro
- packaging/licensing blocker summary
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- commit hash

Phase tracking:
- [x] Add product boundary documentation.
- [x] Align README and roadmap with local/free and candidate Pro positioning.
- [x] Add focused docs governance tests for product boundary, packaging blockers, and deferred paid systems.
- [x] Run required verification.
- [x] Commit scoped PLAN_10 changes.
