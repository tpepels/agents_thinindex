# PLAN_02_DETERMINISTIC_REFERENCE_EXTRACTION.md

Use superpowers:subagent-driven-development.

Goal:
Populate the SQLite `refs` table with useful deterministic references from real source text.

This pass improves reference extraction quality. It does not add `wi pack`, `wi impact`, prediction commands, or new user-facing read plans.

Phase tracking:
- [x] Add named caps/stoplist and deterministic extraction rules.
- [x] Integrate extraction into `build_index` with deterministic SQLite refs.
- [x] Extend fixture tests for extraction, stale cleanup, determinism, caps, and stoplist behavior.
- [ ] Run required formatting, tests, lint, and ignored tests.
- [ ] Commit with `Add deterministic reference extraction`.

Prerequisite:
PLAN_00_SQLITE_INDEX_STORAGE.md and PLAN_01_REFERENCE_GRAPH_FOUNDATION.md must be complete and green. `.dev_index/index.sqlite` is the canonical storage file and contains a `refs` table.

Product rule:
Reference extraction must prefer precision over completeness. Do not create a noisy global substring graph.

Inputs:
Use:

- source files discovered by `build_index`
- indexed records stored in SQLite
- existing extras extractors where useful
- source text already read during indexing where practical

Reference output:
Write references to the SQLite `refs` table.

Do not reintroduce JSONL reference storage.

Extraction rules:

Code text references:
For indexed symbol names, detect references in source text only when:

- symbol length is at least 3
- match occurs at a word boundary
- match is not the defining record itself
- reference is not in ignored paths
- reference count per target is capped
- reference count per file is capped

Emit `ref_kind = "text_reference"`.

Imports:
Extract common imports.

Python:

- `import X`
- `from X import Y`

Rust:

- `use crate::...`
- `use super::...`
- `mod X`

TypeScript/JavaScript:

- `import ... from "..."`
- `export ... from "..."`

Emit `ref_kind = "import"`.

Markdown links:
Extract markdown links:

- `[label](target)`

Emit `ref_kind = "markdown_link"`.

HTML/JSX/CSS usage:
Use existing extras extraction where practical:

- CSS class names
- CSS variables
- HTML ids/classes/data attributes
- JSX className/data-testid

Emit:

- `ref_kind = "css_usage"`
- `ref_kind = "html_usage"`

Test references:
Classify a reference as `test_reference` if:

- path is under `tests/`, `test/`, or `__tests__/`
- or filename contains `_test`, `.test.`, or `.spec.`

If a reference is both a text/import reference and a test reference, prefer `test_reference` for test-path records unless this causes loss of useful import evidence. Keep behavior deterministic.

Ranking metadata:
Do not implement user-facing ranking yet, but include enough fields to rank later:

- `ref_kind`
- `evidence`
- `source`

Evidence:
Evidence must be compact and factual.

Examples:

- `import PromptService`
- `from prompt_service import PromptService`
- `use crate::indexer::build_index`
- `[Guide](docs/guide.md)`
- `.headerNavigation`
- `data-testid`

Noise controls:
Add hard caps with named constants:

- max refs per target name
- max refs per file
- max total refs per build if needed

Do not emit references for extremely common names:

- `test`
- `main`
- `new`
- `run`
- `app`
- `id`
- `name`
- `type`
- `value`

Use a small internal stoplist.

Determinism:
- Reference extraction order must be deterministic.
- Insert order must be deterministic.
- Rebuilding the same repo twice without file changes should produce the same refs.

Staleness:
- Changed files remove old refs for those paths before inserting new refs.
- Deleted files remove stale refs.
- Schema reset clears refs.

Tests:
Add fixture tests for:

- Python import reference
- Rust use/mod reference if easy
- TS/JS import reference if fixture exists or easy to add
- Markdown link reference
- CSS/HTML/JSX usage references
- test path classified as `test_reference`
- common stoplist terms do not flood refs
- changed file removes stale refs
- deleted file removes stale refs
- repeated build is deterministic

Shared integrity:
- Keep using shared ref-integrity checks from PLAN_01.
- Do not duplicate ref integrity assertion logic.

Real repo hardening:
- The ignored `real_repos` test should run ref integrity checks.
- Do not assert exact reference counts for arbitrary real repos.
- Print which repos were checked.

Instruction surfaces:
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical Repository search block.
- Do not reintroduce WI.md.
- Keep `wi --help` as the source of truth for filters/examples/subcommands.
- Update tests whenever command/help text changes.

Acceptance:
- refs are deterministic
- refs are capped/noise-controlled
- refs contain useful import/link/test/style relationships
- no malformed refs
- stale refs are removed correctly
- existing index/search tests still pass
- existing `wi-stats` tests still pass
- no JSONL storage is reintroduced

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if repos exist

Report:
- changed files
- extraction rules implemented
- caps/stoplist values
- validation results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed

<!-- thinindex-plan-instruction-surfaces-start -->

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth for command syntax, filters, examples, and subcommands.
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical `## Repository search` block.
- AGENTS.md should be created if absent.
- CLAUDE.md should be normalized only if present; do not create CLAUDE.md.
- Repeated `wi-init` runs must not duplicate `## Repository search`.
- Remove/normalize legacy markers: `@WI.md`, `See WI.md for repository search/index usage.`, `See `WI.md` for repository search/index usage.`, and old paragraph-style Repository search blocks.
- Update tests whenever help text or canonical Repository search text changes.

<!-- thinindex-plan-instruction-surfaces-end -->
