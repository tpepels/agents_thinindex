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

Universal Ctags has been removed from the active parser path. It is not bundled, detected, or called by thinindex.

Tree-sitter parser dependencies and grammar dependencies must be permissively licensed and audited before release packaging.

The current Tree-sitter-backed language pack covers Rust, Python, JavaScript, JSX, TypeScript, TSX, Java, C#, Scala, Kotlin, Go, C, C++, Shell, Ruby, and PHP. Additional language support must be added through the same registry, grammar, query, fixture, and notice path rather than a second parser architecture.

Before packaging work proceeds, thinindex also needs:

- `THIRD_PARTY_NOTICES`
- full dependency license audit coverage
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
