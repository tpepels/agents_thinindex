mod common;

use std::{fs, path::Path};

use common::*;
use thinindex::agent_instructions::REPOSITORY_SEARCH_BLOCK;

fn write_scorecard_fixture(root: &Path) {
    write_file(root, ".gitignore", ".dev_index/\n");
    write_file(
        root,
        "src/checkout_service.py",
        r#"
from payment_gateway import PaymentGateway

class CheckoutService:
    def checkout_flow(self):
        return PaymentGateway().authorize()
"#,
    );
    write_file(
        root,
        "src/payment_gateway.py",
        r#"
class PaymentGateway:
    def authorize(self):
        return True
"#,
    );
    write_file(
        root,
        "src/checkout_consumer.py",
        r#"
from checkout_service import CheckoutService

def run_checkout():
    return CheckoutService().checkout_flow()
"#,
    );
    write_file(
        root,
        "tests/test_checkout_service.py",
        r#"
from checkout_service import CheckoutService

def test_checkout_flow():
    assert CheckoutService().checkout_flow()
"#,
    );
    write_file(
        root,
        "docs/checkout.md",
        "# Checkout workflow\n\nCheckoutService owns payment authorization.\n",
    );
    write_file(root, "config/checkout.toml", "checkout_enabled = true\n");
    write_file(
        root,
        "AGENTS.md",
        &format!("# AGENTS\n\n{REPOSITORY_SEARCH_BLOCK}\n"),
    );
    write_file(
        root,
        ".cursor/rules/thinindex.mdc",
        &format!("# thinindex\n\n{REPOSITORY_SEARCH_BLOCK}\n"),
    );
    write_file(
        root,
        ".github/copilot-instructions.md",
        &format!("# GitHub Copilot instructions\n\n{REPOSITORY_SEARCH_BLOCK}\n"),
    );
}

#[test]
fn scorecard_command_reports_all_required_dimensions() {
    let repo = temp_repo();
    let root = repo.path();
    write_scorecard_fixture(root);

    let output = wi_scorecard_bin()
        .current_dir(root)
        .args(["--query", "CheckoutService"])
        .output()
        .expect("run wi-scorecard");

    assert!(
        output.status.success(),
        "wi-scorecard failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for needle in [
        "thinindex value scorecard",
        "summary: pass",
        "[pass] wi <term> gives useful file:line results",
        "[pass] stale/missing index auto-recovers",
        "[pass] warm query latency is acceptable",
        "[pass] wi refs <term> gives useful references",
        "[pass] wi pack <term> gives a bounded useful read set",
        "[pass] wi impact <term> gives plausible affected files with reasons",
        "[pass] wi doctor gives actionable state",
        "[pass] wi-init creates useful agent instructions",
        "[pass] generated instructions match actual behavior",
        "[pass] unsupported/experimental parser support is not overclaimed",
    ] {
        assert!(
            stdout.contains(needle),
            "expected scorecard stdout to contain `{needle}`, got:\n{stdout}"
        );
    }
}

#[test]
fn scorecard_failures_are_actionable() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "src/main.py", "class PromptService: pass\n");

    let output = wi_scorecard_bin()
        .current_dir(root)
        .args(["--query", "NoSuchSymbolForScorecard"])
        .output()
        .expect("run wi-scorecard");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("[fail] wi <term> gives useful file:line results"));
    assert!(stdout.contains("action: try a more specific symbol"));
    assert!(stdout.contains("[fail] wi-init creates useful agent instructions"));
    assert!(stdout.contains("action: run `wi-init`"));
}

#[test]
fn normal_scorecard_test_does_not_require_test_repos() {
    let repo = temp_repo();
    let root = repo.path();
    write_scorecard_fixture(root);

    let report = thinindex::scorecard::run_scorecard(
        root,
        &thinindex::scorecard::ScorecardOptions {
            query: "CheckoutService".to_string(),
        },
    )
    .expect("run scorecard");

    assert_eq!(report.dimensions.len(), 10);
    assert_eq!(report.fail_count(), 0);
}

#[test]
#[ignore = "runs against optional ignored local test_repos/fd when present"]
fn ignored_real_repo_scorecard_can_run_when_test_repos_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_repos/fd");
    if !root.exists() {
        eprintln!("skipped: test_repos/fd missing");
        return;
    }

    run_build(&root);
    let output = wi_scorecard_bin()
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")))
        .args(["--repo", "test_repos/fd", "--query", "main"])
        .output()
        .expect("run real-repo wi-scorecard");

    assert!(
        output.status.success(),
        "real-repo scorecard failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("thinindex value scorecard"));
    assert!(stdout.contains("query: main"));
    assert!(stdout.contains("wi pack <term>"));
}

#[test]
fn checked_in_scorecard_docs_explain_interpretation() {
    let docs = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/SCORECARD.md"))
        .expect("read docs/SCORECARD.md");

    for needle in [
        "pass",
        "warn",
        "fail",
        "wi-scorecard",
        "No product claim should be promoted from a warning or failure",
    ] {
        assert!(
            docs.contains(needle),
            "scorecard docs should contain `{needle}`, got:\n{docs}"
        );
    }
}
