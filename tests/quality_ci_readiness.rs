use std::{fs, path::Path};

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn local_ci_script_runs_deterministic_gates_only() {
    let script = repo_file("scripts/check-ci");

    for required in [
        "cargo fmt --check",
        "cargo test",
        "cargo test --test parser_conformance",
        "cargo test --test support_levels",
        "cargo test --test quality",
        "cargo test --test quality_gates",
        "cargo clippy --all-targets --all-features -- -D warnings",
        "cargo deny check licenses",
        "cargo run --bin wi -- --help",
        "cargo run --bin wi-stats -- --version",
    ] {
        assert!(
            script.contains(required),
            "scripts/check-ci should run `{required}`"
        );
    }

    for forbidden in ["--ignored", "test_repos", "ctags"] {
        assert!(
            !script.contains(forbidden),
            "scripts/check-ci must not require manual quality input `{forbidden}`"
        );
    }
}

#[test]
fn ci_workflow_has_fixture_quality_job_without_manual_inputs() {
    let workflow = repo_file(".github/workflows/ci.yml");

    assert!(
        workflow.contains("quality-fixtures")
            && workflow.contains("cargo test --test parser_conformance")
            && workflow.contains("cargo test --test support_levels")
            && workflow.contains("cargo test --test quality")
            && workflow.contains("cargo test --test quality_gates"),
        "CI workflow should run deterministic parser and quality fixture suites"
    );

    for forbidden in ["--ignored", "test_repos", "ctags"] {
        assert!(
            !workflow.contains(forbidden),
            "CI workflow must not require manual quality input `{forbidden}`"
        );
    }
}

#[test]
fn quality_docs_separate_ci_safe_and_manual_only_gates() {
    let quality = repo_file("docs/QUALITY.md");
    let releasing = repo_file("docs/RELEASING.md");
    let readme = repo_file("README.md");
    let makefile = repo_file("Makefile");

    for contents in [quality.as_str(), releasing.as_str(), readme.as_str()] {
        assert!(
            contents.contains("scripts/check-ci"),
            "CI parity docs should mention scripts/check-ci"
        );
    }

    assert!(
        makefile.contains("ci-check:") && makefile.contains("scripts/check-ci"),
        "Makefile should expose scripts/check-ci"
    );

    for required in [
        "## CI-Safe Gates",
        "## Manual-Only Gates",
        "cargo test --test parser_conformance",
        "cargo test --test support_levels",
        "cargo test --test quality",
        "cargo test --test quality_gates",
        "ctags allowlist gate",
        "cargo deny check licenses",
        "real-repo parser integrity",
        "optional comparator quality report",
        "quality improvement cycle",
    ] {
        assert!(
            quality.contains(required),
            "docs/QUALITY.md should document `{required}`"
        );
    }

    assert!(
        readme.contains("does not require local real repositories")
            && readme.contains("optional external comparator commands")
            && releasing.contains("does not require local real repositories")
            && releasing.contains("optional comparator commands"),
        "README and releasing docs should keep CI-safe and manual gates separated"
    );
}
