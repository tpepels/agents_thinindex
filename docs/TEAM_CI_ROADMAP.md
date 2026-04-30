# Team CI And Hosted Value Roadmap

thinindex is currently a local-first agent-navigation tool. This roadmap defines
future team, CI, and hosted-report value without adding accounts, payment
integration, license enforcement, source upload, telemetry, cloud sync, hosted
APIs, or remote indexing.

## Product Principle

Paid or team value must come from proof, reports, integrations, support, update
reliability, and workflow hardening. It must not come from making the free local
core worse.

The free local core remains:

- `wi-init`
- `build_index`
- local `.dev_index/index.sqlite`
- `wi <term>`
- basic filters and ranking
- `wi --help`
- `wi doctor`
- `wi refs`, `wi pack`, and `wi impact`
- `wi-stats`
- local cache rebuilds
- no-network local operation

## Candidate Team/CI Value

Candidate value for future plans:

- checked CI summaries for index freshness, parser support, and doctor status;
- local artifact uploads from CI jobs for human review;
- trend reports for benchmark counts, command latency, and index size;
- team policy packs that document expected `wi` workflow usage;
- exported agent-readiness reports for repository onboarding;
- curated real-repo quality dashboards for supported languages;
- support bundles that redact sensitive values and avoid source dumps;
- signed installers and managed update channels once signing is complete;
- documented enterprise onboarding playbooks.

These are candidates, not current feature gates.

## Hosted Report Candidates

Hosted value is limited to future report viewing and team workflow review. A
hosted product must be able to operate in a no-source-upload mode.

Potential hosted artifacts:

- CI summary JSON generated locally;
- quality dashboard JSON generated locally;
- benchmark summary JSON generated locally;
- doctor/status summary generated locally;
- install/version/support metadata generated locally.

Hosted reports must not require uploading repository source, `.dev_index`,
quality detail JSONL, raw command output with source snippets, local real-repo
contents, or sensitive paths.

## Privacy Constraints

Default team/CI and hosted workflows must be local-first:

- no source upload by default;
- no telemetry by default;
- no background daemon;
- no cloud sync;
- no hosted API dependency for local commands;
- no network activation;
- no remote indexing;
- no automatic upload of `.dev_index/`, `.dev_index/quality/`, or `test_repos/`;
- redaction remains enabled for report-like output.

Any future upload-capable workflow must be opt-in, documented, and separable
from the free local command behavior.

## Local Artifact Shape

A future CI report artifact should be generated locally and may use this compact
JSON shape:

```json
{
  "schema_version": 1,
  "tool": "thinindex",
  "repo_label": "example-repo",
  "generated_at": "2026-04-30T00:00:00Z",
  "source_upload": false,
  "commands": {
    "fmt": "passed",
    "test": "passed",
    "clippy": "passed",
    "doctor": "passed",
    "package_check": "passed"
  },
  "index": {
    "schema_current": true,
    "fresh": true,
    "records": 0,
    "refs": 0
  },
  "parser_support": {
    "supported": 0,
    "experimental": 0,
    "extras_backed": 0,
    "blocked": 0
  },
  "artifacts": [
    {
      "kind": "doctor",
      "path": ".dev_index/quality/TEAM_CI_SUMMARY.json",
      "local_only": true,
      "redacted": true
    }
  ]
}
```

This schema is documentation only. No `wi ci` or hosted command exists yet.

## GitHub Actions Example

A future CI job can stay local-only:

```yaml
jobs:
  thinindex-local-report:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy,rustfmt
      - run: cargo fmt --check
      - run: cargo test
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo run --bin build_index
      - run: cargo run --bin wi -- doctor
      - run: cargo run --bin wi -- bench
```

The example does not upload source, does not require `test_repos/`, and does not
contact a thinindex service.

## Support And Update Channel Model

Future support/update value may include:

- signed installers;
- deterministic release archives;
- support bundles with redaction and no source dumps;
- documented upgrade checks;
- curated compatibility notes for agent workflows;
- team rollout guides.

Support bundles must default to metadata, command versions, doctor output, and
redacted summaries. They must not include source files or local SQLite indexes by
default.

## Explicitly Out Of Scope

This roadmap does not implement:

- accounts;
- payment integration;
- license server behavior;
- network activation;
- telemetry;
- cloud sync;
- hosted API;
- source upload;
- remote indexing;
- feature lockouts;
- paid gates.

Any future plan that adds one of these must update product-boundary, privacy,
licensing, and release docs before implementation.
