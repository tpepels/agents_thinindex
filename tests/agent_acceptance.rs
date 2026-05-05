mod common;

use std::path::Path;

use common::{temp_repo, wi_bin, write_file};

fn run_agent_step(root: &Path, args: &[&str]) -> (String, String) {
    let output = wi_bin()
        .current_dir(root)
        .args(args)
        .output()
        .expect("run wi agent step");

    assert!(
        output.status.success(),
        "agent step failed: wi {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn workflow_row_count(output: &str) -> usize {
    output.lines().filter(|line| line.starts_with("- ")).count()
}

fn assert_rows_have_reasons_and_confidence(output: &str) {
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

fn assert_normal_path_did_not_run_sidecar_checks(root: &Path, outputs: &[&str]) {
    assert!(
        !root.join(".dev_index/quality").exists(),
        "normal agent acceptance workflow must not create .dev_index/quality"
    );

    let combined = outputs.join("\n");
    for unexpected in ["quality report", "comparator", "real-repo", "test_repos"] {
        assert!(
            !combined.contains(unexpected),
            "normal agent acceptance workflow should not run {unexpected} work:\n{combined}"
        );
    }
}

fn write_invoice_fixture(root: &Path, class_name: &str) {
    write_file(root, ".gitignore", ".dev_index/\n");
    write_file(
        root,
        "src/invoice_service.py",
        &format!(
            r#"
from discount_policy import DiscountPolicy
from tax_policy import TaxPolicy

class {class_name}:
    def __init__(self):
        self.discount_policy = DiscountPolicy()
        self.tax_policy = TaxPolicy()

    def quote_total(self, subtotal):
        discounted = self.discount_policy.apply(subtotal)
        return self.tax_policy.add_tax(discounted)
"#
        ),
    );
    write_file(
        root,
        "src/discount_policy.py",
        r#"
class DiscountPolicy:
    def apply(self, subtotal):
        return subtotal
"#,
    );
    write_file(
        root,
        "src/tax_policy.py",
        r#"
class TaxPolicy:
    def add_tax(self, subtotal):
        return round(subtotal * 1.07, 2)
"#,
    );
    write_file(
        root,
        "src/invoice_controller.py",
        &format!(
            r#"
from invoice_service import {class_name}

def quote_invoice(subtotal):
    return {class_name}().quote_total(subtotal)
"#
        ),
    );
    write_file(
        root,
        "tests/test_invoice_service.py",
        &format!(
            r#"
from invoice_service import {class_name}

def test_quote_total_applies_tax():
    assert {class_name}().quote_total(100) == 107
"#
        ),
    );
    write_file(
        root,
        "docs/invoice_workflow.md",
        &format!("# Invoice workflow\n\n{class_name} owns invoice totals for checkout quotes.\n"),
    );
    write_file(root, "config/invoice.toml", "invoice_tax_rate = 0.07\n");
}

#[test]
fn minimum_agent_acceptance_workflow_is_useful_bounded_and_self_healing() {
    let repo = temp_repo();
    let root = repo.path();
    write_invoice_fixture(root, "InvoiceService");

    let (search, missing_stderr) = run_agent_step(root, &["InvoiceService"]);
    assert!(
        search.contains("src/invoice_service.py") && search.contains("class InvoiceService"),
        "agent search should find the likely edit target, got:\n{search}"
    );
    assert!(
        missing_stderr.contains("running `build_index` once"),
        "missing index should self-heal through the first wi search, got:\n{missing_stderr}"
    );
    assert!(
        workflow_row_count(&search) <= 10,
        "initial agent search should stay compact, got:\n{search}"
    );

    let (refs, refs_stderr) = run_agent_step(root, &["refs", "InvoiceService"]);
    assert!(
        refs.contains("Primary:")
            && refs.contains("src/invoice_controller.py")
            && refs.contains("tests/test_invoice_service.py")
            && refs.contains("reason:"),
        "agent refs step should expose callers and tests, got:\n{refs}"
    );
    assert!(
        workflow_row_count(&refs) <= 25,
        "refs output should stay bounded, got:\n{refs}"
    );

    let (pack, pack_stderr) = run_agent_step(root, &["pack", "InvoiceService"]);
    for expected in [
        "src/invoice_service.py",
        "src/discount_policy.py",
        "src/tax_policy.py",
        "src/invoice_controller.py",
        "tests/test_invoice_service.py",
        "docs/invoice_workflow.md",
        "config/invoice.toml",
    ] {
        assert!(
            pack.contains(expected),
            "agent pack should include {expected}, got:\n{pack}"
        );
    }
    assert_rows_have_reasons_and_confidence(&pack);
    assert!(
        workflow_row_count(&pack) <= 20,
        "pack output should stay bounded, got:\n{pack}"
    );

    let (impact, impact_stderr) = run_agent_step(root, &["impact", "InvoiceService"]);
    assert!(
        impact.contains("src/invoice_controller.py")
            && impact.contains("tests/test_invoice_service.py")
            && impact.contains("config/invoice.toml")
            && impact.contains("reason:")
            && impact.contains("confidence:"),
        "agent impact step should expose plausible affected/test/config files, got:\n{impact}"
    );
    assert!(
        workflow_row_count(&impact) <= 25,
        "impact output should stay bounded, got:\n{impact}"
    );

    assert!(
        pack.contains("src/invoice_service.py") && impact.contains("tests/test_invoice_service.py"),
        "agent should be able to identify likely edit and test files from pack/impact"
    );

    write_invoice_fixture(root, "InvoiceWorkflowService");
    let (stale_search, stale_stderr) = run_agent_step(root, &["InvoiceWorkflowService"]);
    assert!(
        stale_search.contains("src/invoice_service.py")
            && stale_search.contains("class InvoiceWorkflowService"),
        "stale-index search should continue against the rebuilt index, got:\n{stale_search}"
    );
    assert!(
        stale_stderr.contains("index is stale")
            && stale_stderr.contains("running `build_index` once"),
        "stale index should self-heal once, got:\n{stale_stderr}"
    );

    let (warm_search, warm_stderr) = run_agent_step(root, &["InvoiceWorkflowService"]);
    assert!(
        warm_search.contains("src/invoice_service.py"),
        "warm second search should still find the edited symbol, got:\n{warm_search}"
    );
    assert!(
        !warm_stderr.contains("running `build_index` once"),
        "immediate second search should not rebuild, got:\n{warm_stderr}"
    );

    assert_normal_path_did_not_run_sidecar_checks(
        root,
        &[
            &search,
            &missing_stderr,
            &refs,
            &refs_stderr,
            &pack,
            &pack_stderr,
            &impact,
            &impact_stderr,
            &stale_search,
            &stale_stderr,
            &warm_search,
            &warm_stderr,
        ],
    );
}
