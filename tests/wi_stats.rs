mod common;

use common::*;

#[test]
fn wi_appends_usage_log() {
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

    let events = thinindex::stats::read_usage_events(root).expect("read usage events");
    let event = events.first().expect("at least one usage event");

    assert!(
        !event.query.is_empty(),
        "missing query field in usage event: {event:?}"
    );
    assert!(
        event.result_count > 0,
        "missing result_count field in usage event: {event:?}"
    );
}

#[test]
fn wi_logs_miss_when_no_results() {
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

    let events = thinindex::stats::read_usage_events(root).expect("read usage events");
    let event = events.last().expect("at least one usage event");

    assert!(!event.hit, "expected hit=false in miss event: {event:?}");
    assert_eq!(
        event.result_count, 0,
        "expected result_count=0 in miss event: {event:?}"
    );
}

#[test]
fn wi_stats_prints_all_windows() {
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
        "Agent workflow audit",
        "Recorded wi events",
        "Context commands",
        "Scope: local wi usage only",
        "Recent misses",
    ] {
        assert!(
            stdout.contains(needle),
            "expected wi-stats stdout to contain '{needle}', got:\n{stdout}"
        );
    }
}

#[test]
fn wi_stats_records_command_categories_for_agent_audit() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/main.py",
        r#"
class PromptService:
    pass

def consume():
    return PromptService()
"#,
    );

    run_build(root);
    run_wi(root, &["PromptService"]);
    run_wi(root, &["pack", "PromptService"]);
    run_wi(root, &["impact", "PromptService"]);

    let events = thinindex::stats::read_usage_events(root).expect("read usage events");
    let commands: Vec<&str> = events.iter().map(|event| event.command.as_str()).collect();

    assert_eq!(commands, vec!["search", "pack", "impact"]);

    let output = wi_stats_bin()
        .current_dir(root)
        .output()
        .expect("run wi-stats");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Context commands: 2 (refs 0, pack 1, impact 1)"),
        "expected command-category audit counts, got:\n{stdout}"
    );
    assert!(
        stdout.contains("Signal: pack and impact usage recorded"),
        "expected implementation workflow signal, got:\n{stdout}"
    );
}

#[test]
fn wi_stats_no_usage_message() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "src/main.py", "class PromptService: pass\n");
    run_build(root);

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
