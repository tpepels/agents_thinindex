# PLAN_35_PRO_LICENSING_FOUNDATION_NO_ENFORCEMENT.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_42_SIGNED_INSTALLER_AND_DISTRIBUTION_HARDENING.md is complete and green.

Goal:
Design and implement the local licensing foundation without enforcing paid gates yet.

Product rule:
Do not make the free/local core worse. This plan prepares licensing structure only.

Hard requirements:
- Do not add payment calls.
- Do not add network activation.
- Do not block current features.
- Do not add telemetry.
- Do not add cloud account behavior.
- Do not hide source/license obligations.
- Keep free/local core usable.

Required:
- Add license state model.
- Add local license file path design.
- Add edition model:
  - free
  - pro
  - unknown/unlicensed
- Add command or internal status display if useful.
- Add validation stub that accepts only explicit local test fixtures.
- Add docs for future activation flow.
- Add tests around local license parsing and status.
- No feature gates yet unless plan explicitly marks them inert/test-only.

Docs:
Update:
- docs/PRODUCT_BOUNDARY.md
- docs/LICENSING.md
- README if needed

Document:
- paid features are not enforced yet
- no license server yet
- no payment integration yet
- free local core remains available

Acceptance:
- licensing model exists
- no paid gates block users
- local license status can be read/tested
- docs are honest about deferred enforcement

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin wi -- --version`
- `cargo run --bin wi -- --help`

Report:
- changed files
- license model
- edition status behavior
- docs updated
- verification results
- commit hash
