use std::{fs, path::Path, process::Command};

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::TempDir;

fn has_ctags() -> bool {
    Command::new("ctags")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn temp_repo() -> TempDir {
    let temp = tempfile::tempdir().expect("create tempdir");
    fs::create_dir_all(temp.path().join(".git")).expect("create .git marker");
    temp
}

fn write_file(root: &Path, relpath: &str, text: &str) {
    let path = root.join(relpath);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }

    fs::write(path, text).expect("write test file");
}

fn build_index_bin() -> Command {
    Command::cargo_bin("build_index").expect("build_index binary")
}

fn wi_bin() -> Command {
    Command::cargo_bin("wi").expect("wi binary")
}

fn wi_init_bin() -> Command {
    Command::cargo_bin("wi-init").expect("wi-init binary")
}

fn wi_stats_bin() -> Command {
    Command::cargo_bin("wi-stats").expect("wi-stats binary")
}

fn run_build(root: &Path) -> String {
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

fn run_wi(root: &Path, args: &[&str]) -> String {
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
"#,
    );

    run_build(root);

    assert!(root.join(".dev_index").exists());
    assert!(root.join(".dev_index/manifest.json").exists());
    assert!(root.join(".dev_index/index.jsonl").exists());

    let index = fs::read_to_string(root.join(".dev_index/index.jsonl")).expect("read index");
    assert!(index.contains("PromptService"));
}

#[test]
fn wi_finds_python_symbol() {
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
"#,
    );

    run_build(root);

    let stdout = run_wi(root, &["PromptService"]);

    assert!(stdout.contains("app/services/prompt_service.py"));
    assert!(stdout.contains("PromptService"));
}

#[test]
fn wi_filters_by_path() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "frontend/header_navigation.py",
        r#"
def HeaderNavigation():
    return "frontend"
"#,
    );

    write_file(
        root,
        "backend/header_navigation.py",
        r#"
def HeaderNavigation():
    return "backend"
"#,
    );

    run_build(root);

    let stdout = run_wi(root, &["HeaderNavigation", "--path", "frontend"]);

    assert!(
        stdout.contains("frontend/header_navigation.py"),
        "expected frontend result, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("backend/header_navigation.py"),
        "backend result should have been filtered out, got:\n{stdout}"
    );
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
fn css_extras_are_indexed() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "frontend/styles/components/header.css",
        r#"
:root {
  --paper-bg: white;
}

.headerNavigation {
  display: flex;
}

#mainHeader {
  position: sticky;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
"#,
    );

    run_build(root);

    let class_result = run_wi(root, &[".headerNavigation", "--type", "css_class"]);
    assert!(class_result.contains(".headerNavigation"));

    let variable_result = run_wi(root, &["--type", "css_variable", "--", "--paper-bg"]);
    assert!(variable_result.contains("--paper-bg"));

    let keyframes_result = run_wi(root, &["fadeIn", "--type", "keyframes"]);
    assert!(keyframes_result.contains("fadeIn"));
}

#[test]
fn markdown_extras_are_indexed() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "README.md",
        "# Thin Index\n\n- [ ] Add ctags integration\n- [x] Add search\n\nSee [docs](docs/index.md).\n",
    );

    run_build(root);

    let heading = run_wi(root, &["Thin Index"]);
    assert!(heading.contains("README.md"));

    let checklist = run_wi(root, &["Add ctags integration", "--type", "checklist"]);
    assert!(checklist.contains("README.md"));

    let link = run_wi(root, &["docs", "--type", "link"]);
    assert!(link.contains("README.md"));
}

#[test]
fn wi_verbose_prints_details() {
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
class PromptService:
    pass
"#,
    );

    run_build(root);

    wi_bin()
        .current_dir(root)
        .args(["PromptService", "--verbose"])
        .assert()
        .success()
        .stdout(predicate::str::contains("kind:"))
        .stdout(predicate::str::contains("lang:"))
        .stdout(predicate::str::contains("source:"))
        .stdout(predicate::str::contains("text:"));
}

#[test]
fn wi_init_writes_wi_and_agents_files() {
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
class PromptService:
    pass
"#,
    );

    Command::cargo_bin("wi-init")
        .expect("wi-init binary")
        .current_dir(root)
        .assert()
        .success();

    let wi = fs::read_to_string(root.join("WI.md")).expect("read WI.md");
    assert!(
        wi.contains("Run `build_index` before discovery"),
        "unexpected WI.md content:\n{wi}"
    );
    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md");
    assert!(agents.contains("See WI.md for repository search/index usage."));

    assert!(root.join(".dev_index/index.jsonl").exists());
}

fn fixture_repo(name: &str) -> TempDir {
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

#[test]
fn fixture_repo_indexes_python_symbols() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    run_build(root);

    let service = run_wi(root, &["PromptService"]);
    assert!(
        service.contains("app/services/prompt_service.py"),
        "expected PromptService result, got:\n{service}"
    );

    let function = run_wi(root, &["create_prompt_service"]);
    assert!(
        function.contains("app/services/prompt_service.py"),
        "expected create_prompt_service result, got:\n{function}"
    );
}

#[test]
fn fixture_repo_indexes_css_extras() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    run_build(root);

    let class_result = run_wi(root, &[".headerNavigation", "--type", "css_class"]);
    assert!(
        class_result.contains("frontend/styles/header.css"),
        "expected css class result, got:\n{class_result}"
    );

    let variable_result = run_wi(root, &["--type", "css_variable", "--", "--paper-bg"]);
    assert!(
        variable_result.contains("frontend/styles/header.css"),
        "expected css variable result, got:\n{variable_result}"
    );

    let keyframes = run_wi(root, &["fadeIn", "--type", "keyframes"]);
    assert!(
        keyframes.contains("frontend/styles/header.css"),
        "expected keyframes result, got:\n{keyframes}"
    );
}

#[test]
fn fixture_repo_indexes_markdown_extras() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    run_build(root);

    let heading = run_wi(root, &["Guide"]);
    assert!(
        heading.contains("docs/guide.md"),
        "expected markdown heading result, got:\n{heading}"
    );

    let checklist = run_wi(root, &["Add ctags integration", "--type", "checklist"]);
    assert!(
        checklist.contains("docs/guide.md"),
        "expected markdown checklist result, got:\n{checklist}"
    );

    let link = run_wi(root, &["README", "--type", "link"]);
    assert!(
        link.contains("docs/guide.md"),
        "expected markdown link result, got:\n{link}"
    );
}

#[test]
fn fixture_repo_indexes_jsx_extras() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    run_build(root);

    let class_result = run_wi(root, &[".headerNavigation"]);
    assert!(
        class_result.contains("frontend/components/HeaderNavigation.tsx")
            || class_result.contains("frontend/styles/header.css"),
        "expected jsx/css class result, got:\n{class_result}"
    );

    let data_attr = run_wi(root, &["data-testid", "--type", "data_attribute"]);
    assert!(
        data_attr.contains("frontend/components/HeaderNavigation.tsx"),
        "expected data attribute result, got:\n{data_attr}"
    );
}

#[test]
fn fixture_repo_remove_command_removes_index() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    Command::cargo_bin("wi-init")
        .expect("wi-init binary")
        .current_dir(root)
        .assert()
        .success();

    assert!(root.join(".dev_index/index.jsonl").exists());

    Command::cargo_bin("wi-init")
        .expect("wi-init binary")
        .current_dir(root)
        .arg("--remove")
        .assert()
        .success();

    assert!(!root.join(".dev_index").exists());
    assert!(root.join("WI.md").exists());
    assert!(root.join("AGENTS.md").exists());
}

#[test]
fn fixture_ignored_repo_skips_default_ignored_dirs() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("ignored_repo");
    let root = repo.path();

    run_build(root);

    let real = run_wi(root, &["RealService"]);
    assert!(
        real.contains("src/real.py"),
        "expected real symbol, got:\n{real}"
    );

    let node_modules = run_wi(root, &["FakeNodeModulesSymbol"]);
    assert!(
        node_modules.trim().is_empty(),
        "node_modules symbol should be ignored, got:\n{node_modules}"
    );

    let next = run_wi(root, &["FakeNextSymbol"]);
    assert!(
        next.trim().is_empty(),
        ".next symbol should be ignored, got:\n{next}"
    );

    let dist = run_wi(root, &["FakeDistSymbol"]);
    assert!(
        dist.trim().is_empty(),
        "dist symbol should be ignored, got:\n{dist}"
    );
}

#[test]
fn fixture_html_repo_indexes_html_extras() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("html_repo");
    let root = repo.path();

    run_build(root);

    let id = run_wi(root, &["#mainHeader", "--type", "html_id"]);
    assert!(
        id.contains("templates/base.html"),
        "expected html id result, got:\n{id}"
    );

    let class_name = run_wi(root, &[".pixelButton", "--type", "html_class"]);
    assert!(
        class_name.contains("frontend/pages/index.html"),
        "expected html class result, got:\n{class_name}"
    );

    let data_attr = run_wi(root, &["data-testid", "--type", "data_attribute"]);
    assert!(
        data_attr.contains("templates/base.html"),
        "expected data attribute result, got:\n{data_attr}"
    );

    let landmark = run_wi(root, &["main", "--type", "html_tag"]);
    assert!(
        landmark.contains("templates/base.html"),
        "expected landmark tag result, got:\n{landmark}"
    );
}

#[test]
fn fixture_todo_repo_indexes_todos_and_fixmes() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("todo_repo");
    let root = repo.path();

    run_build(root);

    let todo = run_wi(root, &["TODO", "--type", "todo"]);
    assert!(
        todo.contains("src/work.py") || todo.contains("docs/notes.md"),
        "expected TODO result, got:\n{todo}"
    );

    let fixme = run_wi(root, &["FIXME", "--type", "fixme"]);
    assert!(
        fixme.contains("src/work.py") || fixme.contains("docs/notes.md"),
        "expected FIXME result, got:\n{fixme}"
    );

    let specific = run_wi(root, &["provider response"]);
    assert!(
        specific.contains("src/work.py"),
        "expected specific FIXME text result, got:\n{specific}"
    );
}

#[test]
fn fixture_thinindexignore_repo_uses_gitignore_style_patterns() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("thinindexignore_repo");
    let root = repo.path();

    run_build(root);

    let visible = run_wi(root, &["VisibleService"]);
    assert!(
        visible.contains("src/visible.py"),
        "expected visible service, got:\n{visible}"
    );

    let generated = run_wi(root, &["GeneratedHiddenService"]);
    assert!(
        generated.trim().is_empty(),
        "generated file should be ignored, got:\n{generated}"
    );

    let secret = run_wi(root, &["SecretService"]);
    assert!(
        secret.trim().is_empty(),
        "secret.py should be ignored, got:\n{secret}"
    );

    let kept = run_wi(root, &["KeptGeneratedService"]);
    assert!(
        kept.contains("generated/keep.py"),
        "negated .thinindexignore pattern should keep generated/keep.py, got:\n{kept}"
    );
}

#[test]
fn fixture_css_double_dash_query_requires_separator() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    run_build(root);

    let variable_result = run_wi(root, &["--type", "css_variable", "--", "--paper-bg"]);
    assert!(
        variable_result.contains("frontend/styles/header.css"),
        "expected css variable result, got:\n{variable_result}"
    );
}

#[test]
fn fixture_keep_index_remove_preserves_dev_index() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    Command::cargo_bin("wi-init")
        .expect("wi-init binary")
        .current_dir(root)
        .assert()
        .success();

    assert!(root.join(".dev_index/index.jsonl").exists());

    Command::cargo_bin("wi-init")
        .expect("wi-init binary")
        .current_dir(root)
        .args(["--remove", "--keep-index"])
        .assert()
        .success();

    assert!(root.join(".dev_index/index.jsonl").exists());
}

#[test]
fn fixture_gitignore_rules_are_respected() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("gitignore_repo");
    let root = repo.path();

    run_build(root);

    let visible = run_wi(root, &["VisibleGitignoreService"]);
    assert!(
        visible.contains("src/visible.py"),
        "expected visible service, got:\n{visible}"
    );

    let ignored = run_wi(root, &["GitignoredService"]);
    assert!(
        ignored.trim().is_empty(),
        ".gitignore ignored file should not be indexed, got:\n{ignored}"
    );

    let kept = run_wi(root, &["KeptGitignoreService"]);
    assert!(
        kept.contains("generated/keep.py"),
        ".gitignore negation should keep generated/keep.py, got:\n{kept}"
    );
}

#[test]
fn wi_init_writes_thinindexignore() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    wi_init_bin().current_dir(root).assert().success();

    let ignore_path = root.join(".thinindexignore");
    assert!(
        ignore_path.exists(),
        ".thinindexignore should exist after wi-init"
    );

    let contents = fs::read_to_string(&ignore_path).expect("read .thinindexignore");
    assert!(
        contents.contains("node_modules/"),
        "expected bundled template content in .thinindexignore, got:\n{contents}"
    );
}

#[test]
fn wi_init_force_overwrites_thinindexignore() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    wi_init_bin().current_dir(root).assert().success();

    let ignore_path = root.join(".thinindexignore");
    fs::write(&ignore_path, "garbage_only\n").expect("overwrite with garbage");

    wi_init_bin()
        .current_dir(root)
        .arg("--force")
        .assert()
        .success();

    let contents = fs::read_to_string(&ignore_path).expect("read .thinindexignore");
    assert!(
        contents.contains("node_modules/"),
        "expected bundled template content after --force, got:\n{contents}"
    );
}

#[test]
fn build_index_respects_thinindexignore() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(root, ".thinindexignore", "secret/\n");
    write_file(
        root,
        "secret/secret.py",
        r#"
class SecretSymbol:
    pass
"#,
    );
    write_file(
        root,
        "src/visible.py",
        r#"
class VisibleSymbol:
    pass
"#,
    );

    run_build(root);

    let visible = run_wi(root, &["VisibleSymbol"]);
    assert!(
        visible.contains("src/visible.py"),
        "expected visible symbol present, got:\n{visible}"
    );

    let secret = run_wi(root, &["SecretSymbol"]);
    assert!(
        secret.trim().is_empty(),
        "secret symbol should be ignored via .thinindexignore, got:\n{secret}"
    );
}

#[test]
fn wi_appends_usage_log() {
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
class PromptService:
    pass
"#,
    );

    run_build(root);
    run_wi(root, &["PromptService"]);

    let log_path = root.join(".dev_index/wi_usage.jsonl");
    assert!(
        log_path.exists(),
        "wi_usage.jsonl should exist after wi run"
    );

    let contents = fs::read_to_string(&log_path).expect("read usage log");
    let first_line = contents
        .lines()
        .next()
        .expect("at least one line in usage log");

    let parsed: serde_json::Value =
        serde_json::from_str(first_line).expect("parse usage log line as JSON");

    assert!(
        parsed.get("query").is_some(),
        "missing 'query' field in usage event: {first_line}"
    );
    assert!(
        parsed.get("result_count").is_some(),
        "missing 'result_count' field in usage event: {first_line}"
    );
}

#[test]
fn wi_logs_miss_when_no_results() {
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
class PromptService:
    pass
"#,
    );

    run_build(root);
    run_wi(root, &["NoSuchSymbolPleaseMissXYZ"]);

    let log_path = root.join(".dev_index/wi_usage.jsonl");
    let contents = fs::read_to_string(&log_path).expect("read usage log");
    let last_line = contents
        .lines()
        .rfind(|l| !l.trim().is_empty())
        .expect("at least one line in usage log");

    let parsed: serde_json::Value =
        serde_json::from_str(last_line).expect("parse usage log line as JSON");

    assert_eq!(
        parsed.get("hit").and_then(|v| v.as_bool()),
        Some(false),
        "expected hit=false in miss event: {last_line}"
    );
    assert_eq!(
        parsed.get("result_count").and_then(|v| v.as_u64()),
        Some(0),
        "expected result_count=0 in miss event: {last_line}"
    );
}

#[test]
fn wi_stats_prints_all_windows() {
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
class PromptService:
    pass
"#,
    );

    run_build(root);
    run_wi(root, &["PromptService"]);
    run_wi(root, &["NoSuchSymbolMissA"]);

    let output = wi_stats_bin()
        .current_dir(root)
        .output()
        .expect("run wi-stats");

    assert!(
        output.status.success(),
        "wi-stats failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    for needle in [
        "WI usage",
        "Window",
        "1d",
        "2d",
        "5d",
        "30d",
        "Hit ratio",
        "Avg results",
        "Hit/miss graph",
        "Recent misses",
    ] {
        assert!(
            stdout.contains(needle),
            "expected wi-stats stdout to contain '{needle}', got:\n{stdout}"
        );
    }
}

#[test]
fn wi_stats_no_usage_message() {
    let repo = temp_repo();
    let root = repo.path();

    let output = wi_stats_bin()
        .current_dir(root)
        .output()
        .expect("run wi-stats");

    assert!(
        output.status.success(),
        "wi-stats failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No wi usage recorded yet"),
        "expected 'No wi usage recorded yet' message, got:\n{stdout}"
    );
}

#[test]
fn wi_stats_recent_misses_section() {
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
class PromptService:
    pass
"#,
    );

    run_build(root);
    run_wi(root, &["MissingQueryAlpha"]);
    run_wi(root, &["MissingQueryBeta"]);

    let output = wi_stats_bin()
        .current_dir(root)
        .output()
        .expect("run wi-stats");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Recent misses"),
        "expected 'Recent misses' heading, got:\n{stdout}"
    );

    let after_heading = stdout
        .split("Recent misses")
        .nth(1)
        .expect("Recent misses heading split");

    assert!(
        after_heading.contains("MissingQueryAlpha"),
        "expected MissingQueryAlpha under Recent misses, got:\n{stdout}"
    );
    assert!(
        after_heading.contains("MissingQueryBeta"),
        "expected MissingQueryBeta under Recent misses, got:\n{stdout}"
    );
}

#[test]
fn wi_stats_recent_misses_none_when_only_hits() {
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
class PromptService:
    pass
"#,
    );

    run_build(root);
    run_wi(root, &["PromptService"]);

    let output = wi_stats_bin()
        .current_dir(root)
        .output()
        .expect("run wi-stats");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    let after_heading = stdout
        .split("Recent misses")
        .nth(1)
        .expect("Recent misses heading split");

    assert!(
        after_heading.contains("None"),
        "expected 'None' under Recent misses when no misses, got:\n{stdout}"
    );
}

#[test]
fn install_script_includes_wi_stats() {
    let install_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("install.sh");
    let contents = fs::read_to_string(&install_path).expect("read install.sh");
    assert!(
        contents.contains("wi-stats"),
        "install.sh should reference wi-stats"
    );
}

#[test]
fn uninstall_script_includes_wi_stats() {
    let uninstall_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("uninstall.sh");
    let contents = fs::read_to_string(&uninstall_path).expect("read uninstall.sh");
    assert!(
        contents.contains("wi-stats"),
        "uninstall.sh should reference wi-stats"
    );
}
