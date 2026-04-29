# PLAN_12A_EXTENDED_JVM_DOTNET_LANGUAGE_PACK.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_11A, PLAN_11B, and PLAN_11C are complete and green.

Goal:
Add C#, Scala, and Kotlin support through the existing generic Tree-sitter framework.

This pass expands the language pack for JVM/.NET-style languages. Do not add a second parser architecture. Do not add hand parsers, line scanners, release packaging, license enforcement, payment behavior, telemetry, cloud behavior, or new product commands.

Product rule:
Adding a language must be routine: grammar registration, extension mapping, query specs, conformance fixture, license entry, docs entry.

Languages:
- C#
- Scala
- Kotlin

Hard requirements:
- Use the existing Tree-sitter extraction framework.
- Do not create a second parser architecture.
- Do not use line-oriented or regex-based code-symbol parsing.
- Do not call or reintroduce ctags.
- Do not add GPL or AGPL dependencies.
- Every grammar/dependency must have a license entry.
- Every supported language must have a conformance fixture.
- No newly built index may emit `source = "ctags"`.

Allowed language-specific work:
- grammar registration
- file extension mapping
- Tree-sitter query specs
- conformance fixture
- license/audit entry
- tiny grammar adapter only if unavoidable and documented

Forbidden language-specific work:
- hand-written parser
- line scanner
- copy/pasted extraction loop
- broad regex parser
- ctags fallback

Per-language artifacts:
For each supported language add:
- grammar registration
- extension mapping
- Tree-sitter query spec
- conformance fixture
- license/audit entry
- support matrix entry
- expected record-kind coverage note

Record kinds:
Where syntax supports them, extract:
- function
- method
- class
- struct
- enum
- interface
- trait
- type
- namespace/package/module
- variable
- constant
- import
- export

Tests:
- every newly supported language has at least one fixture in the shared conformance suite
- multiline declarations are tested where syntax supports them
- comments/strings do not create fake symbols
- line/col are 1-based and accurate
- no duplicate path+line+col records
- no `source = "ctags"`
- representative `wi` commands work for C#, Scala, and Kotlin
- existing representative pack tests still pass
- existing refs/pack/impact/stats tests still pass

Docs:
Update parser support matrix and THIRD_PARTY_NOTICES or equivalent for C#, Scala, and Kotlin.

Docs must state:
- which languages are supported
- which languages are Tree-sitter-backed
- which languages are unsupported/deferred and why
- Tree-sitter is not semantic/LSP-level analysis

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth.
- Keep AGENTS.md and existing CLAUDE.md generation aligned with the canonical Repository search block.

Acceptance:
- C#, Scala, and Kotlin are implemented through the existing generic framework, or explicitly blocked with license/integration reasons
- no second parser architecture is introduced
- no ctags or line-scanner parser backend is reintroduced
- no GPL/AGPL dependency is introduced
- support matrix and license notices are updated
- existing product behavior remains stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- representative `wi` commands for C#, Scala, and Kotlin where supported
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- supported language matrix
- unsupported/deferred language blockers
- grammar dependencies and licenses
- representative smoke outputs
- known extraction gaps
- verification commands and results
- ignored local/real repo test status
- commit hash
