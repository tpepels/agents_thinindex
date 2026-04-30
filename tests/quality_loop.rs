mod common;

use std::{fs, path::Path};

use anyhow::Result;
use thinindex::{
    bench::{BenchmarkRepoSet, ExpectedAbsentSymbol, ExpectedSymbol},
    indexer::build_index,
    model::IndexRecord,
    quality::{
        ComparatorRecord, ComparatorRun, CyclePlanOptions, GapSeverity, GapStatus,
        QualityComparator, QualityCycleStopCondition, QualityCycleVerification, QualityGap,
        QualityGapReport, QualityGateOptions, SuggestedFixType, UniversalCtagsComparator,
        assert_quality_gate_passes, evaluate_quality_gate, finalize_quality_cycle,
        gaps_from_gate_report, generate_cycle_plan, group_gaps, load_quality_repo_set,
        render_quality_cycle_final_report, render_quality_cycle_plan, render_quality_gap_report,
        render_triage_report, run_single_quality_cycle, triage_report_from_quality_report,
        write_quality_cycle_final_report, write_quality_cycle_run, write_triage_report,
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
    let rendered = render_quality_cycle_plan(&plan);
    assert!(
        rendered.contains("- one-cycle limit: true"),
        "plan must document bounded cycle behavior"
    );
    assert!(rendered.contains("## Expected Change Set"));
    assert!(rendered.contains("parser: Tree-sitter query spec for the affected language"));
}

#[test]
fn cycle_plan_caps_requested_gap_limit_at_default_max() {
    let report = QualityGapReport {
        repo_name: "fixture".to_string(),
        repo_path: "/tmp/fixture".to_string(),
        gaps: (0..12)
            .map(|index| {
                gap(
                    &format!("GAP-{index:04}"),
                    "rs",
                    "function",
                    "expected-symbol",
                    GapSeverity::High,
                )
            })
            .collect(),
    };

    let plan = generate_cycle_plan(
        &report,
        CyclePlanOptions {
            cycle_id: "QUALITY_CYCLE_01".to_string(),
            max_gaps: 50,
        },
    );

    assert_eq!(
        plan.max_gaps,
        thinindex::quality::DEFAULT_MAX_GAPS_PER_CYCLE
    );
    assert_eq!(plan.selected_gaps.len(), 10);
    assert_eq!(plan.deferred_gap_ids.len(), 2);
}

#[test]
fn single_cycle_runner_executes_exactly_one_bounded_plan() {
    let gate = evaluate_quality_gate(
        &[record("src/lib.rs", 1, 1, "rs", "function", "present")],
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

    let run = run_single_quality_cycle(&gate, CyclePlanOptions::default());

    assert_eq!(run.cycles_executed, 1);
    assert!(!run.automatic_next_cycle_allowed);
    assert_eq!(run.plan.selected_gaps.len(), 1);
    assert!(render_quality_cycle_plan(&run.plan).contains("- one-cycle limit: true"));
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
fn final_cycle_report_enforces_stop_conditions_and_no_next_cycle() {
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
            ),
            gap(
                "GAP-0002",
                "rs",
                "macro",
                "comparator-only",
                GapSeverity::Medium,
            )
            .with_status(GapStatus::FalsePositive),
        ],
    };
    let plan = generate_cycle_plan(&report, CyclePlanOptions::default());
    let mut current_gaps = report.gaps.clone();
    current_gaps[0] = current_gaps[0].clone().with_status(GapStatus::Fixed);
    let final_report = finalize_quality_cycle(
        &plan,
        &current_gaps,
        vec![
            QualityCycleVerification::passed("cargo test"),
            QualityCycleVerification::skipped(
                "cargo test --test real_repos -- --ignored",
                "test_repos/ missing",
            ),
        ],
    );
    let rendered = render_quality_cycle_final_report(&final_report);

    assert!(!final_report.automatic_next_cycle_allowed);
    assert_eq!(
        final_report.stop_conditions,
        vec![
            QualityCycleStopCondition::SelectedGapsFixed,
            QualityCycleStopCondition::RemainingGapsComparatorFalsePositive,
        ]
    );
    assert!(rendered.contains("- automatic next cycle allowed: no"));
    assert!(rendered.contains("remaining_gaps_comparator_false_positive"));
    assert!(rendered.contains("- Stop after this report."));
}

#[test]
fn final_cycle_report_marks_verification_failure_for_human_review() {
    let report = QualityGapReport {
        repo_name: "fixture".to_string(),
        repo_path: "/tmp/fixture".to_string(),
        gaps: vec![gap(
            "GAP-0001",
            "rs",
            "function",
            "expected-symbol",
            GapSeverity::High,
        )],
    };
    let plan = generate_cycle_plan(&report, CyclePlanOptions::default());
    let final_report = finalize_quality_cycle(
        &plan,
        &report.gaps,
        vec![QualityCycleVerification::failed(
            "cargo clippy --all-targets --all-features -- -D warnings",
            "needs human review",
        )],
    );

    assert!(
        final_report
            .stop_conditions
            .contains(&QualityCycleStopCondition::VerificationFailedNeedsHumanReview)
    );
    assert!(
        render_quality_cycle_final_report(&final_report)
            .contains("verification_failed_needs_human_review")
    );
}

#[test]
fn final_cycle_report_is_deterministic() {
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
            .with_status(GapStatus::Fixed),
            gap(
                "GAP-0002",
                "rs",
                "function",
                "unsupported-extension",
                GapSeverity::Low,
            ),
        ],
    };
    let mut planning_report = report.clone();
    planning_report.gaps[0] = planning_report.gaps[0].clone().with_status(GapStatus::Open);
    let plan = generate_cycle_plan(
        &planning_report,
        CyclePlanOptions {
            cycle_id: "QUALITY_CYCLE_01".to_string(),
            max_gaps: 1,
        },
    );
    let first = finalize_quality_cycle(
        &plan,
        &report.gaps,
        vec![
            QualityCycleVerification::skipped("cargo test --test real_repos -- --ignored", "none"),
            QualityCycleVerification::passed("cargo test"),
        ],
    );
    let second = finalize_quality_cycle(
        &plan,
        &report.gaps,
        vec![
            QualityCycleVerification::skipped("cargo test --test real_repos -- --ignored", "none"),
            QualityCycleVerification::passed("cargo test"),
        ],
    );

    assert_eq!(
        render_quality_cycle_final_report(&first),
        render_quality_cycle_final_report(&second)
    );
    assert!(
        first.stop_conditions.contains(
            &QualityCycleStopCondition::RemainingGapsRequireArchitectureOrLanguageExpansion
        )
    );
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
fn comparator_triage_model_groups_and_renders_promotion_actions() {
    let gate = evaluate_quality_gate(
        &[record(
            "src/lib.rs",
            1,
            1,
            "rs",
            "function",
            "thinindex_only",
        )],
        &[],
        QualityGateOptions::new("fixture", "/tmp/fixture").with_comparator_run(
            ComparatorRun::completed(
                "fake-comparator",
                vec![
                    ComparatorRecord::new(
                        "src/a.rs",
                        10,
                        None,
                        "function",
                        "alpha",
                        Some("rs"),
                        "fake-comparator",
                    ),
                    ComparatorRecord::new(
                        "src/a.rs",
                        12,
                        None,
                        "function",
                        "beta",
                        Some("rs"),
                        "fake-comparator",
                    ),
                ],
            ),
        ),
    )
    .expect("evaluate gate");
    let quality = gate.comparator_report.as_ref().expect("quality report");
    let triage = triage_report_from_quality_report(quality);
    let rendered = render_triage_report(&triage);

    assert_eq!(triage.items.len(), 3);
    assert_eq!(triage.items[0].id, "TRIAGE-0001");
    assert_eq!(triage.items[0].state.as_str(), "open");
    assert!(rendered.contains("- accepted_expected_symbol"));
    assert!(rendered.contains("- fixture_needed"));
    assert!(rendered.contains("- comparator_false_positive"));
    assert!(rendered.contains("- unsupported_syntax"));
    assert!(rendered.contains("- low_value_noise"));
    assert!(rendered.contains("- fixed"));
    assert!(rendered.contains(
        "language=rs kind=function path=src/a.rs count=2 items=TRIAGE-0001, TRIAGE-0002"
    ));
    assert!(rendered.contains("promotion: triage before promoting"));
}

#[test]
fn comparator_triage_state_transitions_are_explicit() {
    let gate = evaluate_quality_gate(
        &[],
        &[],
        QualityGateOptions::new("fixture", "/tmp/fixture").with_comparator_run(
            ComparatorRun::completed(
                "fake-comparator",
                vec![ComparatorRecord::new(
                    "src/lib.rs",
                    10,
                    None,
                    "function",
                    "from_comparator",
                    Some("rs"),
                    "fake-comparator",
                )],
            ),
        ),
    )
    .expect("evaluate gate");
    let triage = triage_report_from_quality_report(gate.comparator_report.as_ref().unwrap());
    let accepted =
        triage.items[0].transition_to(thinindex::quality::TriageState::AcceptedExpectedSymbol);
    let fixture = triage.items[0].transition_to(thinindex::quality::TriageState::FixtureNeeded);
    let unsupported =
        triage.items[0].transition_to(thinindex::quality::TriageState::UnsupportedSyntax);

    assert_eq!(accepted.state.as_str(), "accepted_expected_symbol");
    assert_eq!(
        accepted.promotion_action(),
        "add [[repo.expected_symbol]] or [[repo.expected_symbol_pattern]]"
    );
    assert_eq!(fixture.state.as_str(), "fixture_needed");
    assert_eq!(
        fixture.promotion_action(),
        "add or extend a parser conformance fixture"
    );
    assert_eq!(unsupported.state.as_str(), "unsupported_syntax");
    assert_eq!(
        unsupported.promotion_action(),
        "document unsupported syntax or support-level gap"
    );
    assert!(thinindex::quality::TriageState::from_name("low_value_noise").is_ok());
    assert!(thinindex::quality::TriageState::from_name("false_positive").is_err());
}

#[test]
fn open_triage_does_not_fail_gate_but_can_fail_strict_mode() {
    let gate = evaluate_quality_gate(
        &[],
        &[],
        QualityGateOptions::new("fixture", "/tmp/fixture").with_comparator_run(
            ComparatorRun::completed(
                "fake-comparator",
                vec![ComparatorRecord::new(
                    "src/lib.rs",
                    10,
                    None,
                    "function",
                    "from_comparator",
                    Some("rs"),
                    "fake-comparator",
                )],
            ),
        ),
    )
    .expect("evaluate gate");
    assert_quality_gate_passes(&gate).expect("open comparator-only symbols are triage data");

    let triage = triage_report_from_quality_report(gate.comparator_report.as_ref().unwrap());
    let error = thinindex::quality::assert_triage_has_no_open_items(&triage)
        .expect_err("manual strict triage should fail open items");
    let message = format!("{error:#}");

    assert!(message.contains("strict comparator triage failed for fixture"));
    assert!(message.contains("from_comparator"));
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
    let run = run_single_quality_cycle(&gate, CyclePlanOptions::default());
    let paths = write_quality_cycle_run(temp.path(), &run).expect("write cycle reports");
    let final_report = finalize_quality_cycle(
        &run.plan,
        &run.gap_report.gaps,
        vec![QualityCycleVerification::passed("cargo test")],
    );
    let final_path =
        write_quality_cycle_final_report(temp.path(), &final_report).expect("write final report");

    assert!(
        paths
            .gap_report_path
            .ends_with(".dev_index/quality/QUALITY_GAPS.md")
    );
    assert!(
        paths
            .cycle_plan_path
            .ends_with(".dev_index/quality/QUALITY_CYCLE_01_PLAN.md")
    );
    assert!(final_path.ends_with(".dev_index/quality/QUALITY_CYCLE_01_REPORT.md"));
    assert!(paths.gap_report_path.exists());
    assert!(paths.cycle_plan_path.exists());
    assert!(final_path.exists());
    assert_eq!(
        before,
        load_records(temp.path()).expect("load records after reports")
    );
}

#[test]
fn comparator_triage_report_is_isolated_from_production_database() {
    let temp = temp_repo();
    write_file(temp.path(), "src/lib.rs", "pub fn indexed_symbol() {}\n");
    run_build(temp.path());
    let before = load_records(temp.path()).expect("load records before triage report");
    let gate = evaluate_quality_gate(
        &before,
        &[],
        QualityGateOptions::new("fixture", temp.path().display().to_string()).with_comparator_run(
            ComparatorRun::completed(
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
            ),
        ),
    )
    .expect("evaluate gate");
    let triage = triage_report_from_quality_report(gate.comparator_report.as_ref().unwrap());
    let report_path = write_triage_report(temp.path(), &triage).expect("write triage report");

    assert!(report_path.ends_with(".dev_index/quality/COMPARATOR_TRIAGE.md"));
    assert!(report_path.exists());
    assert_eq!(
        before,
        load_records(temp.path()).expect("load records after triage report")
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

        let run = run_single_quality_cycle(&gate, CyclePlanOptions::default());
        let paths = write_quality_cycle_run(&repo.path, &run)?;
        let final_report = finalize_quality_cycle(
            &run.plan,
            &run.gap_report.gaps,
            vec![QualityCycleVerification::passed(
                "cargo test --test quality_loop -- --ignored",
            )],
        );
        let final_path = write_quality_cycle_final_report(&repo.path, &final_report)?;
        let triage_path = if let Some(comparator_report) = &gate.comparator_report {
            Some(write_triage_report(
                &repo.path,
                &triage_report_from_quality_report(comparator_report),
            )?)
        } else {
            None
        };
        println!("{}", render_quality_gap_report(&run.gap_report));
        println!("{}", render_quality_cycle_plan(&run.plan));
        println!("{}", render_quality_cycle_final_report(&final_report));

        assert!(paths.gap_report_path.exists());
        assert!(paths.cycle_plan_path.exists());
        assert!(final_path.exists());
        assert!(triage_path.is_none_or(|path| path.exists()));
        assert!(run.plan.selected_gaps.len() <= thinindex::quality::DEFAULT_MAX_GAPS_PER_CYCLE);
        assert_eq!(run.cycles_executed, 1);
        assert!(!run.automatic_next_cycle_allowed);
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
