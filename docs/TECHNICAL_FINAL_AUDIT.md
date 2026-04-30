# Technical Final Audit

This audit records the current relationship/navigation layer before security, packaging, licensing, and product-polish work. It is a coherence checkpoint, not a claim that thinindex has complete semantic code understanding.

Status: coherent after Plan 40 cleanup, with caveats documented below.

## Parser

- Code-symbol extraction uses the Tree-sitter registry, query specs, adapters, normalized captures, and shared conformance tests.
- CSS, HTML, Markdown, JSON, TOML, and YAML remain project-owned extras-backed landmarks, not Tree-sitter code-symbol support.
- Universal Ctags is optional quality-comparator tooling only; it is external, not bundled, not required, and not used by production indexing.
- Support claims are governed by `src/support.rs`, `docs/PARSER_SUPPORT.md`, and generated `docs/LANGUAGE_SUPPORT.md`.
- Quality gates live in parser/support/quality tests and keep comparator output out of production `records` and `refs`.

## Dependency Graph

- Dependency edges are stored in SQLite `dependencies` separately from parser records and deterministic refs.
- Resolved, unresolved, and ambiguous edges keep explicit `confidence` and unresolved-reason data.
- Changed or deleted files rebuild the SQLite snapshot so stale records, refs, dependencies, and semantic facts are removed together.
- Resolver behavior and gaps are documented in `docs/DEPENDENCY_GRAPH.md`.
- Dependency edges are used by `wi pack` and `wi impact`, but they do not claim compiler or package-manager completeness.

## References

- References are stored in SQLite `refs` with a concrete source location, target name, kind, confidence, reason, evidence, and source.
- Confidence labels distinguish direct local matches, syntax evidence, dependency evidence, unresolved dependency evidence, and heuristic fallback.
- Broad text fallback is capped and labelled heuristic so it cannot masquerade as semantic resolution.
- Syntax references remain AST-backed observations, not proof of declaration binding.
- Reference behavior and known limits are documented in `docs/REFERENCE_GRAPH.md`.

## Impact

- `wi impact <term>` uses SQLite `records`, `refs`, and `dependencies`.
- Every output row must have a concrete file:line reason from indexed evidence.
- Output groups cover direct definitions, references, dependent files, likely tests, docs, build/config files, and unresolved/unknown areas when evidence exists.
- It is intentionally not an exhaustive semantic impact engine.
- Behavior is documented in `docs/IMPACT_ANALYSIS.md`.

## Pack

- `wi pack <term>` returns a bounded, reasoned read set for implementation work.
- It does not dump full file contents.
- Rows are grouped, ranked, deduplicated by file across non-primary groups, and labelled with confidence.
- Pack remains useful for agents because it gives a small first read set before broader impact or fallback search.
- Behavior is documented in `docs/CONTEXT_PACKS.md`.

## Performance

- `build_index` is explicit and incremental: it compares file metadata, reparses changed indexable files, removes deleted paths, and rewrites deterministic SQLite snapshots.
- Large-file and record/ref caps keep generated, vendor, dependency, lockfile, and minified paths bounded.
- Normal tests do not depend on local `.dev_index/` or `test_repos/`.
- Real-repo and benchmark checks remain ignored/manual and local-only.
- Scale guidance is documented in `docs/PERFORMANCE.md` and `docs/REAL_REPO_MANIFEST.md`.

## Semantic Adapters

- The semantic adapter boundary exists, but real compiler/LSP adapters are not bundled.
- Adapters are optional and disabled by default.
- A successful adapter may write isolated `semantic_facts`; failures or unavailable adapters are skipped cleanly.
- Semantic facts do not pollute parser `records`, deterministic `refs`, or normal baseline command output.
- Future adapter requirements and placeholders are documented in `docs/SEMANTIC_ADAPTERS.md`.

## Agent Integration

- `wi-init` writes or normalizes the canonical Repository search block in `AGENTS.md`.
- Existing `CLAUDE.md` files are normalized when present; `CLAUDE.md` is not created when absent.
- `WI.md` is not generated or restored.
- `wi --help` remains the source of truth for CLI syntax, filters, examples, and subcommands.
- Integration packs live under `integrations/agents/` and are advisory.
- `wi-stats` reports local usage and command-category audit counts, but it cannot observe external grep, find, ls, or file-read activity.

## Remaining Caveats

- Tree-sitter extraction is syntactic, not semantic or LSP-level analysis.
- Dependency resolution is local and deterministic; it does not invoke package managers, build tools, compilers, or LSP servers.
- Experimental languages remain experimental until coverage and real-repo evidence improve.
- Optional comparator and semantic adapter data are quality/advisory signals, not production ground truth.
- Real-repo quality and benchmark confidence depends on local `test_repos/` contents.
- Security/privacy, signed distribution, licensing enforcement, hosted/team workflows, and product polish are separate later plans.
