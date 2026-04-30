# Performance And Scale

thinindex is designed to keep large repository indexing bounded and inspectable. It still builds an explicit repo-local SQLite index; it does not run a daemon, package manager, build tool, watcher, or network service.

## Incremental Builds

`build_index` stores file metadata in `.dev_index/index.sqlite`. On each build it:

- discovers current files through gitignore and `.thinindexignore` rules;
- compares path, mtime, and size against the manifest;
- reparses changed files only;
- removes deleted or newly skipped paths from records;
- rebuilds refs and dependency evidence from indexable text files;
- rewrites the SQLite snapshot deterministically.

The normal summary reports scanned, changed, deleted, and record counts. A second build with no edits should report `changed files: 0`.

## File Size Policy

Very large files are skipped before parsing:

- warning threshold: `LARGE_FILE_WARNING_BYTES` (`512 KiB`)
- hard parse cap: `MAX_INDEXED_FILE_BYTES` (`2 MiB`)

Skipped files are still tracked by metadata, so unchanged oversized files do not repeatedly count as changed. `build_index` prints a warning when it skips large files so potentially important source is not dropped silently.

Large-but-indexed files are reported in `build_index --stats`. If those paths are generated, vendored, minified, lockfiles, or build output, add repo-local ignore rules.

## Stats Report

Use:

```bash
build_index --stats
```

The report includes:

- refs and dependency counts;
- unchanged file count;
- total discovered file bytes;
- file-size thresholds;
- phase timings for discovery, change detection, parsing, dependency extraction, reference extraction, and SQLite save;
- bounded large-file samples;
- SQLite tuning summary.

The report is intentionally compact. It does not write huge snapshots or full per-file parse traces.

## SQLite Tuning

Index connections use local SQLite pragmas suitable for disposable repo-local indexing:

- `journal_mode = WAL`
- `synchronous = NORMAL`
- `temp_store = MEMORY`
- `cache_size = -20000`

The index remains rebuildable; `.dev_index/` is a local cache and should not be committed.

## Monorepo Guidance

For monorepos:

- run `wi-init` to create `.thinindexignore`;
- keep generated, vendored, dependency, build-output, minified, lockfile, coverage, and binary asset paths ignored;
- inspect `build_index --stats` after adding a large repo;
- add local ignore rules for noisy package subtrees before relying on pack/impact output;
- prefer targeted real-repo manifests under ignored `test_repos/` for local hardening.

Useful ignore candidates include:

```gitignore
generated/
vendor/
third_party/
node_modules/
dist/
build/
target/
coverage/
*.min.js
*.min.css
*.lock
```

These ignores are repository-local policy choices. thinindex reports large/skipped files so teams can decide whether to ignore, split, or keep them.
