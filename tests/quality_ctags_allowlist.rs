use std::{panic, path::Path};

use thinindex::{
    model::{IndexRecord, ReferenceRecord},
    quality::{
        assert_no_forbidden_index_sources, check_ctags_allowlist, check_package_artifacts,
        scan_repo_for_ctags_allowlist,
    },
};

const EXPLICIT_BOUNDARY: &str = "Universal Ctags is optional, external, not bundled, not required, and not used by production indexing.";

#[test]
fn allowlist_checker_blocks_forbidden_surfaces_and_allows_quality_boundaries() {
    let violations = check_ctags_allowlist([
        ("src/indexer.rs", "let parser = \"ctags\";\n"),
        ("src/quality/comparator.rs", "let parser = \"ctags\";\n"),
        (
            "tests/quality.rs",
            "assert!(\"ctags\".contains(\"ctags\"));\n",
        ),
        ("tests/release_packaging.rs", "let forbidden = \"ctags\";\n"),
        ("docs/QUALITY.md", "ctags quality docs\n"),
        ("docs/QUALITY_CTAG_BOUNDARY.md", "ctags quality docs\n"),
        ("README.md", EXPLICIT_BOUNDARY),
        ("docs/INSTALLERS.md", EXPLICIT_BOUNDARY),
    ]);

    assert_eq!(
        violations
            .iter()
            .map(|violation| violation.path.as_str())
            .collect::<Vec<_>>(),
        vec![
            "src/indexer.rs",
            "tests/release_packaging.rs",
            "docs/INSTALLERS.md"
        ]
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.reason.contains("quality modules"))
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.reason.contains("non-quality test"))
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.reason.contains("install/release/package"))
    );
}

#[test]
fn explicit_boundary_docs_must_state_every_required_condition() {
    let missing_required_phrase =
        "Universal Ctags is optional, external, not bundled, and not required.";
    let violations = check_ctags_allowlist([
        ("docs/ROADMAP.md", EXPLICIT_BOUNDARY),
        ("docs/LICENSE_AUDIT.md", missing_required_phrase),
    ]);

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].path, "docs/LICENSE_AUDIT.md");
}

#[test]
fn package_artifact_gate_blocks_ctags_binaries_only() {
    let violations = check_package_artifacts([
        "thinindex-1.0.0/wi",
        "thinindex-1.0.0/bin/ctags",
        "thinindex-1.0.0/bin/ctags.exe",
        "thinindex-1.0.0/tools/ctags-extra",
        "thinindex-1.0.0/docs/QUALITY.md",
    ]);

    assert_eq!(
        violations
            .iter()
            .map(|violation| violation.path.as_str())
            .collect::<Vec<_>>(),
        vec![
            "thinindex-1.0.0/bin/ctags",
            "thinindex-1.0.0/bin/ctags.exe",
            "thinindex-1.0.0/tools/ctags-extra",
        ]
    );
}

#[test]
fn production_index_source_gate_blocks_ctags_records_and_refs() {
    let record_result = panic::catch_unwind(|| {
        assert_no_forbidden_index_sources(
            "fixture-record",
            &[IndexRecord::new(
                "src/lib.rs",
                1,
                1,
                "rust",
                "function",
                "symbol",
                "fn symbol() {}",
                "ctags",
            )],
            &[],
        );
    });
    assert!(record_result.is_err());

    let ref_result = panic::catch_unwind(|| {
        assert_no_forbidden_index_sources(
            "fixture-ref",
            &[],
            &[ReferenceRecord::new(
                "src/lib.rs",
                1,
                1,
                "symbol",
                None::<String>,
                "usage",
                "symbol();",
                "ctags",
            )],
        );
    });
    assert!(ref_result.is_err());
}

#[test]
fn repository_ctags_allowlist_gate_passes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let violations = scan_repo_for_ctags_allowlist(root).expect("scan repository");

    assert!(
        violations.is_empty(),
        "ctags allowlist violations: {violations:#?}"
    );
}
