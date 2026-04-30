mod common;

use anyhow::Result;
use thinindex::{
    bench::ExpectedSymbol,
    model::IndexRecord,
    quality::{
        ComparatorRecord, ComparatorRun, ComparatorStatus, CyclePlanOptions, QualityComparator,
        QualityComparisonOptions, QualityGateOptions, QualityReportExportOptions,
        UniversalCtagsComparator, assert_no_forbidden_index_sources, build_quality_report_export,
        compare_quality, evaluate_quality_gate, gaps_from_gate_report, generate_cycle_plan,
        parse_ctags_json_record, render_quality_report, render_quality_report_export_details_jsonl,
        render_quality_report_export_json, render_quality_report_export_markdown,
        write_quality_report, write_quality_report_export,
    },
    store::{load_records, load_refs},
};

use common::{load_index_snapshot_from_sqlite, run_build, temp_repo, write_file};

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
fn quality_reports_redact_secret_like_symbols_by_default() {
    let thinindex = vec![IndexRecord::new(
        "src/lib.rs",
        1,
        1,
        "rs",
        "function",
        "token=thinindex-secret",
        "fn token_secret() {}",
        "tree-sitter",
    )];
    let comparator = ComparatorRun::completed(
        "fake-comparator",
        vec![ComparatorRecord::new(
            "src/lib.rs",
            10,
            None,
            "function",
            "api_key=comparator-secret",
            Some("rs"),
            "fake-comparator",
        )],
    );
    let report = compare_quality(
        &thinindex,
        &comparator,
        QualityComparisonOptions::new("fixture", "/tmp/fixture").with_expected_symbols(vec![
            ExpectedSymbol {
                language: Some("rs".to_string()),
                path: Some("src/lib.rs".to_string()),
                kind: Some("function".to_string()),
                name: "password=missing-secret".to_string(),
            },
        ]),
    );

    let rendered = render_quality_report(&report);
    let temp = temp_repo();
    let path = write_quality_report(temp.path(), "fake-comparator", &rendered)
        .expect("write redacted quality report");
    let written = std::fs::read_to_string(path).expect("read quality report");

    for output in [&rendered, &written] {
        assert!(
            output.contains("token=[REDACTED]")
                && output.contains("api_key=[REDACTED]")
                && output.contains("password=[REDACTED]"),
            "expected redacted quality report, got:\n{output}"
        );
        for leaked in ["thinindex-secret", "comparator-secret", "missing-secret"] {
            assert!(
                !output.contains(leaked),
                "quality report leaked secret-like value {leaked}, got:\n{output}"
            );
        }
    }
}

#[test]
fn quality_report_export_is_deterministic_and_json_parses() {
    let gate_a = export_gate_report("fixture-a", "alpha_missing", "alpha_comparator");
    let gate_b = export_gate_report("fixture-b", "beta_missing", "beta_comparator");
    let gaps_a = gaps_from_gate_report(&gate_a);
    let plan_a = generate_cycle_plan(&gaps_a, CyclePlanOptions::default());

    let first = build_quality_report_export(
        &[gate_b.clone(), gate_a.clone()],
        std::slice::from_ref(&gaps_a),
        std::slice::from_ref(&plan_a),
        QualityReportExportOptions::default(),
    )
    .expect("build first export");
    let second = build_quality_report_export(
        &[gate_a, gate_b],
        &[gaps_a],
        &[plan_a],
        QualityReportExportOptions::default(),
    )
    .expect("build second export");

    let first_markdown = render_quality_report_export_markdown(&first);
    let second_markdown = render_quality_report_export_markdown(&second);
    let first_json = render_quality_report_export_json(&first).expect("render first json");
    let second_json = render_quality_report_export_json(&second).expect("render second json");
    let first_details =
        render_quality_report_export_details_jsonl(&first).expect("render first details");
    let second_details =
        render_quality_report_export_details_jsonl(&second).expect("render second details");
    let parsed: serde_json::Value = serde_json::from_str(&first_json).expect("json parses");

    assert_eq!(first_markdown, second_markdown);
    assert_eq!(first_json, second_json);
    assert_eq!(first_details, second_details);
    assert_eq!(parsed["generated_at"], "deterministic");
    assert_eq!(parsed["repos"][0]["name"], "fixture-a");
    assert!(parsed["repos"][0].get("path").is_none());
}

#[test]
fn quality_report_export_markdown_contains_required_sections() {
    let gate = export_gate_report("fixture", "missing_symbol", "comparator_symbol");
    let gaps = gaps_from_gate_report(&gate);
    let plan = generate_cycle_plan(&gaps, CyclePlanOptions::default());
    let export = build_quality_report_export(&[gate], &[gaps], &[plan], Default::default())
        .expect("build export");
    let markdown = render_quality_report_export_markdown(&export);

    for section in [
        "# Quality Report",
        "## Repos",
        "## Language Support Matrix",
        "## Expected Symbols",
        "## Comparator Symbols",
        "## Parser Errors",
        "## Unsupported Extensions",
        "## Slow Or Noisy Files",
        "## Gap Summary",
        "## Cycle Plan Summary",
        "detail file: QUALITY_REPORT_DETAILS.jsonl",
    ] {
        assert!(markdown.contains(section), "missing section {section}");
    }
}

#[test]
fn quality_report_export_keeps_large_detail_data_out_of_summary() {
    let records = vec![IndexRecord::new(
        "src/lib.rs",
        1,
        1,
        "rs",
        "function",
        "present",
        "fn present() {}",
        "tree-sitter",
    )];
    let comparator_symbols = (0..30)
        .map(|index| {
            ComparatorRecord::new(
                "src/lib.rs",
                index + 10,
                None,
                "function",
                format!("comparator_symbol_{index:02}"),
                Some("rs"),
                "fake-comparator",
            )
        })
        .collect();
    let gate = evaluate_quality_gate(
        &records,
        &[],
        QualityGateOptions::new("fixture", "/tmp/fixture").with_comparator_run(
            ComparatorRun::completed("fake-comparator", comparator_symbols),
        ),
    )
    .expect("evaluate gate");
    let export = build_quality_report_export(
        &[gate],
        &[],
        &[],
        QualityReportExportOptions::default().with_max_summary_items(3),
    )
    .expect("build export");
    let json = render_quality_report_export_json(&export).expect("render json");
    let markdown = render_quality_report_export_markdown(&export);
    let details = render_quality_report_export_details_jsonl(&export).expect("render details");

    assert!(json.contains("comparator_symbol_00"));
    assert!(!json.contains("comparator_symbol_29"));
    assert!(!markdown.contains("comparator_symbol_29"));
    assert!(details.contains("comparator_symbol_29"));
}

#[test]
fn quality_report_exports_redact_secret_like_details() {
    let records = vec![IndexRecord::new(
        "src/lib.rs",
        1,
        1,
        "rs",
        "function",
        "present",
        "fn present() {}",
        "tree-sitter",
    )];
    let gate = evaluate_quality_gate(
        &records,
        &[],
        QualityGateOptions::new("fixture", "/tmp/password=local-secret")
            .with_expected_symbols(vec![ExpectedSymbol {
                language: Some("rs".to_string()),
                path: Some("src/lib.rs".to_string()),
                kind: Some("function".to_string()),
                name: "client_secret=missing-secret".to_string(),
            }])
            .with_comparator_run(ComparatorRun::completed(
                "fake-comparator",
                vec![ComparatorRecord::new(
                    "src/lib.rs",
                    10,
                    None,
                    "function",
                    "token=comparator-secret",
                    Some("rs"),
                    "fake-comparator",
                )],
            )),
    )
    .expect("evaluate gate");
    let export = build_quality_report_export(
        &[gate],
        &[],
        &[],
        QualityReportExportOptions::default().with_local_paths(),
    )
    .expect("build export");
    let json = render_quality_report_export_json(&export).expect("render json");
    let markdown = render_quality_report_export_markdown(&export);
    let details = render_quality_report_export_details_jsonl(&export).expect("render details");

    assert!(
        json.contains("client_secret=[REDACTED]")
            && json.contains("token=[REDACTED]")
            && json.contains("password=[REDACTED]"),
        "expected redacted quality JSON summary, got:\n{json}"
    );
    assert!(
        details.contains("client_secret=[REDACTED]") && details.contains("token=[REDACTED]"),
        "expected redacted quality JSONL details, got:\n{details}"
    );

    for output in [&json, &markdown, &details] {
        for leaked in ["local-secret", "missing-secret", "comparator-secret"] {
            assert!(
                !output.contains(leaked),
                "quality export leaked secret-like value {leaked}, got:\n{output}"
            );
        }
    }
}

#[test]
fn quality_report_export_is_isolated_from_production_database() {
    let temp = temp_repo();
    write_file(temp.path(), "src/lib.rs", "pub fn indexed_symbol() {}\n");
    run_build(temp.path());
    let before_records = load_records(temp.path()).expect("load records before export");
    let before_refs = load_refs(temp.path()).expect("load refs before export");
    let gate = evaluate_quality_gate(
        &before_records,
        &before_refs,
        QualityGateOptions::new("fixture", temp.path().display().to_string()).with_comparator_run(
            ComparatorRun::completed(
                "fake-comparator",
                vec![ComparatorRecord::new(
                    "src/lib.rs",
                    1,
                    None,
                    "function",
                    "indexed_symbol",
                    Some("rs"),
                    "fake-comparator",
                )],
            ),
        ),
    )
    .expect("evaluate gate");
    let gaps = gaps_from_gate_report(&gate);
    let plan = generate_cycle_plan(&gaps, CyclePlanOptions::default());
    let export = build_quality_report_export(&[gate], &[gaps], &[plan], Default::default())
        .expect("build export");
    let paths = write_quality_report_export(temp.path(), &export).expect("write export");

    assert!(
        paths
            .markdown_path
            .ends_with(".dev_index/quality/QUALITY_REPORT.md")
    );
    assert!(
        paths
            .json_path
            .ends_with(".dev_index/quality/QUALITY_REPORT.json")
    );
    assert!(
        paths
            .details_jsonl_path
            .ends_with(".dev_index/quality/QUALITY_REPORT_DETAILS.jsonl")
    );
    assert!(paths.markdown_path.exists());
    assert!(paths.json_path.exists());
    assert!(paths.details_jsonl_path.exists());
    assert_eq!(
        before_records,
        load_records(temp.path()).expect("load records after export")
    );
    assert_eq!(
        before_refs,
        load_refs(temp.path()).expect("load refs after export")
    );
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
    assert_no_forbidden_index_sources("quality-before", &before, &[]);

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
    assert_no_forbidden_index_sources("quality-after", &snapshot.records, &snapshot.refs);
    assert_eq!(before, snapshot.records);
    assert!(
        !snapshot
            .records
            .iter()
            .any(|record| record.path.contains(".dev_index/quality"))
    );
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
    assert_no_forbidden_index_sources("optional-comparator", &load_records(temp.path())?, &[]);
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

fn export_gate_report(
    repo_name: &str,
    missing_symbol: &str,
    comparator_symbol: &str,
) -> thinindex::quality::QualityGateReport {
    let records = vec![IndexRecord::new(
        "src/lib.rs",
        1,
        1,
        "rs",
        "function",
        "present",
        "fn present() {}",
        "tree-sitter",
    )];

    evaluate_quality_gate(
        &records,
        &[],
        QualityGateOptions::new(repo_name, "/tmp/fixture")
            .with_expected_symbols(vec![ExpectedSymbol {
                language: Some("rs".to_string()),
                path: Some("src/lib.rs".to_string()),
                kind: Some("function".to_string()),
                name: missing_symbol.to_string(),
            }])
            .with_comparator_run(ComparatorRun::completed(
                "fake-comparator",
                vec![ComparatorRecord::new(
                    "src/lib.rs",
                    10,
                    None,
                    "function",
                    comparator_symbol,
                    Some("rs"),
                    "fake-comparator",
                )],
            )),
    )
    .expect("evaluate export gate")
}
