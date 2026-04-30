# Quality Plugins

thinindex quality plugins are isolated evaluation tools. They do not participate in production indexing, search, packaging, install, or release artifact creation.

## Boundary

- Production indexing remains Tree-sitter based.
- Comparator records are never written to production `.dev_index/index.sqlite` `records` or `refs`.
- Comparator reports are written only under `.dev_index/quality/`.
- Comparator output is not ground truth. It is a comparison signal alongside thinindex Tree-sitter output and expected-symbol manifests.
- Expected-symbol manifests and conformance fixtures are the stronger quality source when there is disagreement.

## Optional Universal Ctags Comparator

Universal Ctags is optional, external, not bundled, not required, and not used by `build_index`.

When present on a developer machine, the quality adapter can run the local command and normalize its JSON output into comparator records with path, line, optional column, kind, name, optional language, and comparator name. When absent, the comparator is skipped with `skipped: comparator not found`.

Normal tests do not require Universal Ctags. Release packages and installer scripts do not include Universal Ctags.

## Manual Run Shape

The quality layer is library tooling in this phase. A manual or ignored test run should:

1. Build the thinindex SQLite index with `cargo run --bin build_index`.
2. Load thinindex `records`.
3. Run an optional comparator adapter.
4. Compare thinindex records, comparator records, and expected symbols.
5. Write a report under `.dev_index/quality/<comparator>.txt`.

The ignored test `optional_external_ctags_comparator_generates_isolated_quality_report_or_skips` demonstrates this flow. It runs only when explicitly requested and skips cleanly if the optional command is missing.

## Report Contents

Quality reports are compact text files with:

- per-language thinindex record count
- per-language comparator record count
- matched symbols
- thinindex-only symbols
- comparator-only symbols
- expected-symbol pass/fail counts
- unknown comparator kinds
- duplicate record count
- malformed record count
- unsupported extensions

These reports are intended for parser-quality triage and must not be imported into production indexes.
