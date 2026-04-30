mod common;

use std::fs;

use common::*;
use rusqlite::Connection;

fn sqlite_table_exists(root: &std::path::Path, table: &str) -> bool {
    let conn = Connection::open(root.join(".dev_index/index.sqlite")).expect("open sqlite index");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table],
            |row| row.get(0),
        )
        .expect("query sqlite_master");

    count == 1
}

fn assert_ref_exists(refs: &[thinindex::model::ReferenceRecord], ref_kind: &str, to_name: &str) {
    assert!(
        refs.iter()
            .any(|reference| reference.ref_kind == ref_kind && reference.to_name == to_name),
        "expected ref kind `{ref_kind}` to `{to_name}`, got:\n{refs:#?}"
    );
}

#[test]
fn build_creates_index_files() {
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
    assert!(root.join(".dev_index/index.sqlite").exists());
    assert!(!root.join(".dev_index/manifest.json").exists());
    assert!(!root.join(".dev_index/index.jsonl").exists());

    let records = thinindex::store::load_records(root).expect("load records");
    assert!(
        records
            .iter()
            .any(|record| record.name == "PromptService" || record.name == "top_level_function")
    );
}

#[test]
fn unchanged_files_are_skipped_on_second_build() {
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
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "docs/guide.md",
        r#"
# Guide

## Tests

Content.

See [PromptService](../src/service.py).
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

    assert!(sqlite_table_exists(root, "refs"));

    let snapshot = load_index_snapshot_from_sqlite(root);

    run_named_index_integrity_checks(
        "fixture index integrity",
        &snapshot,
        &[
            "docs/guide.md",
            "frontend/page.html",
            "frontend/styles/header.css",
            "src/service.py",
        ],
    );

    assert!(
        snapshot
            .refs
            .iter()
            .any(|reference| reference.to_name == "../src/service.py"
                && reference.ref_kind == "markdown_link"),
        "expected markdown link reference to ../src/service.py, got:\n{:#?}",
        snapshot.refs
    );
}

#[test]
fn fixture_reference_repo_extracts_plan_02_refs() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/prompt_service.py",
        r#"
class PromptService:
    pass
"#,
    );
    write_file(
        root,
        "src/consumer.py",
        r#"
from prompt_service import PromptService

def consume():
    return PromptService()
"#,
    );
    write_file(
        root,
        "src/lib.rs",
        r#"
mod prompt_service;
use crate::prompt_service::PromptService;
"#,
    );
    write_file(
        root,
        "frontend/lib/prompt-service.ts",
        r#"
export class PromptService {}
"#,
    );
    write_file(
        root,
        "frontend/components/HeaderNavigation.tsx",
        r#"
import { PromptService } from "../lib/prompt-service";

export function HeaderNavigation() {
  return <header className="headerNavigation" data-testid="header-nav">{PromptService.name}</header>;
}
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
  color: var(--paper-bg);
}
"#,
    );
    write_file(
        root,
        "frontend/page.html",
        r#"
<header id="mainHeader" class="headerNavigation" data-testid="main-header"></header>
"#,
    );
    write_file(
        root,
        "docs/guide.md",
        "[PromptService](../src/prompt_service.py)\n",
    );
    write_file(
        root,
        "tests/test_prompt_service.py",
        r#"
from prompt_service import PromptService

def test_prompt_service():
    assert PromptService()
"#,
    );

    run_build(root);

    let snapshot = load_index_snapshot_from_sqlite(root);
    run_named_index_integrity_checks("plan 02 reference fixture", &snapshot, &[]);

    let refs = snapshot.refs;
    assert_ref_exists(&refs, "import", "PromptService");
    assert_ref_exists(&refs, "import", "prompt_service");
    assert_ref_exists(&refs, "markdown_link", "../src/prompt_service.py");
    assert_ref_exists(&refs, "css_usage", ".headerNavigation");
    assert_ref_exists(&refs, "css_usage", "--paper-bg");
    assert_ref_exists(&refs, "html_usage", "#mainHeader");
    assert_ref_exists(&refs, "html_usage", ".headerNavigation");
    assert_ref_exists(&refs, "html_usage", "data-testid");
    assert_ref_exists(&refs, "test_reference", "PromptService");
}

#[test]
fn refs_are_deterministic_on_repeated_build() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/service.py", "class PromptService: pass\n");
    write_file(
        root,
        "src/use_service.py",
        "from service import PromptService\nPromptService()\n",
    );
    write_file(
        root,
        "docs/guide.md",
        "[PromptService](../src/service.py)\n",
    );

    run_build(root);
    let first = thinindex::store::load_refs(root).expect("load refs after first build");
    let second_output = run_build(root);
    let second = thinindex::store::load_refs(root).expect("load refs after second build");

    assert!(
        second_output.contains("changed files: 0"),
        "second build should skip unchanged files, got:\n{second_output}"
    );
    assert_eq!(first, second);
}

#[test]
fn changed_python_file_rewrites_stale_refs() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/service.py", "class PromptService: pass\n");
    write_file(root, "src/consumer.py", "from service import OldService\n");

    run_build(root);
    let before = thinindex::store::load_refs(root).expect("load refs before");
    assert_ref_exists(&before, "import", "OldService");

    write_file(
        root,
        "src/consumer.py",
        "from service import PromptService\n",
    );

    let second = run_build(root);
    assert!(
        second.contains("changed files: 1"),
        "expected one changed file, got:\n{second}"
    );

    let after = thinindex::store::load_refs(root).expect("load refs after");
    assert!(
        !after
            .iter()
            .any(|reference| reference.to_name == "OldService"),
        "stale Python import ref should be removed, got:\n{after:#?}"
    );
    assert_ref_exists(&after, "import", "PromptService");
}

#[test]
fn changed_files_rewrite_stale_refs() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/service.py", "class PromptService: pass\n");
    write_file(root, "docs/guide.md", "[Old](../src/old.py)\n");

    run_build(root);
    let before = thinindex::store::load_refs(root).expect("load refs before");
    assert!(
        before
            .iter()
            .any(|reference| reference.to_name == "../src/old.py"),
        "expected initial markdown ref, got:\n{before:#?}"
    );

    write_file(root, "docs/guide.md", "[New](../src/service.py)\n");

    let second = run_build(root);
    assert!(
        second.contains("changed files: 1"),
        "expected one changed file, got:\n{second}"
    );

    let after = thinindex::store::load_refs(root).expect("load refs after");
    assert!(
        !after
            .iter()
            .any(|reference| reference.to_name == "../src/old.py"),
        "stale ref should be removed after changed file rebuild, got:\n{after:#?}"
    );
    assert!(
        after
            .iter()
            .any(|reference| reference.to_name == "../src/service.py"),
        "new ref should be inserted after changed file rebuild, got:\n{after:#?}"
    );
}

#[test]
fn deleted_files_remove_refs() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/service.py", "class PromptService: pass\n");
    write_file(
        root,
        "docs/deleted.md",
        "[PromptService](../src/service.py)\n",
    );

    run_build(root);
    let before = thinindex::store::load_refs(root).expect("load refs before");
    assert!(
        before
            .iter()
            .any(|reference| reference.from_path == "docs/deleted.md"),
        "expected ref from docs/deleted.md, got:\n{before:#?}"
    );

    fs::remove_file(root.join("docs/deleted.md")).expect("delete markdown file");

    let second = run_build(root);
    assert!(
        second.contains("deleted files: 1"),
        "expected one deleted file, got:\n{second}"
    );

    let after = thinindex::store::load_refs(root).expect("load refs after");
    assert!(
        !after
            .iter()
            .any(|reference| reference.from_path == "docs/deleted.md"),
        "deleted file refs should be removed, got:\n{after:#?}"
    );
}

#[test]
fn binary_files_are_skipped() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/service.py",
        r#"
class PromptService:
    pass
"#,
    );

    // A blob with NUL bytes — the canonical "this is binary" signal.
    fs::create_dir_all(root.join("assets")).expect("create assets dir");
    fs::write(
        root.join("assets/icon.bin"),
        b"\x00\x01\x02PNG\x00\x00garbage",
    )
    .expect("write binary fixture");

    // Build must succeed even though a binary file is present.
    run_build(root);

    let records = thinindex::store::load_records(root).expect("load records");

    // The text file is indexed.
    assert!(
        records.iter().any(|record| record.path == "src/service.py"),
        "expected text file to be indexed, got:\n{records:#?}"
    );

    // The binary file is not.
    assert!(
        !records
            .iter()
            .any(|record| record.path == "assets/icon.bin"),
        "binary file must not be indexed, got:\n{records:#?}"
    );

    // File metadata also excludes it — otherwise we'd re-attempt it every build.
    let manifest = thinindex::store::load_manifest(root).expect("load manifest");
    assert!(
        !manifest.files.contains_key("assets/icon.bin"),
        "binary file must not be tracked in file metadata, got:\n{manifest:#?}"
    );
}

#[test]
fn markdown_heading_is_canonicalized_to_section() {
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
