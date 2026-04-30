mod common;

use std::fs;

use anyhow::Result;
use thinindex::{
    bench::ExpectedSymbol,
    model::IndexRecord,
    quality::{
        ComparatorRecord, ComparatorRun, ComparatorStatus, QualityComparator,
        QualityComparisonOptions, UniversalCtagsComparator, compare_quality,
        parse_ctags_json_record, render_quality_report, write_quality_report,
    },
    store::load_records,
};

use common::{
    assert_no_ctags_source, load_index_snapshot_from_sqlite, run_build, temp_repo, write_file,
};

struct FakeComparator {
    records: Vec<ComparatorRecord>,
}

impl QualityComparator for FakeComparator {
    fn name(&self) -> &str {
        "fake-comparator"
    }

    fn run(&self, _repo_root: &std::path::Path) -> anyhow::Result<ComparatorRun> {
        Ok(ComparatorRun::completed(self.name(), self.records.clone()))
    }
}

#[test]
fn fake_comparator_adapter_works() {
    let comparator = FakeComparator {
        records: vec![ComparatorRecord::new(
            "src/lib.rs",
            10,
            Some(4),
            "function",
            "build_index",
            Some("rust"),
            "fake-comparator",
        )],
    };

    let temp = temp_repo();
    let run = comparator.run(temp.path()).expect("run fake comparator");

    assert_eq!(run.status, ComparatorStatus::Completed);
    assert_eq!(run.comparator, "fake-comparator");
    assert_eq!(run.records[0].name, "build_index");
}

#[test]
fn comparator_record_parsing_uses_ctags_json_shape() {
    let record = parse_ctags_json_record(
        r#"{"_type":"tag","name":"parse_me","path":"./src/lib.rs","line":12,"column":7,"kind":"function","language":"Rust"}"#,
        "universal-ctags",
    )
    .expect("parse ctags JSON")
    .expect("tag record");

    assert_eq!(record.path, "src/lib.rs");
    assert_eq!(record.line, 12);
    assert_eq!(record.column, Some(7));
    assert_eq!(record.kind, "function");
    assert_eq!(record.name, "parse_me");
    assert_eq!(record.language.as_deref(), Some("Rust"));
    assert_eq!(record.comparator, "universal-ctags");
}

#[test]
fn kind_mapping_and_matching_use_line_proximity() {
    let thinindex = vec![IndexRecord::new(
        "src/lib.rs",
        10,
        1,
        "rust",
        "function",
        "nearby",
        "fn nearby() {}",
        "tree-sitter",
    )];
    let comparator = ComparatorRun::completed(
        "fake-comparator",
        vec![ComparatorRecord::new(
            "src/lib.rs",
            12,
            None,
            "f",
            "nearby",
            Some("Rust"),
            "fake-comparator",
        )],
    );

    let report = compare_quality(
        &thinindex,
        &comparator,
        QualityComparisonOptions::new("fixture", "/tmp/fixture"),
    );

    assert_eq!(report.metrics[0].matched_symbol_count, 1);
    assert!(report.thinindex_only.is_empty());
    assert!(report.comparator_only.is_empty());
}

#[test]
fn report_includes_comparator_only_and_thinindex_only_symbols() {
    let thinindex = vec![
        IndexRecord::new(
            "src/lib.rs",
            10,
            1,
            "rust",
            "function",
            "shared",
            "fn shared() {}",
            "tree-sitter",
        ),
        IndexRecord::new(
            "src/lib.rs",
            20,
            1,
            "rust",
            "function",
            "thinindex_only",
            "fn thinindex_only() {}",
            "tree-sitter",
        ),
    ];
    let comparator = ComparatorRun::completed(
        "fake-comparator",
        vec![
            ComparatorRecord::new(
                "src/lib.rs",
                10,
                None,
                "function",
                "shared",
                Some("rust"),
                "fake-comparator",
            ),
            ComparatorRecord::new(
                "src/lib.rs",
                30,
                None,
                "function",
                "comparator_only",
                Some("rust"),
                "fake-comparator",
            ),
        ],
    );

    let report = compare_quality(
        &thinindex,
        &comparator,
        QualityComparisonOptions::new("fixture", "/tmp/fixture").with_expected_symbols(vec![
            ExpectedSymbol {
                language: Some("rust".to_string()),
                path: Some("src/lib.rs".to_string()),
                kind: Some("function".to_string()),
                name: "shared".to_string(),
            },
            ExpectedSymbol {
                language: Some("rust".to_string()),
                path: Some("src/lib.rs".to_string()),
                kind: Some("function".to_string()),
                name: "missing_expected".to_string(),
            },
        ]),
    );
    let rendered = render_quality_report(&report);

    assert!(rendered.contains("Thinindex-only:"));
    assert!(rendered.contains("thinindex_only"));
    assert!(rendered.contains("Comparator-only:"));
    assert!(rendered.contains("comparator_only"));
    assert_eq!(report.expected_symbols_checked, 2);
    assert_eq!(report.expected_symbols_missing.len(), 1);
}

#[test]
fn missing_optional_comparator_skips_cleanly() {
    let comparator = UniversalCtagsComparator::new("definitely-no-such-thinindex-ctags-command");
    let temp = temp_repo();
    let run = comparator
        .run(temp.path())
        .expect("missing comparator skips");

    assert_eq!(run.status, ComparatorStatus::Skipped);
    assert!(run.records.is_empty());
    assert!(
        run.message
            .as_deref()
            .unwrap_or_default()
            .contains("comparator not found")
    );
}

#[test]
fn comparator_output_is_not_written_to_production_index_tables() {
    let temp = temp_repo();
    write_file(temp.path(), "src/lib.rs", "pub fn indexed_symbol() {}\n");
    run_build(temp.path());
    let before = load_records(temp.path()).expect("load records before quality report");
    assert_no_ctags_source("quality-before", &before);

    let report = compare_quality(
        &before,
        &ComparatorRun::completed(
            "fake-comparator",
            vec![ComparatorRecord::new(
                "src/lib.rs",
                1,
                None,
                "function",
                "indexed_symbol",
                Some("rust"),
                "fake-comparator",
            )],
        ),
        QualityComparisonOptions::new("fixture", temp.path().display().to_string()),
    );
    let report_path = write_quality_report(
        temp.path(),
        "fake-comparator",
        &render_quality_report(&report),
    )
    .expect("write isolated quality report");

    assert!(report_path.ends_with(".dev_index/quality/fake-comparator.txt"));
    assert!(report_path.exists());

    let snapshot = load_index_snapshot_from_sqlite(temp.path());
    assert_no_ctags_source("quality-after", &snapshot.records);
    assert_eq!(before, snapshot.records);
    assert!(
        !snapshot
            .records
            .iter()
            .any(|record| record.path.contains(".dev_index/quality"))
    );
}

#[test]
fn quality_ctags_mentions_are_isolated_or_explicitly_forbidden_boundary_checks() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let paths = [
        "src",
        "tests",
        "docs",
        "README.md",
        "Cargo.toml",
        "install.sh",
        "uninstall.sh",
        "THIRD_PARTY_NOTICES",
        "scripts",
    ];

    for path in paths {
        let path = root.join(path);
        if !path.exists() {
            continue;
        }
        scan_ctags_mentions(&path, root);
    }
}

#[test]
#[ignore]
fn optional_external_ctags_comparator_generates_isolated_quality_report_or_skips() -> Result<()> {
    let temp = temp_repo();
    write_file(
        temp.path(),
        "src/lib.rs",
        "pub fn indexed_symbol() {}\npub struct IndexedType;\n",
    );
    run_build(temp.path());

    let comparator = UniversalCtagsComparator::default();
    let run = comparator.run(temp.path())?;
    if run.is_skipped() {
        eprintln!(
            "optional comparator skipped: {}",
            run.message.as_deref().unwrap_or("no message")
        );
        return Ok(());
    }

    let records = load_records(temp.path())?;
    let report = compare_quality(
        &records,
        &run,
        QualityComparisonOptions::new("fixture", temp.path().display().to_string()),
    );
    let rendered = render_quality_report(&report);
    let report_path = write_quality_report(temp.path(), &run.comparator, &rendered)?;

    assert!(report_path.exists());
    assert!(report_path.to_string_lossy().contains(".dev_index/quality"));
    assert_no_ctags_source("optional-comparator", &load_records(temp.path())?);
    Ok(())
}

#[test]
#[ignore]
fn malformed_thinindex_records_fail_quality_checks() {
    let thinindex = vec![IndexRecord::new(
        "",
        0,
        0,
        "rust",
        "function",
        "broken",
        "fn broken() {}",
        "tree-sitter",
    )];
    let report = compare_quality(
        &thinindex,
        &ComparatorRun::completed("fake-comparator", Vec::new()),
        QualityComparisonOptions::new("fixture", "/tmp/fixture"),
    );

    assert!(
        thinindex::quality::report::assert_quality_report_has_no_malformed_thinindex_records(
            &report
        )
        .is_err()
    );
}

fn scan_ctags_mentions(path: &std::path::Path, root: &std::path::Path) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("read scan dir") {
            let entry = entry.expect("read scan entry");
            let path = entry.path();
            if path
                .components()
                .any(|component| component.as_os_str() == ".git")
            {
                continue;
            }
            scan_ctags_mentions(&path, root);
        }
        return;
    }

    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    let relpath = path.strip_prefix(root).unwrap_or(path).to_string_lossy();

    for (line_number, line) in contents.lines().enumerate() {
        if !line.to_ascii_lowercase().contains("ctags") {
            continue;
        }

        assert!(
            allowed_ctags_reference(&relpath, line),
            "{}:{} contains an unapproved ctags reference: {}",
            relpath,
            line_number + 1,
            line,
        );
    }
}

fn allowed_ctags_reference(relpath: &str, line: &str) -> bool {
    if relpath.starts_with("src/quality/")
        || relpath.starts_with("tests/quality")
        || relpath.starts_with("docs/QUALITY")
    {
        return true;
    }

    if relpath.starts_with("tests/")
        && (relpath.contains("release")
            || relpath.contains("install")
            || relpath.contains("license")
            || relpath.contains("common"))
    {
        return true;
    }

    if relpath.starts_with("scripts/") && line.contains("reject_entry") {
        return true;
    }

    let normalized = line.to_ascii_lowercase();
    normalized.contains("removed")
        || normalized.contains("not bundled")
        || normalized.contains("not used")
        || normalized.contains("not required")
        || normalized.contains("not called")
        || normalized.contains("not detected")
        || normalized.contains("no longer shells out")
        || normalized.contains("must not")
}
