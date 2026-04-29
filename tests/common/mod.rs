#![allow(dead_code)]

use std::{collections::HashSet, fs, path::Path, process::Command};

use assert_cmd::prelude::*;
use tempfile::TempDir;
use thinindex::model::IndexRecord;

pub fn has_ctags() -> bool {
    Command::new("ctags")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn temp_repo() -> TempDir {
    let temp = tempfile::tempdir().expect("create tempdir");
    fs::create_dir_all(temp.path().join(".git")).expect("create .git marker");
    temp
}

pub fn write_file(root: &Path, relpath: &str, text: &str) {
    let path = root.join(relpath);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }

    fs::write(path, text).expect("write test file");
}

pub fn build_index_bin() -> Command {
    Command::cargo_bin("build_index").expect("build_index binary")
}

pub fn wi_bin() -> Command {
    Command::cargo_bin("wi").expect("wi binary")
}

pub fn wi_init_bin() -> Command {
    Command::cargo_bin("wi-init").expect("wi-init binary")
}

pub fn wi_stats_bin() -> Command {
    Command::cargo_bin("wi-stats").expect("wi-stats binary")
}

pub fn run_build(root: &Path) -> String {
    let output = build_index_bin()
        .current_dir(root)
        .output()
        .expect("run build_index");

    assert!(
        output.status.success(),
        "build_index failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn run_wi(root: &Path, args: &[&str]) -> String {
    let output = wi_bin()
        .current_dir(root)
        .args(args)
        .output()
        .expect("run wi");

    assert!(
        output.status.success(),
        "wi failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn fixture_repo(name: &str) -> TempDir {
    let temp = tempfile::tempdir().expect("create tempdir");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);

    copy_dir_all(&source, temp.path()).expect("copy fixture repo");

    temp
}

fn copy_dir_all(source: &Path, target: &Path) -> std::io::Result<()> {
    fs::create_dir_all(target)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Index-integrity check framework
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct IndexSnapshot {
    pub records: Vec<IndexRecord>,
}

pub fn load_index_snapshot_from_sqlite(root: &Path) -> IndexSnapshot {
    IndexSnapshot {
        records: thinindex::store::load_records(root).unwrap_or_else(|error| {
            panic!(
                "failed to load SQLite index snapshot for {}\nerror: {error:#}",
                root.display()
            )
        }),
    }
}

pub fn assert_required_fields(name: &str, records: &[IndexRecord]) {
    for rec in records {
        assert!(
            !rec.path.is_empty(),
            "[{name}] `path` must not be empty for {rec:?}"
        );
        assert!(
            rec.line >= 1,
            "[{name}] `line` must be >= 1, got {} for {rec:?}",
            rec.line
        );
        assert!(
            rec.col >= 1,
            "[{name}] `col` must be >= 1, got {} for {rec:?}",
            rec.col
        );
        assert!(
            !rec.kind.is_empty(),
            "[{name}] `kind` must not be empty for {rec:?}"
        );
        assert!(
            !rec.source.is_empty(),
            "[{name}] `source` must not be empty for {rec:?}"
        );
    }
}

/// Assert there are no two records with the same (path, line, col) triple.
pub fn assert_no_duplicate_locations(name: &str, records: &[IndexRecord]) {
    let mut seen: HashSet<(String, usize, usize)> = HashSet::new();
    for rec in records {
        let key = (rec.path.clone(), rec.line, rec.col);
        assert!(
            seen.insert(key.clone()),
            "[{name}] duplicate location: path={} line={} col={}",
            key.0,
            key.1,
            key.2,
        );
    }
}

/// Assert that no record's `path` field contains `.dev_index`.
pub fn assert_no_dev_index_paths(name: &str, records: &[IndexRecord]) {
    for rec in records {
        assert!(
            !rec.path.contains(".dev_index"),
            "[{name}] record path contains `.dev_index`: {}",
            rec.path,
        );
    }
}

/// Assert that each string in `expected_paths` appears as a substring in at
/// least one record's `path` field.  No-op when `expected_paths` is empty.
pub fn assert_expected_paths_present(name: &str, records: &[IndexRecord], expected_paths: &[&str]) {
    for &expected in expected_paths {
        assert!(
            records.iter().any(|r| r.path.contains(expected)),
            "[{name}] expected path substring `{expected}` not found in any record path",
        );
    }
}

/// Run the full index-integrity check suite against loaded SQLite records.
///
/// * `name` – a human-readable label (repo name, test name, …)
/// * `snapshot` – records loaded from `.dev_index/index.sqlite`
/// * `expected_paths` – optional path substrings that must appear; pass `&[]`
///   to skip that check
pub fn run_named_index_integrity_checks(
    name: &str,
    snapshot: &IndexSnapshot,
    expected_paths: &[&str],
) {
    assert_required_fields(name, &snapshot.records);
    assert_no_duplicate_locations(name, &snapshot.records);
    assert_no_dev_index_paths(name, &snapshot.records);
    assert_expected_paths_present(name, &snapshot.records, expected_paths);
}
