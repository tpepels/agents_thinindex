use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::{Result, bail};
use regex::Regex;

use crate::{
    bench::{BenchmarkRepo, ExpectedSymbol, ExpectedSymbolPattern, QualityThreshold},
    model::{IndexRecord, ReferenceRecord},
    quality::{
        comparator::ComparatorRun,
        report::{QualityComparisonOptions, QualityReport, compare_quality},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityGateOptions {
    pub repo_name: String,
    pub repo_path: String,
    pub expected_symbol_names: Vec<String>,
    pub expected_symbol_name_patterns: Vec<String>,
    pub expected_symbols: Vec<ExpectedSymbol>,
    pub expected_symbol_patterns: Vec<ExpectedSymbolPattern>,
    pub quality_thresholds: Vec<QualityThreshold>,
    pub comparator_run: Option<ComparatorRun>,
}

impl QualityGateOptions {
    pub fn new(repo_name: impl Into<String>, repo_path: impl Into<String>) -> Self {
        Self {
            repo_name: repo_name.into(),
            repo_path: repo_path.into(),
            expected_symbol_names: Vec::new(),
            expected_symbol_name_patterns: Vec::new(),
            expected_symbols: Vec::new(),
            expected_symbol_patterns: Vec::new(),
            quality_thresholds: Vec::new(),
            comparator_run: None,
        }
    }

    pub fn from_benchmark_repo(repo: &BenchmarkRepo) -> Self {
        Self {
            repo_name: repo.name.clone(),
            repo_path: repo.path.display().to_string(),
            expected_symbol_names: repo.expected_symbols.clone(),
            expected_symbol_name_patterns: repo.expected_symbol_patterns.clone(),
            expected_symbols: repo.expected_symbol_specs.clone(),
            expected_symbol_patterns: repo.expected_symbol_pattern_specs.clone(),
            quality_thresholds: repo.quality_thresholds.clone(),
            comparator_run: None,
        }
    }

    pub fn with_expected_symbols(mut self, expected_symbols: Vec<ExpectedSymbol>) -> Self {
        self.expected_symbols = expected_symbols;
        self
    }

    pub fn with_expected_symbol_patterns(
        mut self,
        expected_symbol_patterns: Vec<ExpectedSymbolPattern>,
    ) -> Self {
        self.expected_symbol_patterns = expected_symbol_patterns;
        self
    }

    pub fn with_quality_thresholds(mut self, quality_thresholds: Vec<QualityThreshold>) -> Self {
        self.quality_thresholds = quality_thresholds;
        self
    }

    pub fn with_comparator_run(mut self, comparator_run: ComparatorRun) -> Self {
        self.comparator_run = Some(comparator_run);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThresholdFailure {
    pub language: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityGateReport {
    pub repo_name: String,
    pub repo_path: String,
    pub languages_checked: Vec<String>,
    pub records_by_language: Vec<(String, usize)>,
    pub refs_by_language: Vec<(String, usize)>,
    pub expected_symbols_checked: usize,
    pub expected_symbols_missing: Vec<String>,
    pub expected_patterns_checked: usize,
    pub expected_patterns_failing: Vec<String>,
    pub thresholds_checked: usize,
    pub threshold_failures: Vec<ThresholdFailure>,
    pub duplicate_record_count: usize,
    pub duplicate_ref_count: usize,
    pub malformed_record_count: usize,
    pub malformed_ref_count: usize,
    pub dev_index_path_count: usize,
    pub ctags_source_count: usize,
    pub unsupported_extensions: Vec<(String, usize)>,
    pub comparator_report: Option<QualityReport>,
}

pub fn evaluate_quality_gate(
    records: &[IndexRecord],
    refs: &[ReferenceRecord],
    options: QualityGateOptions,
) -> Result<QualityGateReport> {
    let record_metrics = collect_record_metrics(records);
    let ref_metrics = collect_ref_metrics(refs, &record_metrics.language_by_path);
    let expected = check_expected_symbols(records, &options)?;
    let thresholds = check_thresholds(&options.quality_thresholds, &record_metrics);
    let comparator_report = options.comparator_run.as_ref().map(|run| {
        compare_quality(
            records,
            run,
            QualityComparisonOptions::new(options.repo_name.clone(), options.repo_path.clone())
                .with_expected_symbols(options.expected_symbols.clone()),
        )
    });

    let mut languages = BTreeSet::new();
    languages.extend(record_metrics.records_by_language.keys().cloned());
    languages.extend(ref_metrics.refs_by_language.keys().cloned());
    languages.extend(
        options
            .quality_thresholds
            .iter()
            .map(|threshold| threshold.language.clone()),
    );
    languages.extend(
        options
            .expected_symbols
            .iter()
            .filter_map(|symbol| symbol.language.clone()),
    );
    languages.extend(
        options
            .expected_symbol_patterns
            .iter()
            .filter_map(|pattern| pattern.language.clone()),
    );

    Ok(QualityGateReport {
        repo_name: options.repo_name,
        repo_path: options.repo_path,
        languages_checked: languages.into_iter().collect(),
        records_by_language: map_to_vec(record_metrics.records_by_language),
        refs_by_language: map_to_vec(ref_metrics.refs_by_language),
        expected_symbols_checked: expected.symbols_checked,
        expected_symbols_missing: expected.symbols_missing,
        expected_patterns_checked: expected.patterns_checked,
        expected_patterns_failing: expected.patterns_failing,
        thresholds_checked: options.quality_thresholds.len(),
        threshold_failures: thresholds,
        duplicate_record_count: record_metrics.duplicate_record_count,
        duplicate_ref_count: ref_metrics.duplicate_ref_count,
        malformed_record_count: record_metrics.malformed_record_count,
        malformed_ref_count: ref_metrics.malformed_ref_count,
        dev_index_path_count: record_metrics.dev_index_path_count
            + ref_metrics.dev_index_path_count,
        ctags_source_count: record_metrics.ctags_source_count,
        unsupported_extensions: map_to_vec(record_metrics.unsupported_extensions),
        comparator_report,
    })
}

pub fn assert_quality_gate_passes(report: &QualityGateReport) -> Result<()> {
    if report.expected_symbols_missing.is_empty()
        && report.expected_patterns_failing.is_empty()
        && report.threshold_failures.is_empty()
        && report.duplicate_record_count == 0
        && report.duplicate_ref_count == 0
        && report.malformed_record_count == 0
        && report.malformed_ref_count == 0
        && report.dev_index_path_count == 0
        && report.ctags_source_count == 0
    {
        return Ok(());
    }

    bail!(
        "quality drift gate failed for {}\n{}",
        report.repo_name,
        render_quality_gate_report(report)
    )
}

pub fn render_quality_gate_report(report: &QualityGateReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("Repo: {}\n", report.repo_name));
    out.push_str(&format!("- path: {}\n", report.repo_path));
    out.push_str(&format!(
        "- languages checked: {}\n",
        render_values(&report.languages_checked)
    ));
    out.push_str(&format!(
        "- expected symbols: {} checked, {} missing\n",
        report.expected_symbols_checked,
        report.expected_symbols_missing.len()
    ));
    out.push_str(&format!(
        "- expected patterns: {} checked, {} failing\n",
        report.expected_patterns_checked,
        report.expected_patterns_failing.len()
    ));
    out.push_str(&format!(
        "- thresholds: {} checked, {} failing\n",
        report.thresholds_checked,
        report.threshold_failures.len()
    ));
    out.push_str(&format!(
        "- integrity: duplicate-records={} duplicate-refs={} malformed-records={} malformed-refs={} dev-index-paths={} ctags-sources={}\n",
        report.duplicate_record_count,
        report.duplicate_ref_count,
        report.malformed_record_count,
        report.malformed_ref_count,
        report.dev_index_path_count,
        report.ctags_source_count,
    ));
    out.push_str(&format!(
        "- records by language: {}\n",
        render_counts(&report.records_by_language)
    ));
    out.push_str(&format!(
        "- refs by language: {}\n",
        render_counts(&report.refs_by_language)
    ));
    out.push_str(&format!(
        "- unsupported extensions: {}\n",
        render_counts(&report.unsupported_extensions)
    ));

    if let Some(comparator) = &report.comparator_report {
        out.push_str(&format!("- comparator: {}\n", comparator.comparator_name));
        if comparator.skipped {
            out.push_str(&format!(
                "- comparator skipped: {}\n",
                comparator
                    .skip_reason
                    .as_deref()
                    .unwrap_or("comparator skipped")
            ));
        } else {
            out.push_str(&format!(
                "- comparator symbols: thinindex-only={} comparator-only={}\n",
                comparator.thinindex_only.len(),
                comparator.comparator_only.len()
            ));
        }
    } else {
        out.push_str("- comparator: not run\n");
    }

    if !report.expected_symbols_missing.is_empty() {
        out.push_str("\nMissing expected symbols:\n");
        for missing in &report.expected_symbols_missing {
            out.push_str(&format!("- {missing}\n"));
        }
    }

    if !report.expected_patterns_failing.is_empty() {
        out.push_str("\nFailing expected patterns:\n");
        for failing in &report.expected_patterns_failing {
            out.push_str(&format!("- {failing}\n"));
        }
    }

    if !report.threshold_failures.is_empty() {
        out.push_str("\nThreshold failures:\n");
        for failure in &report.threshold_failures {
            out.push_str(&format!("- {}: {}\n", failure.language, failure.message));
        }
    }

    if let Some(comparator) = &report.comparator_report
        && !comparator.skipped
    {
        out.push_str("\nComparator-only symbols:\n");
        if comparator.comparator_only.is_empty() {
            out.push_str("- none\n");
        } else {
            for symbol in &comparator.comparator_only {
                out.push_str(&format!(
                    "- {}:{} {} {} ({})\n",
                    symbol.path, symbol.line, symbol.kind, symbol.name, symbol.language
                ));
            }
        }

        out.push_str("\nThinindex-only symbols:\n");
        if comparator.thinindex_only.is_empty() {
            out.push_str("- none\n");
        } else {
            for symbol in &comparator.thinindex_only {
                out.push_str(&format!(
                    "- {}:{} {} {} ({})\n",
                    symbol.path, symbol.line, symbol.kind, symbol.name, symbol.language
                ));
            }
        }
    }

    out
}

#[derive(Debug, Default)]
struct RecordMetrics {
    records_by_language: BTreeMap<String, usize>,
    duplicate_locations_by_language: BTreeMap<String, usize>,
    malformed_records_by_language: BTreeMap<String, usize>,
    language_by_path: BTreeMap<String, String>,
    unsupported_extensions: BTreeMap<String, usize>,
    duplicate_record_count: usize,
    malformed_record_count: usize,
    dev_index_path_count: usize,
    ctags_source_count: usize,
}

#[derive(Debug, Default)]
struct RefMetrics {
    refs_by_language: BTreeMap<String, usize>,
    duplicate_ref_count: usize,
    malformed_ref_count: usize,
    dev_index_path_count: usize,
}

#[derive(Debug, Default)]
struct ExpectedCheck {
    symbols_checked: usize,
    symbols_missing: Vec<String>,
    patterns_checked: usize,
    patterns_failing: Vec<String>,
}

fn collect_record_metrics(records: &[IndexRecord]) -> RecordMetrics {
    let mut metrics = RecordMetrics::default();
    let mut locations = BTreeSet::new();

    for record in records {
        let language = normalize_language(&record.lang);
        *metrics
            .records_by_language
            .entry(language.clone())
            .or_default() += 1;
        metrics
            .language_by_path
            .entry(normalize_path(&record.path))
            .or_insert_with(|| language.clone());

        if !locations.insert((normalize_path(&record.path), record.line, record.col)) {
            metrics.duplicate_record_count += 1;
            *metrics
                .duplicate_locations_by_language
                .entry(language.clone())
                .or_default() += 1;
        }

        if record.path.is_empty()
            || record.line == 0
            || record.col == 0
            || record.lang.is_empty()
            || record.kind.is_empty()
            || record.name.is_empty()
            || record.source.is_empty()
        {
            metrics.malformed_record_count += 1;
            *metrics
                .malformed_records_by_language
                .entry(language)
                .or_default() += 1;
        }

        if record.path.contains(".dev_index") {
            metrics.dev_index_path_count += 1;
        }

        if record.source == "ctags" {
            metrics.ctags_source_count += 1;
        }

        if let Some(extension) = extension(&record.path)
            && record.lang == "unknown"
        {
            *metrics.unsupported_extensions.entry(extension).or_default() += 1;
        }
    }

    metrics
}

fn collect_ref_metrics(
    refs: &[ReferenceRecord],
    language_by_path: &BTreeMap<String, String>,
) -> RefMetrics {
    let mut metrics = RefMetrics::default();
    let mut refs_seen = BTreeSet::new();

    for reference in refs {
        let language = language_by_path
            .get(&normalize_path(&reference.from_path))
            .cloned()
            .or_else(|| extension(&reference.from_path))
            .map(|extension| extension.trim_start_matches('.').to_string())
            .unwrap_or_else(|| "unknown".to_string());

        *metrics.refs_by_language.entry(language).or_default() += 1;

        if !refs_seen.insert((
            normalize_path(&reference.from_path),
            reference.from_line,
            reference.from_col,
            reference.to_name.clone(),
            reference.ref_kind.clone(),
        )) {
            metrics.duplicate_ref_count += 1;
        }

        if reference.from_path.is_empty()
            || reference.from_line == 0
            || reference.from_col == 0
            || reference.to_name.is_empty()
            || reference.ref_kind.is_empty()
            || reference.evidence.is_empty()
            || reference.source.is_empty()
        {
            metrics.malformed_ref_count += 1;
        }

        if reference.from_path.contains(".dev_index") {
            metrics.dev_index_path_count += 1;
        }
    }

    metrics
}

fn check_expected_symbols(
    records: &[IndexRecord],
    options: &QualityGateOptions,
) -> Result<ExpectedCheck> {
    let mut check = ExpectedCheck::default();

    for name in &options.expected_symbol_names {
        check.symbols_checked += 1;
        if !records.iter().any(|record| record.name == *name) {
            check.symbols_missing.push(format!("name={name}"));
        }
    }

    for expected in &options.expected_symbols {
        check.symbols_checked += 1;
        if !records
            .iter()
            .any(|record| record_matches_symbol(record, expected))
        {
            check.symbols_missing.push(format!(
                "missing expected symbol {}",
                format_symbol(expected)
            ));
        }
    }

    for pattern in &options.expected_symbol_name_patterns {
        check.patterns_checked += 1;
        let regex = Regex::new(pattern)?;
        if !records.iter().any(|record| regex.is_match(&record.name)) {
            check
                .patterns_failing
                .push(format!("name_regex={pattern} min_count=1 actual=0"));
        }
    }

    for pattern in &options.expected_symbol_patterns {
        check.patterns_checked += 1;
        let regex = Regex::new(&pattern.name_regex)?;
        let count = records
            .iter()
            .filter(|record| record_matches_pattern(record, pattern, &regex))
            .count();

        if count < pattern.min_count {
            check.patterns_failing.push(format!(
                "language={:?} path_glob={:?} kind={:?} name_regex={} min_count={} actual={count}",
                pattern.language,
                pattern.path_glob,
                pattern.kind,
                pattern.name_regex,
                pattern.min_count
            ));
        }
    }

    check.symbols_missing.sort();
    check.patterns_failing.sort();
    Ok(check)
}

fn check_thresholds(
    thresholds: &[QualityThreshold],
    metrics: &RecordMetrics,
) -> Vec<ThresholdFailure> {
    let mut failures = Vec::new();

    for threshold in thresholds {
        let language = normalize_language(&threshold.language);
        let record_count = metrics
            .records_by_language
            .get(&language)
            .copied()
            .unwrap_or(0);
        let duplicate_count = metrics
            .duplicate_locations_by_language
            .get(&language)
            .copied()
            .unwrap_or(0);
        let malformed_count = metrics
            .malformed_records_by_language
            .get(&language)
            .copied()
            .unwrap_or(0);

        if let Some(min_records) = threshold.min_records
            && record_count < min_records
        {
            failures.push(ThresholdFailure {
                language: language.clone(),
                message: format!("min_records={min_records} actual={record_count}"),
            });
        }

        if let Some(max_duplicate_locations) = threshold.max_duplicate_locations
            && duplicate_count > max_duplicate_locations
        {
            failures.push(ThresholdFailure {
                language: language.clone(),
                message: format!(
                    "max_duplicate_locations={max_duplicate_locations} actual={duplicate_count}"
                ),
            });
        }

        if let Some(max_malformed_records) = threshold.max_malformed_records
            && malformed_count > max_malformed_records
        {
            failures.push(ThresholdFailure {
                language,
                message: format!(
                    "max_malformed_records={max_malformed_records} actual={malformed_count}"
                ),
            });
        }
    }

    failures.sort_by(|left, right| {
        left.language
            .cmp(&right.language)
            .then(left.message.cmp(&right.message))
    });
    failures
}

fn record_matches_symbol(record: &IndexRecord, expected: &ExpectedSymbol) -> bool {
    record.name == expected.name
        && expected
            .language
            .as_ref()
            .is_none_or(|language| normalize_language(&record.lang) == normalize_language(language))
        && expected
            .path
            .as_ref()
            .is_none_or(|path| normalize_path(&record.path) == normalize_path(path))
        && expected
            .kind
            .as_ref()
            .is_none_or(|kind| record.kind == *kind)
}

fn record_matches_pattern(
    record: &IndexRecord,
    pattern: &ExpectedSymbolPattern,
    name_regex: &Regex,
) -> bool {
    pattern
        .language
        .as_ref()
        .is_none_or(|language| normalize_language(&record.lang) == normalize_language(language))
        && pattern
            .kind
            .as_ref()
            .is_none_or(|kind| record.kind == *kind)
        && pattern
            .path_glob
            .as_ref()
            .is_none_or(|path_glob| glob_matches(path_glob, &record.path))
        && name_regex.is_match(&record.name)
}

fn format_symbol(symbol: &ExpectedSymbol) -> String {
    format!(
        "language={:?} path={:?} kind={:?} name={}",
        symbol.language, symbol.path, symbol.kind, symbol.name
    )
}

fn glob_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.replace('\\', "/");
    let value = value.replace('\\', "/");

    if wildcard_match(pattern.as_bytes(), value.as_bytes()) {
        return true;
    }

    if let Some(double_star_dir) = pattern.find("**/") {
        let mut without_empty_dir_segment = pattern.clone();
        without_empty_dir_segment.replace_range(double_star_dir..double_star_dir + 3, "");
        return wildcard_match(without_empty_dir_segment.as_bytes(), value.as_bytes());
    }

    false
}

fn wildcard_match(pattern: &[u8], value: &[u8]) -> bool {
    if pattern.is_empty() {
        return value.is_empty();
    }

    if pattern[0] == b'*' {
        let mut next = 1;
        while next < pattern.len() && pattern[next] == b'*' {
            next += 1;
        }
        let remainder = &pattern[next..];

        return (0..=value.len()).any(|index| wildcard_match(remainder, &value[index..]));
    }

    if !value.is_empty() && (pattern[0] == b'?' || pattern[0] == value[0]) {
        return wildcard_match(&pattern[1..], &value[1..]);
    }

    false
}

fn map_to_vec(values: BTreeMap<String, usize>) -> Vec<(String, usize)> {
    values.into_iter().collect()
}

fn render_counts(counts: &[(String, usize)]) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }

    counts
        .iter()
        .map(|(name, count)| format!("{name}={count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_values(values: &[String]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }

    values.join(", ")
}

fn normalize_language(language: &str) -> String {
    let language = language.trim();
    if language.is_empty() {
        "unknown".to_string()
    } else {
        language.to_ascii_lowercase()
    }
}

fn normalize_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    normalized
        .strip_prefix("./")
        .unwrap_or(&normalized)
        .to_string()
}

fn extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy().to_ascii_lowercase()))
}
