mod common;

use common::*;

#[test]
fn fixture_ignored_repo_skips_default_ignored_dirs() {
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
        node_modules.contains("No matches for: FakeNodeModulesSymbol"),
        "node_modules symbol should be ignored, got:\n{node_modules}"
    );

    let next = run_wi(root, &["FakeNextSymbol"]);
    assert!(
        next.contains("No matches for: FakeNextSymbol"),
        ".next symbol should be ignored, got:\n{next}"
    );

    let dist = run_wi(root, &["FakeDistSymbol"]);
    assert!(
        dist.contains("No matches for: FakeDistSymbol"),
        "dist symbol should be ignored, got:\n{dist}"
    );
}

#[test]
fn fixture_thinindexignore_repo_uses_gitignore_style_patterns() {
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
        generated.contains("No matches for: GeneratedHiddenService"),
        "generated file should be ignored, got:\n{generated}"
    );

    let secret = run_wi(root, &["SecretService"]);
    assert!(
        secret.contains("No matches for: SecretService"),
        "secret.py should be ignored, got:\n{secret}"
    );

    let kept = run_wi(root, &["KeptGeneratedService"]);
    assert!(
        kept.contains("generated/keep.py"),
        "negated .thinindexignore pattern should keep generated/keep.py, got:\n{kept}"
    );
}

#[test]
fn fixture_gitignore_rules_are_respected() {
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
        ignored.contains("No matches for: GitignoredService"),
        ".gitignore ignored file should not be indexed, got:\n{ignored}"
    );

    let kept = run_wi(root, &["KeptGitignoreService"]);
    assert!(
        kept.contains("generated/keep.py"),
        ".gitignore negation should keep generated/keep.py, got:\n{kept}"
    );
}

#[test]
fn build_index_respects_thinindexignore() {
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
        secret.contains("No matches for: SecretSymbol"),
        "secret symbol should be ignored via .thinindexignore, got:\n{secret}"
    );
}
