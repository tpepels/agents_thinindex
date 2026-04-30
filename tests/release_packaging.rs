use std::{fs, path::Path};

const BINARIES: &[&str] = &["wi", "build_index", "wi-init", "wi-stats"];

fn repo_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn release_package_script_stages_expected_payload() {
    let script = repo_file("scripts/package-release");
    let makefile = repo_file("Makefile");

    assert!(
        script.starts_with("#!/usr/bin/env sh"),
        "release package script should be a portable shell script"
    );
    assert!(
        script.contains("cargo build --release"),
        "release package script should build release binaries"
    );

    for binary in BINARIES {
        assert!(
            script.contains(binary),
            "release package script should stage {binary}"
        );
    }

    for required in [
        "README.md",
        "INSTALL.md",
        "docs/RELEASING.md",
        "docs/INSTALLERS.md",
        "THIRD_PARTY_NOTICES",
        "scripts/install-archive-unix",
        "scripts/uninstall-archive-unix",
        "scripts/windows/install.ps1",
        "scripts/windows/uninstall.ps1",
    ] {
        assert!(
            script.contains(required),
            "release package script should include {required}"
        );
    }

    assert!(
        script.contains("LICENSE") && script.contains("[ -f \"LICENSE\" ]"),
        "release package script should include LICENSE when present"
    );
    assert!(
        script.contains(".dev_index")
            && script.contains("test_repos")
            && script.contains("target build junk")
            && script.contains("source checkout"),
        "release package script should document excluded local/source artifacts"
    );
    assert!(
        !script.to_ascii_lowercase().contains("ctags"),
        "release package script must not mention or bundle ctags"
    );
    assert!(
        !script.contains("cp -R .") && !script.contains("cp -r ."),
        "release package script should stage explicit files, not the source checkout"
    );
    assert!(
        makefile.contains("package-release:") && makefile.contains("scripts/package-release"),
        "Makefile should expose the release package script"
    );
}

#[test]
fn release_package_script_has_archive_and_checksum_logic() {
    let script = repo_file("scripts/package-release");

    assert!(
        script.contains("tar.gz") && script.contains("tar -czf"),
        "release package script should create tar.gz archives for Unix-like targets"
    );
    assert!(
        script.contains("zip") && script.contains("zip -qr"),
        "release package script should include Windows zip archive logic"
    );
    assert!(
        script.contains("sha256sum")
            && script.contains("shasum -a 256")
            && script.contains(".sha256"),
        "release package script should write SHA256 checksums"
    );
    assert!(
        script.contains("--target")
            && script.contains("--version")
            && script.contains("--out-dir")
            && script.contains("--dry-run"),
        "release package script should expose simple testable options"
    );
    assert!(
        script.contains("cargo metadata --format-version 1 --no-deps")
            && script.contains("rustc -vV"),
        "release package script should derive version and default target locally"
    );
}

#[test]
fn release_docs_describe_archive_install_and_boundaries() {
    let readme = repo_file("README.md");
    let releasing = repo_file("docs/RELEASING.md");
    let checklist = repo_file("docs/RELEASE_CHECKLIST.md");
    let roadmap = repo_file("docs/ROADMAP.md");

    for (name, contents) in [
        ("README.md", readme.as_str()),
        ("docs/RELEASING.md", releasing.as_str()),
        ("docs/RELEASE_CHECKLIST.md", checklist.as_str()),
    ] {
        assert!(
            contents.contains("scripts/package-release"),
            "{name} should document the release archive command"
        );
        assert!(
            contents.contains("THIRD_PARTY_NOTICES"),
            "{name} should state that third-party notices ship with release artifacts"
        );
        assert!(
            contents.contains(".dev_index/index.sqlite")
                || contents.contains("`.dev_index/`")
                || contents.contains(".dev_index/"),
            "{name} should state repo-local index caches are not release artifacts"
        );
        assert!(
            contents.contains("native installers")
                || contents.contains("Native installers")
                || contents.contains("installers"),
            "{name} should keep native installers as later work"
        );
    }

    assert!(
        releasing.contains("wi` or `wi.exe")
            && releasing.contains("build_index` or `build_index.exe")
            && releasing.contains("wi-init` or `wi-init.exe")
            && releasing.contains("wi-stats` or `wi-stats.exe"),
        "release docs should list every archive binary"
    );
    assert!(
        releasing.contains("scripts/install-archive-unix")
            && releasing.contains("scripts/windows/install.ps1")
            && releasing.contains("docs/INSTALLERS.md"),
        "release docs should mention archive install helpers and installer docs"
    );
    assert!(
        releasing.contains("Universal Ctags is not bundled and not required"),
        "release docs should state ctags is not bundled or required"
    );
    assert!(
        roadmap
            .contains("Release archive packaging is available through `scripts/package-release`")
            && roadmap.contains("SHA256 checksum"),
        "roadmap should describe the shipped archive packaging surface"
    );
}
