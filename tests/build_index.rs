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

fn assert_ref_state(
    refs: &[thinindex::model::ReferenceRecord],
    from_path: &str,
    ref_kind: &str,
    to_name: &str,
    confidence: &str,
    reason: &str,
) {
    assert!(
        refs.iter().any(|reference| {
            reference.from_path == from_path
                && reference.ref_kind == ref_kind
                && reference.to_name == to_name
                && reference.confidence == confidence
                && reference.reason.as_deref() == Some(reason)
        }),
        "expected ref {from_path} {ref_kind} {to_name} confidence={confidence} reason={reason}, got:\n{refs:#?}"
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

fn assert_dependency_state(
    dependencies: &[thinindex::model::DependencyEdge],
    from_path: &str,
    import_path: &str,
    target_path: Option<&str>,
    dependency_kind: &str,
    confidence: &str,
    unresolved_reason: Option<&str>,
) {
    assert!(
        dependencies.iter().any(|dependency| {
            dependency.from_path == from_path
                && dependency.import_path == import_path
                && dependency.target_path.as_deref() == target_path
                && dependency.dependency_kind == dependency_kind
                && dependency.confidence == confidence
                && dependency.unresolved_reason.as_deref() == unresolved_reason
        }),
        "expected dependency {from_path} {dependency_kind} {import_path:?} -> {target_path:?} confidence={confidence} reason={unresolved_reason:?}, got:\n{dependencies:#?}"
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
    assert!(sqlite_table_exists(root, "semantic_facts"));
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
fn large_source_files_are_skipped_reported_and_tracked_incrementally() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/small.py", "class SmallService: pass\n");
    let huge_symbol_text = "class HugeGeneratedService:\n    pass\n";
    let padding = "x".repeat(thinindex::indexer::MAX_INDEXED_FILE_BYTES as usize + 1);
    write_file(
        root,
        "src/huge_generated.py",
        &format!("{huge_symbol_text}{padding}\n"),
    );

    let first = run_build(root);
    assert!(
        first.contains("changed files: 2"),
        "expected small and large files to be tracked as changed, got:\n{first}"
    );
    assert!(
        first.contains("skipped large files: 1")
            && first.contains("warning: skipped large file src/huge_generated.py"),
        "expected skipped large file warning, got:\n{first}"
    );

    let small = run_wi(root, &["SmallService"]);
    assert!(
        small.contains("src/small.py"),
        "small source file should be indexed, got:\n{small}"
    );
    let huge = run_wi(root, &["HugeGeneratedService"]);
    assert!(
        huge.trim().is_empty(),
        "large skipped source should not emit records, got:\n{huge}"
    );

    let second = run_build(root);
    assert!(
        second.contains("changed files: 0"),
        "skipped large file metadata should be tracked, got:\n{second}"
    );
}

#[test]
fn build_index_stats_reports_scale_metrics_without_large_snapshots() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/service.py", "class StatsService: pass\n");

    let output = build_index_bin()
        .current_dir(root)
        .arg("--stats")
        .output()
        .expect("run build_index --stats");

    assert!(
        output.status.success(),
        "build_index --stats failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for needle in [
        "refs:",
        "dependencies:",
        "total file bytes:",
        "max indexed file bytes:",
        "performance:",
        "large files:",
        "sqlite tuning:",
    ] {
        assert!(
            stdout.contains(needle),
            "missing stats field {needle:?}, got:\n{stdout}"
        );
    }
}

#[test]
fn largeish_fixture_build_stays_bounded_and_incremental() {
    let repo = temp_repo();
    let root = repo.path();

    for index in 0..150 {
        write_file(
            root,
            &format!("crates/pkg_{index}/src/lib.py"),
            &format!("class Service{index}: pass\n"),
        );
    }

    let first = run_build(root);
    assert!(
        first.contains("changed files: 150"),
        "expected all fixture files to index once, got:\n{first}"
    );

    let records = thinindex::store::load_records(root).expect("load records");
    assert!(
        records.len() >= 150 && records.len() < thinindex::indexer::MAX_RECORDS_PER_FILE,
        "large-ish fixture should stay bounded, got {} records",
        records.len()
    );

    let second = run_build(root);
    assert!(
        second.contains("changed files: 0"),
        "expected unchanged fixture rebuild to stay incremental, got:\n{second}"
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
fn dependency_resolver_pack_handles_rust_and_python() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/lib.rs",
        "pub mod local_mod;\nuse crate::local_mod::LocalMod;\nuse self::nested::Nested;\nuse serde::Serialize;\n",
    );
    write_file(root, "src/local_mod.rs", "pub struct LocalMod;\n");
    write_file(root, "src/nested.rs", "pub struct Nested;\n");
    write_file(
        root,
        "src/app.py",
        "from my_pkg.service import build_service\nimport requests\n",
    );
    write_file(
        root,
        "src/my_pkg/service.py",
        "def build_service():\n    return 1\n",
    );

    run_build(root);
    let dependencies = thinindex::store::load_dependencies(root).expect("load dependencies");

    assert_dependency_state(
        &dependencies,
        "src/lib.rs",
        "local_mod",
        Some("src/local_mod.rs"),
        "rust_module",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "src/lib.rs",
        "crate::local_mod::LocalMod",
        Some("src/local_mod.rs"),
        "rust_use",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "src/lib.rs",
        "self::nested::Nested",
        Some("src/nested.rs"),
        "rust_use",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "src/lib.rs",
        "serde::Serialize",
        None,
        "rust_use",
        "unresolved",
        Some("external_package"),
    );
    assert_dependency_state(
        &dependencies,
        "src/app.py",
        "my_pkg.service",
        Some("src/my_pkg/service.py"),
        "python_import",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "src/app.py",
        "requests",
        None,
        "python_import",
        "unresolved",
        Some("external_package"),
    );
}

#[test]
fn dependency_resolver_pack_handles_js_ts_go_and_dart() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "package.json", r#"{"name": "@acme/app"}"#);
    write_file(root, "go.mod", "module example.com/project\n");
    write_file(root, "pubspec.yaml", "name: demo_app\n");
    write_file(
        root,
        "web/app.ts",
        "import { Widget } from \"./components\";\nimport { local } from \"@acme/app/web/local\";\nimport React from \"react\";\n",
    );
    write_file(
        root,
        "web/components/index.ts",
        "export function Widget() {}\n",
    );
    write_file(root, "web/local.ts", "export const local = 1;\n");
    write_file(
        root,
        "go/main.go",
        "package main\n\nimport \"example.com/project/pkg/helper\"\nimport \"github.com/acme/external\"\n",
    );
    write_file(
        root,
        "pkg/helper/helper.go",
        "package helper\nfunc Help() {}\n",
    );
    write_file(
        root,
        "lib/main.dart",
        "import 'package:demo_app/src/tool.dart';\nimport 'dart:async';\n",
    );
    write_file(root, "lib/src/tool.dart", "class Tool {}\n");

    run_build(root);
    let dependencies = thinindex::store::load_dependencies(root).expect("load dependencies");

    assert_dependency_state(
        &dependencies,
        "web/app.ts",
        "./components",
        Some("web/components/index.ts"),
        "js_import",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "web/app.ts",
        "@acme/app/web/local",
        Some("web/local.ts"),
        "js_import",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "web/app.ts",
        "react",
        None,
        "js_import",
        "unresolved",
        Some("external_package"),
    );
    assert_dependency_state(
        &dependencies,
        "go/main.go",
        "example.com/project/pkg/helper",
        Some("pkg/helper/helper.go"),
        "go_import",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "go/main.go",
        "github.com/acme/external",
        None,
        "go_import",
        "unresolved",
        Some("external_package"),
    );
    assert_dependency_state(
        &dependencies,
        "lib/main.dart",
        "package:demo_app/src/tool.dart",
        Some("lib/src/tool.dart"),
        "dart_import",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "lib/main.dart",
        "dart:async",
        None,
        "dart_import",
        "unresolved",
        Some("external_package"),
    );
}

#[test]
fn dependency_resolver_pack_handles_jvm_and_dotnet_imports() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/main/java/com/example/App.java",
        "package com.example;\nimport com.example.lib.Helper;\nimport java.util.List;\nclass App {}\n",
    );
    write_file(
        root,
        "src/main/java/com/example/lib/Helper.java",
        "package com.example.lib;\nclass Helper {}\n",
    );
    write_file(
        root,
        "src/main/kotlin/com/example/KtApp.kt",
        "package com.example\nimport com.example.kt.KtHelper\nimport kotlin.collections.List\nclass KtApp\n",
    );
    write_file(
        root,
        "src/main/kotlin/com/example/kt/KtHelper.kt",
        "package com.example.kt\nclass KtHelper\n",
    );
    write_file(
        root,
        "src/main/scala/com/example/ScalaApp.scala",
        "package com.example\nimport com.example.scala.ScalaHelper\nimport scala.collection.mutable\nclass ScalaApp\n",
    );
    write_file(
        root,
        "src/main/scala/com/example/scala/ScalaHelper.scala",
        "package com.example.scala\nclass ScalaHelper\n",
    );
    write_file(
        root,
        "src/Acme/App.cs",
        "using Acme.Core;\nusing System.Text;\nnamespace Acme { class App {} }\n",
    );
    write_file(
        root,
        "src/Acme/Core/Helper.cs",
        "namespace Acme.Core { class Helper {} }\n",
    );

    run_build(root);
    let dependencies = thinindex::store::load_dependencies(root).expect("load dependencies");

    assert_dependency_state(
        &dependencies,
        "src/main/java/com/example/App.java",
        "com.example.lib.Helper",
        Some("src/main/java/com/example/lib/Helper.java"),
        "java_import",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "src/main/java/com/example/App.java",
        "java.util.List",
        None,
        "java_import",
        "unresolved",
        Some("external_package"),
    );
    assert_dependency_state(
        &dependencies,
        "src/main/kotlin/com/example/KtApp.kt",
        "com.example.kt.KtHelper",
        Some("src/main/kotlin/com/example/kt/KtHelper.kt"),
        "kotlin_import",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "src/main/kotlin/com/example/KtApp.kt",
        "kotlin.collections.List",
        None,
        "kotlin_import",
        "unresolved",
        Some("external_package"),
    );
    assert_dependency_state(
        &dependencies,
        "src/main/scala/com/example/ScalaApp.scala",
        "com.example.scala.ScalaHelper",
        Some("src/main/scala/com/example/scala/ScalaHelper.scala"),
        "scala_import",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "src/main/scala/com/example/ScalaApp.scala",
        "scala.collection.mutable",
        None,
        "scala_import",
        "unresolved",
        Some("external_package"),
    );
    assert_dependency_state(
        &dependencies,
        "src/Acme/App.cs",
        "Acme.Core",
        Some("src/Acme/Core/Helper.cs"),
        "cs_import",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "src/Acme/App.cs",
        "System.Text",
        None,
        "cs_import",
        "unresolved",
        Some("external_package"),
    );
}

#[test]
fn dependency_resolver_pack_handles_c_cpp_ruby_php_shell_and_nix() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "c/main.c",
        "#include \"include/local.h\"\n#include <stdio.h>\n",
    );
    write_file(root, "c/include/local.h", "struct Local { int value; };\n");
    write_file(root, "cpp/main.cpp", "#include \"shared.hpp\"\n");
    write_file(root, "include/shared.hpp", "class Shared {};\n");
    write_file(
        root,
        "ruby/app.rb",
        "require \"app/tool\"\nrequire \"json\"\n",
    );
    write_file(root, "app/tool.rb", "class Tool\nend\n");
    write_file(
        root,
        "php/app.php",
        "<?php\ninclude \"lib/tool.php\";\nrequire \"missing.php\";\n",
    );
    write_file(root, "php/lib/tool.php", "<?php\nclass Tool {}\n");
    write_file(
        root,
        "scripts/run.sh",
        "source helpers/lib.sh\n. /etc/profile\n",
    );
    write_file(
        root,
        "scripts/helpers/lib.sh",
        "run_helper() { echo ok; }\n",
    );
    write_file(
        root,
        "default.nix",
        "{ }:\nlet tool = import ./nix/tool.nix;\nin tool\n",
    );
    write_file(root, "nix/tool.nix", "{ name = \"tool\"; }\n");

    run_build(root);
    let dependencies = thinindex::store::load_dependencies(root).expect("load dependencies");

    assert_dependency_state(
        &dependencies,
        "c/main.c",
        "include/local.h",
        Some("c/include/local.h"),
        "c_include",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "c/main.c",
        "stdio.h",
        None,
        "c_include",
        "unresolved",
        Some("system_include"),
    );
    assert_dependency_state(
        &dependencies,
        "cpp/main.cpp",
        "shared.hpp",
        Some("include/shared.hpp"),
        "cpp_include",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "ruby/app.rb",
        "app/tool",
        Some("app/tool.rb"),
        "ruby_require",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "ruby/app.rb",
        "json",
        None,
        "ruby_require",
        "unresolved",
        Some("external_package"),
    );
    assert_dependency_state(
        &dependencies,
        "php/app.php",
        "lib/tool.php",
        Some("php/lib/tool.php"),
        "php_include",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "scripts/run.sh",
        "helpers/lib.sh",
        Some("scripts/helpers/lib.sh"),
        "shell_source",
        "resolved",
        None,
    );
    assert_dependency_state(
        &dependencies,
        "scripts/run.sh",
        "/etc/profile",
        None,
        "shell_source",
        "unresolved",
        Some("absolute_path"),
    );
    assert_dependency_state(
        &dependencies,
        "default.nix",
        "./nix/tool.nix",
        Some("nix/tool.nix"),
        "nix_import",
        "resolved",
        None,
    );
}

#[test]
fn dependency_resolver_pack_marks_ambiguous_matches_explicitly() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "go/main.go",
        "package main\n\nimport \"example.com/project/pkg/helper\"\n",
    );
    write_file(root, "apps/one/pkg/helper/helper.go", "package helper\n");
    write_file(root, "apps/two/pkg/helper/helper.go", "package helper\n");

    run_build(root);
    let dependencies = thinindex::store::load_dependencies(root).expect("load dependencies");

    assert_dependency_state(
        &dependencies,
        "go/main.go",
        "example.com/project/pkg/helper",
        None,
        "go_import",
        "ambiguous",
        Some("ambiguous_match"),
    );
    run_named_dependency_integrity_checks("ambiguous dependency fixture", &dependencies);
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
    assert_ref_exists(&refs, "call", "PromptService");
    assert_ref_exists(&refs, "module_dependency", "src/prompt_service.py");
    assert_ref_exists(&refs, "markdown_link", "../src/prompt_service.py");
    assert_ref_exists(&refs, "css_usage", ".headerNavigation");
    assert_ref_exists(&refs, "css_usage", "--paper-bg");
    assert_ref_exists(&refs, "html_usage", "#mainHeader");
    assert_ref_exists(&refs, "html_usage", ".headerNavigation");
    assert_ref_exists(&refs, "html_usage", "data-testid");
    assert_ref_exists(&refs, "test_reference", "PromptService");
    assert_ref_state(
        &refs,
        "src/consumer.py",
        "call",
        "PromptService",
        "exact_local",
        "local_symbol_match",
    );
    assert_ref_state(
        &refs,
        "src/consumer.py",
        "module_dependency",
        "src/prompt_service.py",
        "dependency",
        "dependency_graph_resolved_file",
    );
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
