use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::model::{IndexRecord, ReferenceRecord};

const CTAGS: &str = "ctags";
const EXPLICIT_DOC_PHRASES: &[&str] = &[
    "optional",
    "external",
    "not bundled",
    "not required",
    "not used by production indexing",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtagsAllowlistViolation {
    pub path: String,
    pub line: usize,
    pub reason: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageArtifactViolation {
    pub path: String,
    pub reason: String,
}

pub fn scan_repo_for_ctags_allowlist(root: &Path) -> Result<Vec<CtagsAllowlistViolation>> {
    let mut files = Vec::new();
    collect_scanned_files(root, root, &mut files)?;

    let mut scanned = Vec::new();
    for path in files {
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        let relpath = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        scanned.push((relpath, contents));
    }

    Ok(check_ctags_allowlist(scanned.iter().map(
        |(path, contents)| (path.as_str(), contents.as_str()),
    )))
}

pub fn check_ctags_allowlist<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Vec<CtagsAllowlistViolation> {
    let mut violations = Vec::new();

    for (path, contents) in files {
        if !contains_ctags(contents) {
            continue;
        }

        if ctags_path_allowed(path, contents) {
            continue;
        }

        for (index, line) in contents.lines().enumerate() {
            if contains_ctags(line) {
                violations.push(CtagsAllowlistViolation {
                    path: path.to_string(),
                    line: index + 1,
                    reason: forbidden_reason(path),
                    text: line.to_string(),
                });
            }
        }
    }

    violations
}

pub fn assert_no_forbidden_index_sources(
    name: &str,
    records: &[IndexRecord],
    refs: &[ReferenceRecord],
) {
    for record in records {
        assert_ne!(
            record.source, CTAGS,
            "[{name}] production record source must not be {CTAGS}: {record:?}"
        );
    }

    for reference in refs {
        assert_ne!(
            reference.source, CTAGS,
            "[{name}] production ref source must not be {CTAGS}: {reference:?}"
        );
    }
}

pub fn check_package_artifacts<'a>(
    entries: impl IntoIterator<Item = &'a str>,
) -> Vec<PackageArtifactViolation> {
    entries
        .into_iter()
        .filter(|entry| package_entry_is_ctags_artifact(entry))
        .map(|entry| PackageArtifactViolation {
            path: entry.to_string(),
            reason: "release/package artifact must not include ctags".to_string(),
        })
        .collect()
}

fn ctags_path_allowed(path: &str, contents: &str) -> bool {
    normalized_path_starts_with(path, "src/quality/")
        || normalized_path_starts_with(path, "tests/quality")
        || path == "docs/QUALITY.md"
        || normalized_path_starts_with(path, "docs/QUALITY_")
        || explicit_boundary_doc_allowed(path, contents)
}

fn explicit_boundary_doc_allowed(path: &str, contents: &str) -> bool {
    let doc_path = path == "README.md"
        || path == "THIRD_PARTY_NOTICES"
        || (normalized_path_starts_with(path, "docs/") && path != "docs/INSTALLERS.md");

    if !doc_path {
        return false;
    }

    let normalized = contents.to_ascii_lowercase();
    EXPLICIT_DOC_PHRASES
        .iter()
        .all(|phrase| normalized.contains(phrase))
}

fn forbidden_reason(path: &str) -> String {
    if path.starts_with("src/") {
        "ctags reference outside isolated quality modules".to_string()
    } else if path.starts_with("scripts/")
        || path == "install.sh"
        || path == "uninstall.sh"
        || path == "docs/INSTALLERS.md"
    {
        "ctags reference in install/release/package surface".to_string()
    } else if path.starts_with("tests/") {
        "ctags reference in non-quality test".to_string()
    } else {
        "ctags reference outside structural allowlist".to_string()
    }
}

fn package_entry_is_ctags_artifact(entry: &str) -> bool {
    Path::new(entry)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            let name = name.to_ascii_lowercase();
            name == CTAGS || name == "ctags.exe" || name.starts_with("ctags-")
        })
        .unwrap_or(false)
}

fn collect_scanned_files(root: &Path, path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if is_skipped_path(root, path) {
        return Ok(());
    }

    if path.is_file() {
        files.push(path.to_path_buf());
        return Ok(());
    }

    if !path.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let entry =
            entry.with_context(|| format!("failed to read entry under {}", path.display()))?;
        collect_scanned_files(root, &entry.path(), files)?;
    }

    Ok(())
}

fn is_skipped_path(root: &Path, path: &Path) -> bool {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.components().any(|component| {
        matches!(component, Component::Normal(name) if matches!(
            name.to_str(),
            Some(".git" | ".dev_index" | "target" | "test_repos" | "prompts" | "dist")
        ))
    })
}

fn contains_ctags(value: &str) -> bool {
    value.to_ascii_lowercase().contains(CTAGS)
}

fn normalized_path_starts_with(path: &str, prefix: &str) -> bool {
    path.replace('\\', "/").starts_with(prefix)
}
