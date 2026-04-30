mod common;

use assert_cmd::prelude::*;
use common::*;

fn write_context_fixture(root: &std::path::Path) {
    write_file(
        root,
        "src/prompt_service.py",
        r#"
from helpers import format_prompt

class PromptService:
    pass
"#,
    );
    write_file(
        root,
        "src/helpers.py",
        "def format_prompt():\n    return 'ok'\n",
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
    write_file(root, "src/module_consumer.py", "import prompt_service\n");
    write_file(root, "docs/guide.md", "[PromptService](PromptService)\n");
    write_file(
        root,
        "config/impact.json",
        r#"{ "service": "PromptService" }"#,
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
    write_file(
        root,
        "frontend/components/HeaderNavigation.tsx",
        r#"
export function HeaderNavigation() {
  return <header className="headerNavigation" data-testid="header-nav">Header</header>;
}
"#,
    );
    write_file(
        root,
        "frontend/styles/header.css",
        r#"
.headerNavigation {
  color: var(--paper-bg);
}
"#,
    );
}

fn context_ref_rows(output: &str) -> Vec<&str> {
    output
        .lines()
        .skip_while(|line| *line != "References:")
        .skip(1)
        .filter(|line| line.starts_with("- "))
        .collect()
}

fn suggested_rows(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("- ") && !line.ends_with("none"))
        .collect()
}

fn group_rows<'a>(output: &'a str, heading: &str) -> Vec<&'a str> {
    output
        .lines()
        .skip_while(|line| *line != heading)
        .skip(1)
        .take_while(|line| !line.ends_with(':'))
        .filter(|line| line.starts_with("- ") && !line.ends_with("none"))
        .collect()
}

fn non_primary_impact_rows(output: &str) -> Vec<&str> {
    [
        "References:",
        "Dependent files:",
        "Likely tests:",
        "Related docs:",
        "Build/config files:",
        "Unresolved/unknown areas:",
    ]
    .into_iter()
    .flat_map(|heading| group_rows(output, heading))
    .collect()
}

fn assert_impact_rows_have_reasons_and_confidence(output: &str) {
    let lines: Vec<_> = output.lines().collect();

    for (index, line) in lines.iter().enumerate() {
        if line.starts_with("- ") && !line.ends_with("none") {
            let reason = lines.get(index + 1).copied().unwrap_or_default();
            let confidence = lines.get(index + 2).copied().unwrap_or_default();
            assert!(
                reason.trim_start().starts_with("reason:"),
                "row missing following reason line:\n{line}\n\nfull output:\n{output}"
            );
            assert!(
                confidence.trim_start().starts_with("confidence:"),
                "row missing following confidence line:\n{line}\n\nfull output:\n{output}"
            );
        }
    }
}

#[test]
fn wi_finds_python_symbol() {
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

    let parser = run_wi(root, &[".headerNavigation", "-s", "tree_sitter", "-v"]);
    assert!(
        !parser.contains("source: extras"),
        "tree_sitter-filtered result should not include extras records, got:\n{parser}"
    );
}

#[test]
fn wi_limits_results() {
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
fn wi_help_mentions_context_commands() {
    wi_bin()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("wi refs PromptService"))
        .stdout(predicates::str::contains("wi pack PromptService"))
        .stdout(predicates::str::contains("wi impact PromptService"))
        .stdout(predicates::str::contains("wi bench"));
}

#[test]
fn wi_refs_prompt_service_includes_primary_and_refs() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    run_build(root);

    let output = run_wi(root, &["refs", "PromptService"]);

    assert!(
        output.contains("Primary:"),
        "missing Primary group:\n{output}"
    );
    assert!(
        output.contains("References:"),
        "missing References group:\n{output}"
    );
    assert!(
        output.contains("src/prompt_service.py"),
        "missing primary file:\n{output}"
    );
    assert!(
        output.contains("src/consumer.py") && output.contains("import PromptService"),
        "missing import reference:\n{output}"
    );
    assert!(
        output.contains("tests/test_prompt_service.py")
            && output.contains("test_reference PromptService"),
        "missing test reference:\n{output}"
    );
    assert!(
        output.contains("docs/guide.md") && output.contains("markdown_link PromptService"),
        "missing markdown link reference:\n{output}"
    );
    assert!(output.contains("reason:"), "missing reasons:\n{output}");
}

#[test]
fn wi_refs_order_is_deterministic_and_ranked() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    run_build(root);

    let first = run_wi(root, &["refs", "PromptService"]);
    let second = run_wi(root, &["refs", "PromptService"]);

    assert_eq!(first, second);
    assert!(first.find("Primary:") < first.find("References:"));
    let import_pos = first.find("src/consumer.py").expect("import ref present");
    let test_pos = first
        .find("tests/test_prompt_service.py")
        .expect("test ref present");
    let doc_pos = first.rfind("docs/guide.md").expect("doc ref present");
    assert!(
        import_pos < test_pos && test_pos < doc_pos,
        "expected import before test before doc, got:\n{first}"
    );
}

#[test]
fn wi_refs_respects_limit() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    for i in 0..5 {
        write_file(
            root,
            &format!("src/consumer_{i}.py"),
            "from prompt_service import PromptService\n",
        );
    }
    run_build(root);

    let output = run_wi(root, &["refs", "PromptService", "-n", "2"]);
    let rows = context_ref_rows(&output);

    assert_eq!(rows.len(), 2, "expected two refs, got:\n{output}");
    assert!(
        output.contains("Primary:"),
        "primary should still print:\n{output}"
    );
}

#[test]
fn wi_refs_missing_refs_shows_primary_non_error() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "src/lonely.py", "class LonelyService: pass\n");
    run_build(root);

    let output = run_wi(root, &["refs", "LonelyService"]);

    assert!(
        output.contains("Primary:"),
        "primary should print:\n{output}"
    );
    assert!(
        output.contains("no references found for LonelyService"),
        "missing no-refs message:\n{output}"
    );
}

#[test]
fn wi_pack_prompt_service_groups_compact_read_set() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    run_build(root);

    let output = run_wi(root, &["pack", "PromptService"]);

    for heading in [
        "Primary definitions:",
        "Direct references:",
        "Dependencies:",
        "Dependents:",
        "Tests:",
        "Configs/build files:",
        "Docs/examples:",
        "Unresolved hints:",
    ] {
        assert!(output.contains(heading), "missing {heading}\n{output}");
    }
    assert!(
        output.contains("src/prompt_service.py")
            && output.contains("tests/test_prompt_service.py")
            && output.contains("src/consumer.py")
            && output.contains("docs/guide.md"),
        "missing expected read-plan files:\n{output}"
    );
    assert!(
        output.contains("src/helpers.py")
            && output.contains("src/module_consumer.py")
            && output.contains("config/impact.json"),
        "missing dependency/dependent/config read-plan files:\n{output}"
    );
    assert_impact_rows_have_reasons_and_confidence(&output);
    assert!(
        output.contains("confidence: direct")
            && output.contains("confidence: dependency")
            && output.contains("confidence: test-related")
            && output.contains("confidence: heuristic"),
        "missing expected pack confidence labels:\n{output}"
    );
    assert!(
        !output.contains("class PromptService:") && !output.contains("def consume():"),
        "pack must not dump source contents:\n{output}"
    );
}

#[test]
fn wi_refs_includes_related_ui_for_style_query() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    run_build(root);

    let output = run_wi(root, &["refs", ".headerNavigation"]);

    assert!(
        output.contains("References:") && output.contains("html_usage .headerNavigation"),
        "expected related UI/style refs, got:\n{output}"
    );
}

#[test]
fn wi_pack_dedupes_files_and_respects_limit() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    for i in 0..5 {
        write_file(
            root,
            &format!("src/extra_consumer_{i}.py"),
            "from prompt_service import PromptService\nPromptService()\n",
        );
    }
    run_build(root);

    let output = run_wi(root, &["pack", "PromptService", "-n", "4"]);
    let rows = suggested_rows(&output);
    let mut paths = std::collections::BTreeSet::new();

    assert!(
        rows.len() <= 4,
        "expected at most four suggested rows, got:\n{output}"
    );
    for row in rows {
        let path = row
            .trim_start_matches("- ")
            .split(':')
            .next()
            .expect("path before colon");
        assert!(
            paths.insert(path.to_string()),
            "duplicate path in:\n{output}"
        );
    }
}

#[test]
fn wi_pack_order_is_deterministic_and_ranked() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    write_file(
        root,
        "fixtures/example_prompt.py",
        "from prompt_service import PromptService\nPromptService()\n",
    );
    run_build(root);

    let first = run_wi(root, &["pack", "PromptService"]);
    let second = run_wi(root, &["pack", "PromptService"]);

    assert_eq!(first, second);
    let primary_pos = first
        .find("Primary definitions:")
        .expect("primary heading present");
    let refs_pos = first
        .find("Direct references:")
        .expect("direct references heading present");
    let dependencies_pos = first
        .find("Dependencies:")
        .expect("dependencies heading present");
    let dependents_pos = first
        .find("Dependents:")
        .expect("dependents heading present");
    let tests_pos = first.find("Tests:").expect("tests heading present");
    let configs_pos = first
        .find("Configs/build files:")
        .expect("configs heading present");
    let docs_pos = first
        .find("Docs/examples:")
        .expect("docs/examples heading present");
    let unresolved_pos = first
        .find("Unresolved hints:")
        .expect("unresolved heading present");

    assert!(
        primary_pos < refs_pos
            && refs_pos < dependencies_pos
            && dependencies_pos < dependents_pos
            && dependents_pos < tests_pos
            && tests_pos < configs_pos
            && configs_pos < docs_pos
            && docs_pos < unresolved_pos,
        "expected deterministic pack group order:\n{first}"
    );
}

#[test]
fn wi_impact_prompt_service_groups_affected_files() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    run_build(root);

    let output = run_wi(root, &["impact", "PromptService"]);

    for heading in [
        "Direct definitions:",
        "References:",
        "Dependent files:",
        "Likely tests:",
        "Related docs:",
        "Build/config files:",
        "Unresolved/unknown areas:",
    ] {
        assert!(output.contains(heading), "missing {heading}\n{output}");
    }
    assert!(
        output.contains("src/prompt_service.py")
            && output.contains("tests/test_prompt_service.py")
            && output.contains("src/consumer.py")
            && output.contains("docs/guide.md"),
        "missing expected impact files:\n{output}"
    );
    assert!(
        output.contains("src/module_consumer.py") && output.contains("config/impact.json"),
        "missing dependency/config impact files:\n{output}"
    );
    assert_impact_rows_have_reasons_and_confidence(&output);
    assert!(
        output.contains("confidence: direct")
            && output.contains("confidence: dependency")
            && output.contains("confidence: test-related")
            && output.contains("confidence: heuristic"),
        "missing expected impact confidence labels:\n{output}"
    );
}

#[test]
fn wi_impact_order_is_deterministic_and_ranked() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    write_file(
        root,
        "fixtures/example_prompt.py",
        "from prompt_service import PromptService\nPromptService()\n",
    );
    run_build(root);

    let first = run_wi(root, &["impact", "PromptService"]);
    let second = run_wi(root, &["impact", "PromptService"]);

    assert_eq!(first, second);
    let primary_pos = first
        .find("Direct definitions:")
        .expect("primary heading present");
    let refs_pos = first
        .find("References:")
        .expect("references heading present");
    let dependents_pos = first
        .find("Dependent files:")
        .expect("dependents heading present");
    let tests_pos = first.find("Likely tests:").expect("tests heading present");
    let docs_pos = first.find("Related docs:").expect("docs heading present");
    let unknown_pos = first
        .find("Unresolved/unknown areas:")
        .expect("unknown heading present");
    let fixture_pos = first
        .find("fixtures/example_prompt.py")
        .expect("fixture ref present");

    assert!(
        primary_pos < refs_pos
            && refs_pos < dependents_pos
            && dependents_pos < tests_pos
            && tests_pos < docs_pos,
        "expected direct definitions, references, dependents, tests, docs order:\n{first}"
    );
    assert!(
        unknown_pos < fixture_pos,
        "fixture/example rows should be in the last group:\n{first}"
    );
}

#[test]
fn wi_impact_respects_group_limits() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    for i in 0..7 {
        write_file(
            root,
            &format!("tests/test_prompt_service_{i}.py"),
            "from prompt_service import PromptService\n\ndef test_prompt_service_extra():\n    assert PromptService()\n",
        );
        write_file(
            root,
            &format!("src/consumer_{i}.py"),
            "from prompt_service import PromptService\nPromptService()\n",
        );
        write_file(
            root,
            &format!("docs/guide_{i}.md"),
            "[PromptService](PromptService)\n",
        );
        write_file(
            root,
            &format!("frontend/components/prompt_{i}.tsx"),
            "export const label = 'PromptService';\n",
        );
    }
    run_build(root);

    let output = run_wi(root, &["impact", "PromptService"]);

    assert!(group_rows(&output, "References:").len() <= 5);
    assert!(group_rows(&output, "Dependent files:").len() <= 5);
    assert!(group_rows(&output, "Likely tests:").len() <= 5);
    assert!(group_rows(&output, "Related docs:").len() <= 3);
    assert!(group_rows(&output, "Build/config files:").len() <= 5);
    assert!(group_rows(&output, "Unresolved/unknown areas:").len() <= 5);
    assert!(
        non_primary_impact_rows(&output).len() <= 15,
        "expected at most 15 non-primary rows, got:\n{output}"
    );
}

#[test]
fn wi_impact_respects_n_as_total_non_primary_limit() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    run_build(root);

    let output = run_wi(root, &["impact", "PromptService", "-n", "2"]);

    assert!(
        output.contains("Direct definitions:") && output.contains("src/prompt_service.py"),
        "primary should still print:\n{output}"
    );
    assert_eq!(
        non_primary_impact_rows(&output).len(),
        2,
        "expected two non-primary rows, got:\n{output}"
    );
}

#[test]
fn wi_impact_missing_refs_shows_primary_non_error() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "src/lonely.py", "class LonelyService: pass\n");
    run_build(root);

    let output = run_wi(root, &["impact", "LonelyService"]);

    assert!(
        output.contains("Direct definitions:"),
        "primary should print:\n{output}"
    );
    assert!(
        output.contains("no impact references found for LonelyService"),
        "missing no-impact-refs message:\n{output}"
    );
}

#[test]
fn wi_impact_no_primary_preserves_no_result_behavior_and_logs_miss() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    run_build(root);

    let output = run_wi(root, &["impact", "NoSuchSymbolPleaseMissXYZ"]);

    assert!(
        output.trim().is_empty() && !output.contains("Direct definitions:"),
        "impact miss should preserve no-result output behavior:\n{output}"
    );

    let events = thinindex::stats::read_usage_events(root).expect("read usage events");
    let event = events.last().expect("at least one usage event");
    assert_eq!(event.query, "impact NoSuchSymbolPleaseMissXYZ");
    assert!(!event.hit, "expected miss event: {event:?}");
    assert_eq!(event.result_count, 0, "expected zero results: {event:?}");
}

#[test]
fn wi_impact_does_not_dump_full_file_contents() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    run_build(root);

    let output = run_wi(root, &["impact", "PromptService"]);

    for needle in [
        "class PromptService:",
        "def consume():",
        "def test_prompt_service():",
    ] {
        assert!(
            !output.contains(needle),
            "impact must not dump file contents; found {needle} in:\n{output}"
        );
    }
}

#[test]
fn wi_impact_dedupes_one_best_row_per_file_per_group() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    write_file(
        root,
        "src/repeated_consumer.py",
        "from prompt_service import PromptService\nPromptService()\nPromptService()\n",
    );
    run_build(root);

    let output = run_wi(root, &["impact", "PromptService"]);
    let mut paths = std::collections::BTreeSet::new();
    for row in non_primary_impact_rows(&output) {
        let path = row
            .trim_start_matches("- ")
            .split(':')
            .next()
            .expect("path before colon");
        assert!(
            paths.insert(path.to_string()),
            "duplicate impacted file in:\n{output}"
        );
    }

    let repeated_rows = group_rows(&output, "References:")
        .into_iter()
        .filter(|row| row.contains("src/repeated_consumer.py"))
        .count();

    assert_eq!(
        repeated_rows, 1,
        "expected one best callers/importers row per file, got:\n{output}"
    );
}

#[test]
fn wi_impact_stale_dependency_changes_update_output() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "src/prompt_service.py", "class PromptService: pass\n");
    write_file(root, "src/consumer.py", "import prompt_service\n");
    run_build(root);

    let before = run_wi(root, &["impact", "PromptService"]);
    assert!(
        group_rows(&before, "Dependent files:")
            .iter()
            .any(|row| row.contains("src/consumer.py")),
        "expected consumer dependency before change:\n{before}"
    );

    write_file(root, "src/consumer.py", "import other_service\n");
    write_file(root, "src/other_service.py", "class OtherService: pass\n");
    run_build(root);

    let after = run_wi(root, &["impact", "PromptService"]);
    assert!(
        !group_rows(&after, "Dependent files:")
            .iter()
            .any(|row| row.contains("src/consumer.py")),
        "stale dependency should be removed from impact output:\n{after}"
    );
}

#[test]
fn wi_impact_filter_options_apply_to_primary_search() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    write_file(
        root,
        "legacy/prompt_service.py",
        "class PromptService: pass\n",
    );
    run_build(root);

    let output = run_wi(root, &["impact", "PromptService", "-p", "src/"]);
    let primary_rows = group_rows(&output, "Direct definitions:");

    assert!(
        primary_rows
            .iter()
            .any(|row| row.contains("src/prompt_service.py")),
        "expected filtered primary result:\n{output}"
    );
    assert!(
        primary_rows
            .iter()
            .all(|row| !row.contains("legacy/prompt_service.py")),
        "primary search should honor path filter:\n{output}"
    );
}

#[test]
fn wi_impact_logs_usage_with_subcommand() {
    let repo = temp_repo();
    let root = repo.path();
    write_context_fixture(root);
    run_build(root);

    run_wi(root, &["impact", "PromptService"]);

    let events = thinindex::stats::read_usage_events(root).expect("read usage events");
    let event = events.last().expect("at least one usage event");
    assert_eq!(event.query, "impact PromptService");
    assert!(event.hit, "expected impact hit event: {event:?}");
    assert!(
        event.result_count > 0,
        "expected positive result_count: {event:?}"
    );
}

#[test]
fn wi_impact_without_term_remains_normal_search() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(
        root,
        "src/impact_symbol.py",
        "def impact():\n    return 1\n",
    );
    run_build(root);

    let output = run_wi(root, &["impact"]);

    assert!(
        output.contains("src/impact_symbol.py") && !output.contains("Direct definitions:"),
        "`wi impact` without a term should remain normal search, got:\n{output}"
    );
}

#[test]
fn wi_refs_without_term_remains_normal_search() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "src/refs_symbol.py", "def refs():\n    return 1\n");
    run_build(root);

    let output = run_wi(root, &["refs"]);

    assert!(
        output.contains("src/refs_symbol.py") && !output.contains("Primary:"),
        "`wi refs` without a term should remain normal search, got:\n{output}"
    );
}

#[test]
fn css_extras_are_indexed() {
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
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "README.md",
        "# Thin Index\n\n- [ ] Add parser integration\n- [x] Add search\n\nSee [docs](docs/index.md).\n",
    );

    run_build(root);

    let heading = run_wi(root, &["Thin Index"]);
    assert!(heading.contains("README.md"));

    let checklist = run_wi(root, &["Add parser integration", "-t", "checklist"]);
    assert!(checklist.contains("README.md"));

    let link = run_wi(root, &["docs", "-t", "link"]);
    assert!(link.contains("README.md"));
}

#[test]
fn fixture_repo_indexes_python_symbols() {
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
    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    run_build(root);

    let heading = run_wi(root, &["Guide"]);
    assert!(
        heading.contains("docs/guide.md"),
        "expected markdown heading result, got:\n{heading}"
    );

    let checklist = run_wi(root, &["Add parser integration", "-t", "checklist"]);
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
fn fixture_repo_indexes_config_extras_without_scalar_noise() {
    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    run_build(root);

    let json_key = run_wi(root, &["parserConfigEnabled", "-t", "key"]);
    assert!(
        json_key.contains("config/app.json"),
        "expected JSON key result, got:\n{json_key}"
    );

    let toml_table = run_wi(root, &["tool.thinindex", "-t", "table"]);
    assert!(
        toml_table.contains("config/thinindex.toml"),
        "expected TOML table result, got:\n{toml_table}"
    );

    let yaml_section = run_wi(root, &["pipeline", "-t", "section"]);
    assert!(
        yaml_section.contains("config/pipeline.yaml"),
        "expected YAML section result, got:\n{yaml_section}"
    );

    let scalar_value = run_wi(root, &["YamlStringFake"]);
    assert!(
        scalar_value.trim().is_empty(),
        "YAML scalar string should not be indexed as a symbol, got:\n{scalar_value}"
    );
}

#[test]
fn fixture_repo_indexes_jsx_extras() {
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
    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    run_build(root);

    let variable_result = run_wi(root, &["-t", "css_variable", "--", "--paper-bg"]);
    assert!(
        variable_result.contains("frontend/styles/header.css"),
        "expected css variable result, got:\n{variable_result}"
    );
}

#[test]
fn wi_exact_symbol_match_beats_doc_or_path_match() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "docs/HeaderNavigation.md",
        "# HeaderNavigation notes\n",
    );
    write_file(
        root,
        "src/header.py",
        r#"
class HeaderNavigation:
    pass
"#,
    );

    run_build(root);

    let out = run_wi(root, &["HeaderNavigation", "-n", "1"]);
    assert!(
        out.contains("src/header.py"),
        "exact symbol match should rank first, got:\n{out}"
    );
}

#[test]
fn wi_source_result_beats_test_when_match_quality_is_equal() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "tests/prompt_service.py",
        r#"
class PromptService:
    pass
"#,
    );
    write_file(
        root,
        "src/prompt_service.py",
        r#"
class PromptService:
    pass
"#,
    );

    run_build(root);

    let out = run_wi(root, &["PromptService", "-n", "1"]);
    assert!(
        out.contains("src/prompt_service.py"),
        "source file should rank before tests for equal match, got:\n{out}"
    );
}

#[test]
fn wi_prefix_match_beats_substring_match() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/service.py",
        r#"
class AlphaPrompt:
    pass

class PromptService:
    pass
"#,
    );

    run_build(root);

    let out = run_wi(root, &["Prompt", "-n", "1"]);
    assert!(
        out.contains("PromptService"),
        "prefix match should rank before substring match, got:\n{out}"
    );
}

#[test]
fn wi_limit_returns_highest_ranked_result() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "docs/prompt.md", "# PromptService\n");
    write_file(
        root,
        "src/prompt_service.py",
        r#"
class PromptService:
    pass
"#,
    );

    run_build(root);

    let out = run_wi(root, &["PromptService", "-n", "1"]);
    assert!(
        out.contains("src/prompt_service.py"),
        "-n 1 should return the highest-ranked result, got:\n{out}"
    );
}

#[test]
fn wi_reports_stale_index_when_a_file_changes_after_initial_build() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/foo.py",
        r#"
class OldName:
    pass
"#,
    );

    run_build(root);

    let before = run_wi(root, &["OldName"]);
    assert!(
        before.contains("src/foo.py"),
        "OldName should be found before edit, got:\n{before}"
    );

    // Edit the file in-place, replacing the symbol. wi should not rebuild
    // automatically; it should tell the user to run build_index.
    write_file(
        root,
        "src/foo.py",
        r#"
class NewName:
    pass
"#,
    );

    let output = wi_bin()
        .current_dir(root)
        .arg("OldName")
        .output()
        .expect("run stale wi");
    assert!(
        !output.status.success(),
        "stale wi should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("run `build_index`"),
        "stale wi should tell user to run build_index, got:\n{stderr}"
    );

    run_build(root);
    let after_new = run_wi(root, &["NewName"]);
    assert!(
        after_new.contains("src/foo.py"),
        "NewName must appear after explicit rebuild, got:\n{after_new}"
    );
}

#[test]
fn wi_does_not_pollute_stdout_when_index_is_fresh() {
    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/foo.py",
        r#"
class OnlyName:
    pass
"#,
    );

    run_build(root);

    // Two consecutive wi calls without intervening edits: the second must
    // not emit any rebuild noise on stdout — only the search result.
    let _warm = run_wi(root, &["OnlyName"]);
    let stdout = run_wi(root, &["OnlyName"]);

    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(
        lines.len(),
        1,
        "fresh-index wi run should print exactly one result line, got:\n{stdout}"
    );
    assert!(
        lines[0].contains("src/foo.py"),
        "result line should reference src/foo.py, got:\n{stdout}"
    );
}
