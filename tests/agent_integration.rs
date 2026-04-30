use std::{fs, path::Path};

const REPOSITORY_SEARCH_BLOCK: &str = "\
## Repository search

- Before broad repository discovery, run `build_index`.
- Run `wi --help` if you need search filters, examples, or subcommands.
- Use `wi <term>` before grep/find/ls/Read to locate code.
- For implementation work, prefer `wi pack <term>` to get a compact read set.
- Before editing a symbol or feature area, run `wi impact <term>` to find related tests/docs/callers.
- Read only files returned by `wi` unless the result is insufficient.
- If `wi` returns no useful result, rerun `build_index` once and retry.
- Fall back to grep/find/Read only after that retry fails.";

fn repo_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn integration_packs_include_canonical_repository_search_block() {
    for path in [
        "integrations/agents/codex/AGENTS.md",
        "integrations/agents/claude/CLAUDE.md",
    ] {
        let text = repo_file(path);
        assert!(
            text.contains(REPOSITORY_SEARCH_BLOCK),
            "{path} must include the canonical Repository search block, got:\n{text}"
        );
    }
}

#[test]
fn agent_integration_docs_are_local_and_advisory() {
    for path in [
        "docs/AGENT_INTEGRATION.md",
        "integrations/agents/generic/README.md",
        "integrations/agents/mcp/README.md",
    ] {
        let text = repo_file(path);
        assert!(
            text.contains("local") || text.contains("Local"),
            "{path} should describe local-only behavior, got:\n{text}"
        );
        assert!(
            text.contains("advisory") || text.contains("cannot detect"),
            "{path} should avoid claiming hard enforcement, got:\n{text}"
        );
        assert!(
            !text.contains("required telemetry") && !text.contains("hosted telemetry"),
            "{path} should not introduce required telemetry behavior, got:\n{text}"
        );
    }
}

#[test]
fn wi_md_is_not_reintroduced() {
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR")).join("WI.md").exists(),
        "WI.md must not be reintroduced"
    );
}
