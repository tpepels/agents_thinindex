# PLAN_39_ONBOARDING_DOCTOR_AND_PRODUCT_POLISH.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_43_PRO_LICENSING_FOUNDATION_NO_ENFORCEMENT.md is complete and green.

Goal:
Add user-facing onboarding and product polish so thinindex is easier to install, initialize, diagnose, and use.

Product rule:
The product should explain what to do next when something is missing, stale, unsupported, or misconfigured.

Required:
- Add or improve `wi doctor` if not already present.
- Add first-run guidance.
- Improve error messages.
- Improve missing index/stale index messages.
- Improve unsupported language messages.
- Improve parser support explanation.
- Improve `wi --help` and examples.
- Add docs/tutorial quickstart.
- Add terminal examples for agents.

Doctor checks:
- index exists
- schema current
- parser support matrix present
- AGENTS.md present/current
- existing CLAUDE.md current if present
- `.dev_index` ignored
- quality plugin optional state
- license status if implemented
- package/install status if relevant

Tests:
- doctor passes on good fixture repo
- doctor reports missing index
- doctor reports stale index
- doctor reports stale AGENTS/CLAUDE block
- help text has current commands
- no WI.md reintroduced

Docs:
Update:
- README quickstart
- docs/GETTING_STARTED.md
- docs/TROUBLESHOOTING.md

Acceptance:
- onboarding is clear
- doctor/status command exists or is improved
- common failures have actionable messages
- docs match current behavior

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin wi -- --help`
- `cargo run --bin wi -- doctor` if implemented
- `cargo run --bin build_index`

Report:
- changed files
- doctor checks
- onboarding docs
- error message improvements
- verification results
- commit hash
