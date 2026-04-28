# AGENTS

- After changing the index schema, correcting index bugs, or changing extraction/search semantics, increment `INDEX_SCHEMA_VERSION`.
- Run `cargo fmt`, `cargo test`, and `cargo clippy --all-targets --all-features -- -D warnings`; do not leave warnings.
- Prefer small typed helpers over boolean/stringly typed control flow.
- Use `Result` for recoverable errors; reserve panics/unwraps for tests or impossible invariants.
- Keep CLI output stable and test-visible when behavior changes.
- Add or update tests, and if needed fixtures, with every bug fix or feature.
- Keep `.rs` files shorter than 500 lines unless there is a reason not to do so. Leave a comment in the file explaining the reason.

## Repository search

Before broad repository discovery, run `build_index`, then use `wi <term>` to find file:line landmarks from the repo-local thin index. Use `wi` before grep/find/ls/Read when locating code. Read only files returned by `wi` unless the result is insufficient. If `wi` returns no useful result, rerun `build_index` once and retry before falling back to grep/find/Read. See `WI.md` for filters and examples (`-t KIND`, `-l EXT`, `-p PATH`, `-s SOURCE`, `-n N`, `-v`).
