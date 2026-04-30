use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{
    quality::{
        cycle::{GapSeverity, GapStatus, QualityCyclePlan, QualityGapReport},
        gate::QualityGateReport,
        manifest::quality_report_dir,
        report::{ComparatorOnlySymbol, ThinindexOnlySymbol},
    },
    support::support_matrix,
};

pub const QUALITY_EXPORT_MARKDOWN_FILE: &str = "QUALITY_REPORT.md";
pub const QUALITY_EXPORT_JSON_FILE: &str = "QUALITY_REPORT.json";
pub const QUALITY_EXPORT_DETAILS_JSONL_FILE: &str = "QUALITY_REPORT_DETAILS.jsonl";
pub const DEFAULT_SUMMARY_DETAIL_LIMIT: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityReportExportOptions {
    pub deterministic: bool,
    pub generated_at: Option<String>,
    pub include_local_paths: bool,
    pub max_summary_items: usize,
}

impl Default for QualityReportExportOptions {
    fn default() -> Self {
        Self {
            deterministic: true,
            generated_at: None,
            include_local_paths: false,
            max_summary_items: DEFAULT_SUMMARY_DETAIL_LIMIT,
        }
    }
}

impl QualityReportExportOptions {
    pub fn with_generated_at(mut self, generated_at: impl Into<String>) -> Self {
        self.generated_at = Some(generated_at.into());
        self.deterministic = false;
        self
    }

    pub fn with_local_paths(mut self) -> Self {
        self.include_local_paths = true;
        self
    }

    pub fn with_max_summary_items(mut self, max_summary_items: usize) -> Self {
        self.max_summary_items = max_summary_items;
        self
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct QualityReportExport {
    pub summary: QualityReportExportSummary,
    #[serde(skip)]
    pub details: Vec<QualityReportDetailRecord>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct QualityReportExportSummary {
    pub generated_at: String,
    pub deterministic: bool,
    pub repos: Vec<RepoQualitySummary>,
    pub language_support: Vec<LanguageSupportSummary>,
    pub gap_summaries: Vec<GapSummary>,
    pub cycle_plans: Vec<CyclePlanSummary>,
    pub detail_file: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RepoQualitySummary {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub languages_checked: Vec<String>,
    pub records_by_language: Vec<NamedCount>,
    pub refs_by_language: Vec<NamedCount>,
    pub expected: ExpectedQualitySummary,
    pub comparator: ComparatorQualitySummary,
    pub parser_errors: Vec<String>,
    pub unsupported_extensions: Vec<NamedCount>,
    pub slow_noisy_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExpectedQualitySummary {
    pub symbols_checked: usize,
    pub symbols_missing: usize,
    pub symbol_missing_sample: Vec<String>,
    pub patterns_checked: usize,
    pub patterns_failing: usize,
    pub pattern_failing_sample: Vec<String>,
    pub absent_symbols_checked: usize,
    pub absent_symbols_found: usize,
    pub absent_symbol_found_sample: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ComparatorQualitySummary {
    pub status: String,
    pub name: Option<String>,
    pub thinindex_only: usize,
    pub thinindex_only_sample: Vec<SymbolSummary>,
    pub comparator_only: usize,
    pub comparator_only_sample: Vec<SymbolSummary>,
    pub unknown_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolSummary {
    pub path: String,
    pub line: usize,
    pub language: String,
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LanguageSupportSummary {
    pub name: String,
    pub language_id: Option<String>,
    pub extensions: Vec<String>,
    pub support_level: String,
    pub backend: String,
    pub record_kinds: Vec<String>,
    pub known_gaps: String,
    pub license_status: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GapSummary {
    pub repo: String,
    pub total: usize,
    pub by_status: Vec<NamedCount>,
    pub by_severity: Vec<NamedCount>,
    pub by_evidence: Vec<NamedCount>,
    pub open_sample: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CyclePlanSummary {
    pub cycle_id: String,
    pub max_gaps: usize,
    pub selected_gaps: usize,
    pub selected_gap_ids: Vec<String>,
    pub deferred_gaps: usize,
    pub deferred_gap_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct NamedCount {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct QualityReportDetailRecord {
    pub repo: String,
    pub detail_kind: String,
    pub path: Option<String>,
    pub line: Option<usize>,
    pub language: Option<String>,
    pub symbol_kind: Option<String>,
    pub name: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityReportExportPaths {
    pub markdown_path: PathBuf,
    pub json_path: PathBuf,
    pub details_jsonl_path: PathBuf,
}

pub fn build_quality_report_export(
    gate_reports: &[QualityGateReport],
    gap_reports: &[QualityGapReport],
    cycle_plans: &[QualityCyclePlan],
    options: QualityReportExportOptions,
) -> Result<QualityReportExport> {
    let max_summary_items = options.max_summary_items;
    let mut gate_reports = gate_reports.to_vec();
    gate_reports.sort_by_key(|report| report.repo_name.clone());

    let mut gap_reports = gap_reports.to_vec();
    gap_reports.sort_by_key(|report| report.repo_name.clone());

    let mut cycle_plans = cycle_plans.to_vec();
    cycle_plans.sort_by_key(|plan| plan.cycle_id.clone());

    let repos = gate_reports
        .iter()
        .map(|report| repo_summary(report, &options))
        .collect::<Vec<_>>();
    let language_support = language_support_summary();
    let gap_summaries = gap_reports
        .iter()
        .map(|report| gap_summary(report, max_summary_items))
        .collect::<Vec<_>>();
    let cycle_plans = cycle_plans
        .iter()
        .map(cycle_plan_summary)
        .collect::<Vec<_>>();
    let mut details = Vec::new();

    for report in &gate_reports {
        append_gate_details(&mut details, report);
    }
    for report in &gap_reports {
        append_gap_details(&mut details, report);
    }
    details.sort_by_key(detail_sort_key);

    Ok(QualityReportExport {
        summary: QualityReportExportSummary {
            generated_at: if options.deterministic {
                "deterministic".to_string()
            } else {
                options
                    .generated_at
                    .unwrap_or_else(|| "not-recorded".to_string())
            },
            deterministic: options.deterministic,
            repos,
            language_support,
            gap_summaries,
            cycle_plans,
            detail_file: QUALITY_EXPORT_DETAILS_JSONL_FILE.to_string(),
        },
        details,
    })
}

pub fn render_quality_report_export_markdown(export: &QualityReportExport) -> String {
    let summary = &export.summary;
    let mut out = String::new();
    out.push_str("# Quality Report\n\n");
    out.push_str(&format!("- generated_at: {}\n", summary.generated_at));
    out.push_str(&format!(
        "- deterministic: {}\n",
        yes_no(summary.deterministic)
    ));
    out.push_str(&format!("- repos: {}\n", summary.repos.len()));
    out.push_str(&format!("- detail file: {}\n\n", summary.detail_file));

    out.push_str("## Repos\n\n");
    if summary.repos.is_empty() {
        out.push_str("- none\n\n");
    } else {
        for repo in &summary.repos {
            out.push_str(&format!(
                "- {}: languages={} records={} refs={} expected_missing={} comparator_only={} thinindex_only={} unsupported_extensions={}\n",
                repo.name,
                render_strings(&repo.languages_checked),
                render_counts(&repo.records_by_language),
                render_counts(&repo.refs_by_language),
                repo.expected.symbols_missing,
                repo.comparator.comparator_only,
                repo.comparator.thinindex_only,
                render_counts(&repo.unsupported_extensions),
            ));
        }
        out.push('\n');
    }

    out.push_str("## Language Support Matrix\n\n");
    out.push_str("| Language | Level | Backend | Extensions | Kinds |\n");
    out.push_str("| --- | --- | --- | --- | --- |\n");
    for entry in &summary.language_support {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            entry.name,
            entry.support_level,
            entry.backend,
            render_strings(&entry.extensions),
            render_strings(&entry.record_kinds),
        ));
    }
    out.push('\n');

    out.push_str("## Expected Symbols\n\n");
    for repo in &summary.repos {
        out.push_str(&format!(
            "- {}: symbols {}/{} missing, patterns {}/{} failing, absent {}/{} found\n",
            repo.name,
            repo.expected.symbols_missing,
            repo.expected.symbols_checked,
            repo.expected.patterns_failing,
            repo.expected.patterns_checked,
            repo.expected.absent_symbols_found,
            repo.expected.absent_symbols_checked,
        ));
    }
    out.push('\n');

    out.push_str("## Comparator Symbols\n\n");
    for repo in &summary.repos {
        out.push_str(&format!(
            "- {}: status={} comparator={} comparator-only={} thinindex-only={}\n",
            repo.name,
            repo.comparator.status,
            repo.comparator.name.as_deref().unwrap_or("none"),
            repo.comparator.comparator_only,
            repo.comparator.thinindex_only,
        ));
    }
    out.push('\n');

    out.push_str("## Parser Errors\n\n");
    let mut any_parser_errors = false;
    for repo in &summary.repos {
        if !repo.parser_errors.is_empty() {
            any_parser_errors = true;
            out.push_str(&format!(
                "- {}: {}\n",
                repo.name,
                repo.parser_errors.join("; ")
            ));
        }
    }
    if !any_parser_errors {
        out.push_str("- none\n");
    }
    out.push('\n');

    out.push_str("## Unsupported Extensions\n\n");
    for repo in &summary.repos {
        out.push_str(&format!(
            "- {}: {}\n",
            repo.name,
            render_counts(&repo.unsupported_extensions)
        ));
    }
    out.push('\n');

    out.push_str("## Slow Or Noisy Files\n\n");
    out.push_str("- not collected by this export\n\n");

    out.push_str("## Gap Summary\n\n");
    if summary.gap_summaries.is_empty() {
        out.push_str("- none\n\n");
    } else {
        for gaps in &summary.gap_summaries {
            out.push_str(&format!(
                "- {}: total={} status={} severity={} evidence={} open_sample={}\n",
                gaps.repo,
                gaps.total,
                render_counts(&gaps.by_status),
                render_counts(&gaps.by_severity),
                render_counts(&gaps.by_evidence),
                render_strings(&gaps.open_sample),
            ));
        }
        out.push('\n');
    }

    out.push_str("## Cycle Plan Summary\n\n");
    if summary.cycle_plans.is_empty() {
        out.push_str("- none\n");
    } else {
        for plan in &summary.cycle_plans {
            out.push_str(&format!(
                "- {}: max_gaps={} selected={} deferred={} selected_ids={} deferred_ids={}\n",
                plan.cycle_id,
                plan.max_gaps,
                plan.selected_gaps,
                plan.deferred_gaps,
                render_strings(&plan.selected_gap_ids),
                render_strings(&plan.deferred_gap_ids),
            ));
        }
    }

    out
}

pub fn render_quality_report_export_json(export: &QualityReportExport) -> Result<String> {
    serde_json::to_string_pretty(&export.summary).context("failed to render quality JSON summary")
}

pub fn render_quality_report_export_details_jsonl(export: &QualityReportExport) -> Result<String> {
    let mut out = String::new();
    for detail in &export.details {
        out.push_str(
            &serde_json::to_string(detail)
                .context("failed to render quality JSONL detail record")?,
        );
        out.push('\n');
    }
    Ok(out)
}

pub fn write_quality_report_export(
    repo_root: &Path,
    export: &QualityReportExport,
) -> Result<QualityReportExportPaths> {
    let report_dir = quality_report_dir(repo_root);
    fs::create_dir_all(&report_dir).with_context(|| {
        format!(
            "failed to create quality report dir {}",
            report_dir.display()
        )
    })?;

    let markdown_path = report_dir.join(QUALITY_EXPORT_MARKDOWN_FILE);
    let json_path = report_dir.join(QUALITY_EXPORT_JSON_FILE);
    let details_jsonl_path = report_dir.join(QUALITY_EXPORT_DETAILS_JSONL_FILE);

    fs::write(
        &markdown_path,
        render_quality_report_export_markdown(export),
    )
    .with_context(|| format!("failed to write {}", markdown_path.display()))?;
    fs::write(&json_path, render_quality_report_export_json(export)?)
        .with_context(|| format!("failed to write {}", json_path.display()))?;
    fs::write(
        &details_jsonl_path,
        render_quality_report_export_details_jsonl(export)?,
    )
    .with_context(|| format!("failed to write {}", details_jsonl_path.display()))?;

    Ok(QualityReportExportPaths {
        markdown_path,
        json_path,
        details_jsonl_path,
    })
}

fn repo_summary(
    report: &QualityGateReport,
    options: &QualityReportExportOptions,
) -> RepoQualitySummary {
    RepoQualitySummary {
        name: report.repo_name.clone(),
        path: options
            .include_local_paths
            .then(|| report.repo_path.clone()),
        languages_checked: sorted_strings(report.languages_checked.clone()),
        records_by_language: named_counts_from_pairs(&report.records_by_language),
        refs_by_language: named_counts_from_pairs(&report.refs_by_language),
        expected: expected_summary(report, options.max_summary_items),
        comparator: comparator_summary(report, options.max_summary_items),
        parser_errors: parser_errors(report),
        unsupported_extensions: named_counts_from_pairs(&report.unsupported_extensions),
        slow_noisy_files: Vec::new(),
    }
}

fn expected_summary(
    report: &QualityGateReport,
    max_summary_items: usize,
) -> ExpectedQualitySummary {
    ExpectedQualitySummary {
        symbols_checked: report.expected_symbols_checked,
        symbols_missing: report.expected_symbols_missing.len(),
        symbol_missing_sample: sample_strings(&report.expected_symbols_missing, max_summary_items),
        patterns_checked: report.expected_patterns_checked,
        patterns_failing: report.expected_patterns_failing.len(),
        pattern_failing_sample: sample_strings(
            &report.expected_patterns_failing,
            max_summary_items,
        ),
        absent_symbols_checked: report.expected_absent_symbols_checked,
        absent_symbols_found: report.expected_absent_symbols_found.len(),
        absent_symbol_found_sample: sample_strings(
            &report.expected_absent_symbols_found,
            max_summary_items,
        ),
    }
}

fn comparator_summary(
    report: &QualityGateReport,
    max_summary_items: usize,
) -> ComparatorQualitySummary {
    let Some(comparator) = &report.comparator_report else {
        return ComparatorQualitySummary {
            status: "not_run".to_string(),
            name: None,
            thinindex_only: 0,
            thinindex_only_sample: Vec::new(),
            comparator_only: 0,
            comparator_only_sample: Vec::new(),
            unknown_kinds: Vec::new(),
        };
    };

    ComparatorQualitySummary {
        status: if comparator.skipped {
            "skipped".to_string()
        } else {
            "completed".to_string()
        },
        name: Some(comparator.comparator_name.clone()),
        thinindex_only: comparator.thinindex_only.len(),
        thinindex_only_sample: sample_thinindex_symbols(
            &comparator.thinindex_only,
            max_summary_items,
        ),
        comparator_only: comparator.comparator_only.len(),
        comparator_only_sample: sample_comparator_symbols(
            &comparator.comparator_only,
            max_summary_items,
        ),
        unknown_kinds: sample_strings(&comparator.unknown_comparator_kinds, max_summary_items),
    }
}

fn language_support_summary() -> Vec<LanguageSupportSummary> {
    let mut entries = support_matrix()
        .iter()
        .map(|entry| LanguageSupportSummary {
            name: entry.name.to_string(),
            language_id: entry.language_id.map(str::to_string),
            extensions: sorted_strings(entry.extensions.iter().map(|value| value.to_string())),
            support_level: entry.support_level.as_str().to_string(),
            backend: entry.backend.as_str().to_string(),
            record_kinds: sorted_strings(entry.record_kinds.iter().map(|value| value.to_string())),
            known_gaps: entry.known_gaps.to_string(),
            license_status: entry.license_status.to_string(),
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| (entry.support_level.clone(), entry.name.clone()));
    entries
}

fn gap_summary(report: &QualityGapReport, max_summary_items: usize) -> GapSummary {
    let mut by_status = BTreeMap::<String, usize>::new();
    let mut by_severity = BTreeMap::<String, usize>::new();
    let mut by_evidence = BTreeMap::<String, usize>::new();
    let mut open = Vec::new();

    for gap in &report.gaps {
        *by_status
            .entry(gap_status(gap.status).to_string())
            .or_default() += 1;
        *by_severity
            .entry(gap_severity(gap.severity).to_string())
            .or_default() += 1;
        *by_evidence.entry(gap.evidence_source.clone()).or_default() += 1;
        if gap.status == GapStatus::Open {
            open.push(gap.id.clone());
        }
    }
    open.sort();

    GapSummary {
        repo: report.repo_name.clone(),
        total: report.gaps.len(),
        by_status: named_counts_from_map(by_status),
        by_severity: named_counts_from_map(by_severity),
        by_evidence: named_counts_from_map(by_evidence),
        open_sample: open.into_iter().take(max_summary_items).collect(),
    }
}

fn cycle_plan_summary(plan: &QualityCyclePlan) -> CyclePlanSummary {
    let selected_gap_ids = plan
        .selected_gaps
        .iter()
        .map(|gap| gap.id.clone())
        .collect::<Vec<_>>();
    let deferred_gap_ids = plan.deferred_gap_ids.clone();

    CyclePlanSummary {
        cycle_id: plan.cycle_id.clone(),
        max_gaps: plan.max_gaps,
        selected_gaps: selected_gap_ids.len(),
        selected_gap_ids,
        deferred_gaps: deferred_gap_ids.len(),
        deferred_gap_ids,
    }
}

fn append_gate_details(details: &mut Vec<QualityReportDetailRecord>, report: &QualityGateReport) {
    for symbol in &report.expected_symbols_missing {
        details.push(QualityReportDetailRecord {
            repo: report.repo_name.clone(),
            detail_kind: "expected_symbol_missing".to_string(),
            path: None,
            line: None,
            language: None,
            symbol_kind: None,
            name: Some(symbol.clone()),
            detail: None,
        });
    }

    for pattern in &report.expected_patterns_failing {
        details.push(QualityReportDetailRecord {
            repo: report.repo_name.clone(),
            detail_kind: "expected_pattern_failing".to_string(),
            path: None,
            line: None,
            language: None,
            symbol_kind: None,
            name: None,
            detail: Some(pattern.clone()),
        });
    }

    for symbol in &report.expected_absent_symbols_found {
        details.push(QualityReportDetailRecord {
            repo: report.repo_name.clone(),
            detail_kind: "expected_absent_symbol_found".to_string(),
            path: None,
            line: None,
            language: None,
            symbol_kind: None,
            name: Some(symbol.clone()),
            detail: None,
        });
    }

    if let Some(comparator) = &report.comparator_report {
        for symbol in &comparator.comparator_only {
            details.push(detail_from_comparator_symbol(
                &report.repo_name,
                "comparator_only",
                symbol,
            ));
        }
        for symbol in &comparator.thinindex_only {
            details.push(detail_from_thinindex_symbol(
                &report.repo_name,
                "thinindex_only",
                symbol,
            ));
        }
    }
}

fn append_gap_details(details: &mut Vec<QualityReportDetailRecord>, report: &QualityGapReport) {
    for gap in &report.gaps {
        details.push(QualityReportDetailRecord {
            repo: report.repo_name.clone(),
            detail_kind: "gap".to_string(),
            path: gap.path.clone(),
            line: None,
            language: Some(gap.language.clone()),
            symbol_kind: gap.kind.clone(),
            name: gap.symbol.clone(),
            detail: Some(format!(
                "{} status={} severity={}",
                gap.id,
                gap_status(gap.status),
                gap_severity(gap.severity)
            )),
        });
    }
}

fn detail_from_comparator_symbol(
    repo: &str,
    detail_kind: &str,
    symbol: &ComparatorOnlySymbol,
) -> QualityReportDetailRecord {
    QualityReportDetailRecord {
        repo: repo.to_string(),
        detail_kind: detail_kind.to_string(),
        path: Some(symbol.path.clone()),
        line: Some(symbol.line),
        language: Some(symbol.language.clone()),
        symbol_kind: Some(symbol.kind.clone()),
        name: Some(symbol.name.clone()),
        detail: None,
    }
}

fn detail_from_thinindex_symbol(
    repo: &str,
    detail_kind: &str,
    symbol: &ThinindexOnlySymbol,
) -> QualityReportDetailRecord {
    QualityReportDetailRecord {
        repo: repo.to_string(),
        detail_kind: detail_kind.to_string(),
        path: Some(symbol.path.clone()),
        line: Some(symbol.line),
        language: Some(symbol.language.clone()),
        symbol_kind: Some(symbol.kind.clone()),
        name: Some(symbol.name.clone()),
        detail: None,
    }
}

fn parser_errors(report: &QualityGateReport) -> Vec<String> {
    let mut errors = Vec::new();
    if report.malformed_record_count > 0 {
        errors.push(format!(
            "malformed records: {}",
            report.malformed_record_count
        ));
    }
    if report.malformed_ref_count > 0 {
        errors.push(format!("malformed refs: {}", report.malformed_ref_count));
    }
    errors
}

fn sample_strings(values: &[String], max_summary_items: usize) -> Vec<String> {
    let mut values = values.to_vec();
    values.sort();
    values.into_iter().take(max_summary_items).collect()
}

fn sample_comparator_symbols(
    symbols: &[ComparatorOnlySymbol],
    max_summary_items: usize,
) -> Vec<SymbolSummary> {
    let mut symbols = symbols.iter().map(SymbolSummary::from).collect::<Vec<_>>();
    symbols.sort();
    symbols.into_iter().take(max_summary_items).collect()
}

fn sample_thinindex_symbols(
    symbols: &[ThinindexOnlySymbol],
    max_summary_items: usize,
) -> Vec<SymbolSummary> {
    let mut symbols = symbols.iter().map(SymbolSummary::from).collect::<Vec<_>>();
    symbols.sort();
    symbols.into_iter().take(max_summary_items).collect()
}

impl From<&ComparatorOnlySymbol> for SymbolSummary {
    fn from(symbol: &ComparatorOnlySymbol) -> Self {
        Self {
            path: symbol.path.clone(),
            line: symbol.line,
            language: symbol.language.clone(),
            kind: symbol.kind.clone(),
            name: symbol.name.clone(),
        }
    }
}

impl From<&ThinindexOnlySymbol> for SymbolSummary {
    fn from(symbol: &ThinindexOnlySymbol) -> Self {
        Self {
            path: symbol.path.clone(),
            line: symbol.line,
            language: symbol.language.clone(),
            kind: symbol.kind.clone(),
            name: symbol.name.clone(),
        }
    }
}

fn named_counts_from_pairs(pairs: &[(String, usize)]) -> Vec<NamedCount> {
    let map = pairs.iter().cloned().collect::<BTreeMap<_, _>>();
    named_counts_from_map(map)
}

fn named_counts_from_map(map: BTreeMap<String, usize>) -> Vec<NamedCount> {
    map.into_iter()
        .map(|(name, count)| NamedCount { name, count })
        .collect()
}

fn sorted_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let set = values.into_iter().collect::<BTreeSet<_>>();
    set.into_iter().collect()
}

fn render_counts(counts: &[NamedCount]) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }

    counts
        .iter()
        .map(|count| format!("{}={}", count.name, count.count))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_strings(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn detail_sort_key(
    detail: &QualityReportDetailRecord,
) -> (String, String, String, usize, String, String, String) {
    (
        detail.repo.clone(),
        detail.detail_kind.clone(),
        detail.path.clone().unwrap_or_default(),
        detail.line.unwrap_or_default(),
        detail.language.clone().unwrap_or_default(),
        detail.symbol_kind.clone().unwrap_or_default(),
        detail
            .name
            .clone()
            .or_else(|| detail.detail.clone())
            .unwrap_or_default(),
    )
}

fn gap_status(status: GapStatus) -> &'static str {
    match status {
        GapStatus::Open => "open",
        GapStatus::Fixed => "fixed",
        GapStatus::Unsupported => "unsupported",
        GapStatus::FalsePositive => "false-positive",
    }
}

fn gap_severity(severity: GapSeverity) -> &'static str {
    match severity {
        GapSeverity::Critical => "critical",
        GapSeverity::High => "high",
        GapSeverity::Medium => "medium",
        GapSeverity::Low => "low",
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
