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
fn docs_describe_current_agent_navigation_storage_and_commands() {
    let readme = repo_file("README.md");
    let roadmap = repo_file("docs/ROADMAP.md");
    let uninstall = repo_file("uninstall.sh");

    assert!(
        readme.contains("local agent-navigation layer"),
        "README should position thinindex as an agent-navigation layer"
    );
    assert!(
        readme.contains("wi --help"),
        "README should direct users to wi --help"
    );
    assert!(
        readme.contains("wi pack <term>") && readme.contains("wi impact <term>"),
        "README should document pack and impact commands"
    );
    assert!(
        readme.contains("wi bench"),
        "README should document the benchmark command"
    );
    assert!(
        readme.contains(".dev_index/index.sqlite") && readme.contains("disposable local cache"),
        "README should document disposable SQLite cache behavior"
    );
    assert!(
        roadmap.contains(".dev_index/index.sqlite"),
        "roadmap should describe SQLite as current storage"
    );
    assert!(
        !readme.contains("faster grep") && !roadmap.contains("faster grep"),
        "docs should not position thinindex as faster grep"
    );
    assert!(
        !readme.contains("ML prediction") && !roadmap.contains("ML prediction"),
        "docs should not claim ML prediction"
    );
    assert!(
        !uninstall.contains("WI.md"),
        "uninstall.sh must not mention WI.md"
    );
    assert!(
        readme.contains("SQLite engine is bundled"),
        "README should document bundled SQLite behavior"
    );
}

#[test]
fn docs_do_not_describe_legacy_files_as_current_instruction_or_storage() {
    let readme = repo_file("README.md");
    let roadmap = repo_file("docs/ROADMAP.md");

    for (name, contents) in [("README.md", readme), ("docs/ROADMAP.md", roadmap)] {
        assert!(
            !contents.contains("@WI.md")
                && !contents.contains("WI.md /")
                && !contents.contains("WI.md integration")
                && !contents.contains("See WI.md")
                && !contents.contains("See `WI.md`"),
            "{name} must not describe WI.md as a generated/current instruction surface"
        );
        assert!(
            !contents.contains(".dev_index/index.jsonl")
                && !contents.contains(".dev_index/manifest.json")
                && !contents.contains(".dev_index/wi_usage.jsonl"),
            "{name} must not describe JSONL files as current canonical storage"
        );
    }
}

#[test]
fn docs_describe_self_contained_parser_and_release_audit_boundary() {
    let readme = repo_file("README.md");
    let roadmap = repo_file("docs/ROADMAP.md");
    let release = repo_file("docs/RELEASE_CHECKLIST.md");
    let product_boundary = repo_file("docs/PRODUCT_BOUNDARY.md");

    assert!(
        readme.contains("Indexing is self-contained")
            && readme.contains("does not require an external parser command"),
        "README should describe self-contained parser behavior"
    );
    assert!(
        roadmap.contains("Indexing uses native Rust parser code")
            && roadmap.contains("No external parser command is required"),
        "roadmap should describe the native parser path as current"
    );
    assert!(
        readme.contains("Native Rust parsing is supported")
            && roadmap.contains("Rust native parsing currently covers")
            && product_boundary.contains("introduces no third-party parser or grammar dependency"),
        "docs should describe Rust native parser support and dependency status"
    );
    assert!(
        product_boundary.contains("permissively licensed and audited")
            && product_boundary.contains("dependency license audit coverage"),
        "product boundary should preserve release audit blockers"
    );
    assert!(
        release.contains("dependency license audit coverage")
            && release.contains("Smoke-test generated artifacts"),
        "release checklist should require release audit and artifact smoke tests"
    );
}

#[test]
fn product_boundary_protects_local_core_and_defers_paid_systems() {
    let readme = repo_file("README.md");
    let roadmap = repo_file("docs/ROADMAP.md");
    let product_boundary = repo_file("docs/PRODUCT_BOUNDARY.md");

    assert!(
        product_boundary.contains("local indexing")
            && product_boundary.contains("`wi <term>`")
            && product_boundary.contains("`wi --help`")
            && product_boundary.contains("no-network local operation"),
        "product boundary should protect the local/free core"
    );
    assert!(
        product_boundary.contains("Do not charge for basic local search or basic repo indexing"),
        "product boundary should prohibit charging for basic local search/indexing"
    );
    assert!(
        product_boundary
            .contains("These are roadmap candidates. They are not active feature gates"),
        "candidate Pro features should not be documented as active gates"
    );
    assert!(
        product_boundary.contains("license enforcement")
            && product_boundary.contains("payments")
            && product_boundary.contains("network calls")
            && product_boundary.contains("feature lockouts"),
        "product boundary should state paid/network/gating systems are not implemented"
    );
    assert!(
        readme.contains("currently a local/free tool")
            && readme.contains("There is no license enforcement")
            && readme.contains("Future Pro candidates")
            && readme.contains("candidates, not current restrictions"),
        "README should frame Pro as deferred candidates, not current gates"
    );
    assert!(
        roadmap.contains("No payment, account, license enforcement")
            && roadmap.contains("Future paid work is documented"),
        "roadmap should keep monetization separate from shipped behavior"
    );
    assert!(
        !readme.contains("paid features are currently gated")
            && !roadmap.contains("paid features are currently gated")
            && !product_boundary.contains("paid features are currently gated"),
        "docs must not claim current paid feature gating"
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
