# PLAN_52_FILE_REFERENCE_GRAPH_REAL_REPO_HARDENING.md

Use superpowers:subagent-driven-development.

Do not implement this until recovery is complete and `prompts/recovery/RECOVERY_STATUS.md` says old-roadmap work may resume.

Goal:
Harden the file-reference graph against real repositories so `wi refs`, `wi pack`, and `wi impact` benefit from file-level relationships outside fixture-only cases.

Context:
The file-reference graph was implemented in commit `1403da64a32454eb680c4b1b78ba0f795f2a8605`.

It added:
- `src/file_refs.rs`
- `file_references` SQLite table
- reference kinds: import, include, require, source, link, asset, script, stylesheet, config_path, package_entry, fixture
- integration into refs/pack/impact
- fixture tests and docs

This plan is not about adding a new architecture. It is a bounded hardening pass against real repos.

Hard prerequisite:
Before starting this plan, the installed binary/schema mismatch from `prompts/recovery/RECOVERY_NEXT.md` must be fixed.

Do not start if:
- PATH `wi` and source `cargo run --bin wi` disagree on schema/version
- PATH `build_index` can still rebuild indexes to an older schema
- `prompts/recovery/RECOVERY_STATUS.md` says recovery is incomplete

Scope:
Improve extraction, resolution, reporting, and quality coverage for file references on `test_repos/`.

Do not add:
- package-manager execution
- network access
- LSP/compiler dependencies
- broad semantic analysis
- release packaging
- license enforcement
- payment behavior
- telemetry
- cloud behavior
- ctags as parser backend
- JSONL canonical storage
- `WI.md`

Phases:
- [x] Confirm recovery status allows roadmap work.
- [x] Confirm PATH/source binary schema mismatch is fixed.
- [x] Run current file-reference fixture tests.
- [x] Run `build_index` and file-reference smoke commands on the main repo.
- [x] Run ignored real-repo tests.
- [x] Inspect file-reference coverage on `test_repos/`.
- [x] Identify missing or weak file-reference patterns.
- [x] Select a bounded set of real-repo hardening fixes.
- [x] Add fixture/regression tests for each fixed pattern where practical.
- [x] Improve extraction/resolution only within existing file-reference architecture.
- [x] Update docs if supported behavior or known gaps change.
- [x] Run full verification.
- [x] Commit.

Real-repo coverage to inspect:
For each repo under `test_repos/`, collect:
- files scanned
- file references emitted
- resolved references
- unresolved references
- references by kind
- references by language/format
- top unresolved reasons
- high-value missed references
- noisy false positives
- refs/pack/impact examples using file-reference evidence

Reference types to harden:
- JS/TS relative imports
- Python relative/local imports
- Rust module references where practical
- C/C++ includes
- Ruby/PHP requires/includes
- Shell source/dot references
- Markdown links/images
- HTML scripts/stylesheets/assets/links
- CSS url(...)
- package/config path fields
- test fixture references

Resolution behavior:
Preserve these rules:
- resolve relative local paths
- resolve extensionless imports where practical
- resolve directory index targets where practical
- preserve unresolved local-looking references with reason
- do not resolve external URLs as local files
- do not treat package names as local files unless local evidence exists
- do not silently drop ambiguous references
- keep output deterministic

Quality policy:
Do not chase every unresolved reference.

Prioritize:
1. supported-language local file references that should resolve
2. refs that improve `wi pack` or `wi impact`
3. common real-repo patterns seen repeatedly
4. low-risk extraction fixes with clear fixture coverage

Defer or document:
- package-manager-specific resolution requiring installation
- aliases/path mappings that need config not currently modeled
- framework routing
- generated paths
- macro/compiler-generated references
- remote URLs
- ambiguous references without enough evidence

Tests:
Add/update tests for:
- real-repo-derived reference patterns
- resolved local references
- unresolved local-looking references with reason
- external URL exclusion
- ambiguous resolution
- stale cleanup
- duplicate edge dedupe
- deterministic ordering
- refs/pack/impact surfacing file-reference evidence
- `.dev_index` and `test_repos` exclusion in normal repo builds

Docs:
Update docs only if behavior changes:
- `docs/FILE_REFERENCES.md`
- `docs/REFERENCE_GRAPH.md`
- `docs/CONTEXT_PACKS.md`
- `docs/IMPACT_ANALYSIS.md`
- `docs/README.md`

Acceptance:
- file-reference graph works on real repos, not only fixtures
- selected real-repo gaps are fixed or explicitly deferred
- every fixed pattern has fixture/regression coverage where practical
- refs/pack/impact demonstrate useful file-reference evidence
- unresolved references remain honest and explainable
- no normal build indexes `.dev_index` or `test_repos`
- no ctags, WI.md, or JSONL regression
- source and PATH-installed binaries no longer disagree on schema/version

Execution notes:
- Recovery status allowed roadmap work, and PATH/source binaries both reported
  `0.1.4 (index schema 12)`.
- Real-repo inspection covered the 26 repos in `test_repos/MANIFEST.toml`.
  Representative rebuilt coverage included:
  - `web-50projects`: 113 file references, 102 resolved;
  - `httpx`: 281 file references, 58 resolved;
  - `gray-matter`: 33 file references, 25 resolved;
  - `fd`: 157 file references, 63 resolved;
  - `static-site-template`: 5 file references, 4 resolved;
  - `csharp-paint`: 99 file references, 9 resolved.
- Fixed bounded high-value gaps:
  - Markdown/query/fragment targets now resolve the local file portion while
    preserving the raw target string.
  - HTML `srcset` assets are extracted.
  - CSS/SCSS `@import` targets are extracted, including common Sass partials
    under `_sass/`.
  - `.csproj`/project-file `Include`, `Update`, `Remove`, `Project`, and
    `HintPath` values are extracted and resolved relative to the project file.
  - Config/package scanning now skips template expressions, variables, route
    patterns, URI schemes, package URLs, and version-looking values instead of
    recording noisy false local paths.
- Deferred intentionally:
  - root-relative web paths such as `/src/main.jsx`, because resolving them
    honestly needs project root/base-path conventions not currently modeled;
  - package self-import aliases such as `zustand/middleware`, because they need
    package export-map semantics and can be ambiguous without package-manager
    execution;
  - framework template URLs and generated paths, because they are not concrete
    local file paths;
  - binary/static assets that are not in the indexed file set, because the
    current resolver only matches discovered indexable files.

Verification:
- `which wi`
- `which build_index`
- `wi --version`
- `build_index --version` if supported
- `cargo run --bin wi -- --version`
- `cargo run --bin build_index -- --version` if supported
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- immediate second `cargo run --bin build_index -- --stats` if supported
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Commit:
Harden file references on real repos

Final response:
- recovery status checked
- binary/schema status checked
- repos checked
- file-reference gaps found
- gaps fixed
- gaps deferred and why
- tests added
- docs updated
- refs/pack/impact evidence
- verification commands and results
- commit hash
