use std::{fs, path::Path};

fn repo_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn team_ci_roadmap_defines_value_without_cloud_behavior() {
    let roadmap = repo_file("docs/TEAM_CI_ROADMAP.md");

    for required in [
        "free local core remains",
        "Candidate Team/CI Value",
        "Hosted Report Candidates",
        "no-source-upload mode",
        "Local Artifact Shape",
        "GitHub Actions Example",
        "Support And Update Channel Model",
        "Explicitly Out Of Scope",
        "source_upload",
        "redacted",
        "local_only",
    ] {
        assert!(
            roadmap.contains(required),
            "TEAM_CI_ROADMAP.md should mention {required}"
        );
    }

    for forbidden in [
        "hosted backend is implemented",
        "source upload is enabled by default",
        "telemetry is enabled",
        "accounts are required",
        "payment integration is active",
        "feature gates are active",
    ] {
        assert!(
            !roadmap.contains(forbidden),
            "TEAM_CI_ROADMAP.md must not claim {forbidden}"
        );
    }
}

#[test]
fn ci_integration_is_local_only_and_artifact_oriented() {
    let ci = repo_file("docs/CI_INTEGRATION.md");

    for required in [
        "local-only",
        "No-Source-Upload Mode",
        "Local Artifact Format",
        "GitHub Actions Example",
        "cargo run --bin build_index",
        "cargo run --bin wi -- doctor",
        "cargo run --bin wi -- bench",
        "does not upload source",
        "does not contact a thinindex backend",
    ] {
        assert!(
            ci.contains(required),
            "CI_INTEGRATION.md should mention {required}"
        );
    }

    for forbidden in [
        "requires test_repos",
        "requires a hosted backend",
        "uploads repository source",
        "telemetry is collected",
        "license server is required",
    ] {
        assert!(
            !ci.contains(forbidden),
            "CI_INTEGRATION.md must not claim {forbidden}"
        );
    }
}

#[test]
fn product_and_release_docs_link_team_ci_boundaries() {
    let readme = repo_file("README.md");
    let product_boundary = repo_file("docs/PRODUCT_BOUNDARY.md");
    let roadmap = repo_file("docs/ROADMAP.md");
    let package_script = repo_file("scripts/package-release");
    let package_check = repo_file("scripts/check-package-contents");

    for (name, contents) in [
        ("README.md", readme.as_str()),
        ("docs/PRODUCT_BOUNDARY.md", product_boundary.as_str()),
        ("docs/ROADMAP.md", roadmap.as_str()),
    ] {
        assert!(
            contents.contains("TEAM_CI_ROADMAP.md") && contents.contains("CI_INTEGRATION.md"),
            "{name} should link team/CI roadmap docs"
        );
        assert!(
            contents.contains("no-source-upload")
                || contents.contains("no hosted backend")
                || contents.contains("no hosted backend, source upload"),
            "{name} should keep team/CI privacy boundaries visible"
        );
    }

    for required in ["docs/TEAM_CI_ROADMAP.md", "docs/CI_INTEGRATION.md"] {
        assert!(
            package_script.contains(required) && package_check.contains(required),
            "release packaging should include/check {required}"
        );
    }
}
