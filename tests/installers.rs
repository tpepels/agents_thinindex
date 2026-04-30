use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::TempDir;

const UNIX_INSTALL: &str = "scripts/install-archive-unix";
const UNIX_UNINSTALL: &str = "scripts/uninstall-archive-unix";
const WINDOWS_INSTALL: &str = "scripts/windows/install.ps1";
const WINDOWS_UNINSTALL: &str = "scripts/windows/uninstall.ps1";
const BINARIES: &[&str] = &["wi", "build_index", "wi-init", "wi-stats"];

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn archive_installers_cover_all_binaries_without_repo_mutation() {
    let unix_install = repo_file(UNIX_INSTALL);
    let unix_uninstall = repo_file(UNIX_UNINSTALL);
    let windows_install = repo_file(WINDOWS_INSTALL);
    let windows_uninstall = repo_file(WINDOWS_UNINSTALL);

    for binary in BINARIES {
        assert!(
            unix_install.contains(binary),
            "{UNIX_INSTALL} should install {binary}"
        );
        assert!(
            unix_uninstall.contains(binary),
            "{UNIX_UNINSTALL} should uninstall {binary}"
        );
        assert!(
            windows_install.contains(&format!("{binary}.exe")),
            "{WINDOWS_INSTALL} should install {binary}.exe"
        );
        assert!(
            windows_uninstall.contains(&format!("{binary}.exe")),
            "{WINDOWS_UNINSTALL} should uninstall {binary}.exe"
        );
    }

    for (path, contents) in [
        (UNIX_INSTALL, unix_install.as_str()),
        (UNIX_UNINSTALL, unix_uninstall.as_str()),
        (WINDOWS_INSTALL, windows_install.as_str()),
        (WINDOWS_UNINSTALL, windows_uninstall.as_str()),
    ] {
        assert!(
            contents.contains("THIRD_PARTY_NOTICES") || path.contains("uninstall"),
            "{path} should either reference notices or be uninstall-only"
        );
        assert!(
            !contents.contains("wi-init --remove") || path.contains("uninstall"),
            "{path} must not auto-run wi-init against user repositories"
        );
        assert!(
            !contents.contains("build_index")
                || contents.contains("--version")
                || path.contains("uninstall"),
            "{path} should not run build_index except for version smoke"
        );
        assert!(
            !contents.contains("rm -rf .dev_index")
                && !contents.contains("rm -r .dev_index")
                && !contents.contains("Remove-Item -Recurse .dev_index"),
            "{path} must not delete repo-local .dev_index"
        );
    }
}

#[test]
fn installer_docs_are_honest_about_platform_status_and_signing() {
    let installers = repo_file("docs/INSTALLERS.md");
    let releasing = repo_file("docs/RELEASING.md");
    let readme = repo_file("README.md");
    let checklist = repo_file("docs/RELEASE_CHECKLIST.md");

    for platform in ["Windows", "macOS", "Linux"] {
        assert!(
            installers.contains(platform),
            "installer docs should describe {platform} status"
        );
    }

    for binary in BINARIES {
        assert!(
            installers.contains(binary),
            "installer docs should mention {binary}"
        );
    }

    assert!(
        installers.contains("THIRD_PARTY_NOTICES")
            && releasing.contains("THIRD_PARTY_NOTICES")
            && checklist.contains("THIRD_PARTY_NOTICES"),
        "installer/release docs should require notices with artifacts"
    );
    assert!(
        installers.contains("Authenticode signing is not implemented")
            && installers.contains("Developer ID signing is not implemented")
            && installers.contains("Notarization and stapling are not implemented")
            && installers.contains("Linux package signing is not implemented"),
        "installer docs should not claim signing/notarization is complete"
    );
    assert!(
        installers.contains("scripts/sign-release-artifact")
            && installers.contains("THININDEX_WINDOWS_CERT_PATH")
            && installers.contains("THININDEX_APPLE_NOTARY_PROFILE")
            && installers.contains("THININDEX_LINUX_GPG_KEY_ID"),
        "installer docs should document the signing scaffold and environment placeholders"
    );
    assert!(
        installers.contains("MSI")
            && installers.contains("MSIX")
            && installers.contains("`.pkg`")
            && installers.contains("`.deb`")
            && installers.contains("`.rpm`")
            && installers.contains("AppImage"),
        "installer docs should be explicit about platform package scaffolds"
    );
    assert!(
        installers.contains("No signing certificates, private keys, or secrets are committed")
            && installers
                .contains("Unsigned binaries may trigger Microsoft Defender SmartScreen warnings")
            && installers.contains("Unsigned binaries may be quarantined by Gatekeeper"),
        "installer docs should disclose unsigned platform trust caveats"
    );
    assert!(
        !installers.contains("production-ready")
            && !installers.contains("signing is complete")
            && !installers.contains("notarization is complete")
            && !readme.contains("signing is complete"),
        "docs must not overclaim installer signing readiness"
    );
}

#[test]
fn signing_scaffold_fails_clearly_without_secret_environment() {
    let temp = TempDir::new().expect("create temp dir");
    let artifact = temp
        .path()
        .join("thinindex-9.9.9-x86_64-pc-windows-msvc.zip");
    fs::write(&artifact, "archive").expect("write artifact");

    let output = Command::new(repo_root().join("scripts/sign-release-artifact"))
        .args([
            "--platform",
            "windows",
            "--artifact",
            artifact.to_str().expect("artifact path"),
        ])
        .env_remove("THININDEX_WINDOWS_CERT_PATH")
        .env_remove("THININDEX_WINDOWS_CERT_PASSWORD")
        .env_remove("THININDEX_WINDOWS_TIMESTAMP_URL")
        .output()
        .expect("run signing scaffold");

    assert!(
        !output.status.success(),
        "signing scaffold should fail without signing environment"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing required environment variable: THININDEX_WINDOWS_CERT_PATH"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn no_signing_secret_material_is_committed() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_repo_files(root, &mut files);

    for path in files {
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let name = rel
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        assert!(
            !name.ends_with(".p12")
                && !name.ends_with(".pfx")
                && !name.ends_with(".key")
                && !name.ends_with(".pem")
                && !name.ends_with(".mobileprovision"),
            "signing secret/certificate-like file should not be committed: {}",
            rel.display()
        );

        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };

        let markers = [
            format!("-----BEGIN {}-----", "PRIVATE KEY"),
            format!("-----BEGIN RSA {}-----", "PRIVATE KEY"),
            format!("-----BEGIN EC {}-----", "PRIVATE KEY"),
            format!("-----BEGIN OPENSSH {}-----", "PRIVATE KEY"),
            format!("{}=", "PFX_PASSWORD"),
            format!("{}=", "APPLE_ID_PASSWORD"),
        ];

        for marker in markers {
            assert!(
                !contents.contains(&marker),
                "signing secret marker `{marker}` should not appear in {}",
                rel.display()
            );
        }
    }
}

fn collect_repo_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("failed to read {dir:?}: {error}"))
    {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read directory entry: {error}"));
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();

        if path.is_dir() {
            if matches!(
                file_name,
                ".git" | ".dev_index" | "target" | "dist" | "test_repos"
            ) {
                continue;
            }
            collect_repo_files(&path, files);
        } else {
            files.push(path);
        }
    }
}
