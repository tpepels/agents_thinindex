# PLAN_45_TREE_SITTER_REAL_REPO_CONVERGENCE_LOOP.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_44_ONBOARDING_DOCTOR_AND_PRODUCT_POLISH.md is complete and green.

Goal:
Continuously improve Tree-sitter parser/query quality until every configured `test_repos/` repository passes declared parser, symbol, dependency, reference, pack, and impact quality gates.

This is a focused Tree-sitter quality-convergence plan. It is not packaging, licensing, product polish, telemetry, cloud, or hosted functionality.

Product rule:
A supported language is only credible if it survives real repositories and captures the important symbols those repositories contain.

Scope:
Improve the existing Tree-sitter framework, query specs, conformance fixtures, expected-symbol manifests, and quality reports.

Do not add:
- a second parser architecture
- hand-written parsers
- line scanners for code symbols
- ctags as parser backend
- ctags as required comparator
- release packaging
- license enforcement
- payment behavior
- telemetry
- cloud behavior
- hosted features

Hard requirements:
- Use the existing Tree-sitter framework.
- Keep ctags isolated as optional quality comparator only, if present.
- Do not require ctags.
- Do not emit `source = "ctags"` in production records.
- Do not weaken existing conformance, quality, dependency, refs, pack, or impact gates.
- Do not promote experimental/blocked languages to supported without evidence.
- Every fixed parser miss should get a fixture or expected-symbol regression case where practical.

Convergence target:
All configured `test_repos/` must pass for declared supported languages.

Passing means:
- no parser panics
- no malformed records
- no duplicate `path + line + col` records
- no production `source = "ctags"` records
- no `.dev_index/` indexing
- expected symbols pass
- expected symbol patterns pass
- expected absent symbols stay absent
- supported-language files emit useful records
- dependency graph checks pass where configured
- refs checks pass where configured
- pack smoke checks pass
- impact smoke checks pass
- unsupported/deferred syntax is documented and not falsely claimed supported

Check → Plan → Act loop:
Run exactly one bounded convergence cycle per implementation run.

Check:
- run normal tests
- run local ignored test
- run real-repo ignored test
- run quality report
- run expected-symbol checks
- run dependency/ref/pack/impact smoke checks
- run optional comparator report only if configured and available

Plan:
- write/update `.dev_index/quality/TREE_SITTER_GAPS.md`
- group gaps by language, repo, syntax construct, and severity
- select at most 10 fixable gaps
- prefer supported-language missed symbols and parser panics over comparator-only noise
- do not mix unrelated architecture changes

Act:
- update Tree-sitter query specs, capture mapping, fixtures, expected-symbol manifests, or docs
- add regression fixture for each fixed parser miss where practical
- rerun full verification
- commit once green

Stop after one cycle:
Do not automatically start another cycle. Report remaining gaps and whether they are:
- fixed
- still open
- unsupported syntax
- blocked by grammar limitation
- blocked by license/integration issue
- comparator false positive
- low-value noise

Real-repo manifest:
Use `test_repos/MANIFEST.toml` when present.

Support:
- expected symbols
- expected symbol patterns
- expected absent symbols
- thresholds
- skip reasons
- unsupported syntax notes
- repo category/language metadata

Failure policy:
Fail the convergence run if:
- any declared supported language fails expected-symbol checks
- any declared supported language has real files but zero useful records without documented reason
- parser panics on supported-language files
- malformed records/refs/dependency edges are emitted
- duplicate record locations occur
- dependency/ref/pack/impact checks fail
- production records contain `source = "ctags"`
- ctags appears outside allowed quality-comparator locations

Allowed non-failures:
- missing `test_repos/`, but report skipped clearly
- unsupported language files, if documented
- generated/vendor/minified files, if ignored or documented
- grammar limitation explicitly marked blocked
- comparator-only symbols not yet triaged

Docs:
Update docs only when behavior/support status changes:
- parser support matrix
- known syntax gaps
- quality docs
- real-repo manifest docs
- troubleshooting docs

Acceptance:
- one convergence cycle is completed
- selected gaps are fixed or explicitly reclassified
- fixture/manifest regression coverage is added for fixed misses where practical
- all declared supported languages in configured `test_repos/` pass quality gates, or remaining blockers are documented accurately
- no parser architecture drift is introduced
- no ctags runtime/build/package dependency is introduced
- existing index/search/refs/pack/impact/quality behavior remains stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- ctags allowlist gate
- license audit command if configured
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- quality report/gate command if available
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- repos checked
- languages checked
- gaps selected
- gaps fixed
- regression fixtures added
- expected symbols/patterns added or updated
- remaining unsupported/blocked gaps
- verification commands and results
- commit hash
