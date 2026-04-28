#![allow(dead_code)]

use std::{collections::HashSet, fs, path::Path, process::Command};

use assert_cmd::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

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

/// Parsed, validated fields from a single `index.jsonl` record.
#[derive(Debug, Clone)]
pub struct IndexJsonRecord {
    pub path: String,
    pub line: u64,
    pub col: u64,
    pub lang: String,
    pub kind: String,
    pub name: String,
    pub text: String,
    pub source: String,
}

/// Parse every non-empty line of `index` as JSON and validate required fields.
///
/// Panics immediately on any malformed or invalid line, including `name` in
/// the failure message so the caller knows which repo or check failed.
pub fn parse_index_jsonl(name: &str, index: &str) -> Vec<IndexJsonRecord> {
    let mut records = Vec::new();

    for (line_no, raw) in index.lines().enumerate() {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }

        let v: Value = serde_json::from_str(raw).unwrap_or_else(|e| {
            panic!("[{name}] line {line_no}: failed to parse JSON: {e}\n  line: {raw}")
        });

        // --- path: string, non-empty ---
        let path = v
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                panic!("[{name}] line {line_no}: `path` missing or not a string\n  line: {raw}")
            })
            .to_owned();
        assert!(
            !path.is_empty(),
            "[{name}] line {line_no}: `path` must not be empty\n  line: {raw}"
        );

        // --- line: integer >= 1 ---
        let line_val = v
            .get("line")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| {
                panic!(
                    "[{name}] line {line_no}: `line` missing or not a non-negative integer\n  line: {raw}"
                )
            });
        assert!(
            line_val >= 1,
            "[{name}] line {line_no}: `line` must be >= 1, got {line_val}\n  line: {raw}"
        );

        // --- col: integer >= 1 ---
        let col_val = v
            .get("col")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| {
                panic!(
                    "[{name}] line {line_no}: `col` missing or not a non-negative integer\n  line: {raw}"
                )
            });
        assert!(
            col_val >= 1,
            "[{name}] line {line_no}: `col` must be >= 1, got {col_val}\n  line: {raw}"
        );

        // --- lang: string (may be empty) ---
        let lang = v
            .get("lang")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                panic!("[{name}] line {line_no}: `lang` missing or not a string\n  line: {raw}")
            })
            .to_owned();

        // --- kind: string, non-empty ---
        let kind = v
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                panic!("[{name}] line {line_no}: `kind` missing or not a string\n  line: {raw}")
            })
            .to_owned();
        assert!(
            !kind.is_empty(),
            "[{name}] line {line_no}: `kind` must not be empty\n  line: {raw}"
        );

        // --- name: string (may be empty) ---
        let rec_name = v
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                panic!("[{name}] line {line_no}: `name` missing or not a string\n  line: {raw}")
            })
            .to_owned();

        // --- text: string (may be empty) ---
        let text = v
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                panic!("[{name}] line {line_no}: `text` missing or not a string\n  line: {raw}")
            })
            .to_owned();

        // --- source: string, non-empty ---
        let source = v
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                panic!("[{name}] line {line_no}: `source` missing or not a string\n  line: {raw}")
            })
            .to_owned();
        assert!(
            !source.is_empty(),
            "[{name}] line {line_no}: `source` must not be empty\n  line: {raw}"
        );

        records.push(IndexJsonRecord {
            path,
            line: line_val,
            col: col_val,
            lang,
            kind,
            name: rec_name,
            text,
            source,
        });
    }

    records
}

/// Assert there are no two records with the same (path, line, col) triple.
pub fn assert_no_duplicate_locations(name: &str, records: &[IndexJsonRecord]) {
    let mut seen: HashSet<(String, u64, u64)> = HashSet::new();
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
pub fn assert_no_dev_index_paths(name: &str, records: &[IndexJsonRecord]) {
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
pub fn assert_expected_paths_present(
    name: &str,
    records: &[IndexJsonRecord],
    expected_paths: &[&str],
) {
    for &expected in expected_paths {
        assert!(
            records.iter().any(|r| r.path.contains(expected)),
            "[{name}] expected path substring `{expected}` not found in any record path",
        );
    }
}

/// Run the full index-integrity check suite against raw `index.jsonl` text.
///
/// * `name` – a human-readable label (repo name, test name, …)
/// * `index` – the full text of an `index.jsonl` file
/// * `expected_paths` – optional path substrings that must appear; pass `&[]`
///   to skip that check
pub fn run_named_index_integrity_checks(name: &str, index: &str, expected_paths: &[&str]) {
    let records = parse_index_jsonl(name, index);
    assert_no_duplicate_locations(name, &records);
    assert_no_dev_index_paths(name, &records);
    assert_expected_paths_present(name, &records, expected_paths);
}
