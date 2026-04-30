use std::{fs, path::Path};

fn repo_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn cargo_deny_policy_is_committed_and_permissive_only() {
    let cargo = repo_file("Cargo.toml");
    let deny = repo_file("deny.toml");
    let makefile = repo_file("Makefile");

    assert!(
        cargo.contains("publish = false"),
        "workspace crate should be private so cargo-deny ignores the proprietary root license"
    );
    assert!(
        deny.contains("[licenses]") && deny.contains("[licenses.private]"),
        "deny.toml should configure license policy and private workspace handling"
    );
    assert!(
        deny.contains("ignore = true"),
        "deny.toml should ignore private unpublished workspace crates"
    );

    for allowed in [
        "MIT",
        "Apache-2.0",
        "Apache-2.0 WITH LLVM-exception",
        "BSD-2-Clause",
        "BSD-3-Clause",
        "ISC",
        "Zlib",
        "Unicode-3.0",
        "CC0-1.0",
        "Unlicense",
    ] {
        assert!(
            deny.contains(&format!("\"{allowed}\"")),
            "deny.toml should explicitly allow {allowed}"
        );
    }

    for forbidden in ["\"GPL", "\"AGPL", "\"LGPL", "\"MPL", "\"EPL", "\"CDDL"] {
        assert!(
            !deny.contains(forbidden),
            "deny.toml must not globally allow forbidden/review license {forbidden}"
        );
    }

    assert!(
        deny.contains("unknown-registry = \"deny\"")
            && deny.contains("unknown-git = \"deny\"")
            && deny.contains("allow-registry = [\"https://github.com/rust-lang/crates.io-index\"]"),
        "deny.toml should reject unknown dependency sources"
    );
    assert!(
        makefile.contains("license-audit:") && makefile.contains("cargo deny check licenses"),
        "Makefile should expose the documented license audit command"
    );
}

#[test]
fn audit_command_and_packaging_blockers_are_documented() {
    let readme = repo_file("README.md");
    let license_audit = repo_file("docs/LICENSE_AUDIT.md");
    let product_boundary = repo_file("docs/PRODUCT_BOUNDARY.md");
    let roadmap = repo_file("docs/ROADMAP.md");
    let release = repo_file("docs/RELEASE_CHECKLIST.md");
    let notices = repo_file("THIRD_PARTY_NOTICES");

    for (name, contents) in [
        ("README.md", readme.as_str()),
        ("docs/LICENSE_AUDIT.md", license_audit.as_str()),
        ("docs/PRODUCT_BOUNDARY.md", product_boundary.as_str()),
        ("docs/ROADMAP.md", roadmap.as_str()),
        ("docs/RELEASE_CHECKLIST.md", release.as_str()),
        ("THIRD_PARTY_NOTICES", notices.as_str()),
    ] {
        assert!(
            contents.contains("cargo deny check licenses"),
            "{name} should document the dependency license audit command"
        );
    }

    for (name, contents) in [
        ("README.md", readme.as_str()),
        ("docs/LICENSE_AUDIT.md", license_audit.as_str()),
        ("docs/PRODUCT_BOUNDARY.md", product_boundary.as_str()),
        ("docs/RELEASE_CHECKLIST.md", release.as_str()),
        ("THIRD_PARTY_NOTICES", notices.as_str()),
    ] {
        assert!(
            contents.contains("THIRD_PARTY_NOTICES"),
            "{name} should identify THIRD_PARTY_NOTICES as part of release packaging"
        );
        assert!(
            contents.contains("GPL")
                && contents.contains("AGPL")
                && (contents.contains("blocked") || contents.contains("block")),
            "{name} should state that forbidden/copyleft findings block packaging"
        );
    }

    assert!(
        !readme.contains("GPL dependencies are commercially safe")
            && !license_audit.contains("GPL dependencies are commercially safe")
            && !product_boundary.contains("GPL dependencies are commercially safe")
            && !release.contains("GPL dependencies are commercially safe"),
        "docs must not claim GPL dependencies are commercially safe"
    );
}

#[test]
fn third_party_notices_cover_runtime_parsers_and_sqlite() {
    let notices = repo_file("THIRD_PARTY_NOTICES");

    for runtime in [
        "anyhow",
        "clap",
        "ignore",
        "regex",
        "rusqlite",
        "serde",
        "serde_json",
    ] {
        assert_notice_entry(&notices, runtime, "License expression:", "Accepted reason:");
    }

    for parser in [
        "tree-sitter",
        "tree-sitter-language",
        "tree-sitter-bash",
        "tree-sitter-c",
        "tree-sitter-c-sharp",
        "tree-sitter-cpp",
        "tree-sitter-dart",
        "tree-sitter-go",
        "tree-sitter-java",
        "tree-sitter-javascript",
        "tree-sitter-kotlin-ng",
        "tree-sitter-nix",
        "tree-sitter-php",
        "tree-sitter-python",
        "tree-sitter-ruby",
        "tree-sitter-rust",
        "tree-sitter-scala",
        "tree-sitter-swift",
        "tree-sitter-typescript",
    ] {
        assert_notice_entry(
            &notices,
            parser,
            "License expression: MIT",
            "Accepted reason:",
        );
        assert_notice_entry(&notices, parser, "Upstream:", "Notice/source text:");
    }

    assert!(
        notices.contains("libsqlite3-sys")
            && notices.contains("rusqlite is configured with the `bundled` feature")
            && notices.contains("public domain"),
        "THIRD_PARTY_NOTICES should document bundled SQLite/libsqlite3-sys status"
    );
    assert!(
        notices.contains("Generated Tree-sitter parser code")
            && notices.contains("bundled through the Tree-sitter grammar crates"),
        "THIRD_PARTY_NOTICES should document generated parser code status"
    );
}

fn assert_notice_entry(notices: &str, package: &str, first: &str, second: &str) {
    let package_line = format!("- Package: {package}");
    let start = notices
        .find(&package_line)
        .unwrap_or_else(|| panic!("THIRD_PARTY_NOTICES missing package entry for {package}"));
    let rest = &notices[start..];
    let end = rest.find("\n\n").unwrap_or(rest.len());
    let section = &rest[..end];

    assert!(
        section.contains(first) && section.contains(second),
        "THIRD_PARTY_NOTICES entry for {package} should include `{first}` and `{second}`.\n{section}"
    );
}
