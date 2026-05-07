# PLAN_63_SEMANTIC_FACT_USER_VALUE_DECISION.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_62_TARGET_PLATFORM_RELEASE_SMOKE.md is complete and green.

Goal:
Decide whether semantic facts should remain isolated/internal or become a scoped user-facing feature.

Context:
The feature wiring audit found that `semantic_facts` table/model and tests exist, but normal `wi`, `refs`, `pack`, and `impact` workflows do not consume semantic facts.

Scope:
Decision and minimal alignment only. Do not add full LSP/compiler integration, package-manager execution, network access, hosted features, telemetry, payment/licensing enforcement, ctags production use, JSONL canonical storage, or `WI.md`.

Product rule:
Do not keep ambiguous half-features. Semantic facts should either be explicitly deferred/internal or connected to a narrow user-visible value.

Decision options:
- keep semantic facts internal/deferred
- remove or hide misleading docs if it is purely internal
- add a small optional/fake-adapter-only user-visible diagnostic
- create a later scoped real semantic-adapter plan
- explicitly decline semantic feature work for now

Phases:
- [ ] Inspect semantic data model, tests, docs, and any CLI mentions.
- [ ] Confirm whether normal user flows consume semantic facts.
- [ ] Identify overclaims or confusing docs.
- [ ] Decide internal/deferred vs scoped user-facing path.
- [ ] Implement only minimal docs/tests/CLI cleanup required by the decision.
- [ ] Do not add real language-server integrations in this plan.
- [ ] Run verification.
- [ ] Commit.

If deferred:
- docs must say semantic facts are internal/experimental/deferred.
- normal user docs must not imply semantic analysis is active.
- support matrix must not overclaim semantic support.

If scoped user-facing:
- feature must be optional and clearly labeled.
- output must be bounded.
- tests must prove no normal workflow depends on unavailable LSP/compiler tools.

Acceptance:
- semantic facts have a clear product status.
- docs and tests match actual behavior.
- no user-facing semantic overclaim remains.
- no new external tool dependency is introduced.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index -- --stats`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`

Commit:
Decide semantic fact product status
