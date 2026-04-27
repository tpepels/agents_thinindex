# AGENTS

- After changing the index schema, correcting index bugs, or changing extraction/search semantics, increment `INDEX_SCHEMA_VERSION`.
- Keep Rust changes idiomatic: run `cargo fmt`; run `cargo clippy -- -D warnings`; do not leave warnings.
- Prefer small typed helpers over boolean/stringly-typed control flow.
- Use `Result` for recoverable errors; reserve panics/unwraps for tests or impossible invariants.
- Keep CLI output stable and test-visible when behavior changes.
- Add or update tests with every bug fix or feature.