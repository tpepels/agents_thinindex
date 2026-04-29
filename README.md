# thinindex

thinindex is a local agent-navigation layer for coding repositories. It builds a repo-local SQLite index so coding agents and the developers supervising them can start with file:line landmarks instead of broad file reads.

No daemon. No embeddings. No vector database. No source upload. The SQLite engine is bundled into the Rust binaries, so users do not need a system SQLite package.

## What thinindex is

thinindex indexes named repository landmarks: functions, classes, methods, CSS selectors, HTML ids/classes, Markdown headings, TODO/FIXME markers, and related deterministic references. The `wi` command returns compact file:line results from that index.

The context commands use the same local data:

- `wi refs <term>` shows deterministic references around a landmark.
- `wi pack <term>` returns a compact read set for implementation work.
- `wi impact <term>` returns evidence-backed files to inspect before edits.

Native parsing is self-contained. Universal Ctags is not bundled, not required, and not used.

Supported parser coverage:

- Rust: functions, methods, structs, enums, traits, modules, constants/statics, type aliases, and imports.
- Python: classes, functions, methods, async functions/methods, imports, and conservative uppercase constants.
- JavaScript/TypeScript/JSX/TSX: functions, arrow-function declarations, classes, practical class methods, imports, exports, interfaces/types, and JSX component usage through existing extras.
- CSS/HTML/Markdown: selectors, ids/classes, tags, headings, checklist items, links, TODO/FIXME, and related extras.

The index is local-first and repo-local. It lives under `.dev_index/` and is intended to be disposable.

## Why agents need it

Agents waste context when they explore a codebase by reading files blindly. thinindex gives them a cheap first pass:

- use file:line landmarks before broad reads
- read fewer irrelevant files
- gather compact read sets from refs, pack, and impact
- measure index and command behavior with local benchmarks

thinindex is for navigation. It is not a hosted search service, an IDE replacement, an LSP replacement, or full semantic code understanding.

## Quickstart

Install the binaries, then run these commands inside a repository:

```bash
wi-init
build_index
wi --help
wi <term>
wi pack <term>
wi impact <term>
```

Example:

```bash
wi build_index
wi pack build_index
wi impact build_index
```

Run `wi --help` for the current command syntax, filters, examples, and subcommands. Keep that help output as the source of truth for CLI details.

## Agent workflow

The canonical agent workflow is:

1. Before broad repository discovery, run `build_index`.
2. Run `wi --help` if search filters, examples, or subcommands are needed.
3. Use `wi <term>` before grep/find/ls/Read to locate code.
4. For implementation work, prefer `wi pack <term>` to get a compact read set.
5. Before editing a symbol or feature area, run `wi impact <term>` to find related tests/docs/callers.
6. Read only files returned by `wi` unless the result is insufficient.
7. If `wi` returns no useful result, rerun `build_index` once and retry.
8. Fall back to grep/find/Read only after that retry fails.

`wi-init` creates or normalizes this workflow in `AGENTS.md` and normalizes an existing `CLAUDE.md` when present. It does not generate a `WI.md` instruction file.

## Commands

Installed commands:

- `build_index`: builds or updates `.dev_index/index.sqlite`.
- `wi <term>`: searches named landmarks and returns compact file:line results.
- `wi refs <term>`: shows deterministic references for matching landmarks.
- `wi pack <term>`: returns a compact, deduplicated read set for implementation work.
- `wi impact <term>`: returns conservative related files with concrete reasons.
- `wi bench`: measures build, search, context-command, size, count, latency, and integrity behavior.
- `wi-stats`: shows local usage stats and hit/miss graphs.
- `wi-init`: prepares a repository for agent use.
- `wi-init --remove`: removes the repo-local index.

Search filters and examples are documented by `wi --help`, not duplicated here.

All installed binaries support `--version`.

## Storage model

The canonical index path is:

```text
.dev_index/index.sqlite
```

`.dev_index` is a disposable local cache. Do not commit it. If the schema changes, `build_index` may rebuild it automatically.

Pre-alpha JSONL `.dev_index` caches are also disposable. `build_index` detects the old cache shape and rebuilds `.dev_index/index.sqlite`.

`wi` does not silently rebuild a missing or stale index. It tells users to run `build_index` so rebuilds are explicit.

Usage stats are stored in the same SQLite database. `make uninstall` removes installed binaries only; it does not remove repo-local caches.

## Real-repo hardening

Normal tests use fixtures and do not depend on local clones. Real-repo validation is opt-in:

- `test_repos/` is ignored and local-only.
- Clone third-party repositories there manually when needed.
- No third-party repository contents are committed.
- `test_repos/MANIFEST.toml` records local benchmark and integrity targets when present.
- Ignored tests validate indexing, references, context commands, and benchmark behavior against those repos.

Run real-repo checks with:

```bash
cargo test --test local_index -- --ignored
cargo test --test real_repos -- --ignored
cargo test --test bench_repos -- --ignored
```

## Benchmarks/value measurement

`wi bench` measures local behavior without asserting fragile timing promises. The benchmark reports:

- build time
- database size
- record and reference counts
- hit/miss behavior for deterministic queries
- search latency
- pack and impact output sizes
- integrity status

Use those numbers to evaluate whether thinindex helps a particular repository and workflow. Do not infer broad agent-performance gains from a single benchmark run.

## Limitations

thinindex is intentionally conservative:

- It is not full semantic code understanding.
- It is not an IDE or LSP replacement.
- References are deterministic and incomplete.
- Impact output is evidence-backed but not exhaustive.
- Agents can still ignore repository instructions.
- Generated, build, vendor, dependency, and large fixture paths should be ignored.
- Bundled parser dependencies must be permissively licensed and audited before commercial release artifacts.
- Native Rust, Python, and JS/TS parser support is useful but not a complete AST, macro expansion, JSX syntax, or type-analysis engine.
- Known parser gaps include macro-expanded Rust, dynamic Python assignment patterns, multi-line JS/TS import/export forms, computed JS/TS class members, and full JSX/TypeScript type analysis.
- Universal Ctags has been removed; proprietary packaging is no longer blocked by ctags, but still requires the dependency audit and release hardening documented in `THIRD_PARTY_NOTICES` and `docs/PRODUCT_BOUNDARY.md`.

## Free/local and future Pro

thinindex is currently a local/free tool. There is no license enforcement, payment flow, account login, cloud sync, telemetry, remote indexing, or feature lockout.

The free/local core includes local indexing, `build_index`, `wi <term>`, basic filters, `wi --help`, `wi-init`, repo-local SQLite storage, `wi-stats`, AGENTS.md setup, existing CLAUDE.md normalization, local cache rebuilds, and no-network operation.

Future Pro candidates are documented in [docs/PRODUCT_BOUNDARY.md](docs/PRODUCT_BOUNDARY.md). They are candidates, not current restrictions.

## Install/uninstall

Requires Rust/Cargo. Indexing is self-contained and does not require an external parser command.

Arch Linux:

```bash
sudo pacman -S rust
```

Debian / Ubuntu:

```bash
sudo apt update
sudo apt install cargo
```

Fedora:

```bash
sudo dnf install rust cargo
```

macOS with Homebrew:

```bash
brew install rust
```

Install:

```bash
make install
```

Default install location:

```text
$HOME/.local/bin
```

Check:

```bash
build_index --version
wi --version
wi-init --version
wi-stats --version
```

Uninstall installed binaries:

```bash
make uninstall
```

Uninstall does not delete `.dev_index`, `.thinindexignore`, `AGENTS.md`, or `CLAUDE.md`.

Remove a repo-local index:

```bash
wi-init --remove
```

## Packaging/licensing caveat

SQLite is bundled through the Rust dependency configuration. The parser path is native Rust code in this repository.

Proprietary Windows/macOS/Linux packages still require dependency license audit coverage and release artifact hardening before they are ready.

## Development

Build and test:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Manual smoke:

```bash
rm -rf .dev_index
cargo run --bin build_index
cargo run --bin wi -- build_index
cargo run --bin wi -- refs build_index
cargo run --bin wi -- pack build_index
cargo run --bin wi -- impact build_index
cargo run --bin wi-stats
```

Generated and local files:

- `.dev_index/` is ignored and disposable.
- `.thinindexignore` is repo-local configuration for thinindex-only ignores.
- `AGENTS.md` is the canonical repository instruction surface created by `wi-init`.
- Existing `CLAUDE.md` files are normalized when present; `wi-init` does not create one.
