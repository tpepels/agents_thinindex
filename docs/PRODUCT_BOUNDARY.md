# Product Boundary

Thinindex is currently a local/free agent-navigation tool. This document defines the boundary for possible future paid work without adding license enforcement, payment integration, network behavior, telemetry, feature lockouts, release installers, or parser feature gates.

## Product principle

Free thinindex must remain a useful local agent-navigation tool. Paid value should come from proof, hardening, integrations, packaging convenience, and advanced workflows, not from making basic local navigation worse.

Do not charge for basic local search or basic repo indexing.

## Free/local edition

The current free/local core includes:

- `wi-init`
- `build_index`
- local indexing
- local `.dev_index/index.sqlite` storage
- `wi <term>`
- basic filters and ranking
- `wi --help`
- `AGENTS.md` setup and existing `CLAUDE.md` normalization
- ability to remove `.dev_index`
- local cache rebuilds
- basic `wi-stats`
- index integrity checks
- no-network local operation

No account, license key, payment, cloud service, telemetry, or remote indexing is required for this local workflow.

## Licensing foundation

An inert local license state model exists for future product work. It supports
`free`, `pro`, and `unknown/unlicensed` status values, a local license file path
design, and a fixture-only validation stub. It does not add payment calls,
network activation, telemetry, account behavior, or feature lockouts.

There are no paid gates active. Missing, invalid, unreadable, unsupported, or
non-fixture license data must not block the current workflow. The free local core remains available without a license file.

The current local path design is documented in [LICENSING.md](LICENSING.md):
`THININDEX_LICENSE_FILE`, then the platform user config directory. The only
accepted Pro status today is an explicit `local-test-fixture` fixture for tests.

## Candidate Pro features

Possible future Pro value should be treated as candidates, not implemented gates:

- advanced context packs beyond the current local `wi pack` output
- advanced impact analysis beyond the current conservative `wi impact` output
- benchmark reports for audits or team review
- CI/reporting integrations
- curated real-repo benchmark reports
- team/shared policy packs
- richer agent integration packs
- exported agent-readiness reports
- advanced reference graph quality features
- signed installers and a managed update channel after parser/licensing blockers are removed
- release-quality Tree-sitter parser coverage if it materially improves bundled parser quality

These are roadmap candidates. They are not active feature gates in the current tool.

## Candidate team/CI and hosted value

Future team and CI value is documented in [TEAM_CI_ROADMAP.md](TEAM_CI_ROADMAP.md)
and [CI_INTEGRATION.md](CI_INTEGRATION.md). It is roadmap-only today.

Candidate value may include local CI summaries, agent-readiness reports,
redacted support bundles, team policy packs, benchmark trend reports, hosted
report viewing, signed installers, and managed update channels.

The required privacy boundary is no-source-upload mode by default. Local
commands must continue to work without accounts, payment integration, network
activation, telemetry, cloud sync, hosted APIs, remote indexing, or feature
lockouts.

Hosted reports, if implemented by a future plan, must accept explicit
user-provided report artifacts. They must not require uploading repository
source, `.dev_index/`, `test_repos/`, raw quality detail dumps, or unredacted
secrets.

## What must never be paywalled

The following must remain part of the local/free core:

- local indexing
- `wi <term>`
- basic filters
- `wi --help`
- `wi-init`
- `AGENTS.md` setup and existing `CLAUDE.md` normalization
- ability to remove `.dev_index`
- local cache rebuilds
- no-network local operation

## Packaging/licensing blockers

No external parser command is required for production indexing. Universal Ctags is optional, external, not bundled, not required, and not used by production indexing.

Tree-sitter parser dependencies and grammar dependencies must be permissively licensed and audited before release packaging.

The repeatable dependency audit command is `cargo deny check licenses`. Only explicitly allowed permissive licenses are accepted. GPL, AGPL, LGPL-only, MPL-only, EPL, CDDL, unknown, no-license, custom, and non-commercial dependency terms block proprietary packaging unless a future plan adds a documented review exception.

Parser claims use explicit support levels: `supported`, `experimental`, `blocked`, and `extras-backed`. The source of truth is `src/support.rs`, mirrored in `README.md` and `docs/PARSER_SUPPORT.md`.

Current supported Tree-sitter code-symbol extraction covers Rust, Python, JavaScript, JSX, TypeScript, TSX, Java, Go, C, C#, C++, Shell, Ruby, and PHP. Scala, Kotlin, Swift, Dart, and Nix are experimental until real-repo coverage and documented gaps are stronger. Additional language support must be added through the same registry, grammar, query, fixture, and notice path rather than a second parser architecture.

CSS, HTML, Markdown, JSON, TOML, and YAML are extras-backed by project-owned extractors, not Tree-sitter-backed code-symbol parsers. Config extraction records useful keys, tables, and sections without treating every scalar value as a symbol.

Blocked entries include Vue/Svelte single-file components, Objective-C/Objective-C++, SQL, XML, Lua, Haskell, and Elixir. Formats and languages not listed in the README support matrix are unsupported. They must not be claimed through line scanning, ctags fallback, or undocumented parser dependencies.

Real-repo parser coverage is checked with shared integrity rules and optional manifest expected-symbol entries. These targeted checks are preferred over exact total record counts because generated code, comments, and unsupported syntax can change totals without changing navigation quality.

Parser performance gates are local and report-oriented: normal tests cover deterministic fixture regressions, while ignored real-repo tests surface slow files, noisy files, large files, parse errors, and unsupported extensions for local hardening.

Before packaging work proceeds, thinindex also needs:

- passing `cargo deny check licenses` output for the committed `Cargo.lock`
- `THIRD_PARTY_NOTICES` included with release artifacts
- release documentation that matches the audited parser dependency set
- installer and archive smoke tests on target platforms

## What is not being built yet

This boundary does not add:

- license enforcement
- payments
- account login
- cloud sync
- telemetry
- remote indexing
- feature lockouts
- network calls
- release installers
- parser paywalls

## Licensing/payment deferred decisions

Pricing, license provider, activation flow, local license cache shape, paid update policy, and team licensing are deferred decisions.

Do not encode pricing or edition limits in code until the product has evidence that a paid edition is useful and the parser/package blocker has been resolved.

## Evidence needed before charging

Charging for a Pro edition requires evidence such as:

- benchmark output showing reduced discovery waste
- real agent tasks showing fewer broad reads and grep calls
- improved read-set precision from `wi pack`
- useful affected-file recall from `wi impact`
- real-repo benchmark stability
- user workflow examples that show repeatable value

## Open questions

- Which advanced context workflows are valuable enough to pay for?
- Which benchmark reports are useful to supervising developers?
- What evidence proves fewer irrelevant reads in real agent tasks?
- Which Tree-sitter grammar and query coverage gives enough quality without licensing risk?
- What packaging channel is appropriate after parser and license audits are complete?
