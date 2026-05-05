# Performance And Scale

thinindex is designed to keep large repository indexing bounded and inspectable. It still builds an explicit repo-local SQLite index; it does not run a daemon, package manager, build tool, watcher, or network service.

## Incremental Builds

`build_index` stores file metadata in `.dev_index/index.sqlite`. On each build it:

- discovers current files through gitignore and `.thinindexignore` rules;
- compares path, mtime, and size against the manifest;
- reparses changed files only;
- removes deleted or newly skipped paths from records;
- rebuilds refs and dependency evidence from indexable text files when the
  file set changed;
- rewrites the SQLite snapshot deterministically when indexed content changed.

The normal summary reports scanned, changed, deleted, and record counts. A
second build with no edits should report `changed files: 0` and should use the
metadata fast path instead of regenerating refs or rewriting SQLite.

## Current Budgets

These budgets are practical local targets for the thinindex repository on a
developer workstation. They are not hard promises for every repository; use
`build_index --stats` to recalibrate after adding large source trees.

| Path | Budget |
| --- | ---: |
| warm `wi <query>` | under 150 ms |
| `wi refs <query>` | under 200 ms |
| `wi pack <query>` | under 250 ms |
| `wi impact <query>` | under 250 ms |
| immediate no-change `build_index` | under 250 ms |
| one-file stale `build_index` or stale `wi <query>` auto-rebuild | under 6 s |
| cold build of this repository | under 10 s |

RECOVERY_03 profiling on 2026-05-05 measured the pre-fix bottleneck in
reference extraction:

- no-change build: 24488 ms total, with 24304 ms in refs;
- one-file stale build: 24329 ms total, with 24172 ms in refs;
- stale `wi <query>` auto-rebuild: 24.206 s wall time.

The fix added a no-change metadata fast path and made text-reference extraction
scan identifier tokens against a precomputed target map instead of checking
every target name against every line. Post-fix measurements on the same
workstation:

- no-change build: 16-29 ms total, with 0 ms in refs and 0 ms in save;
- one-file stale build: 4117-4220 ms total, with about 3950 ms in refs;
- cold build: 7379 ms total, with 3287 ms parse and 3942 ms refs;
- stale `wi <query>` auto-rebuild: 4.140 s wall time;
- warm `wi <query>`: 0.051-0.071 s wall time;
- `wi refs`, `wi pack`, and `wi impact`: 0.087 s, 0.128 s, and 0.124 s.

If these budgets regress, inspect refs/dependency generation first, then check
whether generated, vendored, ignored, or local `test_repos/` paths are being
included accidentally.

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

- refs, dependency, and semantic fact counts;
- capped sensitive-looking path warning count;
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
