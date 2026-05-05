use std::{fs, path::Path};

const REPOSITORY_SEARCH_BLOCK: &str = "\
## Repository search

- Use `wi <term>` directly before grep/find/ls/Read for repository discovery; `wi` auto-builds or auto-rebuilds a missing/stale index once before searching.
- Use `wi refs <term>` before broad reference searches.
- Use `wi pack <term>` before implementation to get a compact read set.
- Use `wi impact <term>` before edits to find related tests/docs/callers.
- Use `wi --help` for filters, examples, subcommands, and command details.
- Read only files returned by `wi` unless the result is insufficient.
- Run `build_index` manually only when you want an explicit rebuild or when `wi` reports that auto-build failed.
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
        "integrations/agents/cursor/thinindex.mdc",
        "integrations/agents/copilot/copilot-instructions.md",
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
        "integrations/agents/cursor/README.md",
        "integrations/agents/copilot/README.md",
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
