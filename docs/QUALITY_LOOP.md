# Quality Loop

The quality loop is a bounded Check -> Plan -> Act workflow for improving parser and index quality from evidence. It lives in the quality plugin layer and writes local reports under `.dev_index/quality/`.

It is not a production indexing path. It does not write comparator output to production SQLite `records` or `refs`, and it does not add install, release, network, telemetry, payment, or license-enforcement behavior.

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
- `.dev_index/quality/QUALITY_CYCLE_01_PLAN.md`

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

Cycle plans are bounded to one pass and at most 10 gaps by default. Prefer supported-language missing symbols and failing expected patterns over comparator-only noise.

## Act

For the selected batch:

1. Reproduce each gap with the narrowest check.
2. Implement only the selected parser/query/fixture/manifest fixes.
3. Add a conformance fixture for each fixed parser miss where practical.
4. Add a manifest expected symbol for important real-repo misses where practical.
5. Rerun normal and applicable ignored quality gates.
6. Mark remaining comparator-only findings as open, unsupported, or false-positive.
7. Stop after this cycle and commit the bounded fix batch.

Do not automatically start a second cycle in the same execution.

## Comparator Findings

Comparator-only symbols are not ground truth. Triage each one as:

- expected-symbol addition
- fixture or conformance addition
- accepted comparator false positive
- documented unsupported syntax

Do not add a parser rule solely because an optional comparator produced a symbol.

## Stop Criteria

Stop the cycle when the selected batch is fixed and verified, or when the remaining gaps are documented unsupported cases, accepted comparator false positives, or low-value noise.

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
