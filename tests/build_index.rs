mod common;

use std::fs;

use common::*;

#[test]
fn build_creates_index_files() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "app/services/prompt_service.py",
        r#"
class PromptService:
    def build_prompt(self):
        return "ok"

def top_level_function():
    return 1
"#,
    );

    run_build(root);

    assert!(root.join(".dev_index").exists());
    assert!(root.join(".dev_index/manifest.json").exists());
    assert!(root.join(".dev_index/index.jsonl").exists());

    let index = fs::read_to_string(root.join(".dev_index/index.jsonl")).expect("read index");
    assert!(index.contains("PromptService") || index.contains("top_level_function"));
}

#[test]
fn unchanged_files_are_skipped_on_second_build() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "app/services/prompt_service.py",
        r#"
class PromptService:
    pass
"#,
    );

    run_build(root);
    let second = run_build(root);

    assert!(
        second.contains("changed files: 0"),
        "expected unchanged second build, got:\n{second}"
    );
}

#[test]
fn changed_files_are_reindexed() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "app/services/prompt_service.py",
        r#"
class PromptService:
    pass
"#,
    );

    run_build(root);

    write_file(
        root,
        "app/services/prompt_service.py",
        r#"
class PromptService:
    pass

class RankingService:
    pass
"#,
    );

    let second = run_build(root);
    assert!(
        second.contains("changed files: 1"),
        "expected one changed file, got:\n{second}"
    );

    let stdout = run_wi(root, &["RankingService"]);
    assert!(stdout.contains("RankingService"));
}

#[test]
fn deleted_files_are_removed_from_index() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    let relpath = "app/services/deleted_service.py";

    write_file(
        root,
        relpath,
        r#"
class DeletedService:
    pass
"#,
    );

    run_build(root);

    let before = run_wi(root, &["DeletedService"]);
    assert!(before.contains("DeletedService"));

    fs::remove_file(root.join(relpath)).expect("delete file");

    let second = run_build(root);
    assert!(
        second.contains("deleted files: 1"),
        "expected one deleted file, got:\n{second}"
    );

    let after = run_wi(root, &["DeletedService"]);
    assert!(
        after.trim().is_empty(),
        "deleted symbol should not remain in index:\n{after}"
    );
}
#[test]
fn build_index_does_not_emit_duplicate_records_at_same_location() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    use std::collections::BTreeSet;

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "docs/guide.md",
        r#"
# Guide

## Tests

Content.
"#,
    );

    write_file(
        root,
        "frontend/page.html",
        r#"
<header id="mainHeader" class="siteHeader sticky" data-testid="main-header">
  Hello
</header>
"#,
    );

    write_file(
        root,
        "src/service.py",
        r#"
class PromptService:
    pass

def build_prompt():
    return "ok"
"#,
    );

    run_build(root);

    let index = fs::read_to_string(root.join(".dev_index/index.jsonl")).expect("read index");

    let mut seen = BTreeSet::new();
    let mut duplicates = Vec::new();

    for line in index.lines().filter(|line| !line.trim().is_empty()) {
        let record: serde_json::Value =
            serde_json::from_str(line).expect("parse index record JSON");

        let key = (
            record
                .get("path")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string(),
            record
                .get("line")
                .and_then(|value| value.as_u64())
                .unwrap_or(0),
            record
                .get("col")
                .and_then(|value| value.as_u64())
                .unwrap_or(0),
        );

        if !seen.insert(key) {
            duplicates.push(line.to_string());
        }
    }

    assert!(
        duplicates.is_empty(),
        "index should not contain duplicate path+line+col records:\n{}",
        duplicates.join("\n")
    );
}
