# Getting Started

This guide is the shortest path from a fresh checkout to useful `wi` results.

## Install

Build locally with Cargo:

```bash
cargo build --release
```

Or install from a release archive by extracting it and running the archive
installer:

```bash
scripts/install-archive-unix
```

On Windows, run from the extracted archive root:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\install.ps1
```

## Initialize A Repository

Run these commands inside the repository you want agents to work in:

```bash
wi-init
wi doctor
wi --help
```

`wi-init` writes `.thinindexignore`, creates or updates `AGENTS.md`, normalizes
an existing `CLAUDE.md` when present, and builds the first local index.

`wi-init` builds the first `.dev_index/index.sqlite`. The index is local,
disposable, and should not be committed.

`wi doctor` checks whether the index, schema, freshness, instruction files,
ignore rules, parser support matrix, optional quality state, license status,
and binary path look sane.

## First Searches

Search for a symbol or landmark:

```bash
wi build_index
wi PromptService
wi '.headerNavigation' -t css_class
wi 'Tests' -t section
```

Use context commands before reading broad files:

```bash
wi refs PromptService
wi pack PromptService
wi impact PromptService
wi-scorecard --query PromptService
```

`wi refs` shows direct deterministic reference evidence. `wi pack` returns a
compact read set for implementation. `wi impact` returns evidence-backed files
to inspect before editing. `wi-scorecard` reports compact pass/warn/fail
evidence for whether the local workflow is producing useful results.

## Agent Terminal Example

A practical agent loop is:

```bash
build_index
wi doctor
wi --help
wi pack <feature-or-symbol>
wi impact <feature-or-symbol>
```

Read the returned file:line rows first. `wi` auto-builds or auto-rebuilds a
missing/stale index once before searching. Fall back to grep/find only when
`wi` does not return useful results after that self-healing query path or after
fixing any explicit auto-build failure.

## Unsupported Files

Parser support is intentionally explicit. Supported and experimental languages
are listed in `README.md`, `docs/PARSER_SUPPORT.md`, and
`docs/LANGUAGE_SUPPORT.md`.

Unsupported languages are skipped by design. Add ignore rules for generated,
vendor, minified, binary, or noisy files that do not help navigation.
