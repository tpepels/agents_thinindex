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

fn assert_dependency(
    dependencies: &[thinindex::model::DependencyEdge],
    from_path: &str,
    import_path: &str,
    target_path: Option<&str>,
    dependency_kind: &str,
) {
    assert!(
        dependencies.iter().any(|dependency| {
            dependency.from_path == from_path
                && dependency.import_path == import_path
                && dependency.target_path.as_deref() == target_path
                && dependency.dependency_kind == dependency_kind
        }),
        "expected dependency {from_path} {dependency_kind} {import_path:?} -> {target_path:?}, got:\n{dependencies:#?}"
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
fn non_utf8_files_do_not_abort_index_build() {
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

    let asset_dir = root.join("assets");
    fs::create_dir_all(&asset_dir).expect("create assets dir");
    fs::write(asset_dir.join("repository_cover.ai"), [0xff, 0xfe, 0xfd])
        .expect("write non-utf8 asset");

    run_build(root);

    let records = thinindex::store::load_records(root).expect("load records");
    assert!(
        records.iter().any(|record| record.name == "PromptService"),
        "valid UTF-8 source should still be indexed"
    );
    assert!(
        records
            .iter()
            .all(|record| record.path != "assets/repository_cover.ai"),
        "non-UTF-8 asset should not emit index records"
    );

    let second = run_build(root);
    assert!(
        second.contains("changed files: 0"),
        "skipped non-UTF-8 files should still be tracked by metadata, got:\n{second}"
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
    assert!(sqlite_table_exists(root, "dependencies"));

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
fn dependency_graph_records_representative_ecosystems() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/lib.rs",
        "mod local_mod;\nuse crate::local_mod::LocalMod;\nuse crate::missing::Missing;\n",
    );
    write_file(root, "src/local_mod.rs", "pub struct LocalMod;\n");
    write_file(
        root,
        "py/app.py",
        "from helpers.util import build_py\nimport missing_pkg\n",
    );
    write_file(
        root,
        "py/helpers/util.py",
        "def build_py():\n    return 1\n",
    );
    write_file(
        root,
        "web/app.ts",
        "import { Widget } from \"./widget\";\nimport React from \"react\";\n",
    );
    write_file(root, "web/widget.ts", "export function Widget() {}\n");
    write_file(
        root,
        "go/main.go",
        "package main\n\nimport \"example.com/project/pkg/helper\"\nimport \"fmt\"\n",
    );
    write_file(
        root,
        "go/pkg/helper/helper.go",
        "package helper\nfunc Help() {}\n",
    );
    write_file(
        root,
        "java/src/main/java/com/example/App.java",
        "package com.example;\nimport com.example.lib.Helper;\nimport java.util.List;\nclass App {}\n",
    );
    write_file(
        root,
        "java/src/main/java/com/example/lib/Helper.java",
        "package com.example.lib;\nclass Helper {}\n",
    );
    write_file(
        root,
        "c/main.c",
        "#include \"local.h\"\n#include <stdio.h>\nint main(void) { return 0; }\n",
    );
    write_file(root, "c/local.h", "struct LocalHeader { int value; };\n");
    write_file(
        root,
        "cpp/main.cpp",
        "#include \"local.hpp\"\nclass CppMain {};\n",
    );
    write_file(root, "cpp/local.hpp", "class CppLocal {};\n");
    write_file(
        root,
        "ruby/app.rb",
        "require_relative \"lib/tool\"\nrequire \"json\"\n",
    );
    write_file(root, "ruby/lib/tool.rb", "class RubyTool\nend\n");
    write_file(
        root,
        "php/app.php",
        "<?php\ninclude \"lib/tool.php\";\nrequire \"missing.php\";\n",
    );
    write_file(root, "php/lib/tool.php", "<?php\nclass PhpTool {}\n");
    write_file(root, "scripts/run.sh", "source ./lib.sh\n. ./missing.sh\n");
    write_file(root, "scripts/lib.sh", "run_helper() { echo ok; }\n");

    run_build(root);
    let dependencies = thinindex::store::load_dependencies(root).expect("load dependencies");

    assert_dependency(
        &dependencies,
        "src/lib.rs",
        "local_mod",
        Some("src/local_mod.rs"),
        "rust_module",
    );
    assert_dependency(
        &dependencies,
        "src/lib.rs",
        "crate::local_mod::LocalMod",
        Some("src/local_mod.rs"),
        "rust_use",
    );
    assert_dependency(
        &dependencies,
        "src/lib.rs",
        "crate::missing::Missing",
        None,
        "rust_use",
    );
    assert_dependency(
        &dependencies,
        "py/app.py",
        "helpers.util",
        Some("py/helpers/util.py"),
        "python_import",
    );
    assert_dependency(
        &dependencies,
        "py/app.py",
        "missing_pkg",
        None,
        "python_import",
    );
    assert_dependency(
        &dependencies,
        "web/app.ts",
        "./widget",
        Some("web/widget.ts"),
        "js_import",
    );
    assert_dependency(&dependencies, "web/app.ts", "react", None, "js_import");
    assert_dependency(
        &dependencies,
        "go/main.go",
        "example.com/project/pkg/helper",
        Some("go/pkg/helper/helper.go"),
        "go_import",
    );
    assert_dependency(&dependencies, "go/main.go", "fmt", None, "go_import");
    assert_dependency(
        &dependencies,
        "java/src/main/java/com/example/App.java",
        "com.example.lib.Helper",
        Some("java/src/main/java/com/example/lib/Helper.java"),
        "java_import",
    );
    assert_dependency(
        &dependencies,
        "java/src/main/java/com/example/App.java",
        "java.util.List",
        None,
        "java_import",
    );
    assert_dependency(
        &dependencies,
        "c/main.c",
        "local.h",
        Some("c/local.h"),
        "c_include",
    );
    assert_dependency(&dependencies, "c/main.c", "stdio.h", None, "c_include");
    assert_dependency(
        &dependencies,
        "cpp/main.cpp",
        "local.hpp",
        Some("cpp/local.hpp"),
        "cpp_include",
    );
    assert_dependency(
        &dependencies,
        "ruby/app.rb",
        "lib/tool",
        Some("ruby/lib/tool.rb"),
        "ruby_require",
    );
    assert_dependency(&dependencies, "ruby/app.rb", "json", None, "ruby_require");
    assert_dependency(
        &dependencies,
        "php/app.php",
        "lib/tool.php",
        Some("php/lib/tool.php"),
        "php_include",
    );
    assert_dependency(
        &dependencies,
        "php/app.php",
        "missing.php",
        None,
        "php_include",
    );
    assert_dependency(
        &dependencies,
        "scripts/run.sh",
        "./lib.sh",
        Some("scripts/lib.sh"),
        "shell_source",
    );
    assert_dependency(
        &dependencies,
        "scripts/run.sh",
        "./missing.sh",
        None,
        "shell_source",
    );

    assert!(
        dependencies
            .iter()
            .filter(|dependency| dependency.target_path.is_none())
            .all(|dependency| dependency.unresolved_reason.is_some()
                && dependency.confidence == "unresolved"),
        "unresolved dependencies should include reasons, got:\n{dependencies:#?}"
    );

    run_named_dependency_integrity_checks("dependency graph fixture", &dependencies);
}

#[test]
fn dependency_graph_is_deterministic_and_has_no_duplicate_edges() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/lib.rs", "mod local_mod;\nmod local_mod;\n");
    write_file(root, "src/local_mod.rs", "pub struct LocalMod;\n");

    run_build(root);
    let first = thinindex::store::load_dependencies(root).expect("load first dependencies");
    let second_output = run_build(root);
    let second = thinindex::store::load_dependencies(root).expect("load second dependencies");

    assert!(
        second_output.contains("changed files: 0"),
        "second build should skip unchanged files, got:\n{second_output}"
    );
    assert_eq!(first, second);
    assert_eq!(
        first
            .iter()
            .filter(|dependency| {
                dependency.from_path == "src/lib.rs" && dependency.import_path == "local_mod"
            })
            .count(),
        1,
        "duplicate dependency edges should be deduped, got:\n{first:#?}"
    );
}

#[test]
fn changed_and_deleted_files_remove_stale_dependencies() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "py/old.py", "def old():\n    return 1\n");
    write_file(root, "py/new.py", "def new():\n    return 1\n");
    write_file(root, "py/app.py", "import old\n");

    run_build(root);
    let before = thinindex::store::load_dependencies(root).expect("load dependencies before");
    assert_dependency(
        &before,
        "py/app.py",
        "old",
        Some("py/old.py"),
        "python_import",
    );

    write_file(root, "py/app.py", "import new\n");
    let changed = run_build(root);
    assert!(
        changed.contains("changed files: 1"),
        "expected one changed file, got:\n{changed}"
    );
    let after_change =
        thinindex::store::load_dependencies(root).expect("load dependencies after change");
    assert!(
        !after_change
            .iter()
            .any(|dependency| dependency.import_path == "old"),
        "stale old dependency should be removed, got:\n{after_change:#?}"
    );
    assert_dependency(
        &after_change,
        "py/app.py",
        "new",
        Some("py/new.py"),
        "python_import",
    );

    fs::remove_file(root.join("py/app.py")).expect("delete dependency source");
    let deleted = run_build(root);
    assert!(
        deleted.contains("deleted files: 1"),
        "expected one deleted file, got:\n{deleted}"
    );
    let after_delete =
        thinindex::store::load_dependencies(root).expect("load dependencies after delete");
    assert!(
        !after_delete
            .iter()
            .any(|dependency| dependency.from_path == "py/app.py"),
        "deleted file dependencies should be removed, got:\n{after_delete:#?}"
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
