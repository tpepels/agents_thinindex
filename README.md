# thinindex

Agents waste tokens when they explore a codebase by reading files blindly.

`thinindex` gives them a cheap repo-local structured index first. Agents use `wi` to get compact file/line targets before reading files.

It installs:

- `build_index` — builds/updates `.dev_index/`
- `wi` — searches the index and returns compact file/line results
- `wi-init` — sets up a repo for agent use
- `wi-stats` — shows usage stats and ASCII hit/miss graphs for 1/2/5/30-day windows

No daemon. No embeddings. No vector database. No MCP. No background updater. The SQLite engine is bundled into the Rust binaries; users do not need a system SQLite package.

## What agents will do

After `wi-init`, agents that follow `AGENTS.md` should:

1. Run `build_index` before broad repository discovery.
2. Run `wi --help` when they need filters, examples, or subcommands.
3. Use `wi <term>` before grep/find/ls/Read to locate code.
4. Use `wi pack <term>` for implementation work.
5. Use `wi impact <term>` before editing a symbol or feature area.
6. Read only files returned by `wi` unless the result is insufficient.
7. Retry once after `build_index` before falling back to grep/find/Read.

This gives agents a cheap first pass over the repo instead of burning tokens on blind file reads.

## Install

Requires Rust/Cargo and Universal Ctags. Universal Ctags is an external user-installed dependency for indexing until thinindex has a native parser; release archives/installers do not bundle ctags.

Arch Linux:

```bash
sudo pacman -S rust universal-ctags
```

Debian / Ubuntu:

```bash
sudo apt update
sudo apt install cargo universal-ctags
```

Fedora:

```bash
sudo dnf install rust cargo ctags
```

macOS with Homebrew:

```bash
brew install rust universal-ctags
```

Then install `thinindex`:

```bash
make install
```

Default install location:

```text
$HOME/.local/bin
```

Make sure that is on your `PATH`.

Check:

```bash
build_index --version
wi --version
wi-init --version
wi-stats --version
ctags --version
```

`ctags --version` should mention Universal Ctags.

Install and uninstall are idempotent. They install or remove only commands under the selected `BIN_DIR`; they do not delete repo-local `.dev_index` caches.

## Initialize a repo

Run this inside a repository:

```bash
wi-init
```

This will:

- create `.thinindexignore`
- create or normalize the canonical Repository search block in `AGENTS.md`
- normalize an existing `CLAUDE.md` when present
- add `.dev_index/` to `.gitignore` if `.gitignore` exists and does not already ignore it
- run `build_index` once
- create `.dev_index/`

It does not install cron or any background service.

To overwrite `.thinindexignore` from the bundled template:

```bash
wi-init --force
```

To remove the index from a repo:

```bash
wi-init --remove
```

To remove setup but keep the index:

```bash
wi-init --remove --keep-index
```

## Use

Build or refresh the index:

```bash
build_index
```

Show current syntax, filters, examples, and subcommands:

```bash
wi --help
```

Search:

```bash
wi HeaderNavigation
wi prompt -p app
wi pixel -l css
wi ranking -t function
wi HeaderNavigation -n 10
wi HeaderNavigation -v
```

Reference and context commands:

```bash
wi refs HeaderNavigation
wi pack HeaderNavigation
wi impact HeaderNavigation
wi bench
```

Options:

```text
-t <kind>   filter by kind/type
-l <lang>   filter by language
-p <path>   filter by path substring
-s <source> filter by source
-n <n>      result limit
-v          verbose output
```

For search terms that start with `-`:

```bash
wi -- --paper-bg
```

## Usage stats

`wi-stats` shows usage, hits, misses, hit ratio, average results, and terminal hit/miss graphs for 1/2/5/30-day windows.

Data is collected automatically each time `wi` is run, stored in:

```text
.dev_index/index.sqlite
```

Run:

```bash
wi-stats
```

There are no flags.

## What gets indexed

Primary symbols come from Universal Ctags.

Extra extraction includes:

- CSS selectors and variables
- HTML ids/classes/data attributes
- Markdown headings/checklists/links
- JSX component usage
- TODO/FIXME markers

Index files live in:

```text
.dev_index/
  index.sqlite
```

`.dev_index/index.sqlite` is a disposable local cache. If the index schema changes, `build_index` may delete and rebuild `.dev_index/`. Old JSONL `.dev_index` caches from pre-alpha builds are automatically rebuilt by `build_index`.

## Ignore extra paths

`thinindex` respects normal `.gitignore` rules.

`wi-init` writes a default `.thinindexignore` in the repo root using gitignore-style patterns. Edit or extend it to add thinindex-only ignores:

```text
generated/
*.large.json
.env*
!generated/keep.py
```

Use this for generated files, large fixtures, local artifacts, or anything agents should not discover through `wi`.

## Uninstall

Remove commands:

```bash
make uninstall
```

This removes installed commands only. It does not delete any repo-local `.dev_index/`, `.thinindexignore`, `AGENTS.md`, or `CLAUDE.md` files.

Remove a repo-local index before uninstalling:

```bash
wi-init --remove
```
