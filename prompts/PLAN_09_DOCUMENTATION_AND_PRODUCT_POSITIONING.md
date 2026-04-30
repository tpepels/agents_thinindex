# PLAN_09_DOCUMENTATION_AND_PRODUCT_POSITIONING.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_08 are complete and green.

Goal:
Rewrite thinindex documentation and product positioning so the project is clearly described as a local agent-navigation layer, not a faster grep clone.

This pass is primarily documentation and product-surface cleanup. Do not add new search semantics, reference extraction rules, storage changes, ML prediction, parser backend changes, packaging behavior, or new commands unless required to fix documented behavior that is currently false.

Product rule:
Docs must describe task workflows, not implementation internals. If a workflow requires explaining too many internals, improve or simplify the product surface instead of documenting around it.

Core positioning:
thinindex is a local agent-navigation layer that reduces blind repository discovery.

Primary value:
- agents read fewer irrelevant files
- agents use file:line landmarks before broad reads
- agents get compact read sets from refs/pack/impact
- users can measure value with benchmarks
- `.dev_index/index.sqlite` is a disposable local cache

Do not position thinindex as:
- a faster grep
- a replacement for IDE/LSP
- a semantic code intelligence engine
- a hosted search product
- an ML prediction system
- ready for proprietary commercial packaging before license audit, release hardening, and signing/installer work are complete

Commercial packaging caveat:
- Universal Ctags must not be bundled into proprietary release artifacts.
- Do not claim ctags can be bundled.
- Do not claim cross-platform commercial packaging is ready if any production parser path requires ctags.
- The Tree-sitter parser stack from PLAN_11A through PLAN_11C removes the old production ctags blocker; future packaging caveats should focus on current license audit, signing, installer, and release-hardening state.
- Universal Ctags may only be documented as optional external quality-comparator tooling.

Documentation targets:
Audit and update docs that exist in the repo, likely including:
- README.md
- docs/ROADMAP.md
- install/release docs if present
- command/help docs if present
- any references to WI.md
- any references to JSONL canonical storage
- any stale references to manifest.json, index.jsonl, wi_usage.jsonl
- any old AGENTS/CLAUDE workflow text
- any claim that Universal Ctags can be bundled or is safe for proprietary packaging

Required README structure:
Prefer a task-oriented README with these sections:

1. What thinindex is
2. Why agents need it
3. Quickstart
4. Agent workflow
5. Commands
6. Storage model
7. Real-repo hardening
8. Benchmarks/value measurement
9. Limitations
10. Install/uninstall
11. Packaging/licensing caveat
12. Development

What thinindex is:
State clearly:
- thinindex builds a repo-local SQLite index
- `wi` returns file:line landmarks
- `refs`, `pack`, and `impact` use deterministic references
- it is local-first
- it is intended for coding agents and developers supervising them

Quickstart:
Include the current normal flow:
- install
- `wi-init`
- `build_index`
- `wi --help`
- `wi <term>`
- `wi pack <term>`
- `wi impact <term>`

Agent workflow:
Document the canonical workflow:
- run `build_index`
- use `wi --help` when needed
- use `wi <term>` before grep/find/ls/Read
- use `wi pack <term>` for implementation work
- use `wi impact <term>` before edits
- read only returned files unless insufficient
- retry after one rebuild before fallback

Commands:
Document current commands only:
- `build_index`
- `wi <term>`
- `wi refs <term>`
- `wi pack <term>`
- `wi impact <term>`
- `wi bench` if implemented
- `wi-stats`
- `wi-init`
- `wi-init --remove`

Do not document commands that do not exist.

Storage model:
Document:
- `.dev_index/index.sqlite` is canonical
- `.dev_index` is disposable local cache
- old JSONL caches are rebuilt by `build_index`
- `wi` does not silently rebuild; it tells users to run `build_index`
- no WI.md instruction file is generated

Real-repo hardening:
Document:
- `test_repos/` is ignored/local
- users may clone repos there manually
- ignored tests can validate real repos
- `test_repos/MANIFEST.toml` if implemented
- no third-party repos are committed

Benchmarks/value measurement:
Document what the benchmark measures:
- build time
- DB size
- record/ref count
- hit/miss rate
- latency
- pack/impact sizes
- integrity status

Do not claim thinindex improves agent performance unless benchmark output supports that claim. Use measured language.

Limitations:
Document explicitly:
- not full semantic code understanding
- not an LSP replacement
- refs are deterministic and incomplete
- impact is evidence-backed but not exhaustive
- agents can still ignore instructions
- generated/build/vendor files should be ignored
- proprietary packaging is blocked if Universal Ctags returns to the required parser path
- bundled parser dependencies must be permissively licensed and audited before commercial release artifacts

Install/uninstall:
Keep docs aligned with PLAN_08:
- all binaries installed
- all binaries support `--version`
- uninstall does not delete `.dev_index`
- `wi-init --remove` removes repo-local index
- if ctags is referenced, document it as optional external quality-comparator tooling only
- do not imply ctags will ship inside release archives

ROADMAP:
Update docs/ROADMAP.md if present:
- reflect SQLite storage as current if implemented
- reflect refs/pack/impact/bench as current if implemented
- move completed phases out of future tense where appropriate
- keep future work separate from shipped behavior
- add or preserve a blocker item if production indexing ever again requires ctags before proprietary cross-platform packaging

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
Update or add docs tests if the repo has docs governance.

At minimum:
- tests should fail on references to generated `WI.md` as an instruction file
- tests should fail on stale canonical storage claims if they mention JSONL as current storage
- tests should verify README mentions `wi --help`
- tests should verify README mentions `.dev_index/index.sqlite`
- tests should verify README mentions `wi pack` and `wi impact` if those commands exist
- tests should verify README does not claim ML prediction
- tests should verify docs do not claim Universal Ctags can be bundled into proprietary releases
- tests should verify docs mention ctags as a packaging blocker only if ctags returns to production indexing

Do not make tests brittle around long prose. Prefer focused substring/anti-substring checks.

Acceptance:
- README accurately describes current product behavior.
- Docs use “agent-navigation layer” positioning.
- Docs no longer position thinindex as just search/faster grep.
- Docs do not mention WI.md as generated/current instruction surface.
- Docs do not describe JSONL as canonical storage.
- Command docs match actual `wi --help`.
- Real-repo and benchmark docs match implemented behavior.
- Docs clearly state that proprietary release packaging is blocked if production indexing again requires ctags.
- Docs do not claim ctags can be bundled.
- Existing code behavior remains unchanged unless fixing documented falsehoods.
- No normal test depends on local repos or `test_repos/`.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin wi -- --help`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- docs rewritten/updated
- positioning summary
- stale docs removed
- Tree-sitter production parser and optional ctags-comparator packaging caveat documented
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- commit hash

Phase tracking:
- [x] Rewrite README around the agent-navigation positioning and required sections.
- [x] Update roadmap to separate shipped behavior from future work and remove stale storage/instruction claims.
- [x] Add focused documentation governance tests for README, roadmap, storage, commands, and packaging caveats.
- [x] Run required verification.
- [x] Commit scoped documentation/test updates.
