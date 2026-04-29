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
description = "fixture manifest repo"
queries = ["PromptService", "config", "PromptService"]
expected_paths = ["src/"]

[[repo]]
name = "skipped"
path = "skipped_app"
queries = ["ignored"]
skip = true
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
    assert_eq!(
        repos[0].queries.as_ref().expect("manifest queries"),
        &vec!["PromptService".to_string(), "config".to_string()]
    );
    assert_eq!(repos[0].expected_paths, vec!["src/".to_string()]);
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
