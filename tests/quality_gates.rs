mod common;

use std::{fs, path::Path};

use anyhow::Result;
use thinindex::{
    bench::{
        BenchmarkRepoSet, ExpectedAbsentSymbol, ExpectedSymbol, ExpectedSymbolPattern,
        QualityThreshold,
    },
    indexer::build_index,
    model::{IndexRecord, ReferenceRecord},
    quality::{
        ComparatorRecord, ComparatorRun, QualityComparator, QualityGateOptions,
        UniversalCtagsComparator, assert_quality_gate_passes, evaluate_quality_gate,
        load_quality_repo_set, render_quality_gate_report,
    },
};

use common::{load_index_snapshot_from_sqlite, run_build, temp_repo, write_file};

#[test]
fn normal_deterministic_quality_gate_passes_for_tiny_fixture_repo() {
    let temp = temp_repo();
    write_file(
        temp.path(),
        "src/lib.rs",
        "pub fn gate_symbol() {}\npub struct GateType;\n",
    );
    run_build(temp.path());
    let snapshot = load_index_snapshot_from_sqlite(temp.path());

    let report = evaluate_quality_gate(
        &snapshot.records,
        &snapshot.refs,
        QualityGateOptions::new("tiny-fixture", temp.path().display().to_string())
            .with_expected_symbols(vec![ExpectedSymbol {
                language: Some("rs".to_string()),
                path: Some("src/lib.rs".to_string()),
                kind: Some("function".to_string()),
                name: "gate_symbol".to_string(),
            }])
            .with_expected_symbol_patterns(vec![ExpectedSymbolPattern {
                language: Some("rs".to_string()),
                path_glob: Some("src/**/*.rs".to_string()),
                kind: Some("function".to_string()),
                name_regex: "^gate_.*".to_string(),
                min_count: 1,
            }])
            .with_expected_absent_symbols(vec![ExpectedAbsentSymbol {
                language: Some("rs".to_string()),
                path: Some("src/lib.rs".to_string()),
                kind: Some("function".to_string()),
                name: "gate_symbol_from_comment".to_string(),
            }])
            .with_quality_thresholds(vec![QualityThreshold {
                language: "rs".to_string(),
                min_records: Some(2),
                max_duplicate_locations: Some(0),
                max_malformed_records: Some(0),
            }]),
    )
    .expect("evaluate gate");

    assert_quality_gate_passes(&report).expect("quality gate passes");
    let rendered = render_quality_gate_report(&report);
    assert!(rendered.contains("- expected symbols: 1 checked, 0 missing"));
    assert!(rendered.contains("- expected patterns: 1 checked, 0 failing"));
    assert!(rendered.contains("- expected absent symbols: 1 checked, 0 found"));
    assert!(rendered.contains("- records by language: rs="));
}

#[test]
fn missing_expected_symbol_message_is_actionable() {
    let records = vec![record("src/lib.rs", 1, 1, "rs", "function", "present")];
    let report = evaluate_quality_gate(
        &records,
        &[],
        QualityGateOptions::new("fixture", "/tmp/fixture").with_expected_symbols(vec![
            ExpectedSymbol {
                language: Some("rs".to_string()),
                path: Some("src/lib.rs".to_string()),
                kind: Some("function".to_string()),
                name: "missing_symbol".to_string(),
            },
        ]),
    )
    .expect("evaluate gate");
    let error = assert_quality_gate_passes(&report).expect_err("missing symbol should fail");
    let message = format!("{error:#}");

    assert!(message.contains("quality drift gate failed for fixture"));
    assert!(message.contains("missing expected symbol"));
    assert!(message.contains("repo=fixture"));
    assert!(message.contains("missing_symbol"));
    assert!(message.contains("src/lib.rs"));
    assert!(message.contains("nearby=src/lib.rs:1:1 function present (rs)"));
}

#[test]
fn expected_symbol_patterns_and_thresholds_are_checked() {
    let records = vec![
        record("src/a.rs", 1, 1, "rs", "function", "build_alpha"),
        record("src/b.rs", 2, 1, "rs", "function", "build_beta"),
    ];

    let passing = evaluate_quality_gate(
        &records,
        &[],
        QualityGateOptions::new("fixture", "/tmp/fixture")
            .with_expected_symbol_patterns(vec![ExpectedSymbolPattern {
                language: Some("rs".to_string()),
                path_glob: Some("src/**/*.rs".to_string()),
                kind: Some("function".to_string()),
                name_regex: "^build_.*".to_string(),
                min_count: 2,
            }])
            .with_quality_thresholds(vec![QualityThreshold {
                language: "rs".to_string(),
                min_records: Some(2),
                max_duplicate_locations: Some(0),
                max_malformed_records: Some(0),
            }]),
    )
    .expect("evaluate passing gate");
    assert_quality_gate_passes(&passing).expect("pattern and threshold should pass");

    let failing = evaluate_quality_gate(
        &records,
        &[],
        QualityGateOptions::new("fixture", "/tmp/fixture")
            .with_expected_symbol_patterns(vec![ExpectedSymbolPattern {
                language: Some("rs".to_string()),
                path_glob: Some("src/**/*.rs".to_string()),
                kind: Some("function".to_string()),
                name_regex: "^build_.*".to_string(),
                min_count: 3,
            }])
            .with_quality_thresholds(vec![QualityThreshold {
                language: "rs".to_string(),
                min_records: Some(3),
                max_duplicate_locations: Some(0),
                max_malformed_records: Some(0),
            }]),
    )
    .expect("evaluate failing gate");
    let rendered = render_quality_gate_report(&failing);

    assert!(rendered.contains("min_count=3 actual=2"));
    assert!(rendered.contains("min_records=3 actual=2"));
    assert!(assert_quality_gate_passes(&failing).is_err());
}

#[test]
fn expected_absent_symbols_fail_when_found() {
    let records = vec![
        record(
            "src/lib.rs",
            1,
            1,
            "rs",
            "function",
            "NotARealSymbolFromComment",
        ),
        record("src/lib.rs", 2, 1, "rs", "function", "present"),
    ];
    let report = evaluate_quality_gate(
        &records,
        &[],
        QualityGateOptions::new("fixture", "/tmp/fixture").with_expected_absent_symbols(vec![
            ExpectedAbsentSymbol {
                language: Some("rs".to_string()),
                path: Some("src/lib.rs".to_string()),
                kind: Some("function".to_string()),
                name: "NotARealSymbolFromComment".to_string(),
            },
        ]),
    )
    .expect("evaluate gate");
    let rendered = render_quality_gate_report(&report);

    assert!(rendered.contains("- expected absent symbols: 1 checked, 1 found"));
    assert!(rendered.contains("Found expected-absent symbols:"));
    assert!(rendered.contains("repo=fixture"));
    assert!(rendered.contains("NotARealSymbolFromComment"));
    assert!(rendered.contains("matches=src/lib.rs:1:1 function NotARealSymbolFromComment (rs)"));
    assert!(assert_quality_gate_passes(&report).is_err());
}

#[test]
fn gate_fails_integrity_regressions() {
    let records = vec![
        record("src/lib.rs", 1, 1, "rs", "function", "duplicate"),
        record("src/lib.rs", 1, 1, "rs", "function", "duplicate_again"),
        IndexRecord::new(
            ".dev_index/quality/report.txt",
            0,
            0,
            "rs",
            "function",
            "bad",
            "bad",
            "ctags",
        ),
    ];
    let refs = vec![ReferenceRecord::new(
        ".dev_index/quality/report.txt",
        0,
        0,
        "",
        None::<String>,
        "",
        "",
        "",
    )];

    let report = evaluate_quality_gate(
        &records,
        &refs,
        QualityGateOptions::new("fixture", "/tmp/fixture"),
    )
    .expect("evaluate gate");
    let rendered = render_quality_gate_report(&report);

    assert_eq!(report.duplicate_record_count, 1);
    assert_eq!(report.malformed_record_count, 1);
    assert_eq!(report.malformed_ref_count, 1);
    assert_eq!(report.dev_index_path_count, 2);
    assert_eq!(report.ctags_source_count, 1);
    assert!(rendered.contains("ctags-sources=1"));
    assert!(assert_quality_gate_passes(&report).is_err());
}

#[test]
fn comparator_only_symbols_do_not_fail_normal_deterministic_gate() {
    let records = vec![record(
        "src/lib.rs",
        1,
        1,
        "rs",
        "function",
        "thinindex_symbol",
    )];
    let comparator_run = ComparatorRun::completed(
        "fake-comparator",
        vec![ComparatorRecord::new(
            "src/lib.rs",
            10,
            None,
            "function",
            "comparator_only",
            Some("rs"),
            "fake-comparator",
        )],
    );

    let report = evaluate_quality_gate(
        &records,
        &[],
        QualityGateOptions::new("fixture", "/tmp/fixture").with_comparator_run(comparator_run),
    )
    .expect("evaluate gate");

    assert_quality_gate_passes(&report).expect("comparator-only symbols are triage data");
    assert_eq!(
        report
            .comparator_report
            .as_ref()
            .expect("comparator report")
            .comparator_only
            .len(),
        1
    );
    assert!(render_quality_gate_report(&report).contains("comparator-only=1"));
}

#[test]
fn drift_report_ordering_is_deterministic() {
    let records = vec![
        record("src/b.py", 2, 1, "py", "function", "beta"),
        record("src/a.rs", 1, 1, "rs", "function", "alpha"),
    ];
    let options = QualityGateOptions::new("fixture", "/tmp/fixture").with_quality_thresholds(vec![
        QualityThreshold {
            language: "rs".to_string(),
            min_records: Some(1),
            max_duplicate_locations: Some(0),
            max_malformed_records: Some(0),
        },
    ]);

    let first = evaluate_quality_gate(&records, &[], options.clone()).expect("first gate");
    let second = evaluate_quality_gate(&records, &[], options).expect("second gate");
    let rendered = render_quality_gate_report(&first);

    assert_eq!(rendered, render_quality_gate_report(&second));
    assert!(rendered.contains("- languages checked: py, rs"));
    assert!(rendered.contains("- records by language: py=1, rs=1"));
}

#[test]
#[ignore = "rebuilds .dev_index for every repo under test_repos/; run with: cargo test --test quality_gates -- --ignored"]
fn real_repo_quality_gate_uses_manifest_and_optional_comparator_report() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_repos");
    let repo_set = load_quality_repo_set(&root)?;

    let BenchmarkRepoSet::Repos {
        manifest_used,
        repos,
    } = repo_set
    else {
        match repo_set {
            BenchmarkRepoSet::MissingRoot => println!("skipped: test_repos/ missing"),
            BenchmarkRepoSet::Empty => println!("skipped: test_repos/ has no repo directories"),
            BenchmarkRepoSet::Repos { .. } => unreachable!(),
        }
        return Ok(());
    };

    println!(
        "quality gate testing {} repo(s){}:",
        repos.len(),
        if manifest_used {
            " from MANIFEST.toml"
        } else {
            ""
        }
    );

    for repo in repos {
        let dev_index = repo.path.join(".dev_index");
        if dev_index.exists() {
            fs::remove_dir_all(&dev_index)?;
        }
        build_index(&repo.path)?;
        let snapshot = load_index_snapshot_from_sqlite(&repo.path);

        let comparator = UniversalCtagsComparator::default();
        let comparator_run = comparator.run(&repo.path)?;

        let report = evaluate_quality_gate(
            &snapshot.records,
            &snapshot.refs,
            QualityGateOptions::from_benchmark_repo(&repo).with_comparator_run(comparator_run),
        )?;
        println!("{}", render_quality_gate_report(&report));
        assert_quality_gate_passes(&report)?;
    }

    Ok(())
}

fn record(path: &str, line: usize, col: usize, lang: &str, kind: &str, name: &str) -> IndexRecord {
    IndexRecord::new(
        path,
        line,
        col,
        lang,
        kind,
        name,
        format!("{kind} {name}"),
        "tree-sitter",
    )
}
