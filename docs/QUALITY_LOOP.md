# Quality Loop

The quality loop is a bounded Check -> Plan -> Act workflow for improving parser and index quality from evidence. It lives in the quality plugin layer and writes local reports under `.dev_index/quality/`.

It is not a production indexing path. It does not write comparator output to production SQLite `records` or `refs`, and it does not add install, release, network, telemetry, payment, or license-enforcement behavior.

One execution is exactly one quality cycle. A cycle run may check, write one plan, help drive one bounded fix batch, write one final report, and then stop. Agents must not start a second cycle automatically from the same execution.

## Check

Run the normal deterministic checks:

```sh
cargo test
```

When local real repositories are available, run the ignored checks:

```sh
cargo test --test quality_gates -- --ignored
cargo test --test quality_loop -- --ignored
```

The check phase collects:

- missing expected symbols
- failing expected symbol patterns
- found expected-absent symbols
- parser panics or integrity failures surfaced by tests
- supported-language files that emit zero useful records without a documented reason
- comparator-only and thinindex-only symbols when the optional comparator is available
- noisiest or slowest files when performance data exists

Universal Ctags remains optional, external, not bundled, not required, and not used by `build_index`.

## Plan

The loop writes:

- `.dev_index/quality/QUALITY_GAPS.md`
- `.dev_index/quality/COMPARATOR_TRIAGE.md`
- `.dev_index/quality/QUALITY_CYCLE_01_PLAN.md`
- `.dev_index/quality/QUALITY_CYCLE_01_REPORT.md`

Gaps are grouped by language, syntax construct, severity, and evidence source. Every gap includes:

- gap id
- repo/path
- language
- symbol/kind/pattern
- evidence source
- severity
- suggested fix type
- status: open, fixed, unsupported, or false-positive
- fixture added: yes/no
- manifest added: yes/no

Comparator triage groups comparator-only and thinindex-only symbols by language, kind, and path. Use exactly these states: `open`, `accepted_expected_symbol`, `fixture_needed`, `comparator_false_positive`, `unsupported_syntax`, `low_value_noise`, and `fixed`.

Cycle plans are bounded to one pass and at most 10 gaps by default. Prefer supported-language missing symbols and failing expected patterns over comparator-only noise.

The cycle runner records `cycles_executed = 1` and `automatic_next_cycle_allowed = false`. Raising the requested gap limit above 10 is capped back to 10.

## Act

For the selected batch:

1. Reproduce each gap with the narrowest check.
2. Implement only the selected parser/query/fixture/manifest fixes.
3. Add a conformance fixture for each fixed parser miss where practical.
4. Add a manifest expected symbol for important real-repo misses where practical.
5. Rerun normal and applicable ignored quality gates.
6. Mark remaining comparator-only findings with one of the explicit triage states.
7. Write `.dev_index/quality/QUALITY_CYCLE_01_REPORT.md`.
8. Stop after this cycle and commit the bounded fix batch.

Do not automatically start a second cycle in the same execution.

## Comparator Findings

Comparator-only symbols are not ground truth. Triage each one as:

- `accepted_expected_symbol`: add or update a manifest expected symbol or expected symbol pattern
- `fixture_needed`: add or extend parser conformance before changing manifests
- `comparator_false_positive`: record as external comparator noise
- `unsupported_syntax`: document an unsupported parser gap
- `low_value_noise`: leave unpromoted

Do not add a parser rule solely because an optional comparator produced a symbol.

## Stop Criteria

Stop the cycle when the selected batch is fixed and verified, or when the remaining gaps are documented unsupported cases, accepted comparator false positives, or low-value noise.

The final report uses these stop conditions:

- `no_selected_gaps`
- `selected_gaps_fixed`
- `remaining_gaps_unsupported`
- `remaining_gaps_comparator_false_positive`
- `remaining_gaps_require_architecture_or_language_expansion`
- `verification_failed_needs_human_review`

`verification_failed_needs_human_review` is terminal for the current execution. Fix the underlying failure before any future manually started cycle.

Measurable closure criteria:

- no missing manifest expected symbols
- no failing expected symbol patterns
- no found expected-absent symbols
- no duplicate locations
- no malformed records or refs
- no parser panics
- no supported language emits zero useful records on real files without a documented reason
- comparator-only symbols are triaged
- every parser fix has a regression fixture where practical
- quality report improves or remains stable between runs

## One-Cycle Test Workflow

The ignored quality-loop test is the documented runner workflow for local real repos:

```sh
cargo test --test quality_loop -- --ignored
```

It rebuilds each configured repo under `test_repos/`, runs the normal gate plus optional comparator data, writes the gap report, writes one bounded plan, writes comparator triage when available, writes the final report, and stops. If `test_repos/` is missing or empty, the workflow prints a skip message.
