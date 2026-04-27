# Thinindex Product Roadmap

## Product target

Thinindex becomes a cross-platform local codebase index for coding agents.

Core promise:

> Agents should search a cheap local index before reading files.

Paid product viability depends on:

- one-command install
- Windows/macOS/Linux support
- no fragile external setup
- deterministic local-only behavior
- clear agent integration
- measurable local usage stats
- clean Free/Pro enforcement
- outsourced payment/licensing through Lemon Squeezy
- native commercially viable parser backend
- professional docs and release process

Current state is a developer utility. The roadmap below gets it to a clean Free/Pro product.

## Product constants

Use named constants everywhere in docs, code, tests, and messages so policy can change cleanly.

```text
FREE_MAX_INDEXED_FILES = 50
FREE_MAX_INDEX_RECORDS = 1000
PRO_PRICE_ONE_TIME_EUR = 24
LICENSE_PROVIDER = Lemon Squeezy
PARSER_BACKEND = tree-sitter
```

User-facing messages should render actual values, but implementation/docs should reference the constants where practical.

## Non-negotiables

* Local-first indexing and search.
* No source upload.
* No daemon by default.
* No embeddings in the first paid version.
* No hidden memory injection.
* No telemetry by default.
* No agent-specific lock-in.
* No GPL parser dependency in the paid product.
* CLI remains fast and simple.
* Free limits scale, not core functionality.
* Lemon Squeezy handles payment and license management.
* Thinindex only implements local activation/cache/enforcement.
* User-editable config must not increase Free product limits.
* Paid launch must not require users to install ctags.

---

# Product editions

## Free

Purpose: useful evaluation and real use on small repos.

* `wi-init`
* `build_index`
* `wi`
* `wi-stats`
* local `.dev_index`
* `.thinindexignore`
* `WI.md` / `AGENTS.md` integration
* local usage stats
* max `FREE_MAX_INDEXED_FILES` eligible indexed files per repo
* max `FREE_MAX_INDEX_RECORDS` index records per repo
* native tree-sitter backend, same as Pro, but scale-limited

## Pro

Purpose: one-time paid local tool for real codebases.

* unlimited eligible indexed files
* unlimited index records
* one-time license, target price `PRO_PRICE_ONE_TIME_EUR`
* all current and future 1.x updates
* Lemon Squeezy license activation
* native tree-sitter parser backend
* signed installers when ready
* Windows/macOS/Linux releases
* richer language extraction
* repo health diagnostics
* agent integration packs
* config validation

## Team later

Do not build first.

* shared policy templates
* CI checks
* organization license management
* team docs
* optional license portal

---

# Free vs Pro boundary

## Limit model

Free limits by **both eligible indexed files and index records**, after `.gitignore` and `.thinindexignore` are applied.

```text
Free:
- FREE_MAX_INDEXED_FILES eligible indexed files per repo
- FREE_MAX_INDEX_RECORDS index records per repo

Pro:
- unlimited eligible indexed files
- unlimited index records
```

An **eligible indexed file** is a file selected for indexing after ignore rules are applied.

An **index record** is one searchable item written to `.dev_index/index.jsonl`, such as:

* function
* class
* method
* constant
* import/export
* React component definition
* React component usage
* hook
* CSS selector
* HTML id/class
* Markdown heading
* TODO/FIXME marker

Why both limits:

* file count keeps Free scoped to small repos
* record count prevents bypassing the limit by cramming many symbols into a few files
* both limits are easy to explain
* both limits are calculated locally after ignore rules

## Enforcement behavior

`build_index` enforces both limits.

Enforcement order:

1. discover eligible files after ignore rules
2. check eligible file count
3. index records into memory/temp output
4. check index record count
5. write `.dev_index` only if all limits pass

No partial index should be written when limits are exceeded.

If file count fails:

```text
Free file limit reached: 73 eligible files found, limit is FREE_MAX_INDEXED_FILES.

Reduce indexed files with .thinindexignore or upgrade to Pro for unlimited indexing.
```

If record count fails:

```text
Free index record limit reached: 1,284 records found, limit is FREE_MAX_INDEX_RECORDS.

Reduce indexed symbols with .thinindexignore or upgrade to Pro for unlimited indexing.
```

If both fail:

```text
Free limit reached.

Eligible files: 73 / FREE_MAX_INDEXED_FILES
Index records: 1,284 / FREE_MAX_INDEX_RECORDS

Reduce indexed files with .thinindexignore or upgrade to Pro.
```

Rules:

* Do not write a partial index.
* Do not silently degrade results.
* Do not disable `wi`, `wi-init`, or `wi-stats`.
* `wi` can search an existing index.
* `build_index --check` reports the same limit state.
* Counts are deterministic and based on post-ignore eligible files and generated records.
* Free product limits are not user-configurable.

## Local config rule

Do **not** put authoritative Free limits in user config:

```toml
free_indexed_file_limit = 50
free_index_record_limit = 1000
```

That is meaningless for enforcement.

User config may only reduce local limits, never raise product limits.

```text
effective_file_limit = min(local_file_limit_or_unlimited, edition_file_limit)
effective_record_limit = min(local_record_limit_or_unlimited, edition_record_limit)
```

Edition limits:

```text
Free:
- FREE_MAX_INDEXED_FILES
- FREE_MAX_INDEX_RECORDS

Pro:
- unlimited files
- unlimited records
```

---

# Parser backend decision

## Decision

Use **tree-sitter** as the long-term commercial parser backend.

Universal Ctags is not the paid-product backend.

## Why tree-sitter

* suitable for native cross-platform binaries
* no external parser install for users
* commercially viable licensing model
* stable enough for structural code extraction
* good coverage for common coding-agent repositories
* avoids GPL/distribution issues from bundling Universal Ctags
* lets Thinindex control schema, ranking, language support, and error behavior

## Ctags status

Universal Ctags may remain temporarily as:

* a development/bootstrap backend
* an optional fallback
* a migration aid

But paid launch should not require ctags.

Before paid launch:

* remove ctags from install prerequisites
* make tree-sitter the default backend
* make ctags optional or delete it
* ensure Windows works without ctags

## Backend model

Introduce a parser backend abstraction:

```text
ParserBackend
- TreeSitterBackend
- OptionalCtagsBackend only if kept
```

The rest of the system must not care which parser created the records.

Stable boundary:

```text
eligible files -> parser backend -> IndexRecord[] -> .dev_index/index.jsonl -> wi search
```

Manifest should track:

```text
backend_name
backend_version
schema_version
```

---

# Licensing and payment

## Decision

Use **Lemon Squeezy** for v1.

Thinindex does not build its own licensing service.

Lemon Squeezy handles:

* payment
* VAT/tax
* license key generation
* license activation
* license validation
* license deactivation
* activation limits
* customer/license records

Thinindex handles:

* CLI activation command
* local license cache
* local status display
* Free/Pro enforcement in `build_index`
* privacy docs
* tests

## License commands

Add:

```bash
wi-license activate <license-key>
wi-license status
wi-license deactivate
wi-license refresh
```

Possible later:

```bash
wi-license install license.json
```

Only add manual install if Lemon Squeezy flow or user demand requires it.

## Local license state

Linux/macOS:

```text
~/.config/thinindex/license.json
```

Windows:

```text
%APPDATA%\thinindex\license.json
```

Local cache shape:

```json
{
  "schema": 1,
  "provider": "lemonsqueezy",
  "license_key_hash": "...",
  "instance_id": "...",
  "edition": "pro",
  "activated_at": 1760000000,
  "last_validated_at": 1760000000,
  "cached_valid_until": 1760604800
}
```

Rules:

* Avoid storing raw license keys if possible.
* Store provider activation instance ID.
* Store cached edition/features.
* Keep this independent of repo data.
* No repo contents are sent to Lemon Squeezy.

## Network behavior

Normal indexing/searching is local.

```text
wi-license activate   -> online
wi-license refresh    -> online
wi-license deactivate -> online if possible
build_index           -> offline by default
wi                    -> offline
wi-stats              -> offline
wi-init               -> offline
```

`build_index` uses cached local license state. It should not phone home on every run.

Optional later:

* auto-refresh cached license every 14 or 30 days
* only if it does not make normal use fragile

## License failure behavior

Missing license:

```text
Edition: Free
Indexed file limit: FREE_MAX_INDEXED_FILES
Index record limit: FREE_MAX_INDEX_RECORDS
```

Invalid/expired/unreachable refresh:

* keep Free behavior
* show clear status in `wi-license status`
* do not block Free usage
* do not delete user index automatically

Valid Pro license:

```text
Edition: Pro
Indexed file limit: unlimited
Index record limit: unlimited
```

## Security reality

No local CLI licensing is fully secure. Do not build hostile DRM.

Good enough:

* compiled Free limits
* Lemon Squeezy activation
* local activation cache
* no config bypass
* no repo source upload
* clear messages

Not worth it:

* obfuscation
* hardware fingerprinting
* aggressive online checks
* subscriptions for v1
* frequent mandatory validation

---

# Phase 0 — Product cleanup before expansion

## Goal

Make the current project clean, coherent, testable, and ready to evolve.

## Work

* Stabilize command names:

  * `wi-init`
  * `build_index`
  * `wi`
  * `wi-stats`
* Freeze repo-local files:

  * `.dev_index/index.jsonl`
  * `.dev_index/manifest.json`
  * `.dev_index/wi_usage.jsonl`
  * `.thinindexignore`
  * `WI.md`
* Make README short and user-facing.
* Add:

  * `CHANGELOG.md`
  * `LICENSE`
  * `CONTRIBUTING.md`
  * `docs/ARCHITECTURE.md`
  * `docs/RELEASE.md`
  * `docs/SECURITY.md`

## Technical cleanup

* Centralize constants:

  * file names
  * template names
  * command names
  * schema versions
  * `FREE_MAX_INDEXED_FILES`
  * `FREE_MAX_INDEX_RECORDS`
* Add schema version to all JSON/JSONL record types.
* Add graceful migration behavior for old `.dev_index`.
* Make all generated files documented.
* Keep `wi-init --remove` behavior clear:

  * removes `.dev_index`
  * leaves `WI.md`, `.thinindexignore`, and `AGENTS.md`

## Acceptance

* `cargo fmt`
* `cargo test`
* `cargo clippy -- -D warnings`
* fresh clone install works
* `wi-init`, `build_index`, `wi`, `wi-stats`, `wi-init --remove` all work in a sample repo
* README explains the product in under 30 seconds

---

# Phase 1 — Robust local indexing

## Goal

Make indexing reliable enough that users trust it.

## Work

* Make incremental rebuilds robust:

  * new files
  * changed files
  * deleted files
  * renamed files treated correctly
* Add stale-index detection:

  * `wi` can warn if manifest does not match changed files
  * keep warning cheap
* Improve error handling:

  * parser backend missing/misconfigured
  * unsupported files
  * malformed JSONL
  * permission errors
* Add:

```bash
build_index --clean
build_index --check
```

`--clean`:

* remove `.dev_index`
* rebuild from scratch

`--check`:

* exits non-zero if index is stale
* useful for CI/local scripts

## Ignore behavior

* Respect:

  * `.gitignore`
  * `.thinindexignore`
  * global gitignore
* Keep hard safety excludes:

  * `.git`
  * `.dev_index`
* Document exact semantics.

## Tests

Fixture repos:

* Python
* JS/TS/TSX
* Rust
* CSS
* HTML
* Markdown
* ignored generated files
* large files
* deleted files
* renamed files
* malformed index
* stale manifest

## Acceptance

* repeated `build_index` is fast and reports `changed files: 0`
* `build_index --clean` restores a broken index
* `build_index --check` detects stale files
* no panics on weird repos

---

# Phase 2 — Parser backend abstraction

## Goal

Prepare to replace ctags without rewriting storage/search/stats/licensing.

## Work

Add:

```text
ParserBackend trait
BackendResult
BackendDiagnostic
```

Initial backends:

```text
CtagsBackend       temporary/current
TreeSitterBackend  new native target
```

Rules:

* `indexer.rs` calls backend trait, not ctags directly.
* `IndexRecord` schema remains stable.
* Manifest records backend name/version.
* Backend diagnostics are clear and user-facing.
* Tests can run against backend fixtures.

## Acceptance

* existing behavior still passes with ctags backend
* backend selection is isolated
* tree-sitter backend can be added incrementally
* search/store/stats do not depend on ctags

---

# Phase 3 — Native tree-sitter backend

## Goal

Remove ctags as a required dependency and create the commercially viable parser backend.

## First supported languages

Start with:

* Python
* JavaScript
* TypeScript
* TSX/JSX
* Rust
* CSS/HTML via existing extras or tree-sitter
* Markdown via existing extras

Do not chase every language yet.

## Extraction targets

Python:

* classes
* functions
* methods
* imports
* FastAPI route decorators

JS/TS:

* imports
* exports
* functions
* classes
* consts
* interfaces/types
* React components
* hooks

TSX/JSX:

* component definitions
* component usage
* className usage
* data attributes

Rust:

* functions
* structs
* enums
* traits
* impl methods
* modules
* constants

CSS:

* classes
* IDs
* variables
* keyframes
* media queries

HTML:

* IDs
* classes
* data attributes
* semantic landmarks

Markdown:

* headings
* checklists
* links
* code fences

## Implementation

Use tree-sitter queries for code languages.

Suggested module structure:

```text
src/backend/
  mod.rs
  tree_sitter.rs
  ctags.rs
  languages.rs
  queries/
    python.scm
    javascript.scm
    typescript.scm
    tsx.scm
    rust.scm
```

Keep simple regex extras for:

* Markdown
* CSS if simpler
* HTML if simpler
* TODO/FIXME

Do not build semantic name resolution yet.

## Acceptance

* `build_index` works without ctags
* install docs no longer require ctags
* all fixture tests pass with native backend
* Windows works without external parser install
* Free/Pro limits apply to native records
* ctags is optional fallback or removed

---

# Phase 4 — Free limit enforcement

## Goal

Add the Free/Pro scale boundary before payment activation.

## Work

* Count eligible indexed files after ignore rules.
* Count generated index records before writing final index.
* Add edition detection stub:

  * no license = Free
  * test-only Pro override for tests
* Enforce Free limits:

  * `FREE_MAX_INDEXED_FILES`
  * `FREE_MAX_INDEX_RECORDS`
* Print clear over-limit errors.
* Do not write partial indexes if either limit is exceeded.

## Tests

* `FREE_MAX_INDEXED_FILES - 1` files passes
* `FREE_MAX_INDEXED_FILES` files passes
* `FREE_MAX_INDEXED_FILES + 1` files fails
* `FREE_MAX_INDEX_RECORDS - 1` records passes
* `FREE_MAX_INDEX_RECORDS` records passes
* `FREE_MAX_INDEX_RECORDS + 1` records fails
* ignores reduce count
* Pro test override passes
* existing index is not corrupted on over-limit failure
* local config cannot raise Free limits

## Acceptance

* limits are deterministic
* error messages are clear
* tests prove boundary behavior
* README explains Free limits

---

# Phase 5 — Product-ready free CLI

## Goal

Make the free version clean enough to distribute.

## Work

* Finalize `wi-init`.
* Finalize `.thinindexignore`.
* Add stats disable env var:

```bash
WI_NO_STATS=1
```

* Remove ctags from required docs after tree-sitter backend is default.
* Add GitHub Actions for Linux/macOS.
* Add release artifacts.
* Add install smoke tests.
* Add docs site or compact docs folder.

## Acceptance

* user can install without Rust from release artifact
* user can index without ctags
* `wi-init` creates useful agent instructions
* `wi-stats` works
* Free limits are visible and clear

---

# Phase 6 — Lemon Squeezy licensing client

## Goal

Allow paid users to unlock Pro locally using Lemon Squeezy license keys.

## Work

* Add license module:

  * read local license cache
  * write local license cache
  * hash license key before storing
  * expose edition/features
* Add Lemon Squeezy client:

  * activate
  * validate/refresh
  * deactivate
* Add command:

```bash
wi-license
```

Subcommands:

```bash
wi-license activate <license-key>
wi-license status
wi-license refresh
wi-license deactivate
```

## Enforcement

Only `build_index` enforces Free/Pro scale.

* Missing/invalid license: Free mode
* Valid Pro activation: unlimited indexed files and records
* `wi`, `wi-init`, and `wi-stats` remain available

## Tests

* missing license = Free
* invalid cache = Free with warning
* valid cached Pro = Pro
* Pro unlocks over-limit files
* Pro unlocks over-limit records
* activate stores cache
* deactivate removes/invalidates cache
* network failures do not break Free mode
* license check never reads repo source

## Acceptance

* local cached Lemon Squeezy activation works
* Free behavior works without license file
* no online dependency for normal use
* no repository contents leave the machine

---

# Phase 7 — Lemon Squeezy product setup

## Goal

Connect purchase to license issuance without building a custom licensing service.

## Work

In Lemon Squeezy:

* create product
* create Pro variant
* set one-time price around `PRO_PRICE_ONE_TIME_EUR`
* enable license keys
* configure activation limit
* configure customer emails
* test purchase flow
* test license key activation through API

In Thinindex docs:

* purchase link
* activation instructions
* status/deactivation instructions
* privacy explanation

## Activation flow

```bash
wi-license activate XXXX-XXXX-XXXX
```

Behavior:

1. calls Lemon Squeezy license activate API
2. receives activation instance
3. validates response
4. writes local cache
5. `build_index` now allows Pro limits

Refresh:

```bash
wi-license refresh
```

Deactivation:

```bash
wi-license deactivate
```

## Acceptance

* purchase produces usable license key
* activation writes local license cache
* deactivation works
* refund/revoke process is known
* docs explain that source code is never uploaded

---

# Phase 8 — Cross-platform packaging

## Goal

Make install boring.

## Targets

* Linux x86_64
* Linux arm64
* macOS arm64
* macOS x86_64
* Windows x86_64

## Deliverables

Linux/macOS:

* `.tar.gz`
* `install.sh`

Windows:

* `.zip`
* `install.ps1`
* optional later: MSI

All:

* checksums
* release notes
* smoke tests

## Install behavior

Linux/macOS:

* install to `$HOME/.local/bin` by default
* support `BIN_DIR`
* warn if not on PATH

Windows:

* install to user-local bin directory
* add PATH instruction
* avoid requiring admin
* PowerShell-friendly commands

## Code signing

Required for serious paid Windows/macOS product:

* Windows code signing cert
* macOS signing/notarization

## Acceptance

* GitHub Actions builds all targets
* downloaded binary works without Rust installed
* Windows smoke test passes:

  * `wi-init`
  * `build_index`
  * `wi`
  * `wi-stats`
  * `wi-license status`
  * `wi-init --remove`

---

# Phase 9 — Agent integration packs

## Goal

Make thinindex valuable specifically for coding-agent workflows.

## Supported agents/editors

Initial:

* Claude Code / Claude CLI
* Codex CLI
* Cursor
* GitHub Copilot instructions
* generic `AGENTS.md`

## Work

`wi-init` should support templates:

```bash
wi-init --agent generic
wi-init --agent claude
wi-init --agent codex
wi-init --agent cursor
```

Default remains generic.

Generated files:

* `WI.md`
* `AGENTS.md` reference
* optional `CLAUDE.md` reference
* optional `.cursor/rules/wi.md`
* optional Codex instruction snippet if standard location exists

## Rules

* Never overwrite without `--force`.
* Keep generated instructions short.
* Avoid hidden state.
* Explain that `build_index` should run before discovery and after structural changes.

## Acceptance

* each template has snapshot tests
* user can see exactly what is written
* no vendor-specific behavior leaks into generic mode

---

# Phase 10 — Usage stats and proof of value

## Goal

Make users see that agents are using the tool.

## Current stats

* searches
* hits
* misses
* hit ratio
* avg results
* windows:

  * 1d
  * 2d
  * 5d
  * 30d
* terminal graph

## Improve later

* Track command source if possible:

  * user
  * unknown agent
  * env var `WI_CALLER=claude|codex|cursor`
* Track stale-index retries:

  * miss followed by build followed by hit
* Add:

```bash
wi-stats --json
```

* Add stats disable option:

```bash
WI_NO_STATS=1
```

## Do not do

* Do not claim exact token savings.
* Do not upload stats.
* Do not add telemetry by default.

## Acceptance

* stats are local
* no private code content beyond query strings
* README explains query logging plainly
* user can disable stats

---

# Phase 11 — Configuration

## Goal

Stop hardcoding behavior while keeping defaults simple.

## Add optional config file

```text
.thinindex.toml
```

Example:

```toml
[backend]
default = "native"

[index]
max_file_size = "1MB"
include = ["src/**", "app/**", "frontend/**"]
exclude = ["fixtures/large/**"]
# Optional local policy caps. Can only reduce, never raise product limits.
max_indexed_files = 40
max_index_records = 800

[stats]
enabled = true
```

## Rules

* `.thinindexignore` remains for path ignores.
* `.thinindex.toml` is for behavior/config.
* Default works without config.
* Free limits cannot be increased by local config.
* Config can reduce limits, not bypass licensing.

## Acceptance

* config is optional
* invalid config gives clear errors
* `wi-init --config` can write minimal commented config
* license enforcement cannot be bypassed by config

---

# Phase 12 — Search quality

## Goal

Make `wi` better than grep for agent navigation.

## Improvements

* Ranking refinements:

  * exact name
  * camelCase/PascalCase fuzzy
  * path relevance
  * kind priority
  * recent hit usefulness
* Add query syntax without making it complex:

  * `wi Header --type function`
  * `wi Header --lang tsx`
  * maybe later: `wi kind:function Header`
* Add aliases:

  * `--type component`
  * `--type css`
* Add optional compact context later:

  * one line before/after

## Do not add

* embeddings
* daemon
* LSP dependency
* network calls

## Acceptance

* fixture search ranking tests
* query output remains small
* agents get useful file/line results quickly

---

# Phase 13 — Security and privacy

## Goal

Be safe enough for private codebases.

## Work

* Document:

  * all index/search data is local
  * what files are written
  * query strings are stored in `wi_usage.jsonl`
  * how to disable/remove stats
  * license activation contacts Lemon Squeezy but does not upload source
* Add secret-safety defaults:

  * ignore `.env*`
  * ignore key/cert files
  * ignore common credential paths
* Add `wi-init --privacy-strict` later if needed.
* Add security contact/process.

## Acceptance

* `docs/SECURITY.md`
* README privacy note
* `wi-init --remove` clearly explains what remains
* no telemetry
* no repository source upload

---

# Phase 14 — Release automation

## Goal

Ship repeatably.

## Work

* GitHub Actions:

  * format
  * clippy
  * test
  * cross-platform build
  * release artifacts
  * checksums
* Versioning:

  * SemVer
  * changelog
  * `--version` on all binaries
* Release script:

  * tag
  * build
  * publish
  * smoke test

## Acceptance

* one tag creates draft release
* artifacts are named consistently
* checksums generated
* install instructions verified on clean machine/container
* Free/Pro behavior tested

---

# Phase 15 — Website / landing page

## Goal

Explain the value quickly.

## Page sections

* Problem:

  * agents burn context reading files blindly
* Solution:

  * local index + `wi`
* Demo:

  * screenshot/GIF
* Install:

  * one command
* Free vs Pro:

  * Free: `FREE_MAX_INDEXED_FILES` files and `FREE_MAX_INDEX_RECORDS` records
  * Pro: unlimited, `PRO_PRICE_ONE_TIME_EUR` one-time
* Agent behavior:

  * search first, read less
* Privacy:

  * local only, no source upload
* Pricing:

  * Lemon Squeezy checkout
* Docs:

  * commands
  * integrations
  * ignore/config
  * licensing
  * troubleshooting

## Acceptance

* clear screenshot
* 60-second install/demo
* no vague AI marketing
* direct examples
* Lemon Squeezy checkout path works only when product is ready

---

# Phase 16 — Monetization launch

## Goal

Charge only when product is clean enough.

## Do not start here.

Before payment:

* cross-platform binaries exist
* Free limit enforcement works
* Lemon Squeezy license activation works
* native tree-sitter backend exists
* install is easy
* docs are clear
* support burden is understood

## Pro features at launch

Minimum viable paid offer:

* unlimited indexed files
* unlimited index records
* Lemon Squeezy license key activation
* local license cache
* clean installers
* native parser backend
* all 1.x updates

Do not sell vague future features as the main value.

## Launch checks

* Lemon Squeezy product configured
* test purchase works
* activation/deactivation works
* refund/revoke process exists
* support email exists
* privacy/security docs exist
* Free over-limit upgrade path is clear

## Acceptance

* paid users can install without Rust
* paid users can activate without uploading source
* paid users do not need ctags
* license failure leaves Free mode usable
* Free users understand the file/record limits

---

# Phase 17 — Team/CI product

## Goal

Make it useful for teams after local Pro works.

## Features

* `build_index --check`
* CI job:

  * index freshness
  * ignore coverage
  * oversized file warnings
  * generated files accidentally indexed
* Shared templates:

  * `WI.md`
  * `AGENTS.md`
  * `.thinindexignore`
  * `.thinindex.toml`
* Team license support later through Lemon Squeezy or a dedicated licensing service if Lemon Squeezy becomes insufficient.
* PR comment later, optional

## Acceptance

* CI can run without paid cloud
* failure messages are actionable
* no source upload

---

# Implementation plan sequence

## Plan A — Clean current utility

1. Add docs:

   * `CHANGELOG.md`
   * `docs/ARCHITECTURE.md`
   * `docs/SECURITY.md`
2. Stabilize schema versions.
3. Add `build_index --clean`.
4. Add `build_index --check`.
5. Harden malformed index handling.
6. Expand fixture tests.
7. Polish README.

## Plan B — Backend abstraction and tree-sitter

1. Add `ParserBackend` trait.
2. Move ctags behind `CtagsBackend`.
3. Add `TreeSitterBackend`.
4. Implement Python extraction.
5. Implement Rust extraction.
6. Implement JS/TS/TSX extraction.
7. Keep CSS/HTML/Markdown extras.
8. Make tree-sitter backend default.
9. Remove ctags from required install docs.
10. Decide whether to keep or delete ctags fallback.

## Plan C — Free limits

1. Add eligible indexed file counter.
2. Add generated index record counter.
3. Add edition detection stub.
4. Enforce `FREE_MAX_INDEXED_FILES`.
5. Enforce `FREE_MAX_INDEX_RECORDS`.
6. Add Pro test override.
7. Add over-limit tests.
8. Update README.
9. Ensure no partial index on failure.
10. Ensure local config cannot raise Free limits.

## Plan D — Product-ready free CLI

1. Finalize `wi-init`.
2. Finalize `.thinindexignore`.
3. Add stats disable env var.
4. Add GitHub Actions for Linux/macOS.
5. Add release artifacts.
6. Add install smoke tests.
7. Add docs site or compact docs folder.

## Plan E — Windows support

1. Fix path handling tests.
2. Add PowerShell install/uninstall.
3. Add Windows GitHub Actions.
4. Add Windows fixture tests.
5. Confirm no external parser dependency.
6. Add release ZIP.
7. Document Windows install.

## Plan F — Lemon Squeezy license client

1. Add local license cache schema.
2. Add Lemon Squeezy activation client.
3. Add `wi-license status`.
4. Add `wi-license activate`.
5. Add `wi-license refresh`.
6. Add `wi-license deactivate`.
7. Enforce Free/Pro in `build_index`.
8. Add invalid/missing/valid license tests.
9. Add docs.

## Plan G — Lemon Squeezy product setup

1. Create Lemon Squeezy product/variant.
2. Enable license keys.
3. Configure one-time price around `PRO_PRICE_ONE_TIME_EUR`.
4. Test license key purchase.
5. Test activation from CLI.
6. Add purchase/activation docs.
7. Add refund/revoke process.

## Plan H — Pro packaging

1. Define Pro build/distribution.
2. Add signed release pipeline.
3. Add installer polish.
4. Add landing page.
5. Add purchase/download flow.
6. Beta with real users.

---

# Recommended immediate next phase

Do **Plan A + Plan B + Plan C** next.

Do not start payment, website, Windows, or Lemon Squeezy integration before:

* index is robust
* docs are clean
* install/uninstall behavior is polished
* tree-sitter backend exists and is default
* Free limit enforcement is implemented cleanly
* tests cover broken/stale/malformed/over-limit repos

The immediate next implementation plan should be:

```text
Phase 0/1/2/3/4: Clean, harden, switch to tree-sitter, and add Free file/record limits.
```