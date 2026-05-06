# PLAN_55_REFS_CONFIDENCE_REASON_OUTPUT_ALIGNMENT.md

Use superpowers:subagent-driven-development.

Do not implement this until the release-impacting gaps from `prompts/recovery/FEATURE_GAP_AUDIT.md` are fixed or explicitly waived.

Goal:
Make `wi refs <term>` as useful and honest as `wi pack` and `wi impact` by surfacing stored confidence and reason/evidence labels without making output noisy.

Scope:
Improve refs output only.

Do not add:
- new parser architecture
- package-manager resolution
- LSP/compiler integration
- MCP
- packaging
- licensing enforcement
- telemetry
- cloud behavior
- ctags production use
- JSONL canonical storage
- `WI.md`

Context:
The feature-gap audit says `ReferenceRecord` stores confidence/reason and `pack`/`impact` print reasons/confidence, but `wi refs` does not print confidence and does not clearly expose stored reason labels.

Phases:
- [x] Inspect current `wi refs` output, docs, and tests.
- [x] Identify where confidence/reason is stored but not printed.
- [x] Add compact confidence output for reference rows.
- [x] Add compact reason/evidence output for reference rows.
- [x] Ensure primary definitions remain easy to scan.
- [x] Ensure file-reference rows are distinguishable from symbol references.
- [x] Improve ranking so higher-confidence exact/local/file-reference evidence appears above heuristic/text refs.
- [x] Keep output bounded.
- [x] Update docs/help/examples.
- [x] Add/update tests.
- [x] Run verification.
- [x] Commit.

Required behavior:
- `wi refs <term>` shows confidence for reference results where available.
- `wi refs <term>` shows reason/evidence labels clearly.
- Output remains compact and bounded.
- High-confidence exact/local/file-reference evidence ranks above heuristic/text references.
- Heuristic references are labeled honestly.
- File-reference rows remain distinguishable from symbol references.
- Default output does not dump verbose internal metadata.

Tests:
- refs output includes confidence for stored reference records.
- refs output includes reason/evidence.
- refs output ranks exact/local/file-reference evidence above heuristic refs.
- refs output remains bounded.
- heuristic refs are labeled.
- file-reference evidence is distinguishable.
- docs/help examples match output.

Docs:
Update where relevant:
- `docs/REFERENCE_GRAPH.md`
- `docs/AGENT_INTEGRATION.md`
- README refs examples if present

Acceptance:
- refs confidence/reason output is aligned with pack/impact.
- output is compact and useful for agents.
- docs no longer overclaim or underdescribe refs output.
- no behavior regression in pack/impact.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Commit:
Align refs confidence output
