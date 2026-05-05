# PLAN_54_RELEASE_CANDIDATE_READINESS_CHECKLIST.md

Use superpowers:subagent-driven-development.

Do not implement this until `prompts/recovery/RECOVERY_STATUS.md` says recovery
is complete enough for a release-candidate decision and commit `5b1899b Align
scorecard install docs` has fixed the `wi-scorecard` install/docs blocker.

Goal:
Decide whether thinindex is ready for a release candidate, and fix only
release-blocking inconsistencies found during the checklist.

Scope:
- Audit release-readiness surfaces.
- Clean up only release-blocking mismatches.
- Do not add product features, parser architecture, language support, MCP,
  cloud behavior, telemetry, payment behavior, or license enforcement.

Required checks:
- [x] Installed PATH binaries and source binaries agree on version and schema.
- [x] Release archive includes required binaries: `wi`, `build_index`,
      `wi-init`, `wi-stats`, and `wi-scorecard`.
- [x] Release archive includes required docs, notices, SBOM, checksum, and
      install/uninstall helpers.
- [x] Release archive excludes `.dev_index`, `test_repos`, local quality
      reports, secrets, generated local state, and source/build artifacts.
- [x] Installed `wi doctor` reports ok in this checkout.
- [x] No-change `build_index --stats` is fast enough.
- [x] Installed `wi <query>` self-heals missing and stale indexes in real CLI
      temp-repo runs.
- [x] Installed `wi-scorecard` runs successfully.
- [x] README, GETTING_STARTED, INSTALLERS, RELEASING, RELEASE_CHECKLIST, and
      SCORECARD docs agree with actual installed/archive binaries.
- [x] Product/support claims are not overbroad for RC.
- [x] Universal Ctags is not bundled or required for production indexing.
- [x] `WI.md` is not reintroduced.
- [x] JSONL is not canonical storage.
- [x] `THIRD_PARTY_NOTICES`, generated `SBOM.md`, and release metadata are
      current enough for RC.
- [x] Full verification is green.
- [x] RC status is classified as `ready_for_rc`, `ready_with_caveats`, or
      `blocked`.
- [x] Commit exists.

Verification commands:
- `which wi`
- `which build_index`
- `which wi-scorecard`
- `wi --version`
- `build_index --version`
- `wi-scorecard --version`
- `wi doctor`
- `build_index --stats`
- immediate second `build_index --stats`
- `wi build_index`
- `wi refs build_index`
- `wi pack build_index`
- `wi impact build_index`
- `wi-scorecard`
- `./install.sh`
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `scripts/check-release`
- `scripts/smoke-release-archive <current archive>`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` when `test_repos/` exists

Execution notes:
- Subagent audit found one RC-blocking inconsistency: README still said release
  archives include "all four binaries" after `wi-scorecard` became packaged.
  Fixed README to say archives include all five user-facing binaries and
  scorecard docs.
- `./install.sh` installed `build_index`, `wi`, `wi-init`, `wi-stats`, and
  `wi-scorecard`, and each reported `0.1.4 (index schema 12)`.
- PATH binaries: `/home/tom/.local/bin/wi`,
  `/home/tom/.local/bin/build_index`, and
  `/home/tom/.local/bin/wi-scorecard`.
- PATH and source versions agree for `wi`, `build_index`, and `wi-scorecard`:
  `0.1.4 (index schema 12)`.
- Installed `wi doctor` initially reported the newly added plan file made the
  index stale; installed `wi-scorecard` self-healed it and then `wi doctor`
  reported `overall: ok`.
- `build_index --stats` no-change runs reported `total ms: 12` twice.
- Installed `wi` self-healed both missing and stale temp-repo indexes and
  continued the original query.
- Installed `wi build_index`, `wi refs build_index`, `wi pack build_index`, and
  `wi impact build_index` all returned useful evidence-backed output.
- Installed `wi-scorecard` passed with `pass 10 / warn 0 / fail 0`.
- `scripts/check-release` generated
  `dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz`, validated package
  contents/SBOM/checksum, and smoke-tested the packaged archive.
- Standalone `scripts/smoke-release-archive
  dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz` passed.
- Archive listing confirmed the five required binaries, `README.md`,
  `INSTALL.md`, `SBOM.md`, `THIRD_PARTY_NOTICES`, `docs/RELEASING.md`,
  `docs/INSTALLERS.md`, and `docs/SCORECARD.md`; no forbidden payload matches
  appeared in the inspected listing.
- Product/support claim audit found no RC-blocking overclaims. Support levels
  remain explicit; Universal Ctags remains optional/external/not bundled/not
  required; `WI.md` is not generated; JSONL references are legacy migration or
  quality-export details, not canonical storage.
- Verification passed: `cargo fmt --check`, `cargo test`, `cargo clippy
  --all-targets --all-features -- -D warnings`, `scripts/check-release`,
  standalone archive smoke, `cargo test --test local_index -- --ignored`, and
  `cargo test --test real_repos -- --ignored`.

RC decision:
- `ready_with_caveats`.
- Caveat: archive smoke was performed for the current
  `x86_64-unknown-linux-gnu` archive; other target archives still need their
  own target-platform smoke before publishing them.
- Caveat: native installers, signing, notarization, package-manager publishing,
  hosted services, telemetry, and payment/licensing enforcement remain
  intentionally unimplemented and documented as scaffolded/future work. This
  does not block the local archive RC.
- Caveat: optional local `test_repos/` coverage is checkout-local; it passed in
  this environment, but third-party repos remain uncommitted by design.
- Next RC step: run the release checklist target-platform smoke for any archive
  target being published, then tag/package the RC from a clean worktree.

Commit:
Assess release candidate readiness
