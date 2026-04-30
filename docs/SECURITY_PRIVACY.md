# Security And Privacy

thinindex is local-first, but it still indexes repository text into `.dev_index/index.sqlite`. Treat `.dev_index/` and `.dev_index/quality/` as local disposable caches that may contain project names, file paths, symbols, references, and compact evidence strings.

## Safe Defaults

- No source is uploaded by thinindex.
- No network service, daemon, telemetry, cloud sync, or hosted index is used.
- `.dev_index/`, `.dev_index/quality/`, and `test_repos/` are ignored local paths and must not be committed.
- Release archive checks reject `.dev_index/`, `test_repos/`, build output, dist output, and local report artifacts.
- `wi pack` and `wi impact` return bounded file:line rows and reasons, not full file contents.
- Quality exports omit absolute local paths by default.

## Sensitive Files

Add sensitive project paths to `.thinindexignore` before indexing. `wi-init` includes common examples:

- `.env`, `.env.*`, `.envrc`
- `.netrc`, `.npmrc`, `.pypirc`
- `secrets/`, `secret/`, `credentials/`, `private_keys/`
- `*.pem`, `*.key`, `*.p12`, `*.pfx`
- generated logs, databases, archives, binary assets, dependency caches, and test output

`build_index` warns about a small capped set of sensitive-looking paths when they are still indexed. This is only a lightweight path warning, not a secret scanner.

## Redaction Policy

Human-readable command/report output redacts common secret-like values by default when they appear in evidence, verbose text, symbol names, paths, or quality report details. The redaction is deliberately conservative and pattern-based. It is not a guarantee that all secrets are detected.

Redacted examples include values assigned to names such as `password`, `token`, `api_key`, `access_token`, `client_secret`, `private_key`, and common token shapes such as AWS access keys, GitHub tokens, JWTs, and private-key headers.

Reports should prefer counts, paths, symbol names, line numbers, reasons, and capped samples. They should not dump large source text by default.

## Quality Reports

Quality reports and comparator reports stay under `.dev_index/quality/`. They are local QA artifacts and must not be copied into production SQLite `records` or `refs`.

Default quality exports are safe mode:

- no absolute local paths unless a local workflow explicitly opts in;
- capped summary samples;
- redacted secret-like strings in Markdown, JSON, and JSONL outputs;
- no full source file dumps.

If a future verbose/report mode includes larger samples, it must keep redaction enabled and clearly label the output local-only.

## Release Artifacts

Release archives are assembled from explicit files. They must not include:

- `.dev_index/`
- `.dev_index/quality/`
- `test_repos/`
- `target/`
- `dist/`
- generated benchmark output
- local comparator or quality reports
- signing keys, certificates, provisioning profiles, notarization credentials, or package signing material
- source checkout contents copied wholesale

Release archives include `SBOM.md` and `THIRD_PARTY_NOTICES` so recipients can inspect the shipped binaries, target, checksum sidecar, and dependency notices. Run `scripts/check-package-contents <archive>` after packaging.
