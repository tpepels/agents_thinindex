# PLAN_40_TECHNICAL_FINAL_AUDIT.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_39_AGENT_WORKFLOW_ENFORCEMENT_AND_INTEGRATION_PACKS.md is complete and green.

Progress:
- [x] Phase 1: audit parser, dependency, refs, pack, impact, performance, semantic, and agent docs for drift
- [x] Phase 2: fix stale relationship/navigation claims found during audit
- [x] Phase 3: add final technical audit documentation and coherence tests
- [x] Phase 4: run required verification and commit completed Plan 40 work

Goal:
Audit the technical relationship/navigation layer and remove drift before moving to security, packaging, licensing, and product polish.

Product rule:
Parser, dependency graph, refs, pack, impact, quality, and agent integration must agree. No stale claims.

Audit areas:
1. Parser:
   - Tree-sitter remains code-symbol backbone
   - no ctags production dependency
   - support matrix honest
   - quality gates stable

2. Dependency graph:
   - edges stored correctly
   - stale cleanup works
   - unresolved edges explicit
   - resolver docs match behavior

3. References:
   - confidence labels accurate
   - fallback behavior documented
   - stale refs cleaned

4. Impact:
   - evidence/reasons/confidence shown
   - no exhaustive semantic claims
   - tests/config/dependencies included where practical

5. Pack:
   - bounded output
   - reasoned groups
   - no huge dumps
   - useful for agents

6. Performance:
   - large repo behavior bounded
   - incremental rebuild correct
   - generated/vendor guidance current

7. Semantic adapters:
   - optional only
   - no required external tools
   - docs honest

8. Agent integration:
   - AGENTS/CLAUDE generation current
   - no WI.md
   - usage/audit local only

Required:
- Fix inconsistencies found by audit.
- Add missing tests for discovered gaps.
- Remove stale docs.
- Do not hide uncertainty.
- Do not add major new architecture unless needed to fix a serious inconsistency.

Acceptance:
- technical layers are internally consistent
- docs/tests match implementation
- no major known drift remains
- ready to proceed to security/privacy, packaging, licensing, and product polish plans

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- ctags allowlist gate if present
- license audit command if configured
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- audit findings
- fixes applied
- remaining caveats
- verification commands and results
- commit hash
