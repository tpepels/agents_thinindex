# PLAN_40_TEAM_CI_AND_HOSTED_VALUE_ROADMAP.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_44_ONBOARDING_DOCTOR_AND_PRODUCT_POLISH.md is complete and green.

Goal:
Define and scaffold the next product layer: team/CI value, hosted reports, and enterprise-ready workflows.

This is mostly product architecture/docs with light scaffolding only. Do not build cloud service behavior unless explicitly scoped and safe.

Product rule:
Paid/team value should come from proof, reports, integrations, support, and workflow reliability, not by crippling the free local core.

Required docs:
Add or update:
- docs/TEAM_CI_ROADMAP.md
- docs/PRODUCT_BOUNDARY.md
- docs/CI_INTEGRATION.md if useful

Define:
- local free/core guarantees
- Pro candidates
- team/CI candidates
- hosted report candidates
- privacy constraints
- no-source-upload mode
- artifact/report formats
- support/update channel model
- what remains explicitly out of scope

Possible scaffolding:
- CI report command shape
- JSON report schema
- local artifact format
- GitHub Actions example
- no hosted backend yet

Do not add:
- accounts
- payment integration
- license server
- source upload
- telemetry
- cloud sync
- hosted API

Acceptance:
- next product layer is clearly planned
- free/core boundary remains protected
- team/CI value is defined
- privacy constraints are documented
- no premature cloud/payment behavior is added

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- docs/governance tests if present

Report:
- changed files
- team/CI roadmap summary
- free/pro boundary updates
- verification results
- commit hash
