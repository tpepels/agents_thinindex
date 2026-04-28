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
fn fixture_index_passes_shared_integrity_checks() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

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
        "frontend/styles/header.css",
        r#"
:root {
  --paper-bg: white;
}

.headerNavigation {
  display: flex;
}
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

    run_named_index_integrity_checks(
        "fixture index integrity",
        &index,
        &[
            "docs/guide.md",
            "frontend/page.html",
            "frontend/styles/header.css",
            "src/service.py",
        ],
    );
}

#[test]
fn markdown_heading_is_canonicalized_to_section() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

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

    run_build(root);

    let verbose = run_wi(root, &["Tests", "-l", "md", "-v"]);

    assert!(
        verbose.contains("kind: section"),
        "markdown heading `Tests` should produce a `section` record, got:\n{verbose}"
    );
    assert!(
        !verbose.contains("kind: heading_2"),
        "markdown heading `Tests` should not surface as `heading_2`, got:\n{verbose}"
    );
}
