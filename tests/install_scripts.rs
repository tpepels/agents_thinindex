use std::{fs, path::Path, process::Command};

use assert_cmd::prelude::*;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

const BINARIES: &[&str] = &["build_index", "wi", "wi-init", "wi-stats"];

fn repo_file(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(name))
        .unwrap_or_else(|error| panic!("failed to read {name}: {error}"))
}

#[test]
fn install_script_installs_all_expected_binaries() {
    let contents = repo_file("install.sh");

    for binary in BINARIES {
        assert!(
            contents.contains(&format!("target/release/{binary}")),
            "install.sh should install {binary}"
        );
        assert!(
            contents.contains(&format!("$BIN_DIR/{binary}")),
            "install.sh should verify or print {binary}"
        );
        assert!(
            contents.contains(&format!("\"$BIN_DIR/{binary}\" --version")),
            "install.sh should smoke-test {binary} --version"
        );
    }
}

#[test]
fn uninstall_script_removes_all_expected_binaries() {
    let contents = repo_file("uninstall.sh");

    for binary in BINARIES {
        assert!(
            contents.contains(binary),
            "uninstall.sh should mention {binary}"
        );
    }
}

#[test]
fn install_and_uninstall_scripts_do_not_remove_repo_local_dev_index() {
    let install = repo_file("install.sh");
    let uninstall = repo_file("uninstall.sh");

    for (name, contents) in [
        ("install.sh", install.as_str()),
        ("uninstall.sh", uninstall.as_str()),
    ] {
        assert!(
            !contents.contains("rm -rf .dev_index")
                && !contents.contains("rm -r .dev_index")
                && !contents.contains("remove_dir_all"),
            "{name} must not remove repo-local .dev_index"
        );
    }

    assert!(
        uninstall.contains("does not remove repo-local files"),
        "uninstall.sh should document that repo-local files are left alone"
    );
}

#[test]
fn install_script_uses_release_build_with_bundled_sqlite_dependency() {
    let install = repo_file("install.sh");
    let cargo = repo_file("Cargo.toml");

    assert!(
        install.contains("cargo build --release"),
        "install.sh should build release binaries"
    );
    assert!(
        cargo.contains("rusqlite") && cargo.contains("bundled"),
        "Cargo.toml should keep SQLite bundled so install does not require system SQLite"
    );
}

#[test]
fn docs_do_not_reintroduce_wi_md_or_system_sqlite_requirement() {
    let readme = repo_file("README.md");
    let uninstall = repo_file("uninstall.sh");

    assert!(!readme.contains("WI.md"), "README must not mention WI.md");
    assert!(
        !uninstall.contains("WI.md"),
        "uninstall.sh must not mention WI.md"
    );
    assert!(
        readme.contains("SQLite engine is bundled"),
        "README should document bundled SQLite behavior"
    );
    assert!(
        readme.contains(".dev_index/index.sqlite") && readme.contains("disposable local cache"),
        "README should document disposable SQLite cache behavior"
    );
}

#[test]
fn all_binaries_support_version() {
    for binary in BINARIES {
        Command::cargo_bin(binary)
            .unwrap_or_else(|error| panic!("failed to locate {binary}: {error}"))
            .arg("--version")
            .assert()
            .success()
            .stdout(contains("thinindex").or(contains(*binary)));
    }
}

#[test]
fn wi_help_mentions_current_subcommands() {
    Command::cargo_bin("wi")
        .expect("locate wi")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("wi refs PromptService"))
        .stdout(contains("wi pack PromptService"))
        .stdout(contains("wi impact PromptService"))
        .stdout(contains("wi bench"));
}
