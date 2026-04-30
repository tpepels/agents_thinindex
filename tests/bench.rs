mod common;

use std::time::Duration;

use common::*;
use thinindex::bench::{
    BenchmarkRepoSet, BenchmarkRunOptions, load_benchmark_repo_set, render_benchmark_report,
    run_benchmark,
};

fn write_benchmark_fixture(root: &std::path::Path) {
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
        "tests/test_prompt_service.py",
        r#"
from prompt_service import PromptService

def test_prompt_service():
    assert PromptService()
"#,
    );
    write_file(root, "docs/guide.md", "[PromptService](PromptService)\n");
    write_file(
        root,
        "frontend/components/HeaderNavigation.tsx",
        r#"
export function HeaderNavigation() {
  return <header className="headerNavigation">Header</header>;
}
"#,
    );
}

#[test]
fn fixture_benchmark_reports_sane_metrics() {
    let repo = temp_repo();
    let root = repo.path();
    write_benchmark_fixture(root);
    run_build(root);

    let report = run_benchmark(
        root,
        BenchmarkRunOptions {
            queries: Some(vec![
                "PromptService".to_string(),
                "HeaderNavigation".to_string(),
                "MissingBenchmarkQuery".to_string(),
            ]),
            build_duration: Some(Duration::from_millis(1)),
        },
    )
    .expect("run benchmark");
    let rendered = render_benchmark_report(&report);

    assert!(report.record_count > 0, "expected records:\n{rendered}");
    assert_eq!(report.duplicate_location_count, 0, "{rendered}");
    assert_eq!(report.malformed_record_count, 0, "{rendered}");
    assert_eq!(report.malformed_ref_count, 0, "{rendered}");
    assert_eq!(report.dev_index_path_count, 0, "{rendered}");
    assert_eq!(report.query_count, 3, "{rendered}");
    assert!(report.hit_count >= 1, "{rendered}");
    assert!(rendered.contains("Repo:"));
    assert!(rendered.contains("- integrity: ok"));

    let events = thinindex::stats::read_usage_events(root).expect("read usage events");
    assert!(
        events.is_empty(),
        "benchmark should not create usage events: {events:?}"
    );
}

#[test]
fn wi_bench_prints_compact_report_without_logging_usage() {
    let repo = temp_repo();
    let root = repo.path();
    write_benchmark_fixture(root);
    run_build(root);

    let output = wi_bin()
        .current_dir(root)
        .arg("bench")
        .output()
        .expect("run wi bench");

    assert!(
        output.status.success(),
        "wi bench failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for needle in [
        "Repo:",
        "- path:",
        "- build:",
        "- db:",
        "- files:",
        "- records:",
        "- refs:",
        "- queries:",
        "- hit rate:",
        "- avg wi latency:",
        "- avg pack files:",
        "- avg impact files:",
        "- integrity: ok",
    ] {
        assert!(
            stdout.contains(needle),
            "expected wi bench output to contain {needle}, got:\n{stdout}"
        );
    }

    let events = thinindex::stats::read_usage_events(root).expect("read usage events");
    assert!(
        events.is_empty(),
        "wi bench should not create usage events: {events:?}"
    );
}

#[test]
fn benchmark_manifest_parsing_uses_non_skipped_repos() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let root = temp.path();
    let repo = root.join("python_app");
    std::fs::create_dir_all(repo.join("src")).expect("create repo");
    std::fs::create_dir_all(root.join("skipped_app")).expect("create skipped repo");
    std::fs::write(
        root.join("MANIFEST.toml"),
        r#"
[[repo]]
name = "python-app"
path = "python_app"
kind = "python-cli"
languages = ["py"]
description = "fixture manifest repo"
notes = "fixture manifest includes all quality metadata fields"
ignore_guidance = "ignore generated fixtures if they dominate parser output"
queries = ["PromptService", "config", "PromptService"]
expected_paths = ["src/"]
expected_symbols = ["PromptService"]
expected_symbol_patterns = ["^create_.*_service$"]

[[repo.expected_symbol]]
language = "py"
path = "src/service.py"
kind = "class"
name = "PromptService"

[[repo.expected_symbol_pattern]]
language = "py"
path_glob = "src/**/*.py"
kind = "function"
name_regex = "^create_.*_service$"
min_count = 1

[[repo.expected_absent_symbol]]
language = "py"
path = "src/service.py"
kind = "function"
name = "PromptServiceFromComment"

[[repo.quality_threshold]]
language = "py"
min_records = 2
max_duplicate_locations = 0
max_malformed_records = 0

[[repo]]
name = "skipped"
path = "skipped_app"
skip = true
skip_reason = "fixture skip reason"
"#,
    )
    .expect("write manifest");

    let repo_set = load_benchmark_repo_set(root).expect("load benchmark repo set");
    let BenchmarkRepoSet::Repos {
        manifest_used,
        repos,
    } = repo_set
    else {
        panic!("expected manifest repos, got {repo_set:?}");
    };

    assert!(manifest_used);
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "python-app");
    assert_eq!(repos[0].path, repo);
    assert_eq!(repos[0].kind.as_deref(), Some("python-cli"));
    assert_eq!(repos[0].languages, vec!["py".to_string()]);
    assert_eq!(
        repos[0].notes.as_deref(),
        Some("fixture manifest includes all quality metadata fields")
    );
    assert_eq!(
        repos[0].ignore_guidance.as_deref(),
        Some("ignore generated fixtures if they dominate parser output")
    );
    assert_eq!(
        repos[0].queries.as_ref().expect("manifest queries"),
        &vec!["PromptService".to_string(), "config".to_string()]
    );
    assert_eq!(repos[0].expected_paths, vec!["src/".to_string()]);
    assert_eq!(repos[0].expected_symbols, vec!["PromptService".to_string()]);
    assert_eq!(
        repos[0].expected_symbol_patterns,
        vec!["^create_.*_service$".to_string()]
    );
    assert_eq!(repos[0].expected_symbol_specs.len(), 1);
    assert_eq!(
        repos[0].expected_symbol_specs[0].name,
        "PromptService".to_string()
    );
    assert_eq!(repos[0].expected_symbol_pattern_specs.len(), 1);
    assert_eq!(repos[0].expected_symbol_pattern_specs[0].min_count, 1);
    assert_eq!(repos[0].expected_absent_symbol_specs.len(), 1);
    assert_eq!(
        repos[0].expected_absent_symbol_specs[0].name,
        "PromptServiceFromComment".to_string()
    );
    assert_eq!(repos[0].quality_thresholds.len(), 1);
    assert_eq!(repos[0].quality_thresholds[0].language, "py");
    assert_eq!(repos[0].quality_thresholds[0].min_records, Some(2));
    assert_eq!(
        repos[0].quality_thresholds[0].max_duplicate_locations,
        Some(0)
    );
    assert_eq!(
        repos[0].quality_thresholds[0].max_malformed_records,
        Some(0)
    );
    assert!(repos[0].from_manifest);
}

#[test]
fn benchmark_manifest_missing_repo_fails_clearly() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let root = temp.path();
    std::fs::write(
        root.join("MANIFEST.toml"),
        r#"
[[repo]]
name = "missing"
path = "missing_repo"
kind = "missing-fixture"
languages = ["rs"]
queries = ["Missing"]
"#,
    )
    .expect("write manifest");

    let error = load_benchmark_repo_set(root).expect_err("missing manifest repo should fail");
    let message = format!("{error:#}");

    assert!(
        message.contains("repo `missing` path does not exist"),
        "unexpected error: {message}"
    );
}

#[test]
fn benchmark_manifest_expected_absent_symbol_requires_name() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let root = temp.path();
    let repo = root.join("python_app");
    std::fs::create_dir_all(repo.join("src")).expect("create repo");
    std::fs::write(
        root.join("MANIFEST.toml"),
        r#"
[[repo]]
name = "python-app"
path = "python_app"
kind = "python-cli"
languages = ["py"]
queries = ["PromptService"]

[[repo.expected_absent_symbol]]
language = "py"
path = "src/service.py"
kind = "function"
"#,
    )
    .expect("write manifest");

    let error = load_benchmark_repo_set(root)
        .expect_err("expected_absent_symbol without a name should fail");
    let message = format!("{error:#}");

    assert!(
        message.contains("expected_absent_symbol #1 missing required field `name`"),
        "unexpected error: {message}"
    );
}

#[test]
fn benchmark_manifest_requires_kind_languages_and_queries_for_active_repos() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let root = temp.path();
    let repo = root.join("python_app");
    std::fs::create_dir_all(repo.join("src")).expect("create repo");
    std::fs::write(
        root.join("MANIFEST.toml"),
        r#"
[[repo]]
name = "python-app"
path = "python_app"
languages = ["py"]
queries = ["PromptService"]
"#,
    )
    .expect("write manifest");

    let error = load_benchmark_repo_set(root).expect_err("missing kind should fail");
    let message = format!("{error:#}");
    assert!(
        message.contains("repo `python-app` missing required field `kind`"),
        "unexpected error: {message}"
    );

    std::fs::write(
        root.join("MANIFEST.toml"),
        r#"
[[repo]]
name = "python-app"
path = "python_app"
kind = "python-cli"
queries = ["PromptService"]
"#,
    )
    .expect("write manifest");

    let error = load_benchmark_repo_set(root).expect_err("missing languages should fail");
    let message = format!("{error:#}");
    assert!(
        message.contains("repo `python-app` missing required field `languages`"),
        "unexpected error: {message}"
    );
}

#[test]
fn benchmark_manifest_skipped_repo_requires_skip_reason_but_not_existing_path() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let root = temp.path();
    std::fs::write(
        root.join("MANIFEST.toml"),
        r#"
[[repo]]
name = "skipped"
path = "not-cloned"
skip = true
"#,
    )
    .expect("write manifest");

    let error = load_benchmark_repo_set(root).expect_err("skip reason should be required");
    let message = format!("{error:#}");
    assert!(
        message.contains("skipped repo `skipped` missing required field `skip_reason`"),
        "unexpected error: {message}"
    );

    std::fs::write(
        root.join("MANIFEST.toml"),
        r#"
[[repo]]
name = "skipped"
path = "not-cloned"
skip = true
skip_reason = "optional corpus repo not cloned locally"
"#,
    )
    .expect("write manifest");

    let repo_set = load_benchmark_repo_set(root).expect("skipped missing repo should be accepted");
    assert_eq!(repo_set, BenchmarkRepoSet::Empty);
}

#[test]
fn real_repo_manifest_docs_describe_required_schema_and_local_only_policy() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let docs = std::fs::read_to_string(root.join("docs/REAL_REPO_MANIFEST.md"))
        .expect("read real repo manifest docs");
    let readme = std::fs::read_to_string(root.join("README.md")).expect("read README");
    let quality = std::fs::read_to_string(root.join("docs/QUALITY.md")).expect("read quality docs");

    for field in [
        "name",
        "path",
        "kind",
        "languages",
        "queries",
        "expected_paths",
        "expected_symbols",
        "expected_symbol_patterns",
        "expected_absent_symbol",
        "quality_threshold",
        "skip_reason",
        "notes",
        "ignore_guidance",
    ] {
        assert!(
            docs.contains(field),
            "manifest docs should describe `{field}`"
        );
    }

    assert!(docs.contains("Do not commit third-party repository contents"));
    assert!(docs.contains("Normal tests use temporary fixture manifests"));
    assert!(readme.contains("docs/REAL_REPO_MANIFEST.md"));
    assert!(quality.contains("docs/REAL_REPO_MANIFEST.md"));
}
