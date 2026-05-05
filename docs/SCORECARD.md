# Product Value Scorecard

`wi-scorecard` prints a compact local report for the core thinindex product
loop. It is intended for humans and agents deciding whether the repository
search workflow is actually useful before expanding scope.

Run it inside a repository:

```bash
wi-scorecard --query build_index
```

Use a query that should have useful definitions, references, tests, docs, or
dependents in the current repository. For application code, a service, command,
component, or public function is usually better than an entry point.

## Status Meaning

- `pass`: the dimension has direct local evidence.
- `warn`: the dimension ran, but the evidence is partial or query-dependent.
- `fail`: the dimension lacks useful evidence and includes an action.

No product claim should be promoted from a warning or failure. Fix the listed
action or choose a better evidence query before using the scorecard as release
or roadmap evidence.

## Dimensions

The scorecard covers:

- `wi <term>` useful file:line results;
- missing/stale index recovery;
- warm query latency;
- `wi refs <term>` reference evidence;
- `wi pack <term>` bounded read-set evidence;
- `wi impact <term>` affected-file evidence with reasons;
- `wi doctor` setup state;
- `wi-init` agent instruction surfaces;
- generated instruction wording alignment;
- parser support claim boundaries.

The command is local-only. It does not upload source, call a hosted service,
enable telemetry, or require `test_repos/` for normal runs.

## Interpreting Results

A good scorecard for a release candidate has no `fail` rows. Warnings are
acceptable only when the evidence explains a query limitation, such as an entry
point with little reference evidence. If `wi pack` or `wi impact` warns, rerun
with a symbol that has callers, tests, docs, or dependencies before drawing
product conclusions.

For real-repo hardening, run the ignored scorecard test when local
`test_repos/` exists:

```bash
cargo test --test scorecard -- --ignored
```

That ignored test is optional local evidence. Normal tests do not require
third-party repositories.
