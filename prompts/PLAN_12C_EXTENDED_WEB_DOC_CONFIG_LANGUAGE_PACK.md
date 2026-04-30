# PLAN_12C_EXTENDED_WEB_DOC_CONFIG_LANGUAGE_PACK.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_12B is complete and green.

Goal:
Add or harden CSS, HTML, Markdown, JSON, TOML, and YAML support through the existing generic parser/extras framework.

This pass covers web/document/config formats. Do not add a second parser architecture. Do not add hand parsers for code languages, release packaging, license enforcement, payment behavior, telemetry, cloud behavior, or new product commands.

Product rule:
Web/document/config support should be useful and non-noisy. Do not index every scalar value as a symbol.

Languages/formats:
- CSS
- HTML
- Markdown
- JSON
- TOML
- YAML

Hard requirements:
- Use the existing Tree-sitter extraction framework where Tree-sitter support is chosen.
- Existing project-owned extras may remain for CSS/HTML/Markdown if they are more useful and tested.
- Be explicit in docs whether a format is Tree-sitter-backed or extras-backed.
- Do not create a second parser architecture.
- Do not call or reintroduce ctags.
- Do not add GPL or AGPL dependencies.
- Every grammar/dependency must have a license entry.
- Every supported format must have a conformance fixture.
- No newly built index may emit `source = "ctags"`.

Config/data record policy:
For JSON, TOML, and YAML, define useful, non-noisy records.

Preferred record kinds:
- key
- section/table where applicable
- anchor/reference only if useful and supported

Do not emit every scalar value as a symbol.

CSS:
Keep or add:
- css_class
- css_id
- css_variable
- keyframes

HTML:
Keep or add:
- html_tag
- html_id
- html_class
- data_attribute

Markdown:
Keep or add:
- section
- checklist
- link
- todo/fixme if already covered by extras

Tests:
- every newly supported format has at least one fixture in the shared conformance suite or accepted extras suite
- comments/strings do not create fake code symbols
- line/col are 1-based and accurate
- no duplicate path+line+col records
- no `source = "ctags"`
- representative `wi` commands work for CSS, HTML, Markdown, JSON, TOML, and YAML where supported
- existing representative pack tests still pass
- existing refs/pack/impact/stats tests still pass

Docs:
Update parser/support matrix and THIRD_PARTY_NOTICES or equivalent.

Docs must state:
- which formats are Tree-sitter-backed
- which formats are extras-backed
- which formats are unsupported/deferred and why
- Tree-sitter is not semantic/LSP-level analysis

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth.
- Keep AGENTS.md and existing CLAUDE.md generation aligned with the canonical Repository search block.

Acceptance:
- CSS, HTML, Markdown, JSON, TOML, and YAML support is implemented or explicitly blocked with license/integration reasons
- support is useful and non-noisy
- no second parser architecture is introduced
- no ctags or line-scanner code parser backend is reintroduced
- no GPL/AGPL dependency is introduced
- support matrix and license notices are updated
- existing product behavior remains stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- representative `wi` commands for CSS, HTML, Markdown, JSON, TOML, and YAML where supported
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- supported format matrix
- Tree-sitter-backed vs extras-backed formats
- unsupported/deferred blockers
- grammar dependencies and licenses
- representative smoke outputs
- known extraction gaps
- verification commands and results
- ignored local/real repo test status
- commit hash

## Phase tracking

- [x] Add non-noisy JSON, TOML, and YAML extraction through project-owned extras.
- [x] Keep CSS, HTML, and Markdown on the existing extras path with fixture coverage.
- [x] Add config fixtures and tests for key/table/section behavior and scalar non-indexing.
- [x] Update support matrix, known gaps, help text, and third-party notices.
- [x] Run required 12C verification.
- [x] Commit with `Add extended web doc and config parser pack`.
