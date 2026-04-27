mod common;

use std::fs;

use common::*;

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
        .rfind(|line| !line.trim().is_empty())
        .expect("at least one line in usage log");

    let parsed: serde_json::Value =
        serde_json::from_str(last_line).expect("parse usage log line as JSON");

    assert_eq!(
        parsed.get("hit").and_then(|value| value.as_bool()),
        Some(false),
        "expected hit=false in miss event: {last_line}"
    );
    assert_eq!(
        parsed.get("result_count").and_then(|value| value.as_u64()),
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
