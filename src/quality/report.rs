use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use crate::{
    bench::ExpectedSymbol,
    model::IndexRecord,
    privacy::redact_sensitive_text,
    quality::{
        comparator::{ComparatorRecord, ComparatorRun, ComparatorStatus},
        manifest::quality_report_dir,
    },
};

pub const LINE_PROXIMITY: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityComparisonOptions {
    pub repo_name: String,
    pub repo_path: String,
    pub expected_symbols: Vec<ExpectedSymbol>,
}

impl QualityComparisonOptions {
    pub fn new(repo_name: impl Into<String>, repo_path: impl Into<String>) -> Self {
        Self {
            repo_name: repo_name.into(),
            repo_path: repo_path.into(),
            expected_symbols: Vec::new(),
        }
    }

    pub fn with_expected_symbols(mut self, expected_symbols: Vec<ExpectedSymbol>) -> Self {
        self.expected_symbols = expected_symbols;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageQualityMetrics {
    pub language: String,
    pub thinindex_record_count: usize,
    pub comparator_record_count: usize,
    pub matched_symbol_count: usize,
    pub thinindex_only_symbol_count: usize,
    pub comparator_only_symbol_count: usize,
    pub expected_symbol_pass_count: usize,
    pub expected_symbol_fail_count: usize,
    pub unknown_comparator_kind_count: usize,
    pub duplicate_record_count: usize,
    pub malformed_record_count: usize,
    pub unsupported_extension_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThinindexOnlySymbol {
    pub path: String,
    pub line: usize,
    pub kind: String,
    pub name: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComparatorOnlySymbol {
    pub path: String,
    pub line: usize,
    pub kind: String,
    pub name: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityReport {
    pub repo_name: String,
    pub repo_path: String,
    pub comparator_name: String,
    pub skipped: bool,
    pub skip_reason: Option<String>,
    pub metrics: Vec<LanguageQualityMetrics>,
    pub thinindex_only: Vec<ThinindexOnlySymbol>,
    pub comparator_only: Vec<ComparatorOnlySymbol>,
    pub expected_symbols_checked: usize,
    pub expected_symbols_missing: Vec<String>,
    pub unknown_comparator_kinds: Vec<String>,
    pub duplicate_record_count: usize,
    pub malformed_record_count: usize,
    pub unsupported_extensions: Vec<String>,
}

#[derive(Debug, Default)]
struct MutableLanguageMetrics {
    thinindex_record_count: usize,
    comparator_record_count: usize,
    matched_symbol_count: usize,
    thinindex_only_symbol_count: usize,
    comparator_only_symbol_count: usize,
    expected_symbol_pass_count: usize,
    expected_symbol_fail_count: usize,
    unknown_comparator_kind_count: usize,
    duplicate_record_count: usize,
    malformed_record_count: usize,
    unsupported_extension_count: usize,
}

pub fn compare_quality(
    thinindex_records: &[IndexRecord],
    comparator_run: &ComparatorRun,
    options: QualityComparisonOptions,
) -> QualityReport {
    let mut metrics: BTreeMap<String, MutableLanguageMetrics> = BTreeMap::new();
    let mut thinindex_records = thinindex_records.to_vec();
    thinindex_records.sort_by_key(thinindex_sort_key);

    let mut comparator_records = comparator_run.records.clone();
    comparator_records.sort_by_key(comparator_sort_key);

    let mut duplicate_record_count = 0usize;
    let mut malformed_record_count = 0usize;
    let mut unknown_comparator_kinds = BTreeMap::<String, usize>::new();
    let mut unsupported_extensions = BTreeMap::<String, usize>::new();

    let mut thinindex_duplicate_keys = BTreeSet::new();
    for record in &thinindex_records {
        let language = normalize_language(&record.lang);
        let metric = metrics.entry(language.clone()).or_default();
        metric.thinindex_record_count += 1;

        if !thinindex_duplicate_keys.insert((
            normalize_path(&record.path),
            record.line,
            record.col,
            record.kind.clone(),
            record.name.clone(),
        )) {
            duplicate_record_count += 1;
            metric.duplicate_record_count += 1;
        }

        if is_malformed_thinindex_record(record) {
            malformed_record_count += 1;
            metric.malformed_record_count += 1;
        }
    }

    let mut comparator_duplicate_keys = BTreeSet::new();
    for record in &comparator_records {
        let language = comparator_language(record);
        let metric = metrics.entry(language.clone()).or_default();
        metric.comparator_record_count += 1;

        if !comparator_duplicate_keys.insert((
            normalize_path(&record.path),
            record.line,
            record.column.unwrap_or(0),
            record.kind.clone(),
            record.name.clone(),
        )) {
            duplicate_record_count += 1;
            metric.duplicate_record_count += 1;
        }

        if is_malformed_comparator_record(record) {
            malformed_record_count += 1;
            metric.malformed_record_count += 1;
        }

        if kind_group(&record.kind).is_none() {
            *unknown_comparator_kinds
                .entry(record.kind.clone())
                .or_default() += 1;
            metric.unknown_comparator_kind_count += 1;
        }
    }

    let mut matched_thinindex = BTreeSet::<usize>::new();
    let mut matched_comparator = BTreeSet::<usize>::new();

    for (comparator_index, comparator_record) in comparator_records.iter().enumerate() {
        if is_malformed_comparator_record(comparator_record) {
            continue;
        }

        let Some(match_index) =
            best_match(&thinindex_records, comparator_record, &matched_thinindex)
        else {
            continue;
        };

        matched_thinindex.insert(match_index);
        matched_comparator.insert(comparator_index);
        let language = normalize_language(&thinindex_records[match_index].lang);
        metrics.entry(language).or_default().matched_symbol_count += 1;
    }

    let mut thinindex_only = Vec::new();
    for (index, record) in thinindex_records.iter().enumerate() {
        if matched_thinindex.contains(&index) || is_malformed_thinindex_record(record) {
            continue;
        }
        let language = normalize_language(&record.lang);
        metrics
            .entry(language.clone())
            .or_default()
            .thinindex_only_symbol_count += 1;
        thinindex_only.push(ThinindexOnlySymbol {
            path: normalize_path(&record.path),
            line: record.line,
            kind: record.kind.clone(),
            name: record.name.clone(),
            language,
        });
    }

    let thinindex_languages = thinindex_records
        .iter()
        .map(|record| normalize_language(&record.lang))
        .collect::<BTreeSet<_>>();
    let thinindex_extensions = thinindex_records
        .iter()
        .filter_map(|record| extension(&record.path))
        .collect::<BTreeSet<_>>();

    let mut comparator_only = Vec::new();
    for (index, record) in comparator_records.iter().enumerate() {
        if matched_comparator.contains(&index) || is_malformed_comparator_record(record) {
            continue;
        }
        let language = comparator_language(record);
        metrics
            .entry(language.clone())
            .or_default()
            .comparator_only_symbol_count += 1;
        if !thinindex_languages.contains(&language)
            && let Some(extension) = extension(&record.path)
            && !thinindex_extensions.contains(&extension)
        {
            *unsupported_extensions.entry(extension).or_default() += 1;
            metrics
                .entry(language.clone())
                .or_default()
                .unsupported_extension_count += 1;
        }
        comparator_only.push(ComparatorOnlySymbol {
            path: normalize_path(&record.path),
            line: record.line,
            kind: record.kind.clone(),
            name: record.name.clone(),
            language,
        });
    }

    thinindex_only.sort();
    comparator_only.sort();

    let mut expected_symbols_missing = Vec::new();
    for expected in &options.expected_symbols {
        let language = expected
            .language
            .as_deref()
            .map(normalize_language)
            .unwrap_or_else(|| "unknown".to_string());
        let metric = metrics.entry(language).or_default();

        if expected_symbol_found(&thinindex_records, expected) {
            metric.expected_symbol_pass_count += 1;
        } else {
            metric.expected_symbol_fail_count += 1;
            expected_symbols_missing.push(format_expected_symbol(expected));
        }
    }
    expected_symbols_missing.sort();

    let metrics = metrics
        .into_iter()
        .map(|(language, metric)| LanguageQualityMetrics {
            language,
            thinindex_record_count: metric.thinindex_record_count,
            comparator_record_count: metric.comparator_record_count,
            matched_symbol_count: metric.matched_symbol_count,
            thinindex_only_symbol_count: metric.thinindex_only_symbol_count,
            comparator_only_symbol_count: metric.comparator_only_symbol_count,
            expected_symbol_pass_count: metric.expected_symbol_pass_count,
            expected_symbol_fail_count: metric.expected_symbol_fail_count,
            unknown_comparator_kind_count: metric.unknown_comparator_kind_count,
            duplicate_record_count: metric.duplicate_record_count,
            malformed_record_count: metric.malformed_record_count,
            unsupported_extension_count: metric.unsupported_extension_count,
        })
        .collect();

    QualityReport {
        repo_name: options.repo_name,
        repo_path: options.repo_path,
        comparator_name: comparator_run.comparator.clone(),
        skipped: comparator_run.status == ComparatorStatus::Skipped,
        skip_reason: comparator_run.message.clone(),
        metrics,
        thinindex_only,
        comparator_only,
        expected_symbols_checked: options.expected_symbols.len(),
        expected_symbols_missing,
        unknown_comparator_kinds: repeated_values(unknown_comparator_kinds),
        duplicate_record_count,
        malformed_record_count,
        unsupported_extensions: repeated_values(unsupported_extensions),
    }
}

pub fn render_quality_report(report: &QualityReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Repo: {}\n",
        redact_sensitive_text(&report.repo_name)
    ));
    out.push_str(&format!(
        "- path: {}\n",
        redact_sensitive_text(&report.repo_path)
    ));
    out.push_str(&format!(
        "- comparator: {}\n",
        redact_sensitive_text(&report.comparator_name)
    ));

    if report.skipped {
        out.push_str(&format!(
            "- skipped: {}\n",
            report
                .skip_reason
                .as_deref()
                .map(redact_sensitive_text)
                .unwrap_or_else(|| "comparator skipped".to_string())
        ));
    } else {
        out.push_str("- skipped: no\n");
    }

    out.push_str(&format!(
        "- expected symbols: {} checked, {} missing\n",
        report.expected_symbols_checked,
        report.expected_symbols_missing.len()
    ));
    out.push_str(&format!(
        "- duplicates: {} malformed: {}\n",
        report.duplicate_record_count, report.malformed_record_count
    ));

    out.push_str("\nLanguages:\n");
    if report.metrics.is_empty() {
        out.push_str("- none\n");
    } else {
        for metric in &report.metrics {
            out.push_str(&format!(
                "- {}: thinindex={} comparator={} matched={} thinindex-only={} comparator-only={} expected-pass={} expected-fail={} unknown-kinds={} duplicates={} malformed={} unsupported-exts={}\n",
                redact_sensitive_text(&metric.language),
                metric.thinindex_record_count,
                metric.comparator_record_count,
                metric.matched_symbol_count,
                metric.thinindex_only_symbol_count,
                metric.comparator_only_symbol_count,
                metric.expected_symbol_pass_count,
                metric.expected_symbol_fail_count,
                metric.unknown_comparator_kind_count,
                metric.duplicate_record_count,
                metric.malformed_record_count,
                metric.unsupported_extension_count,
            ));
        }
    }

    out.push_str("\nThinindex-only:\n");
    if report.thinindex_only.is_empty() {
        out.push_str("- none\n");
    } else {
        for symbol in &report.thinindex_only {
            out.push_str(&format!(
                "- {}:{} {} {} ({})\n",
                redact_sensitive_text(&symbol.path),
                symbol.line,
                redact_sensitive_text(&symbol.kind),
                redact_sensitive_text(&symbol.name),
                redact_sensitive_text(&symbol.language)
            ));
        }
    }

    out.push_str("\nComparator-only:\n");
    if report.comparator_only.is_empty() {
        out.push_str("- none\n");
    } else {
        for symbol in &report.comparator_only {
            out.push_str(&format!(
                "- {}:{} {} {} ({})\n",
                redact_sensitive_text(&symbol.path),
                symbol.line,
                redact_sensitive_text(&symbol.kind),
                redact_sensitive_text(&symbol.name),
                redact_sensitive_text(&symbol.language)
            ));
        }
    }

    if !report.expected_symbols_missing.is_empty() {
        out.push_str("\nMissing expected symbols:\n");
        for symbol in &report.expected_symbols_missing {
            out.push_str(&format!("- {}\n", redact_sensitive_text(symbol)));
        }
    }

    if !report.unknown_comparator_kinds.is_empty() {
        out.push_str("\nUnknown comparator kinds:\n");
        for kind in &report.unknown_comparator_kinds {
            out.push_str(&format!("- {}\n", redact_sensitive_text(kind)));
        }
    }

    if !report.unsupported_extensions.is_empty() {
        out.push_str("\nUnsupported extensions:\n");
        for extension in &report.unsupported_extensions {
            out.push_str(&format!("- {}\n", redact_sensitive_text(extension)));
        }
    }

    out
}

pub fn write_quality_report(
    repo_root: &Path,
    comparator_name: &str,
    contents: &str,
) -> Result<PathBuf> {
    let report_dir = quality_report_dir(repo_root);
    fs::create_dir_all(&report_dir).with_context(|| {
        format!(
            "failed to create quality report dir {}",
            report_dir.display()
        )
    })?;
    let file_name = sanitize_report_name(comparator_name);
    let report_path = report_dir.join(format!("{file_name}.txt"));
    fs::write(&report_path, redact_sensitive_text(contents)).with_context(|| {
        format!(
            "failed to write isolated quality report {}",
            report_path.display()
        )
    })?;
    Ok(report_path)
}

pub fn assert_quality_report_has_no_malformed_thinindex_records(
    report: &QualityReport,
) -> Result<()> {
    if report.malformed_record_count == 0 {
        return Ok(());
    }

    bail!(
        "quality report for {} has {} malformed records",
        report.repo_name,
        report.malformed_record_count
    )
}

fn best_match(
    thinindex_records: &[IndexRecord],
    comparator_record: &ComparatorRecord,
    matched_thinindex: &BTreeSet<usize>,
) -> Option<usize> {
    thinindex_records
        .iter()
        .enumerate()
        .filter(|(index, record)| {
            !matched_thinindex.contains(index)
                && records_match(record, comparator_record)
                && !is_malformed_thinindex_record(record)
        })
        .min_by_key(|(_, record)| record.line.abs_diff(comparator_record.line))
        .map(|(index, _)| index)
}

fn records_match(thinindex_record: &IndexRecord, comparator_record: &ComparatorRecord) -> bool {
    normalize_path(&thinindex_record.path) == normalize_path(&comparator_record.path)
        && thinindex_record.name == comparator_record.name
        && compatible_kind(&thinindex_record.kind, &comparator_record.kind)
        && thinindex_record.line.abs_diff(comparator_record.line) <= LINE_PROXIMITY
}

fn compatible_kind(thinindex_kind: &str, comparator_kind: &str) -> bool {
    match (kind_group(thinindex_kind), kind_group(comparator_kind)) {
        (Some(left), Some(right)) => left == right,
        _ => thinindex_kind.eq_ignore_ascii_case(comparator_kind),
    }
}

fn kind_group(kind: &str) -> Option<&'static str> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "function" | "func" | "f" | "method" | "m" | "constructor" => Some("callable"),
        "class" | "c" | "struct" | "s" | "interface" | "trait" | "enum" | "type" | "typedef" => {
            Some("type")
        }
        "module" | "namespace" | "package" => Some("module"),
        "constant" | "const" | "variable" | "var" | "field" | "property" | "member" => {
            Some("value")
        }
        "import" | "export" => Some("module-edge"),
        _ => None,
    }
}

fn expected_symbol_found(records: &[IndexRecord], expected: &ExpectedSymbol) -> bool {
    records.iter().any(|record| {
        record.name == expected.name
            && expected.language.as_ref().is_none_or(|language| {
                normalize_language(&record.lang) == normalize_language(language)
            })
            && expected
                .path
                .as_ref()
                .is_none_or(|path| normalize_path(&record.path).contains(&normalize_path(path)))
            && expected
                .kind
                .as_ref()
                .is_none_or(|kind| compatible_kind(&record.kind, kind))
    })
}

fn format_expected_symbol(expected: &ExpectedSymbol) -> String {
    format!(
        "{}{}{}{}",
        expected
            .language
            .as_deref()
            .map(|language| format!("{language}:"))
            .unwrap_or_default(),
        expected
            .path
            .as_deref()
            .map(|path| format!("{path}:"))
            .unwrap_or_default(),
        expected
            .kind
            .as_deref()
            .map(|kind| format!("{kind}:"))
            .unwrap_or_default(),
        expected.name,
    )
}

fn thinindex_sort_key(record: &IndexRecord) -> (String, usize, usize, String, String) {
    (
        normalize_path(&record.path),
        record.line,
        record.col,
        record.kind.clone(),
        record.name.clone(),
    )
}

fn comparator_sort_key(record: &ComparatorRecord) -> (String, usize, usize, String, String) {
    (
        normalize_path(&record.path),
        record.line,
        record.column.unwrap_or(0),
        record.kind.clone(),
        record.name.clone(),
    )
}

fn comparator_language(record: &ComparatorRecord) -> String {
    record
        .language
        .as_deref()
        .map(normalize_language)
        .unwrap_or_else(|| {
            extension(&record.path)
                .map(|extension| extension.trim_start_matches('.').to_string())
                .unwrap_or_else(|| "unknown".to_string())
        })
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

fn is_malformed_thinindex_record(record: &IndexRecord) -> bool {
    record.path.is_empty()
        || record.line == 0
        || record.col == 0
        || record.kind.is_empty()
        || record.name.is_empty()
        || record.source.is_empty()
}

fn is_malformed_comparator_record(record: &ComparatorRecord) -> bool {
    record.path.is_empty() || record.line == 0 || record.kind.is_empty() || record.name.is_empty()
}

fn repeated_values(values: BTreeMap<String, usize>) -> Vec<String> {
    let mut repeated = Vec::new();
    for (value, count) in values {
        for _ in 0..count {
            repeated.push(value.clone());
        }
    }
    repeated
}

fn sanitize_report_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();

    if sanitized.is_empty() {
        "quality-report".to_string()
    } else {
        sanitized
    }
}
