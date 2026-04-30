# Troubleshooting

Use `wi doctor` first when thinindex does not behave as expected.

```bash
wi doctor
```

The doctor output is intentionally actionable: every `fail` or `warn` row
includes the next command or file to inspect.

## Missing Index

Symptom:

```text
index database missing; run `build_index`
```

Fix:

```bash
build_index
wi doctor
```

The index lives at `.dev_index/index.sqlite`. It is a local disposable cache and
is not created implicitly by `wi <term>`.

## Stale Index

Symptom:

```text
index is stale; run `build_index`
```

Fix:

```bash
build_index
wi <term>
```

`wi` does not silently rebuild because indexing should be explicit before broad
agent discovery or after structural changes.

## Stale AGENTS.md Or CLAUDE.md

Symptom from `wi doctor`:

```text
[fail] AGENTS.md: AGENTS.md is stale or missing the current Repository search block
[fail] CLAUDE.md: CLAUDE.md is stale or missing the current Repository search block
```

Fix:

```bash
wi-init
wi doctor
```

`wi-init` creates `AGENTS.md` when absent and normalizes an existing
`CLAUDE.md`. It does not create `CLAUDE.md`.

## .dev_index Is Not Ignored

Symptom from `wi doctor`:

```text
[warn] ignore: .dev_index/ is not listed in .gitignore or .git/info/exclude
```

Fix:

```gitignore
.dev_index/
```

If the repository already has a `.gitignore`, `wi-init` can add this entry.

## Unsupported Language Or Format

thinindex does not silently parse every file type. Support levels are explicit:

- `supported`
- `experimental`
- `extras-backed`
- `blocked`

Check:

```bash
wi doctor
```

Then review:

- `docs/PARSER_SUPPORT.md`
- `docs/LANGUAGE_SUPPORT.md`
- `README.md`

Unsupported languages and formats are skipped by design. If unsupported files
are generated, vendored, minified, binary, or otherwise noisy, add them to
`.thinindexignore`.

## Optional Quality Reports

Quality reports under `.dev_index/quality/` are local-only. Normal `wi`,
`wi refs`, `wi pack`, `wi impact`, and `wi doctor` do not require optional
quality reports or external comparator tools.

## License Status

The licensing foundation is inert. Missing or invalid license data must not
block the current local workflow. See `docs/LICENSING.md`.

`wi doctor` reports the local license status so future activation work has one
place to surface state, but no current command is gated by that status.
