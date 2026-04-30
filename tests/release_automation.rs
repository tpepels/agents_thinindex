use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::TempDir;

const BINARIES: &[&str] = &["wi", "build_index", "wi-init", "wi-stats"];

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn github_actions_workflows_cover_release_gates() {
    let ci = repo_file(".github/workflows/ci.yml");
    let release = repo_file(".github/workflows/release.yml");

    for workflow in [ci.as_str(), release.as_str()] {
        assert!(
            workflow.contains("cargo fmt --check")
                && workflow.contains("cargo test")
                && workflow.contains("cargo clippy --all-targets --all-features -- -D warnings")
                && workflow.contains("cargo deny check licenses"),
            "workflow should run fmt, test, clippy, and license gates"
        );
        assert!(
            workflow.contains("cargo run --bin wi -- --help")
                && workflow.contains("cargo run --bin wi -- --version")
                && workflow.contains("cargo run --bin build_index -- --version")
                && workflow.contains("cargo run --bin wi-init -- --version")
                && workflow.contains("cargo run --bin wi-stats -- --version"),
            "workflow should smoke all commands"
        );
        assert!(
            workflow.contains("scripts/package-release")
                && workflow.contains("scripts/check-package-contents")
                && workflow.contains("THIRD_PARTY_NOTICES"),
            "workflow should package and content-check release artifacts"
        );
        assert!(
            !workflow.contains("test_repos")
                && !workflow.contains("wi-init --remove")
                && !workflow.contains("actions/create-release")
                && !workflow.contains("softprops/action-gh-release"),
            "workflow should not require local real repos, mutate repos, or publish GitHub Releases"
        );
    }

    assert!(
        ci.contains("pull_request") && ci.contains("push"),
        "CI workflow should run on push and pull request"
    );
    assert!(
        release.contains("workflow_dispatch")
            && release.contains("tags:")
            && release.contains("actions/upload-artifact@v4"),
        "release workflow should be manual/tag-triggered and upload workflow artifacts"
    );
}

#[test]
fn local_release_check_runs_required_gates_without_real_repos() {
    let script = repo_file("scripts/check-release");

    assert!(
        script.contains("cargo fmt --check")
            && script.contains("cargo test")
            && script.contains("cargo clippy --all-targets --all-features -- -D warnings")
            && script.contains("cargo deny check licenses")
            && script.contains("scripts/package-release")
            && script.contains("scripts/check-package-contents"),
        "release-check script should run local release gates and package content checks"
    );
    assert!(
        !script.contains("test_repos") && !script.contains("--ignored"),
        "release-check must not require local real repos or ignored tests"
    );
}

#[test]
fn package_content_check_accepts_expected_archive() {
    let archive = make_archive(&[""]);

    Command::new(repo_root().join("scripts/check-package-contents"))
        .arg(&archive)
        .current_dir(repo_root())
        .assert_success("expected valid package archive to pass content check");
}

#[test]
fn package_content_check_rejects_missing_notices() {
    let archive = make_archive(&["missing-notices"]);

    Command::new(repo_root().join("scripts/check-package-contents"))
        .arg(&archive)
        .current_dir(repo_root())
        .assert_failure("expected archive missing THIRD_PARTY_NOTICES to fail content check");
}

#[test]
fn package_content_check_rejects_forbidden_artifacts() {
    for forbidden in ["dev-index", "test-repos", "quality-report"] {
        let archive = make_archive(&[forbidden]);

        Command::new(repo_root().join("scripts/check-package-contents"))
            .arg(&archive)
            .current_dir(repo_root())
            .assert_failure(&format!(
                "expected archive containing {forbidden} to fail content check"
            ));
    }
}

fn make_archive(options: &[&str]) -> PathBuf {
    let temp = TempDir::new().expect("create temp dir");
    let root = temp.keep();
    let package = root.join("thinindex-9.9.9-test-target");

    fs::create_dir_all(package.join("docs")).expect("create docs");

    for binary in BINARIES {
        fs::write(package.join(binary), "binary").expect("write binary");
    }

    fs::write(package.join("README.md"), "readme").expect("write readme");
    fs::write(package.join("INSTALL.md"), "install").expect("write install");
    fs::write(package.join("docs/RELEASING.md"), "releasing").expect("write releasing");
    fs::write(package.join("docs/INSTALLERS.md"), "installers").expect("write installers");
    fs::write(package.join("docs/SECURITY_PRIVACY.md"), "privacy").expect("write privacy");

    if !options.contains(&"missing-notices") {
        fs::write(package.join("THIRD_PARTY_NOTICES"), "notices").expect("write notices");
    }

    if options.contains(&"dev-index") {
        fs::create_dir_all(package.join(".dev_index")).expect("create .dev_index");
        fs::write(package.join(".dev_index/index.sqlite"), "sqlite").expect("write index");
    }

    if options.contains(&"test-repos") {
        fs::create_dir_all(package.join("test_repos")).expect("create test_repos");
        fs::write(package.join("test_repos/README.md"), "repo").expect("write test repo");
    }

    if options.contains(&"quality-report") {
        fs::write(package.join("QUALITY_REPORT.md"), "local report").expect("write report");
    }

    let archive = root.join("thinindex-9.9.9-test-target.tar.gz");
    Command::new("tar")
        .args([
            "-czf",
            archive.to_str().expect("archive path"),
            "-C",
            root.to_str().expect("root path"),
            "thinindex-9.9.9-test-target",
        ])
        .assert_success("failed to create test archive");

    archive
}

trait CommandExt {
    fn assert_success(&mut self, context: &str);
    fn assert_failure(&mut self, context: &str);
}

impl CommandExt for Command {
    fn assert_success(&mut self, context: &str) {
        let output = self
            .output()
            .unwrap_or_else(|error| panic!("{context}: failed to run command: {error}"));

        assert!(
            output.status.success(),
            "{context}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn assert_failure(&mut self, context: &str) {
        let output = self
            .output()
            .unwrap_or_else(|error| panic!("{context}: failed to run command: {error}"));

        assert!(
            !output.status.success(),
            "{context}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
