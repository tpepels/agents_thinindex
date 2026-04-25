Use superpowers:subagent-driven-development.

Implement the next thinindex feature set: repo-local WI usage stats with terminal graphs, plus template-managed thinindex ignore rules.

Goal:
- Track `wi` hits/misses over time so users can see whether agents are using `wi`.
- Add `wi-stats` with one sane default output.
- Show terminal hit/miss graphs for 1/2/5/30-day rolling windows.
- Move default ignore patterns out of `src/indexer.rs` into a template written by `wi-init`.
- Keep `.git` and `.dev_index` hardcoded as safety ignores.

Do not add cron, daemons, background services, embeddings, MCP, databases, async runtimes, network calls, SVG output, image files, plotting crates, CLI flags for `wi-stats`, or token-savings estimates.

Files to create:
- `templates/.thinindexignore`
- `src/stats.rs`
- `src/bin/wi-stats.rs`

Files to modify:
- `Cargo.toml`
- `src/lib.rs`
- `src/bin/wi.rs`
- `src/bin/wi-init.rs`
- `src/indexer.rs`
- `README.md`
- `install.sh`
- `uninstall.sh`
- `tests/integration.rs`

Required behavior:

1. Template-managed `.thinindexignore`

Create `templates/.thinindexignore` with default ignores:

```gitignore
node_modules/
.next/
dist/
build/
coverage/
playwright-report/
test-results/
__pycache__/
.pytest_cache/
.mypy_cache/
.ruff_cache/
.venv/
venv/
*.lock
*.log
*.png
*.jpg
*.jpeg
*.gif
*.webp
*.pdf
*.zip
*.tar
*.gz
*.sqlite
*.db
````

Update `wi-init`:

* Write `.thinindexignore` from the bundled template if absent.
* `--force` overwrites both `WI.md` and `.thinindexignore`.
* Keep existing behavior: write/update `WI.md`, update `AGENTS.md`, run `build_index` once.
* `--remove` removes `.dev_index` only unless `--keep-index` is passed.
* `--remove` leaves `WI.md`, `.thinindexignore`, and `AGENTS.md` alone.
* `--disable` remains an alias for `--remove`.

Update `src/indexer.rs`:

* Remove broad hardcoded ignore dirs/suffixes now covered by `.thinindexignore`.
* Keep only hardcoded safety ignores:

  * `.git`
  * `.dev_index`
* Continue using `ignore::WalkBuilder`.
* Continue respecting:

  * `.gitignore`
  * global gitignore
  * git excludes
  * `.thinindexignore`

2. WI usage stats

On every successful `wi` invocation, append one JSONL event to:

```text
.dev_index/wi_usage.jsonl
```

Append after search completes, including zero-result searches. Do not log CLI parse failures.

Use a simple event shape. Prefer Unix seconds over extra time dependencies:

```rust
pub struct UsageEvent {
    pub ts: u64,
    pub query: String,
    pub query_len: usize,
    pub result_count: usize,
    pub hit: bool,
    pub used_type: bool,
    pub used_lang: bool,
    pub used_path: bool,
    pub used_limit: bool,
    pub repo: String,
    pub indexed_files: usize,
}
```

Definitions:

* `hit = result_count > 0`
* `miss = result_count == 0`
* `indexed_files` comes from `.dev_index/manifest.json` file count if available.

Add `src/stats.rs` for:

* appending usage events
* reading usage events
* filtering by rolling windows
* aggregating hits/misses/results
* rendering compact terminal stats
* rendering ASCII hit/miss graphs

Stats must include rolling windows for:

* last 1 day
* last 2 days
* last 5 days
* last 30 days

Each window must show:

* total searches
* hits
* misses
* hit ratio
* average results per search

Do not show estimated tokens. Do not show estimated savings.

`src/stats.rs` must include:

* `render_usage_table(...) -> String`
* `render_hit_miss_graph(...) -> String`
* `ascii_bar(value, max_value, max_width) -> String`

ASCII bar rules:

* Use `#`.
* Scale bars to max width 30.
* If `value > 0`, render at least one `#`.
* If `value == 0`, render an empty string.
* Show hits and misses separately.
* Print actual counts next to bars.

3. Add `wi-stats`

Add binary:

```toml
[[bin]]
name = "wi-stats"
path = "src/bin/wi-stats.rs"
```

CLI:

```bash
wi-stats
```

No arguments or flags.

Behavior:

* Resolve repo root like `wi`.
* Read `.dev_index/wi_usage.jsonl`.
* If no usage exists, print a clear short message and exit successfully.
* Default output shows all rolling windows: 1, 2, 5, 30 days.
* Show hits, misses, hit ratio, and average results.
* Include terminal graphs. No SVG, no image files, no plotting crates.
* Include compact recent misses section, limited to the last 10 missed queries.

Output shape:

```text
WI usage

Window    Searches  Hits  Misses  Hit ratio  Avg results
1d        12        10    2       83.3%      3.2
2d        30        25    5       83.3%      2.9
5d        74        61    13      82.4%      3.1
30d       128       103   25      80.5%      3.4

Hit/miss graph
1d   H ########## 10   M ## 2
2d   H ######################### 25   M ##### 5
5d   H ############################## 61   M ###### 13
30d  H ############################## 103  M ####### 25

Recent misses
- HeaderNav
- promptRouter
- --paper-bgg
```

If no misses exist:

```text
Recent misses
None
```

4. Update `wi`

In `src/bin/wi.rs`:

* After `search()` returns, append a usage event.
* Log:

  * query
  * result_count
  * hit
  * whether `--type`, `--lang`, `--path`, `--limit` were used
  * repo root
  * indexed file count from manifest
* Stats logging failure must not make `wi` fail. Print a concise warning to stderr at most.

5. Install/uninstall

Update install/uninstall to include all four commands:

* `build_index`
* `wi`
* `wi-init`
* `wi-stats`

6. README

Keep README short and user-facing.

Document:

* problem: agents waste tokens by blind repo reads
* install prerequisites including Universal Ctags
* `wi-init`
* `build_index`
* `wi`
* `wi-stats`
* `.thinindexignore`
* what agents will do

Mention:

* `wi-init` writes `.thinindexignore`
* `.thinindexignore` uses gitignore-style rules
* normal `.gitignore` is respected
* no background updater
* `wi-stats` shows usage, hits, misses, hit ratio, average results, and terminal hit/miss graphs for 1/2/5/30-day windows

7. Tests

Add/update integration tests:

* `wi-init` writes `.thinindexignore`
* `wi-init --force` overwrites `.thinindexignore`
* `build_index` respects `.thinindexignore`
* `wi` appends `.dev_index/wi_usage.jsonl`
* a miss is logged when result count is zero
* `wi-stats` prints windows for 1d, 2d, 5d, 30d
* `wi-stats` prints total/hits/misses/hit ratio/average results
* `wi-stats` prints the hit/miss graph section
* `wi-stats` prints recent misses
* install/uninstall scripts include `wi-stats` if script contents are tested

Keep tests deterministic. Do not rely on exact wall-clock dates.

Execution rules:

* Use `rtk` for discovery.
* Before editing, list files you will modify/create.
* Implement in small phases.
* Run:

  * `cargo fmt`
  * `cargo test`
  * `cargo clippy -- -D warnings`
* Stop when all pass.