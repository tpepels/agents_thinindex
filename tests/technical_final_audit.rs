use std::{fs, path::Path};

fn repo_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn technical_final_audit_covers_relationship_navigation_layers() {
    let audit = repo_file("docs/TECHNICAL_FINAL_AUDIT.md");

    for section in [
        "# Technical Final Audit",
        "## Parser",
        "## Dependency Graph",
        "## References",
        "## Impact",
        "## Pack",
        "## Performance",
        "## Semantic Adapters",
        "## Agent Integration",
        "## Remaining Caveats",
    ] {
        assert!(
            audit.contains(section),
            "technical final audit should include {section}"
        );
    }

    for required in [
        "Tree-sitter registry",
        "SQLite `dependencies`",
        "SQLite `refs`",
        "`wi impact <term>` uses SQLite `records`, `refs`, and `dependencies`",
        "`wi pack <term>` returns a bounded, reasoned read set",
        "Adapters are optional and disabled by default",
        "`WI.md` is not generated or restored",
        "`wi --help` remains the source of truth",
        "cannot observe external grep, find, ls, or file-read activity",
    ] {
        assert!(
            audit.contains(required),
            "technical final audit should document `{required}`"
        );
    }
}

#[test]
fn technical_audit_is_linked_from_current_docs() {
    for path in [
        "README.md",
        "docs/QUALITY.md",
        "docs/QUALITY_SYSTEM_AUDIT.md",
        "docs/RELEASING.md",
        "docs/RELEASE_CHECKLIST.md",
    ] {
        let contents = repo_file(path);
        assert!(
            contents.contains("docs/TECHNICAL_FINAL_AUDIT.md"),
            "{path} should link the technical final audit"
        );
    }
}

#[test]
fn relationship_docs_do_not_contain_audited_stale_claims() {
    let stale_dependency_claim =
        ["Current CLI output remains stable", " until a later plan"].concat();
    let stale_semantic_claim = [
        "Optional future adapters may write",
        " isolated `semantic_facts`",
    ]
    .concat();
    let stale_stats_claim = ["`wi-stats` for local", " usage stats."].concat();

    for path in [
        "docs/DEPENDENCY_GRAPH.md",
        "docs/REFERENCE_GRAPH.md",
        "docs/ROADMAP.md",
    ] {
        let contents = repo_file(path);
        assert!(
            !contents.contains(&stale_dependency_claim),
            "{path} should not claim dependency edges are unused by CLI output"
        );
        assert!(
            !contents.contains(&stale_semantic_claim),
            "{path} should describe the existing semantic adapter boundary"
        );
        assert!(
            !contents.contains(&stale_stats_claim),
            "{path} should mention the current wi-stats agent workflow audit"
        );
    }
}
