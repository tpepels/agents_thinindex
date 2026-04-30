use std::{fs, path::Path};

fn repo_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn security_privacy_docs_cover_sensitive_surfaces() {
    let docs = repo_file("docs/SECURITY_PRIVACY.md");

    for required in [
        ".dev_index/index.sqlite",
        ".dev_index/quality/",
        "test_repos/",
        ".thinindexignore",
        "not a secret scanner",
        "Redaction Policy",
        "Quality Reports",
        "Release Artifacts",
        "scripts/check-package-contents",
    ] {
        assert!(
            docs.contains(required),
            "security/privacy docs should mention {required}"
        );
    }
}

#[test]
fn thinindexignore_template_includes_sensitive_path_guidance() {
    let template = repo_file("templates/.thinindexignore");

    for required in [
        ".env",
        ".env.*",
        "secrets/",
        "credentials/",
        "private_keys/",
        "*.pem",
        "*.key",
    ] {
        assert!(
            template.contains(required),
            ".thinindexignore template should include {required}"
        );
    }
}

#[test]
fn release_docs_link_security_privacy_policy() {
    for path in [
        "README.md",
        "docs/RELEASING.md",
        "docs/RELEASE_CHECKLIST.md",
    ] {
        let contents = repo_file(path);
        assert!(
            contents.contains("SECURITY_PRIVACY.md"),
            "{path} should link security/privacy policy"
        );
    }
}
