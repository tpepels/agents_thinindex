use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::quality::{
    gate::QualityGateReport,
    manifest::quality_report_dir,
    report::{ComparatorOnlySymbol, ThinindexOnlySymbol},
};

pub const QUALITY_CYCLE_ID: &str = "QUALITY_CYCLE_01";
pub const DEFAULT_MAX_GAPS_PER_CYCLE: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GapSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl GapSeverity {
    fn rank(self) -> u8 {
        match self {
            Self::Critical => 0,
            Self::High => 1,
            Self::Medium => 2,
            Self::Low => 3,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GapStatus {
    Open,
    Fixed,
    Unsupported,
    FalsePositive,
}

impl GapStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Fixed => "fixed",
            Self::Unsupported => "unsupported",
            Self::FalsePositive => "false-positive",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QualityCycleStopCondition {
    NoSelectedGaps,
    SelectedGapsFixed,
    RemainingGapsUnsupported,
    RemainingGapsComparatorFalsePositive,
    RemainingGapsRequireArchitectureOrLanguageExpansion,
    VerificationFailedNeedsHumanReview,
}

impl QualityCycleStopCondition {
    fn as_str(self) -> &'static str {
        match self {
            Self::NoSelectedGaps => "no_selected_gaps",
            Self::SelectedGapsFixed => "selected_gaps_fixed",
            Self::RemainingGapsUnsupported => "remaining_gaps_unsupported",
            Self::RemainingGapsComparatorFalsePositive => {
                "remaining_gaps_comparator_false_positive"
            }
            Self::RemainingGapsRequireArchitectureOrLanguageExpansion => {
                "remaining_gaps_require_architecture_or_language_expansion"
            }
            Self::VerificationFailedNeedsHumanReview => "verification_failed_needs_human_review",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QualityCycleVerificationStatus {
    Passed,
    Failed,
    Skipped,
}

impl QualityCycleVerificationStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityCycleVerification {
    pub command: String,
    pub status: QualityCycleVerificationStatus,
    pub detail: String,
}

impl QualityCycleVerification {
    pub fn passed(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            status: QualityCycleVerificationStatus::Passed,
            detail: String::new(),
        }
    }

    pub fn failed(command: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            status: QualityCycleVerificationStatus::Failed,
            detail: detail.into(),
        }
    }

    pub fn skipped(command: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            status: QualityCycleVerificationStatus::Skipped,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuggestedFixType {
    ParserQuery,
    Fixture,
    ManifestExpectedSymbol,
    ComparatorTriage,
    IntegrityFix,
    PerformanceTriage,
    Documentation,
}

impl SuggestedFixType {
    fn rank(self) -> u8 {
        match self {
            Self::ParserQuery => 0,
            Self::Fixture => 1,
            Self::ManifestExpectedSymbol => 2,
            Self::IntegrityFix => 3,
            Self::ComparatorTriage => 4,
            Self::PerformanceTriage => 5,
            Self::Documentation => 6,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::ParserQuery => "parser-query",
            Self::Fixture => "fixture",
            Self::ManifestExpectedSymbol => "manifest-expected-symbol",
            Self::ComparatorTriage => "comparator-triage",
            Self::IntegrityFix => "integrity-fix",
            Self::PerformanceTriage => "performance-triage",
            Self::Documentation => "documentation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityGap {
    pub id: String,
    pub repo: String,
    pub path: Option<String>,
    pub language: String,
    pub symbol: Option<String>,
    pub kind: Option<String>,
    pub pattern: Option<String>,
    pub evidence_source: String,
    pub severity: GapSeverity,
    pub suggested_fix: SuggestedFixType,
    pub status: GapStatus,
    pub fixture_added: bool,
    pub manifest_added: bool,
    pub detail: String,
}

impl QualityGap {
    pub fn with_status(mut self, status: GapStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_fixture_added(mut self, fixture_added: bool) -> Self {
        self.fixture_added = fixture_added;
        self
    }

    pub fn with_manifest_added(mut self, manifest_added: bool) -> Self {
        self.manifest_added = manifest_added;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityGapReport {
    pub repo_name: String,
    pub repo_path: String,
    pub gaps: Vec<QualityGap>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityGapGroup {
    pub language: String,
    pub syntax_construct: String,
    pub severity: GapSeverity,
    pub evidence_source: String,
    pub gap_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CyclePlanOptions {
    pub cycle_id: String,
    pub max_gaps: usize,
}

impl Default for CyclePlanOptions {
    fn default() -> Self {
        Self {
            cycle_id: QUALITY_CYCLE_ID.to_string(),
            max_gaps: DEFAULT_MAX_GAPS_PER_CYCLE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityCyclePlan {
    pub cycle_id: String,
    pub max_gaps: usize,
    pub selected_gaps: Vec<QualityGap>,
    pub deferred_gap_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityCycleRun {
    pub gap_report: QualityGapReport,
    pub plan: QualityCyclePlan,
    pub cycles_executed: usize,
    pub automatic_next_cycle_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityCycleRunPaths {
    pub gap_report_path: PathBuf,
    pub cycle_plan_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityCycleFinalReport {
    pub cycle_id: String,
    pub selected_gap_ids: Vec<String>,
    pub fixed_gap_ids: Vec<String>,
    pub remaining_gap_ids: Vec<String>,
    pub stop_conditions: Vec<QualityCycleStopCondition>,
    pub verification: Vec<QualityCycleVerification>,
    pub automatic_next_cycle_allowed: bool,
}

pub fn gaps_from_gate_report(report: &QualityGateReport) -> QualityGapReport {
    let mut builder = GapBuilder::new(&report.repo_name, &report.repo_path);

    for missing in &report.expected_symbols_missing {
        builder.push(GapInput {
            path: path_from_detail(missing),
            language: language_from_detail(missing),
            symbol: Some(missing.clone()),
            kind: kind_from_detail(missing),
            pattern: None,
            evidence_source: "expected-symbol".to_string(),
            severity: GapSeverity::High,
            suggested_fix: SuggestedFixType::ParserQuery,
            detail: format!("Expected symbol missing: {missing}"),
        });
    }

    for pattern in &report.expected_patterns_failing {
        builder.push(GapInput {
            path: path_from_detail(pattern),
            language: language_from_detail(pattern),
            symbol: None,
            kind: kind_from_detail(pattern),
            pattern: Some(pattern.clone()),
            evidence_source: "expected-symbol-pattern".to_string(),
            severity: GapSeverity::High,
            suggested_fix: SuggestedFixType::ParserQuery,
            detail: format!("Expected symbol pattern failing: {pattern}"),
        });
    }

    for found in &report.expected_absent_symbols_found {
        builder.push(GapInput {
            path: path_from_detail(found),
            language: language_from_detail(found),
            symbol: Some(found.clone()),
            kind: kind_from_detail(found),
            pattern: None,
            evidence_source: "expected-absent-symbol".to_string(),
            severity: GapSeverity::High,
            suggested_fix: SuggestedFixType::ParserQuery,
            detail: format!("Expected-absent symbol found: {found}"),
        });
    }

    for failure in &report.threshold_failures {
        builder.push(GapInput {
            path: None,
            language: failure.language.clone(),
            symbol: None,
            kind: None,
            pattern: None,
            evidence_source: "quality-threshold".to_string(),
            severity: GapSeverity::Medium,
            suggested_fix: SuggestedFixType::Fixture,
            detail: failure.message.clone(),
        });
    }

    if report.duplicate_record_count > 0 {
        builder.push(integrity_gap(
            "duplicate-records",
            report.duplicate_record_count,
            GapSeverity::Critical,
        ));
    }

    if report.duplicate_ref_count > 0 {
        builder.push(integrity_gap(
            "duplicate-refs",
            report.duplicate_ref_count,
            GapSeverity::Critical,
        ));
    }

    if report.malformed_record_count > 0 {
        builder.push(integrity_gap(
            "malformed-records",
            report.malformed_record_count,
            GapSeverity::Critical,
        ));
    }

    if report.malformed_ref_count > 0 {
        builder.push(integrity_gap(
            "malformed-refs",
            report.malformed_ref_count,
            GapSeverity::Critical,
        ));
    }

    if report.dev_index_path_count > 0 {
        builder.push(integrity_gap(
            "dev-index-paths",
            report.dev_index_path_count,
            GapSeverity::Critical,
        ));
    }

    if report.ctags_source_count > 0 {
        builder.push(integrity_gap(
            "ctags-sources",
            report.ctags_source_count,
            GapSeverity::Critical,
        ));
    }

    for (extension, count) in &report.unsupported_extensions {
        builder.push(GapInput {
            path: None,
            language: "unknown".to_string(),
            symbol: None,
            kind: Some(extension.clone()),
            pattern: None,
            evidence_source: "unsupported-extension".to_string(),
            severity: GapSeverity::Low,
            suggested_fix: SuggestedFixType::Documentation,
            detail: format!("Unsupported extension {extension} seen {count} time(s)"),
        });
    }

    if let Some(comparator) = &report.comparator_report
        && !comparator.skipped
    {
        for symbol in &comparator.comparator_only {
            builder.push(comparator_gap(&report.repo_name, symbol));
        }

        for symbol in &comparator.thinindex_only {
            builder.push(thinindex_only_gap(&report.repo_name, symbol));
        }
    }

    let mut gaps = builder.finish();
    gaps.sort_by_key(gap_sort_key);

    QualityGapReport {
        repo_name: report.repo_name.clone(),
        repo_path: report.repo_path.clone(),
        gaps,
    }
}

pub fn run_single_quality_cycle(
    report: &QualityGateReport,
    options: CyclePlanOptions,
) -> QualityCycleRun {
    let gap_report = gaps_from_gate_report(report);
    let plan = generate_cycle_plan(&gap_report, options);

    QualityCycleRun {
        gap_report,
        plan,
        cycles_executed: 1,
        automatic_next_cycle_allowed: false,
    }
}

pub fn group_gaps(gaps: &[QualityGap]) -> Vec<QualityGapGroup> {
    let mut grouped: BTreeMap<(String, String, GapSeverity, String), Vec<String>> = BTreeMap::new();

    for gap in gaps {
        let construct = gap
            .kind
            .clone()
            .or_else(|| gap.pattern.clone())
            .or_else(|| gap.symbol.clone())
            .unwrap_or_else(|| "general".to_string());
        grouped
            .entry((
                gap.language.clone(),
                construct,
                gap.severity,
                gap.evidence_source.clone(),
            ))
            .or_default()
            .push(gap.id.clone());
    }

    grouped
        .into_iter()
        .map(
            |((language, syntax_construct, severity, evidence_source), mut gap_ids)| {
                gap_ids.sort();
                QualityGapGroup {
                    language,
                    syntax_construct,
                    severity,
                    evidence_source,
                    gap_ids,
                }
            },
        )
        .collect()
}

pub fn generate_cycle_plan(
    report: &QualityGapReport,
    options: CyclePlanOptions,
) -> QualityCyclePlan {
    let max_gaps = options.max_gaps.min(DEFAULT_MAX_GAPS_PER_CYCLE);
    let mut selectable_gaps = report
        .gaps
        .iter()
        .filter(|gap| gap.status == GapStatus::Open && is_actionable_cycle_gap(gap))
        .cloned()
        .collect::<Vec<_>>();
    selectable_gaps.sort_by_key(plan_sort_key);

    let selected_gaps = selectable_gaps
        .iter()
        .take(max_gaps)
        .cloned()
        .collect::<Vec<_>>();
    let selected_ids = selected_gaps
        .iter()
        .map(|gap| gap.id.clone())
        .collect::<BTreeSet<_>>();
    let deferred_gap_ids = open_gaps(&report.gaps)
        .filter(|gap| !selected_ids.contains(&gap.id))
        .map(|gap| gap.id.clone())
        .collect::<Vec<_>>();

    QualityCyclePlan {
        cycle_id: options.cycle_id,
        max_gaps,
        selected_gaps,
        deferred_gap_ids,
    }
}

pub fn finalize_quality_cycle(
    plan: &QualityCyclePlan,
    current_gaps: &[QualityGap],
    verification: Vec<QualityCycleVerification>,
) -> QualityCycleFinalReport {
    let selected_ids = plan
        .selected_gaps
        .iter()
        .map(|gap| gap.id.clone())
        .collect::<BTreeSet<_>>();
    let current_by_id = current_gaps
        .iter()
        .map(|gap| (gap.id.clone(), gap))
        .collect::<BTreeMap<_, _>>();

    let mut fixed_gap_ids = Vec::new();
    let mut remaining_gap_ids = Vec::new();
    let mut remaining_gaps = Vec::new();

    for gap in current_gaps {
        if gap.status == GapStatus::Fixed {
            fixed_gap_ids.push(gap.id.clone());
        } else {
            remaining_gap_ids.push(gap.id.clone());
            if !selected_ids.contains(&gap.id) {
                remaining_gaps.push(gap.clone());
            }
        }
    }

    fixed_gap_ids.sort();
    remaining_gap_ids.sort();
    remaining_gaps.sort_by_key(gap_sort_key);

    let mut stop_conditions = BTreeSet::new();
    if plan.selected_gaps.is_empty() {
        stop_conditions.insert(QualityCycleStopCondition::NoSelectedGaps);
    } else if selected_ids.iter().all(|id| {
        current_by_id
            .get(id)
            .is_some_and(|gap| gap.status == GapStatus::Fixed)
    }) {
        stop_conditions.insert(QualityCycleStopCondition::SelectedGapsFixed);
    } else {
        stop_conditions.insert(QualityCycleStopCondition::VerificationFailedNeedsHumanReview);
    }

    if !remaining_gaps.is_empty()
        && remaining_gaps
            .iter()
            .all(|gap| gap.status == GapStatus::Unsupported)
    {
        stop_conditions.insert(QualityCycleStopCondition::RemainingGapsUnsupported);
    }

    if !remaining_gaps.is_empty()
        && remaining_gaps
            .iter()
            .all(|gap| gap.status == GapStatus::FalsePositive || is_comparator_triage_gap(gap))
    {
        stop_conditions.insert(QualityCycleStopCondition::RemainingGapsComparatorFalsePositive);
    }

    if !remaining_gaps.is_empty() && remaining_gaps.iter().all(gap_requires_later_plan) {
        stop_conditions
            .insert(QualityCycleStopCondition::RemainingGapsRequireArchitectureOrLanguageExpansion);
    }

    if verification
        .iter()
        .any(|item| item.status == QualityCycleVerificationStatus::Failed)
    {
        stop_conditions.insert(QualityCycleStopCondition::VerificationFailedNeedsHumanReview);
    }

    QualityCycleFinalReport {
        cycle_id: plan.cycle_id.clone(),
        selected_gap_ids: selected_ids.into_iter().collect(),
        fixed_gap_ids,
        remaining_gap_ids,
        stop_conditions: stop_conditions.into_iter().collect(),
        verification,
        automatic_next_cycle_allowed: false,
    }
}

pub fn render_quality_gap_report(report: &QualityGapReport) -> String {
    let mut out = String::new();
    out.push_str("# Quality Gaps\n\n");
    out.push_str(&format!("- repo: {}\n", report.repo_name));
    out.push_str(&format!("- path: {}\n", report.repo_path));
    out.push_str(&format!("- gaps: {}\n", report.gaps.len()));
    out.push_str("- cycle: check -> plan -> act\n\n");

    out.push_str("## Groups\n\n");
    let groups = group_gaps(&report.gaps);
    if groups.is_empty() {
        out.push_str("- none\n\n");
    } else {
        for group in groups {
            out.push_str(&format!(
                "- language={} construct={} severity={} evidence={} gaps={}\n",
                group.language,
                group.syntax_construct,
                group.severity.as_str(),
                group.evidence_source,
                group.gap_ids.join(", "),
            ));
        }
        out.push('\n');
    }

    out.push_str("## Gaps\n\n");
    if report.gaps.is_empty() {
        out.push_str("- none\n");
    } else {
        for gap in &report.gaps {
            render_gap(&mut out, gap);
        }
    }

    out
}

pub fn render_quality_cycle_plan(plan: &QualityCyclePlan) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", plan.cycle_id));
    out.push_str("- model: check -> plan -> act\n");
    out.push_str("- one-cycle limit: true\n");
    out.push_str(&format!("- max gaps: {}\n", plan.max_gaps));
    out.push_str(&format!("- selected gaps: {}\n", plan.selected_gaps.len()));
    out.push_str(&format!(
        "- deferred gaps: {}\n\n",
        plan.deferred_gap_ids.len()
    ));

    out.push_str("## Selected Batch\n\n");
    if plan.selected_gaps.is_empty() {
        out.push_str("- none\n\n");
    } else {
        for gap in &plan.selected_gaps {
            render_gap(&mut out, gap);
        }
    }

    out.push_str("## Act Checklist\n\n");
    out.push_str("- [ ] Reproduce each selected gap with the narrowest failing check.\n");
    out.push_str("- [ ] Implement parser/query/fixture/manifest fixes only for selected gaps.\n");
    out.push_str("- [ ] Add regression fixture or manifest expectation where practical.\n");
    out.push_str("- [ ] Rerun normal and applicable ignored quality gates.\n");
    out.push_str(
        "- [ ] Mark remaining comparator-only findings as open, unsupported, or false-positive.\n",
    );
    out.push_str("- [ ] Stop after this cycle and commit the bounded fix batch.\n\n");

    out.push_str("## Expected Change Set\n\n");
    let targets = suggested_change_targets(&plan.selected_gaps);
    if targets.is_empty() {
        out.push_str("- none\n\n");
    } else {
        for target in targets {
            out.push_str(&format!("- {target}\n"));
        }
        out.push('\n');
    }

    out.push_str("## Deferred Gap IDs\n\n");
    if plan.deferred_gap_ids.is_empty() {
        out.push_str("- none\n");
    } else {
        for gap_id in &plan.deferred_gap_ids {
            out.push_str(&format!("- {gap_id}\n"));
        }
    }

    out
}

pub fn render_quality_cycle_final_report(report: &QualityCycleFinalReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {} Final Report\n\n", report.cycle_id));
    out.push_str("- model: check -> plan -> act -> stop\n");
    out.push_str("- one-cycle limit: true\n");
    out.push_str(&format!(
        "- automatic next cycle allowed: {}\n",
        yes_no(report.automatic_next_cycle_allowed)
    ));
    out.push_str(&format!(
        "- selected gaps: {}\n",
        render_values(&report.selected_gap_ids)
    ));
    out.push_str(&format!(
        "- fixed gaps: {}\n",
        render_values(&report.fixed_gap_ids)
    ));
    out.push_str(&format!(
        "- remaining gaps: {}\n",
        render_values(&report.remaining_gap_ids)
    ));
    out.push_str(&format!(
        "- stop conditions: {}\n\n",
        render_stop_conditions(&report.stop_conditions)
    ));

    out.push_str("## Verification\n\n");
    if report.verification.is_empty() {
        out.push_str("- none recorded\n\n");
    } else {
        let mut verification = report.verification.clone();
        verification.sort_by(|left, right| left.command.cmp(&right.command));
        for item in verification {
            if item.detail.is_empty() {
                out.push_str(&format!(
                    "- command: `{}` status: {}\n",
                    item.command,
                    item.status.as_str()
                ));
            } else {
                out.push_str(&format!(
                    "- command: `{}` status: {} detail: {}\n",
                    item.command,
                    item.status.as_str(),
                    item.detail
                ));
            }
        }
        out.push('\n');
    }

    out.push_str("## Agent Boundary\n\n");
    out.push_str("- Stop after this report.\n");
    out.push_str("- Do not automatically start another quality cycle in this execution.\n");

    out
}

pub fn write_quality_gap_report(repo_root: &Path, report: &QualityGapReport) -> Result<PathBuf> {
    let report_dir = quality_report_dir(repo_root);
    fs::create_dir_all(&report_dir).with_context(|| {
        format!(
            "failed to create quality report dir {}",
            report_dir.display()
        )
    })?;
    let path = report_dir.join("QUALITY_GAPS.md");
    fs::write(&path, render_quality_gap_report(report))
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(path)
}

pub fn write_quality_cycle_run(
    repo_root: &Path,
    run: &QualityCycleRun,
) -> Result<QualityCycleRunPaths> {
    let gap_report_path = write_quality_gap_report(repo_root, &run.gap_report)?;
    let cycle_plan_path = write_quality_cycle_plan(repo_root, &run.plan)?;

    Ok(QualityCycleRunPaths {
        gap_report_path,
        cycle_plan_path,
    })
}

pub fn write_quality_cycle_plan(repo_root: &Path, plan: &QualityCyclePlan) -> Result<PathBuf> {
    let report_dir = quality_report_dir(repo_root);
    fs::create_dir_all(&report_dir).with_context(|| {
        format!(
            "failed to create quality report dir {}",
            report_dir.display()
        )
    })?;
    let path = report_dir.join(format!("{}_PLAN.md", sanitize_file_stem(&plan.cycle_id)));
    fs::write(&path, render_quality_cycle_plan(plan))
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(path)
}

pub fn write_quality_cycle_final_report(
    repo_root: &Path,
    report: &QualityCycleFinalReport,
) -> Result<PathBuf> {
    let report_dir = quality_report_dir(repo_root);
    fs::create_dir_all(&report_dir).with_context(|| {
        format!(
            "failed to create quality report dir {}",
            report_dir.display()
        )
    })?;
    let path = report_dir.join(format!(
        "{}_REPORT.md",
        sanitize_file_stem(&report.cycle_id)
    ));
    fs::write(&path, render_quality_cycle_final_report(report))
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(path)
}

struct GapBuilder {
    repo: String,
    repo_path: String,
    next_id: usize,
    gaps: Vec<QualityGap>,
}

impl GapBuilder {
    fn new(repo: &str, repo_path: &str) -> Self {
        Self {
            repo: repo.to_string(),
            repo_path: repo_path.to_string(),
            next_id: 1,
            gaps: Vec::new(),
        }
    }

    fn push(&mut self, input: GapInput) {
        let id = format!("GAP-{:04}", self.next_id);
        self.next_id += 1;
        self.gaps.push(QualityGap {
            id,
            repo: self.repo.clone(),
            path: input.path,
            language: normalize_language(&input.language),
            symbol: input.symbol,
            kind: input.kind,
            pattern: input.pattern,
            evidence_source: input.evidence_source,
            severity: input.severity,
            suggested_fix: input.suggested_fix,
            status: GapStatus::Open,
            fixture_added: false,
            manifest_added: false,
            detail: input.detail,
        });
    }

    fn finish(self) -> Vec<QualityGap> {
        let _ = self.repo_path;
        self.gaps
    }
}

struct GapInput {
    path: Option<String>,
    language: String,
    symbol: Option<String>,
    kind: Option<String>,
    pattern: Option<String>,
    evidence_source: String,
    severity: GapSeverity,
    suggested_fix: SuggestedFixType,
    detail: String,
}

fn integrity_gap(name: &str, count: usize, severity: GapSeverity) -> GapInput {
    GapInput {
        path: None,
        language: "unknown".to_string(),
        symbol: None,
        kind: Some(name.to_string()),
        pattern: None,
        evidence_source: "integrity".to_string(),
        severity,
        suggested_fix: SuggestedFixType::IntegrityFix,
        detail: format!("{name} count={count}"),
    }
}

fn comparator_gap(repo: &str, symbol: &ComparatorOnlySymbol) -> GapInput {
    GapInput {
        path: Some(symbol.path.clone()),
        language: symbol.language.clone(),
        symbol: Some(symbol.name.clone()),
        kind: Some(symbol.kind.clone()),
        pattern: None,
        evidence_source: "comparator-only".to_string(),
        severity: GapSeverity::Medium,
        suggested_fix: SuggestedFixType::ComparatorTriage,
        detail: format!(
            "{repo}: comparator-only symbol {}:{} {} {}",
            symbol.path, symbol.line, symbol.kind, symbol.name
        ),
    }
}

fn thinindex_only_gap(repo: &str, symbol: &ThinindexOnlySymbol) -> GapInput {
    GapInput {
        path: Some(symbol.path.clone()),
        language: symbol.language.clone(),
        symbol: Some(symbol.name.clone()),
        kind: Some(symbol.kind.clone()),
        pattern: None,
        evidence_source: "thinindex-only".to_string(),
        severity: GapSeverity::Low,
        suggested_fix: SuggestedFixType::ComparatorTriage,
        detail: format!(
            "{repo}: thinindex-only symbol {}:{} {} {}",
            symbol.path, symbol.line, symbol.kind, symbol.name
        ),
    }
}

fn render_gap(out: &mut String, gap: &QualityGap) {
    out.push_str(&format!("- id: {}\n", gap.id));
    out.push_str(&format!("  repo: {}\n", gap.repo));
    out.push_str(&format!(
        "  path: {}\n",
        gap.path.as_deref().unwrap_or("n/a")
    ));
    out.push_str(&format!("  language: {}\n", gap.language));
    out.push_str(&format!(
        "  symbol: {}\n",
        gap.symbol.as_deref().unwrap_or("n/a")
    ));
    out.push_str(&format!(
        "  kind: {}\n",
        gap.kind.as_deref().unwrap_or("n/a")
    ));
    out.push_str(&format!(
        "  pattern: {}\n",
        gap.pattern.as_deref().unwrap_or("n/a")
    ));
    out.push_str(&format!("  evidence: {}\n", gap.evidence_source));
    out.push_str(&format!("  severity: {}\n", gap.severity.as_str()));
    out.push_str(&format!(
        "  suggested_fix: {}\n",
        gap.suggested_fix.as_str()
    ));
    out.push_str(&format!("  status: {}\n", gap.status.as_str()));
    out.push_str(&format!("  fixture_added: {}\n", yes_no(gap.fixture_added)));
    out.push_str(&format!(
        "  manifest_added: {}\n",
        yes_no(gap.manifest_added)
    ));
    out.push_str(&format!("  detail: {}\n", gap.detail));
}

fn gap_sort_key(gap: &QualityGap) -> (String, String, u8, u8, String, String) {
    (
        gap.language.clone(),
        gap.evidence_source.clone(),
        gap.severity.rank(),
        gap.suggested_fix.rank(),
        gap.path.clone().unwrap_or_default(),
        gap.symbol
            .clone()
            .or_else(|| gap.pattern.clone())
            .unwrap_or_else(|| gap.detail.clone()),
    )
}

fn plan_sort_key(gap: &QualityGap) -> (u8, u8, u8, String, String, String) {
    (
        gap.severity.rank(),
        evidence_rank(&gap.evidence_source),
        gap.suggested_fix.rank(),
        gap.language.clone(),
        gap.path.clone().unwrap_or_default(),
        gap.id.clone(),
    )
}

fn open_gaps(gaps: &[QualityGap]) -> impl Iterator<Item = &QualityGap> {
    gaps.iter().filter(|gap| gap.status == GapStatus::Open)
}

fn is_actionable_cycle_gap(gap: &QualityGap) -> bool {
    !is_comparator_triage_gap(gap) && gap.suggested_fix != SuggestedFixType::Documentation
}

fn is_comparator_triage_gap(gap: &QualityGap) -> bool {
    gap.suggested_fix == SuggestedFixType::ComparatorTriage
        || matches!(
            gap.evidence_source.as_str(),
            "comparator-only" | "thinindex-only"
        )
}

fn evidence_rank(source: &str) -> u8 {
    match source {
        "expected-symbol" => 0,
        "expected-symbol-pattern" => 1,
        "expected-absent-symbol" => 2,
        "integrity" => 3,
        "quality-threshold" => 4,
        "comparator-only" => 5,
        "thinindex-only" => 6,
        "unsupported-extension" => 7,
        _ => 8,
    }
}

fn gap_requires_later_plan(gap: &QualityGap) -> bool {
    gap.status == GapStatus::Open
        && (gap.evidence_source == "unsupported-extension"
            || gap.suggested_fix == SuggestedFixType::Documentation
            || gap.detail.contains("architecture")
            || gap.detail.contains("language expansion")
            || gap.detail.contains("unsupported syntax"))
}

fn language_from_detail(detail: &str) -> String {
    extract_debug_field(detail, "language")
        .or_else(|| extract_plain_field(detail, "language"))
        .unwrap_or_else(|| "unknown".to_string())
}

fn path_from_detail(detail: &str) -> Option<String> {
    extract_debug_field(detail, "path").or_else(|| extract_plain_field(detail, "path"))
}

fn kind_from_detail(detail: &str) -> Option<String> {
    extract_debug_field(detail, "kind").or_else(|| extract_plain_field(detail, "kind"))
}

fn extract_debug_field(detail: &str, field: &str) -> Option<String> {
    let marker = format!("{field}=Some(\"");
    let start = detail.find(&marker)? + marker.len();
    let rest = &detail[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_plain_field(detail: &str, field: &str) -> Option<String> {
    let marker = format!("{field}=");
    let start = detail.find(&marker)? + marker.len();
    let rest = &detail[start..];
    let value = rest
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(',')
        .trim_matches('"');

    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn sanitize_file_stem(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();

    if sanitized.is_empty() {
        "QUALITY_CYCLE_01".to_string()
    } else {
        sanitized
    }
}

fn normalize_language(language: &str) -> String {
    let language = language.trim();
    if language.is_empty() {
        "unknown".to_string()
    } else {
        language.to_ascii_lowercase()
    }
}

fn render_values(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn render_stop_conditions(conditions: &[QualityCycleStopCondition]) -> String {
    if conditions.is_empty() {
        "none".to_string()
    } else {
        conditions
            .iter()
            .map(|condition| condition.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn suggested_change_targets(gaps: &[QualityGap]) -> Vec<String> {
    let mut targets = BTreeSet::new();

    for gap in gaps {
        if let Some(path) = &gap.path {
            targets.insert(format!("source: {path}"));
        }

        match gap.suggested_fix {
            SuggestedFixType::ParserQuery => {
                targets
                    .insert("parser: Tree-sitter query spec for the affected language".to_string());
                targets
                    .insert("tests: parser conformance fixture for the missed syntax".to_string());
            }
            SuggestedFixType::Fixture => {
                targets.insert("tests: parser or quality conformance fixture".to_string());
            }
            SuggestedFixType::ManifestExpectedSymbol => {
                targets.insert("manifest: test_repos/MANIFEST.toml expected symbol".to_string());
            }
            SuggestedFixType::ComparatorTriage => {
                targets.insert("quality: .dev_index/quality/COMPARATOR_TRIAGE.md".to_string());
            }
            SuggestedFixType::IntegrityFix => {
                targets.insert("tests: index integrity regression coverage".to_string());
            }
            SuggestedFixType::PerformanceTriage => {
                targets.insert("tests: benchmark or performance regression gate".to_string());
            }
            SuggestedFixType::Documentation => {
                targets.insert("docs: parser support or quality-loop documentation".to_string());
            }
        }
    }

    targets.into_iter().collect()
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
