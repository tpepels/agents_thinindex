mod common;

use std::fs;

use common::*;

#[test]
fn index_schema_version_missing_triggers_rebuild() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/main.py",
        r#"
class FreshService:
    pass
"#,
    );

    let dev_index = root.join(".dev_index");
    fs::create_dir_all(&dev_index).unwrap();

    let manifest_path = dev_index.join("manifest.json");
    fs::write(&manifest_path, r#"{"files":{}}"#).unwrap();
    fs::write(dev_index.join("index.jsonl"), "stale\n").unwrap();
    fs::write(dev_index.join("wi_usage.jsonl"), "usage\n").unwrap();

    let output = run_build(root);

    assert!(
        output.contains("index schema changed; rebuilding .dev_index")
            || output.contains("index manifest invalid; rebuilding .dev_index"),
        "should print rebuild message, got:\n{output}"
    );

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();

    assert_eq!(
        manifest.get("schema_version").unwrap().as_u64().unwrap(),
        thinindex::model::INDEX_SCHEMA_VERSION as u64,
        "manifest should have current schema_version after rebuild"
    );

    let index = fs::read_to_string(dev_index.join("index.jsonl")).unwrap();
    assert!(
        !index.contains("stale"),
        "index.jsonl should not contain stale records:\n{index}"
    );
    assert!(
        index.contains("FreshService"),
        "index.jsonl should contain fresh rebuilt records:\n{index}"
    );

    assert!(
        !dev_index.join("wi_usage.jsonl").exists(),
        "schema reset should remove old wi_usage.jsonl"
    );
}

#[test]
fn index_schema_version_mismatch_triggers_rebuild() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/main.py",
        r#"
class FreshService:
    pass
"#,
    );

    let dev_index = root.join(".dev_index");
    fs::create_dir_all(&dev_index).unwrap();

    let manifest_path = dev_index.join("manifest.json");
    fs::write(&manifest_path, r#"{"schema_version":999,"files":{}}"#).unwrap();
    fs::write(dev_index.join("index.jsonl"), "stale\n").unwrap();

    let output = run_build(root);

    assert!(
        output.contains("index schema changed; rebuilding .dev_index"),
        "should print schema changed message, got:\n{output}"
    );

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();

    assert_eq!(
        manifest.get("schema_version").unwrap().as_u64().unwrap(),
        thinindex::model::INDEX_SCHEMA_VERSION as u64,
        "manifest should have current schema_version after rebuild"
    );

    let index = fs::read_to_string(dev_index.join("index.jsonl")).unwrap();
    assert!(
        !index.contains("stale"),
        "index.jsonl should not contain stale records:\n{index}"
    );
    assert!(
        index.contains("FreshService"),
        "index.jsonl should contain fresh rebuilt records:\n{index}"
    );
}

#[test]
fn index_schema_version_reset_removes_usage_log() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/main.py", "class FreshService: pass\n");

    let dev_index = root.join(".dev_index");
    fs::create_dir_all(&dev_index).unwrap();

    fs::write(
        dev_index.join("manifest.json"),
        r#"{"schema_version":999,"files":{}}"#,
    )
    .unwrap();
    fs::write(dev_index.join("index.jsonl"), "stale\n").unwrap();
    fs::write(dev_index.join("wi_usage.jsonl"), "usage\n").unwrap();

    let output = run_build(root);

    assert!(
        output.contains("index schema changed; rebuilding .dev_index"),
        "expected schema rebuild message, got:\n{output}"
    );

    let index = fs::read_to_string(dev_index.join("index.jsonl")).unwrap();
    assert!(!index.contains("stale"));
    assert!(index.contains("FreshService"));

    assert!(
        !dev_index.join("wi_usage.jsonl").exists(),
        "schema reset should remove old wi_usage.jsonl"
    );
}

#[test]
fn index_schema_version_no_rebuild_when_same() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/main.py",
        r#"
class FreshService:
    pass
"#,
    );

    run_build(root);
    let output = run_build(root);

    assert!(
        !output.contains("index schema changed; rebuilding .dev_index"),
        "should not print schema changed message on same version, got:\n{output}"
    );
    assert!(
        output.contains("changed files: 0"),
        "second build should not reindex unchanged files, got:\n{output}"
    );
}

#[test]
fn malformed_manifest_rebuilds_cleanly_and_removes_dev_index() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/main.py", "class FreshService: pass\n");

    let dev_index = root.join(".dev_index");
    fs::create_dir_all(&dev_index).unwrap();
    fs::write(dev_index.join("manifest.json"), "not json").unwrap();
    fs::write(dev_index.join("index.jsonl"), "stale\n").unwrap();
    fs::write(dev_index.join("wi_usage.jsonl"), "usage\n").unwrap();

    let output = run_build(root);

    assert!(
        output.contains("rebuilding .dev_index"),
        "expected rebuild message for malformed manifest, got:\n{output}"
    );

    let index = fs::read_to_string(dev_index.join("index.jsonl")).unwrap();
    assert!(!index.contains("stale"));
    assert!(index.contains("FreshService"));

    assert!(
        !dev_index.join("wi_usage.jsonl").exists(),
        "malformed manifest rebuild should remove old wi_usage.jsonl"
    );
}
