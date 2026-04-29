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
fn docs_state_ctags_is_external_and_blocks_proprietary_packaging() {
    let readme = repo_file("README.md");
    let roadmap = repo_file("docs/ROADMAP.md");
    let release = repo_file("docs/RELEASE_CHECKLIST.md");

    assert!(
        readme.contains("external user-installed dependency")
            && readme.contains("must not bundle Universal Ctags"),
        "README should describe ctags as external-only"
    );
    assert!(
        readme.contains("Proprietary Windows/macOS/Linux packages are blocked")
            && readme.contains("permissively licensed"),
        "README should document native permissive parser packaging blocker"
    );
    assert!(
        roadmap.contains("Proprietary cross-platform packages are blocked")
            && roadmap.contains("permissively licensed bundled parser dependencies"),
        "roadmap should preserve ctags/native-parser packaging blocker"
    );
    assert!(
        release.contains("Do not bundle Universal Ctags")
            && release.contains(
                "blocked until the native parser work removes the ctags runtime dependency"
            ),
        "release checklist should prohibit bundled ctags in proprietary release artifacts"
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
