mod common;

use std::{fs, path::Path};

use anyhow::Result;
use thinindex::{
    bench::{BenchmarkRepoSet, ExpectedAbsentSymbol, ExpectedSymbol},
    indexer::build_index,
    model::IndexRecord,
    quality::{
        ComparatorRecord, ComparatorRun, CyclePlanOptions, GapSeverity, GapStatus,
        QualityComparator, QualityGap, QualityGapReport, QualityGateOptions, SuggestedFixType,
        UniversalCtagsComparator, assert_quality_gate_passes, evaluate_quality_gate,
        gaps_from_gate_report, generate_cycle_plan, group_gaps, load_quality_repo_set,
        render_quality_cycle_plan, render_quality_gap_report, write_quality_cycle_plan,
        write_quality_gap_report,
    },
    store::load_records,
};

use common::{load_index_snapshot_from_sqlite, run_build, temp_repo, write_file};

#[test]
fn quality_gap_model_collects_actionable_evidence() {
    let records = vec![record("src/lib.rs", 1, 1, "rs", "function", "present")];
    let comparator = ComparatorRun::completed(
        "fake-comparator",
        vec![ComparatorRecord::new(
            "src/lib.rs",
            8,
            None,
            "function",
            "from_comparator",
            Some("rs"),
            "fake-comparator",
        )],
    );
    let gate = evaluate_quality_gate(
        &records,
        &[],
        QualityGateOptions::new("fixture", "/tmp/fixture")
            .with_expected_symbols(vec![ExpectedSymbol {
                language: Some("rs".to_string()),
                path: Some("src/lib.rs".to_string()),
                kind: Some("function".to_string()),
                name: "missing_symbol".to_string(),
            }])
            .with_expected_absent_symbols(vec![ExpectedAbsentSymbol {
                language: Some("rs".to_string()),
                path: Some("src/lib.rs".to_string()),
                kind: Some("function".to_string()),
                name: "present".to_string(),
            }])
            .with_comparator_run(comparator),
    )
    .expect("evaluate gate");

    let report = gaps_from_gate_report(&gate);
    let rendered = render_quality_gap_report(&report);

    assert!(report.gaps.iter().any(|gap| {
        gap.evidence_source == "expected-symbol"
            && gap
                .symbol
                .as_deref()
                .is_some_and(|symbol| symbol.contains("missing_symbol"))
            && gap.severity == GapSeverity::High
    }));
    assert!(report.gaps.iter().any(|gap| {
        gap.evidence_source == "expected-absent-symbol"
            && gap
                .symbol
                .as_deref()
                .is_some_and(|symbol| symbol.contains("present"))
            && gap.severity == GapSeverity::High
    }));
    assert!(
        report
            .gaps
            .iter()
            .any(|gap| gap.evidence_source == "comparator-only"
                && gap.symbol.as_deref() == Some("from_comparator"))
    );
    assert!(rendered.contains("fixture_added: no"));
    assert!(rendered.contains("manifest_added: no"));
    assert!(rendered.contains("suggested_fix: parser-query"));
}

#[test]
fn gap_grouping_is_deterministic() {
    let report = QualityGapReport {
        repo_name: "fixture".to_string(),
        repo_path: "/tmp/fixture".to_string(),
        gaps: vec![
            gap(
                "GAP-0002",
                "py",
                "function",
                "comparator-only",
                GapSeverity::Medium,
            ),
            gap(
                "GAP-0001",
                "rs",
                "function",
                "expected-symbol",
                GapSeverity::High,
            ),
            gap(
                "GAP-0003",
                "rs",
                "function",
                "expected-symbol",
                GapSeverity::High,
            ),
        ],
    };

    let groups = group_gaps(&report.gaps);

    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].language, "py");
    assert_eq!(groups[1].language, "rs");
    assert_eq!(groups[1].gap_ids, vec!["GAP-0001", "GAP-0003"]);
}

#[test]
fn cycle_plan_is_bounded_and_prioritizes_expected_symbols_over_comparator_noise() {
    let mut gaps = Vec::new();
    for index in 0..12 {
        let evidence = if index < 2 {
            "comparator-only"
        } else {
            "expected-symbol"
        };
        let severity = if index < 2 {
            GapSeverity::Medium
        } else {
            GapSeverity::High
        };
        gaps.push(gap(
            &format!("GAP-{index:04}"),
            "rs",
            "function",
            evidence,
            severity,
        ));
    }
    let report = QualityGapReport {
        repo_name: "fixture".to_string(),
        repo_path: "/tmp/fixture".to_string(),
        gaps,
    };

    let plan = generate_cycle_plan(&report, CyclePlanOptions::default());

    assert_eq!(plan.selected_gaps.len(), 10);
    assert_eq!(plan.deferred_gap_ids.len(), 2);
    assert!(
        plan.selected_gaps
            .iter()
            .take(2)
            .all(|gap| gap.evidence_source == "expected-symbol")
    );
    assert!(
        render_quality_cycle_plan(&plan).contains("- one-cycle limit: true"),
        "plan must document bounded cycle behavior"
    );
}

#[test]
fn triage_status_handling_excludes_non_open_gaps_from_plan() {
    let report = QualityGapReport {
        repo_name: "fixture".to_string(),
        repo_path: "/tmp/fixture".to_string(),
        gaps: vec![
            gap(
                "GAP-0001",
                "rs",
                "function",
                "expected-symbol",
                GapSeverity::High,
            )
            .with_status(GapStatus::Fixed)
            .with_fixture_added(true),
            gap(
                "GAP-0002",
                "rs",
                "function",
                "comparator-only",
                GapSeverity::Medium,
            )
            .with_status(GapStatus::FalsePositive),
            gap(
                "GAP-0003",
                "rs",
                "macro",
                "expected-symbol",
                GapSeverity::High,
            )
            .with_manifest_added(true),
        ],
    };

    let plan = generate_cycle_plan(&report, CyclePlanOptions::default());
    let rendered = render_quality_gap_report(&report);

    assert_eq!(
        plan.selected_gaps
            .iter()
            .map(|gap| gap.id.as_str())
            .collect::<Vec<_>>(),
        vec!["GAP-0003"]
    );
    assert!(rendered.contains("status: fixed"));
    assert!(rendered.contains("status: false-positive"));
    assert!(rendered.contains("fixture_added: yes"));
    assert!(rendered.contains("manifest_added: yes"));
}

#[test]
fn quality_loop_output_ordering_is_deterministic() {
    let records = vec![
        record("src/b.py", 2, 1, "py", "function", "beta"),
        record("src/a.rs", 1, 1, "rs", "function", "alpha"),
    ];
    let comparator = ComparatorRun::completed(
        "fake-comparator",
        vec![
            ComparatorRecord::new(
                "src/z.rs",
                9,
                None,
                "function",
                "zeta",
                Some("rs"),
                "fake-comparator",
            ),
            ComparatorRecord::new(
                "src/c.py",
                3,
                None,
                "function",
                "charlie",
                Some("py"),
                "fake-comparator",
            ),
        ],
    );
    let gate = evaluate_quality_gate(
        &records,
        &[],
        QualityGateOptions::new("fixture", "/tmp/fixture").with_comparator_run(comparator),
    )
    .expect("evaluate gate");

    let first_report = gaps_from_gate_report(&gate);
    let second_report = gaps_from_gate_report(&gate);
    let first_plan = generate_cycle_plan(&first_report, CyclePlanOptions::default());
    let second_plan = generate_cycle_plan(&second_report, CyclePlanOptions::default());

    assert_eq!(
        render_quality_gap_report(&first_report),
        render_quality_gap_report(&second_report)
    );
    assert_eq!(
        render_quality_cycle_plan(&first_plan),
        render_quality_cycle_plan(&second_plan)
    );
}

#[test]
fn quality_loop_reports_do_not_pollute_production_database() {
    let temp = temp_repo();
    write_file(temp.path(), "src/lib.rs", "pub fn loop_symbol() {}\n");
    run_build(temp.path());
    let before = load_records(temp.path()).expect("load records before loop reports");
    let snapshot = load_index_snapshot_from_sqlite(temp.path());
    let gate = evaluate_quality_gate(
        &snapshot.records,
        &snapshot.refs,
        QualityGateOptions::new("fixture", temp.path().display().to_string()),
    )
    .expect("evaluate gate");
    let gaps = gaps_from_gate_report(&gate);
    let plan = generate_cycle_plan(&gaps, CyclePlanOptions::default());

    let gaps_path = write_quality_gap_report(temp.path(), &gaps).expect("write gap report");
    let plan_path = write_quality_cycle_plan(temp.path(), &plan).expect("write cycle plan");

    assert!(gaps_path.ends_with(".dev_index/quality/QUALITY_GAPS.md"));
    assert!(plan_path.ends_with(".dev_index/quality/QUALITY_CYCLE_01_PLAN.md"));
    assert!(gaps_path.exists());
    assert!(plan_path.exists());
    assert_eq!(
        before,
        load_records(temp.path()).expect("load records after reports")
    );
}

#[test]
#[ignore = "rebuilds .dev_index for every repo under test_repos/; run with: cargo test --test quality_loop -- --ignored"]
fn full_quality_loop_writes_gap_report_and_bounded_plan_for_test_repos() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_repos");
    let repo_set = load_quality_repo_set(&root)?;
    let BenchmarkRepoSet::Repos { repos, .. } = repo_set else {
        match repo_set {
            BenchmarkRepoSet::MissingRoot => println!("skipped: test_repos/ missing"),
            BenchmarkRepoSet::Empty => println!("skipped: test_repos/ has no repo directories"),
            BenchmarkRepoSet::Repos { .. } => unreachable!(),
        }
        return Ok(());
    };

    for repo in repos {
        let dev_index = repo.path.join(".dev_index");
        if dev_index.exists() {
            fs::remove_dir_all(&dev_index)?;
        }
        build_index(&repo.path)?;
        let snapshot = load_index_snapshot_from_sqlite(&repo.path);
        let comparator = UniversalCtagsComparator::default();
        let comparator_run = comparator.run(&repo.path)?;
        let gate = evaluate_quality_gate(
            &snapshot.records,
            &snapshot.refs,
            QualityGateOptions::from_benchmark_repo(&repo).with_comparator_run(comparator_run),
        )?;
        assert_quality_gate_passes(&gate)?;

        let gaps = gaps_from_gate_report(&gate);
        let plan = generate_cycle_plan(&gaps, CyclePlanOptions::default());
        let gaps_path = write_quality_gap_report(&repo.path, &gaps)?;
        let plan_path = write_quality_cycle_plan(&repo.path, &plan)?;
        println!("{}", render_quality_gap_report(&gaps));
        println!("{}", render_quality_cycle_plan(&plan));

        assert!(gaps_path.exists());
        assert!(plan_path.exists());
        assert!(plan.selected_gaps.len() <= thinindex::quality::DEFAULT_MAX_GAPS_PER_CYCLE);
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

fn gap(
    id: &str,
    language: &str,
    kind: &str,
    evidence_source: &str,
    severity: GapSeverity,
) -> QualityGap {
    QualityGap {
        id: id.to_string(),
        repo: "fixture".to_string(),
        path: Some(format!("src/{id}.rs")),
        language: language.to_string(),
        symbol: Some(format!("symbol_{id}")),
        kind: Some(kind.to_string()),
        pattern: None,
        evidence_source: evidence_source.to_string(),
        severity,
        suggested_fix: if evidence_source == "comparator-only" {
            SuggestedFixType::ComparatorTriage
        } else {
            SuggestedFixType::ParserQuery
        },
        status: GapStatus::Open,
        fixture_added: false,
        manifest_added: false,
        detail: format!("{evidence_source} detail {id}"),
    }
}
