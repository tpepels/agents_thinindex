mod common;

use std::fs;

use common::*;
use rusqlite::{Connection, params};

fn sqlite_path(root: &std::path::Path) -> std::path::PathBuf {
    root.join(".dev_index/index.sqlite")
}

fn schema_version(root: &std::path::Path) -> u32 {
    let conn = Connection::open(sqlite_path(root)).expect("open sqlite index");
    let value: String = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .expect("read schema_version");

    value.parse().expect("schema_version should parse")
}

fn force_schema_version(root: &std::path::Path, version: u32) {
    let conn = Connection::open(sqlite_path(root)).expect("open sqlite index");
    conn.execute(
        "UPDATE meta SET value = ?1 WHERE key = 'schema_version'",
        params![version.to_string()],
    )
    .expect("update schema_version");
}

#[test]
fn old_jsonl_storage_triggers_rebuild() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/main.py", "class FreshService: pass\n");

    let dev_index = root.join(".dev_index");
    fs::create_dir_all(&dev_index).unwrap();
    fs::write(dev_index.join("manifest.json"), r#"{"files":{}}"#).unwrap();
    fs::write(dev_index.join("index.jsonl"), "stale\n").unwrap();
    fs::write(dev_index.join("wi_usage.jsonl"), "usage\n").unwrap();

    let output = run_build(root);

    assert!(
        output.contains("old index storage found; rebuilding .dev_index"),
        "should print old-storage rebuild message, got:\n{output}"
    );
    assert!(sqlite_path(root).exists());
    assert!(!dev_index.join("manifest.json").exists());
    assert!(!dev_index.join("index.jsonl").exists());
    assert!(!dev_index.join("wi_usage.jsonl").exists());
    assert_eq!(schema_version(root), thinindex::model::INDEX_SCHEMA_VERSION);

    let records = thinindex::store::load_records(root).unwrap();
    assert!(records.iter().any(|record| record.name == "FreshService"));
}

#[test]
fn sqlite_schema_version_missing_triggers_rebuild() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/main.py", "class FreshService: pass\n");
    run_build(root);

    let conn = Connection::open(sqlite_path(root)).expect("open sqlite index");
    conn.execute("DELETE FROM meta WHERE key = 'schema_version'", [])
        .expect("delete schema_version");

    let output = run_build(root);

    assert!(
        output.contains("index schema changed; rebuilding .dev_index")
            || output.contains("index database invalid; rebuilding .dev_index"),
        "should print rebuild message, got:\n{output}"
    );
    assert_eq!(schema_version(root), thinindex::model::INDEX_SCHEMA_VERSION);

    let records = thinindex::store::load_records(root).unwrap();
    assert!(records.iter().any(|record| record.name == "FreshService"));
}

#[test]
fn sqlite_schema_version_mismatch_triggers_rebuild() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/main.py", "class FreshService: pass\n");
    run_build(root);
    force_schema_version(root, 999);

    let output = run_build(root);

    assert!(
        output.contains("index schema changed; rebuilding .dev_index"),
        "should print schema changed message, got:\n{output}"
    );
    assert_eq!(schema_version(root), thinindex::model::INDEX_SCHEMA_VERSION);

    let records = thinindex::store::load_records(root).unwrap();
    assert!(records.iter().any(|record| record.name == "FreshService"));
}

#[test]
fn wi_schema_version_mismatch_auto_rebuilds_and_continues_query() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/main.py", "class FreshService: pass\n");
    run_build(root);
    force_schema_version(root, 999);

    let output = wi_bin()
        .current_dir(root)
        .arg("FreshService")
        .output()
        .expect("run wi with schema-stale index");

    assert!(
        output.status.success(),
        "wi should auto-rebuild schema-stale index\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("src/main.py:1 class FreshService"),
        "original query should continue after schema rebuild, got:\n{stdout}"
    );
    assert_eq!(schema_version(root), thinindex::model::INDEX_SCHEMA_VERSION);
    assert!(
        stderr.contains("index schema version 999 does not match")
            && stderr.contains("running `build_index` once")
            && stderr.contains("index schema changed; rebuilding .dev_index"),
        "schema-stale wi should explain one-shot auto-rebuild, got:\n{stderr}"
    );
}

#[test]
fn sqlite_schema_version_reset_removes_usage_events() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/main.py", "class FreshService: pass\n");
    run_build(root);
    run_wi(root, &["FreshService"]);
    assert!(
        !thinindex::stats::read_usage_events(root)
            .unwrap()
            .is_empty()
    );

    force_schema_version(root, 999);
    let output = run_build(root);

    assert!(
        output.contains("index schema changed; rebuilding .dev_index"),
        "expected schema rebuild message, got:\n{output}"
    );
    assert!(
        thinindex::stats::read_usage_events(root)
            .unwrap()
            .is_empty()
    );

    let records = thinindex::store::load_records(root).unwrap();
    assert!(records.iter().any(|record| record.name == "FreshService"));
}

#[test]
fn index_schema_version_no_rebuild_when_same() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/main.py", "class FreshService: pass\n");

    run_build(root);
    let output = run_build(root);

    assert!(
        !output.contains("rebuilding .dev_index"),
        "should not print rebuild message on same version, got:\n{output}"
    );
    assert!(
        output.contains("changed files: 0"),
        "second build should not reindex unchanged files, got:\n{output}"
    );
}

#[test]
fn malformed_sqlite_rebuilds_cleanly() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/main.py", "class FreshService: pass\n");

    let dev_index = root.join(".dev_index");
    fs::create_dir_all(&dev_index).unwrap();
    fs::write(sqlite_path(root), "not sqlite").unwrap();

    let output = run_build(root);

    assert!(
        output.contains("index database invalid; rebuilding .dev_index"),
        "expected rebuild message for malformed sqlite, got:\n{output}"
    );
    assert_eq!(schema_version(root), thinindex::model::INDEX_SCHEMA_VERSION);

    let records = thinindex::store::load_records(root).unwrap();
    assert!(records.iter().any(|record| record.name == "FreshService"));
}
