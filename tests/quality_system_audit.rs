use std::{fs, path::Path};

fn repo_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn quality_system_audit_covers_parser_quality_release_surfaces() {
    let audit = repo_file("docs/QUALITY_SYSTEM_AUDIT.md");

    for section in [
        "# Parser Quality System Audit",
        "## Audit Summary",
        "## Parser Framework",
        "## Language Support Claims",
        "## Quality Plugin Isolation",
        "## Real Repos",
        "## Ctags Boundary",
        "## License Audit",
        "## Docs And Release Readiness",
        "## Remaining Caveats",
    ] {
        assert!(
            audit.contains(section),
            "quality system audit should include {section}"
        );
    }

    for required in [
        "LanguageRegistry::default()",
        "docs/PARSER_MAINTENANCE.md",
        "src/support.rs",
        "docs/PARSER_SUPPORT.md",
        "docs/LANGUAGE_SUPPORT.md",
        ".dev_index/quality/",
        "docs/REAL_REPO_MANIFEST.md",
        "docs/QUALITY_CTAG_BOUNDARY.md",
        "cargo test --test quality_ctags_allowlist",
        "cargo deny check licenses",
        ".dev_index/index.sqlite",
        "pre-alpha JSONL cache files are disposable rebuild input only",
        "`WI.md` is not generated",
        "`wi --help` is the source of truth",
    ] {
        assert!(
            audit.contains(required),
            "quality system audit should document `{required}`"
        );
    }
}

#[test]
fn quality_system_docs_link_the_final_audit() {
    for path in [
        "README.md",
        "docs/QUALITY.md",
        "docs/RELEASING.md",
        "docs/RELEASE_CHECKLIST.md",
    ] {
        let contents = repo_file(path);
        assert!(
            contents.contains("docs/QUALITY_SYSTEM_AUDIT.md"),
            "{path} should link the final parser-quality audit"
        );
    }
}

#[test]
fn release_and_ci_docs_include_deterministic_quality_gates() {
    let ci = repo_file(".github/workflows/ci.yml");
    let release_checklist = repo_file("docs/RELEASE_CHECKLIST.md");
    let quality = repo_file("docs/QUALITY.md");

    for required in [
        "cargo test --test parser_conformance",
        "cargo test --test support_levels",
        "cargo test --test quality",
        "cargo test --test quality_gates",
    ] {
        assert!(ci.contains(required), "CI should run `{required}`");
        assert!(
            quality.contains(required),
            "quality docs should document `{required}`"
        );
    }

    assert!(
        release_checklist.contains("scripts/check-ci")
            && release_checklist.contains("deterministic parser/quality fixtures"),
        "release checklist should include CI-safe quality gates"
    );
}

#[test]
fn current_docs_do_not_claim_stale_parser_quality_facts() {
    let checked_docs = [
        ("README.md", repo_file("README.md")),
        ("docs/ROADMAP.md", repo_file("docs/ROADMAP.md")),
        (
            "docs/PRODUCT_BOUNDARY.md",
            repo_file("docs/PRODUCT_BOUNDARY.md"),
        ),
        ("docs/QUALITY.md", repo_file("docs/QUALITY.md")),
        (
            "docs/QUALITY_SYSTEM_AUDIT.md",
            repo_file("docs/QUALITY_SYSTEM_AUDIT.md"),
        ),
        (
            "docs/PARSER_MAINTENANCE.md",
            repo_file("docs/PARSER_MAINTENANCE.md"),
        ),
    ];

    for (path, contents) in checked_docs {
        let lowered = contents.to_ascii_lowercase();
        assert!(
            !lowered.contains("jsonl is canonical")
                && !lowered.contains("canonical jsonl")
                && !lowered.contains("jsonl storage is canonical"),
            "{path} must not claim JSONL storage is canonical"
        );
        assert!(
            !contents.contains("wi-init creates WI.md")
                && !contents.contains("wi-init generates WI.md"),
            "{path} must not claim WI.md is generated"
        );
        assert!(
            !contents.contains("Languages and formats not listed are supported")
                && !contents.contains("blocked entries are supported"),
            "{path} must not overclaim unsupported languages"
        );
    }
}
