mod common;

use std::fs;

use common::*;
use thinindex::model::FileReference;

fn assert_file_ref(
    references: &[FileReference],
    source_path: &str,
    raw_target: &str,
    target_path: Option<&str>,
    reference_kind: &str,
) {
    assert!(
        references.iter().any(|reference| {
            reference.source_path == source_path
                && reference.raw_target == raw_target
                && reference.target_path.as_deref() == target_path
                && reference.reference_kind == reference_kind
        }),
        "expected file reference {source_path} {reference_kind} {raw_target:?} -> {target_path:?}, got:\n{references:#?}"
    );
}

fn write_file_reference_fixture(root: &std::path::Path) {
    write_file(root, "src/utils/helper.py", "def helper():\n    return 1\n");
    write_file(root, "src/app.py", "from utils.helper import helper\n");
    write_file(root, "web/widget.ts", "export const widget = 1;\n");
    write_file(root, "web/app.ts", "import { widget } from './widget';\n");
    write_file(root, "c/defs.h", "#define ANSWER 42\n");
    write_file(root, "c/main.c", "#include \"defs.h\"\n");
    write_file(root, "assets/logo.png", "fake image bytes\n");
    write_file(root, "assets/bg.png", "fake image bytes\n");
    write_file(root, "assets/logo-small.png", "fake image bytes\n");
    write_file(root, "assets/logo-large.png", "fake image bytes\n");
    write_file(root, "templates/base.html", "<main></main>\n");
    write_file(root, "tests/fixtures/sample.json", "{}\n");
    write_file(root, "docs/guide.md", "# Guide\n");
    write_file(
        root,
        "docs/readme.md",
        "[Guide](guide.md#intro)\n[Template](../templates/base.html?cache=1#main)\n![Logo](../assets/logo.png)\n![Logo again](../assets/logo.png)\n[External](https://example.com/logo.png)\n",
    );
    write_file(
        root,
        "web/index.html",
        r#"
<link rel="stylesheet" href="style.css">
<script src="app.js"></script>
<img src="../assets/logo.png">
<img srcset="../assets/logo-small.png 1x, ../assets/logo-large.png 2x">
"#,
    );
    write_file(
        root,
        "web/style.css",
        r#"
@import "../styles/theme";
.hero { background: url("../assets/bg.png"); }
.remote { background: url("https://example.com/remote.png"); }
"#,
    );
    write_file(root, "web/app.js", "console.log('app');\n");
    write_file(root, "styles/theme.scss", "$brand: #fff;\n");
    write_file(root, "_sass/mixins/_buttons.scss", "@mixin button {}\n");
    write_file(root, "styles/app.scss", "@import \"mixins/buttons\";\n");
    write_file(
        root,
        "config/app.json",
        r#"{ "template": "templates/base.html", "fixture": "tests/fixtures/sample.json", "schema": "missing/schema.json", "root": "/blog/:num/", "version": "1.2.3", "generated": "${OUT_DIR}/schema.json" }"#,
    );
    write_file(root, "package.json", r#"{ "main": "web/app.js" }"#);
    write_file(root, "project/App.cs", "class App {}\n");
    write_file(root, "project/App.config", "<configuration />\n");
    write_file(
        root,
        "project/App.csproj",
        r#"
<Project>
  <Reference Include="System.Core" />
  <Compile Include="App.cs" />
  <None Include="App.config" />
</Project>
"#,
    );
}

#[test]
fn file_references_are_extracted_resolved_deduped_and_deterministic() {
    let repo = temp_repo();
    let root = repo.path();
    write_file_reference_fixture(root);
    write_file(root, ".dev_index/ignored.py", "def ignored(): pass\n");
    write_file(
        root,
        "test_repos/sample/src/hidden.py",
        "def hidden(): pass\n",
    );

    run_build(root);
    let first = thinindex::store::load_file_references(root).expect("load file references");
    let second_output = run_build(root);
    let second = thinindex::store::load_file_references(root).expect("load file references again");

    assert!(
        second_output.contains("changed files: 0"),
        "second build should skip unchanged files, got:\n{second_output}"
    );
    assert_eq!(first, second);

    assert_file_ref(
        &first,
        "src/app.py",
        "utils.helper",
        Some("src/utils/helper.py"),
        "import",
    );
    assert_file_ref(
        &first,
        "web/app.ts",
        "./widget",
        Some("web/widget.ts"),
        "import",
    );
    assert_file_ref(&first, "c/main.c", "defs.h", Some("c/defs.h"), "include");
    assert_file_ref(
        &first,
        "docs/readme.md",
        "guide.md#intro",
        Some("docs/guide.md"),
        "link",
    );
    assert_file_ref(
        &first,
        "docs/readme.md",
        "../templates/base.html?cache=1#main",
        Some("templates/base.html"),
        "link",
    );
    assert_file_ref(
        &first,
        "docs/readme.md",
        "../assets/logo.png",
        Some("assets/logo.png"),
        "asset",
    );
    assert_file_ref(
        &first,
        "web/index.html",
        "style.css",
        Some("web/style.css"),
        "stylesheet",
    );
    assert_file_ref(
        &first,
        "web/index.html",
        "app.js",
        Some("web/app.js"),
        "script",
    );
    assert_file_ref(
        &first,
        "web/index.html",
        "../assets/logo.png",
        Some("assets/logo.png"),
        "asset",
    );
    assert_file_ref(
        &first,
        "web/index.html",
        "../assets/logo-small.png",
        Some("assets/logo-small.png"),
        "asset",
    );
    assert_file_ref(
        &first,
        "web/index.html",
        "../assets/logo-large.png",
        Some("assets/logo-large.png"),
        "asset",
    );
    assert_file_ref(
        &first,
        "web/style.css",
        "../styles/theme",
        Some("styles/theme.scss"),
        "stylesheet",
    );
    assert_file_ref(
        &first,
        "styles/app.scss",
        "mixins/buttons",
        Some("_sass/mixins/_buttons.scss"),
        "stylesheet",
    );
    assert_file_ref(
        &first,
        "web/style.css",
        "../assets/bg.png",
        Some("assets/bg.png"),
        "asset",
    );
    assert_file_ref(
        &first,
        "config/app.json",
        "templates/base.html",
        Some("templates/base.html"),
        "config_path",
    );
    assert_file_ref(
        &first,
        "config/app.json",
        "tests/fixtures/sample.json",
        Some("tests/fixtures/sample.json"),
        "fixture",
    );
    assert_file_ref(
        &first,
        "package.json",
        "web/app.js",
        Some("web/app.js"),
        "package_entry",
    );
    assert_file_ref(
        &first,
        "project/App.csproj",
        "App.cs",
        Some("project/App.cs"),
        "package_entry",
    );
    assert_file_ref(
        &first,
        "project/App.csproj",
        "App.config",
        Some("project/App.config"),
        "package_entry",
    );
    assert_file_ref(
        &first,
        "config/app.json",
        "missing/schema.json",
        None,
        "config_path",
    );

    assert!(
        first
            .iter()
            .all(|reference| !reference.raw_target.contains("https://")),
        "external URLs should not become local file references:\n{first:#?}"
    );
    assert!(
        first.iter().all(|reference| {
            !matches!(
                reference.raw_target.as_str(),
                "1.2.3" | "${OUT_DIR}/schema.json" | "/blog/:num/" | "System.Core"
            )
        }),
        "non-local versions, variables, and packages should not become file references:\n{first:#?}"
    );
    assert_eq!(
        first
            .iter()
            .filter(|reference| {
                reference.source_path == "docs/readme.md"
                    && reference.raw_target == "../assets/logo.png"
                    && reference.reference_kind == "asset"
            })
            .count(),
        1,
        "duplicate file reference edges should be deduped:\n{first:#?}"
    );
    assert!(
        first.iter().all(|reference| {
            !reference.source_path.contains(".dev_index")
                && !reference.source_path.starts_with("test_repos/")
                && reference.target_path.as_ref().is_none_or(|target| {
                    !target.contains(".dev_index") && !target.starts_with("test_repos/")
                })
        }),
        ".dev_index and test_repos must not be indexed in normal repo builds:\n{first:#?}"
    );
    run_named_file_reference_integrity_checks("file reference fixture", &first);
}

#[test]
fn deleted_target_file_rebuilds_file_reference_as_unresolved() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "docs/readme.md", "![Logo](../assets/logo.png)\n");
    write_file(root, "assets/logo.png", "fake image bytes\n");

    run_build(root);
    let before = thinindex::store::load_file_references(root).expect("load before");
    assert_file_ref(
        &before,
        "docs/readme.md",
        "../assets/logo.png",
        Some("assets/logo.png"),
        "asset",
    );

    fs::remove_file(root.join("assets/logo.png")).expect("delete target asset");
    run_build(root);
    let after = thinindex::store::load_file_references(root).expect("load after");

    assert!(
        after.iter().any(|reference| {
            reference.source_path == "docs/readme.md"
                && reference.raw_target == "../assets/logo.png"
                && reference.target_path.is_none()
                && reference.unresolved_reason.as_deref() == Some("target_not_found")
        }),
        "deleted target should clear resolved target and keep an unresolved hint:\n{after:#?}"
    );
}

#[test]
fn absolute_and_ambiguous_targets_keep_clear_unresolved_reasons() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "docs/readme.md", "[Absolute](/assets/logo.png)\n");
    write_file(root, "web/app.md", "[Widget](./widget)\n");
    write_file(root, "web/widget.js", "console.log('js');\n");
    write_file(root, "web/widget.ts", "export const widget = 1;\n");

    run_build(root);
    let references = thinindex::store::load_file_references(root).expect("load file references");

    assert!(references.iter().any(|reference| {
        reference.source_path == "docs/readme.md"
            && reference.raw_target == "/assets/logo.png"
            && reference.target_path.is_none()
            && reference.unresolved_reason.as_deref() == Some("absolute_path")
    }));
    assert!(references.iter().any(|reference| {
        reference.source_path == "web/app.md"
            && reference.raw_target == "./widget"
            && reference.target_path.is_none()
            && reference.confidence == "ambiguous"
            && reference.unresolved_reason.as_deref() == Some("ambiguous_match")
    }));
}

#[test]
fn refs_pack_and_impact_surface_file_reference_evidence() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "web/widget.ts", "export class Widget {}\n");
    write_file(root, "web/app.ts", "import './widget';\n");

    run_build(root);

    let refs = run_wi(root, &["refs", "Widget"]);
    assert!(
        refs.contains("web/app.ts")
            && refs.contains("file_import")
            && refs.contains("confidence: resolved")
            && refs.contains("reason: import resolves to web/widget.ts; evidence:"),
        "refs should surface file-reference evidence:\n{refs}"
    );

    let pack = run_wi(root, &["pack", "Widget"]);
    assert!(
        pack.contains("web/app.ts") && pack.contains("file_import"),
        "pack should include related files via file references:\n{pack}"
    );

    let impact = run_wi(root, &["impact", "Widget"]);
    assert!(
        impact.contains("web/app.ts") && impact.contains("file_import"),
        "impact should include reverse file references:\n{impact}"
    );
}
