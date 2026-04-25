# thinindex

Agents waste tokens when they explore a codebase by reading files blindly.

`thinindex` gives them a cheap repo-local index first. It installs:

- `build_index` — builds/updates `.dev_index/`
- `wi` — searches the index and returns compact file/line results
- `wi-init` — sets up a repo for agent use
- `wi-stats` — shows usage stats and ASCII hit/miss graphs for 1/2/5/30-day windows

No daemon. No embeddings. No vector database. No MCP. No background updater.

Replace the install section with:

````md
## Install

Requires Rust/Cargo and Universal Ctags.

Arch Linux:

```bash
sudo pacman -S rust universal-ctags
````

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


## Initialize a repo

Run this inside a repository:

```bash
wi-init
```

This will:

* create `WI.md`
* add a short reference to `AGENTS.md`
* run `build_index` once
* create `.dev_index/`

It does not install cron or any background service.

To overwrite `WI.md` from the bundled template:

```bash
wi-init --force
```

To remove the index from a repo:

```bash
wi-init --remove
```

## Use

Build or refresh the index:

```bash
build_index
```

Search:

```bash
wi HeaderNavigation
wi prompt --path app
wi pixel --lang css
wi ranking --type function
```

Useful filters:

```bash
--type <kind>
--lang <lang>
--path <substring>
--limit <n>
--verbose
```

For search terms that start with `--`:

```bash
wi -- --paper-bg
```

## Usage stats

`wi-stats` shows usage, hits, misses, hit ratio, average results, and terminal hit/miss graphs for 1/2/5/30-day windows.

Data is collected automatically each time `wi` is run, stored at `.dev_index/wi_usage.jsonl`.

```bash
wi-stats
```

There are no flags.

## What agents will do

After `wi-init`, agents that follow `AGENTS.md` should:

1. Read `WI.md`.
2. Run `build_index` before broad discovery.
3. Use `wi <term>` to find files and line numbers.
4. Read only the files returned by `wi`.
5. Run `build_index` again after each implementation phase or structural code change.
6. If `wi` returns nothing, rerun `build_index` once and retry before broader discovery.

This gives agents a cheap first pass over the repo instead of burning tokens on blind file reads.

## What gets indexed

Primary symbols come from Universal Ctags.

Extra extraction includes:

* CSS selectors and variables
* HTML ids/classes/data attributes
* Markdown headings/checklists/links
* JSX component usage
* TODO/FIXME markers

Index files live in:

```text
.dev_index/
  manifest.json
  index.jsonl
```

Put it after the **“What gets indexed”** section and before **“Uninstall”**.

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

Clean a repo before uninstalling:

```bash
wi-init --remove
```

If already uninstalled, clean manually:

```bash
rm -rf .dev_index
rm -f WI.md
```

Then remove the WI.md reference from `AGENTS.md` if desired.