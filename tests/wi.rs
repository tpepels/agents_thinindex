mod common;

use assert_cmd::prelude::*;
use common::*;

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

    let stdout = run_wi(root, &["HeaderNavigation", "-p", "frontend"]);

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
fn wi_filters_by_language() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "src/shared.py", "class SharedSymbol: pass\n");
    write_file(root, "src/shared.rs", "struct SharedSymbol;\n");

    run_build(root);

    let py = run_wi(root, &["SharedSymbol", "-l", "py"]);

    assert!(
        py.contains("src/shared.py"),
        "expected python result, got:\n{py}"
    );
    assert!(
        !py.contains("src/shared.rs"),
        "rust result should be filtered out, got:\n{py}"
    );
}

#[test]
fn wi_filters_by_source() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "frontend/styles/header.css",
        r#"
.headerNavigation {
  display: flex;
}
"#,
    );

    run_build(root);

    let extras = run_wi(root, &[".headerNavigation", "-s", "extras", "-v"]);
    assert!(
        extras.contains("source: extras"),
        "expected extras source result, got:\n{extras}"
    );

    let ctags = run_wi(root, &[".headerNavigation", "-s", "ctags", "-v"]);
    assert!(
        !ctags.contains("source: extras"),
        "ctags-filtered result should not include extras records, got:\n{ctags}"
    );
}

#[test]
fn wi_limits_results() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }
    let repo = temp_repo();
    let root = repo.path();
    for i in 0..5 {
        write_file(
            root,
            &format!("src/file{0}.py", i),
            &format!("class C{0}: pass\n", i),
        );
    }
    run_build(root);
    let out = run_wi(root, &["C", "-n", "1"]);
    let count = out.lines().filter(|l| l.contains("src/file")).count();
    assert_eq!(count, 1, "should limit to 1 result, got:\n{out}");
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
        .args(["PromptService", "-v"])
        .assert()
        .success()
        .stdout(predicates::str::contains("kind:"))
        .stdout(predicates::str::contains("lang:"))
        .stdout(predicates::str::contains("source:"))
        .stdout(predicates::str::contains("text:"));
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

    let class_result = run_wi(root, &[".headerNavigation", "-t", "css_class"]);
    assert!(class_result.contains(".headerNavigation"));

    let variable_result = run_wi(root, &["-t", "css_variable", "--", "--paper-bg"]);
    assert!(variable_result.contains("--paper-bg"));

    let keyframes_result = run_wi(root, &["fadeIn", "-t", "keyframes"]);
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

    let checklist = run_wi(root, &["Add ctags integration", "-t", "checklist"]);
    assert!(checklist.contains("README.md"));

    let link = run_wi(root, &["docs", "-t", "link"]);
    assert!(link.contains("README.md"));
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

    let class_result = run_wi(root, &[".headerNavigation", "-t", "css_class"]);
    assert!(
        class_result.contains("frontend/styles/header.css"),
        "expected css class result, got:\n{class_result}"
    );

    let variable_result = run_wi(root, &["-t", "css_variable", "--", "--paper-bg"]);
    assert!(
        variable_result.contains("frontend/styles/header.css"),
        "expected css variable result, got:\n{variable_result}"
    );

    let keyframes = run_wi(root, &["fadeIn", "-t", "keyframes"]);
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

    let checklist = run_wi(root, &["Add ctags integration", "-t", "checklist"]);
    assert!(
        checklist.contains("docs/guide.md"),
        "expected markdown checklist result, got:\n{checklist}"
    );

    let link = run_wi(root, &["README", "-t", "link"]);
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

    let data_attr = run_wi(root, &["data-testid", "-t", "data_attribute"]);
    assert!(
        data_attr.contains("frontend/components/HeaderNavigation.tsx"),
        "expected data attribute result, got:\n{data_attr}"
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

    let id = run_wi(root, &["#mainHeader", "-t", "html_id"]);
    assert!(
        id.contains("templates/base.html"),
        "expected html id result, got:\n{id}"
    );

    let class_name = run_wi(root, &[".pixelButton", "-t", "html_class"]);
    assert!(
        class_name.contains("frontend/pages/index.html"),
        "expected html class result, got:\n{class_name}"
    );

    let data_attr = run_wi(root, &["data-testid", "-t", "data_attribute"]);
    assert!(
        data_attr.contains("templates/base.html"),
        "expected data attribute result, got:\n{data_attr}"
    );

    let landmark = run_wi(root, &["main", "-t", "html_tag"]);
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

    let todo = run_wi(root, &["TODO", "-t", "todo"]);
    assert!(
        todo.contains("src/work.py") || todo.contains("docs/notes.md"),
        "expected TODO result, got:\n{todo}"
    );

    let fixme = run_wi(root, &["FIXME", "-t", "fixme"]);
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
fn fixture_css_double_dash_query_requires_separator() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    run_build(root);

    let variable_result = run_wi(root, &["-t", "css_variable", "--", "--paper-bg"]);
    assert!(
        variable_result.contains("frontend/styles/header.css"),
        "expected css variable result, got:\n{variable_result}"
    );
}
