use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use crate::quality::{
    manifest::quality_report_dir,
    report::{ComparatorOnlySymbol, QualityReport, ThinindexOnlySymbol},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TriageState {
    Open,
    AcceptedExpectedSymbol,
    FixtureNeeded,
    ComparatorFalsePositive,
    UnsupportedSyntax,
    LowValueNoise,
    Fixed,
}

impl TriageState {
    pub const ALL: [Self; 7] = [
        Self::Open,
        Self::AcceptedExpectedSymbol,
        Self::FixtureNeeded,
        Self::ComparatorFalsePositive,
        Self::UnsupportedSyntax,
        Self::LowValueNoise,
        Self::Fixed,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::AcceptedExpectedSymbol => "accepted_expected_symbol",
            Self::FixtureNeeded => "fixture_needed",
            Self::ComparatorFalsePositive => "comparator_false_positive",
            Self::UnsupportedSyntax => "unsupported_syntax",
            Self::LowValueNoise => "low_value_noise",
            Self::Fixed => "fixed",
        }
    }

    pub fn from_name(value: &str) -> Result<Self> {
        match value {
            "open" => Ok(Self::Open),
            "accepted_expected_symbol" => Ok(Self::AcceptedExpectedSymbol),
            "fixture_needed" => Ok(Self::FixtureNeeded),
            "comparator_false_positive" => Ok(Self::ComparatorFalsePositive),
            "unsupported_syntax" => Ok(Self::UnsupportedSyntax),
            "low_value_noise" => Ok(Self::LowValueNoise),
            "fixed" => Ok(Self::Fixed),
            other => bail!("unknown triage state `{other}`"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TriageSymbolSource {
    ComparatorOnly,
    ThinindexOnly,
}

impl TriageSymbolSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ComparatorOnly => "comparator-only",
            Self::ThinindexOnly => "thinindex-only",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriageSymbol {
    pub id: String,
    pub source: TriageSymbolSource,
    pub state: TriageState,
    pub language: String,
    pub kind: String,
    pub path: String,
    pub line: usize,
    pub name: String,
    pub detail: String,
}

impl TriageSymbol {
    pub fn transition_to(&self, state: TriageState) -> Self {
        let mut next = self.clone();
        next.state = state;
        next
    }

    pub fn promotion_action(&self) -> &'static str {
        match self.state {
            TriageState::Open => "triage before promoting",
            TriageState::AcceptedExpectedSymbol => {
                "add [[repo.expected_symbol]] or [[repo.expected_symbol_pattern]]"
            }
            TriageState::FixtureNeeded => "add or extend a parser conformance fixture",
            TriageState::ComparatorFalsePositive => "record as accepted comparator false positive",
            TriageState::UnsupportedSyntax => "document unsupported syntax or support-level gap",
            TriageState::LowValueNoise => "leave unpromoted as low-value comparator noise",
            TriageState::Fixed => "rerun quality gates and keep regression coverage",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriageReport {
    pub repo_name: String,
    pub repo_path: String,
    pub comparator_name: String,
    pub skipped: bool,
    pub items: Vec<TriageSymbol>,
}

pub fn triage_report_from_quality_report(report: &QualityReport) -> TriageReport {
    let mut items = Vec::new();

    for symbol in &report.comparator_only {
        items.push(triage_symbol_from_comparator_only(items.len() + 1, symbol));
    }

    for symbol in &report.thinindex_only {
        items.push(triage_symbol_from_thinindex_only(items.len() + 1, symbol));
    }

    items.sort_by_key(triage_symbol_sort_key);
    for (index, item) in items.iter_mut().enumerate() {
        item.id = format!("TRIAGE-{:04}", index + 1);
    }

    TriageReport {
        repo_name: report.repo_name.clone(),
        repo_path: report.repo_path.clone(),
        comparator_name: report.comparator_name.clone(),
        skipped: report.skipped,
        items,
    }
}

pub fn open_triage_items(report: &TriageReport) -> Vec<&TriageSymbol> {
    report
        .items
        .iter()
        .filter(|item| item.state == TriageState::Open)
        .collect()
}

pub fn assert_triage_has_no_open_items(report: &TriageReport) -> Result<()> {
    let open = open_triage_items(report);
    if open.is_empty() {
        return Ok(());
    }

    bail!(
        "strict comparator triage failed for {}: {} open item(s)\n{}",
        report.repo_name,
        open.len(),
        render_triage_report(report)
    )
}

pub fn render_triage_report(report: &TriageReport) -> String {
    let mut out = String::new();
    out.push_str("# Comparator Triage\n\n");
    out.push_str(&format!("- repo: {}\n", report.repo_name));
    out.push_str(&format!("- path: {}\n", report.repo_path));
    out.push_str(&format!("- comparator: {}\n", report.comparator_name));
    out.push_str(&format!("- skipped: {}\n", yes_no(report.skipped)));
    out.push_str(&format!("- items: {}\n", report.items.len()));
    out.push_str(&format!("- open: {}\n\n", open_triage_items(report).len()));

    out.push_str("## States\n\n");
    for state in TriageState::ALL {
        out.push_str(&format!("- {}\n", state.as_str()));
    }

    out.push_str("\n## Comparator-Only Groups\n\n");
    render_groups(&mut out, report, TriageSymbolSource::ComparatorOnly);

    out.push_str("\n## Thinindex-Only Groups\n\n");
    render_groups(&mut out, report, TriageSymbolSource::ThinindexOnly);

    out.push_str("\n## Items\n\n");
    if report.items.is_empty() {
        out.push_str("- none\n");
    } else {
        for item in &report.items {
            render_item(&mut out, item);
        }
    }

    out
}

pub fn write_triage_report(repo_root: &Path, report: &TriageReport) -> Result<PathBuf> {
    let report_dir = quality_report_dir(repo_root);
    fs::create_dir_all(&report_dir).with_context(|| {
        format!(
            "failed to create quality report dir {}",
            report_dir.display()
        )
    })?;
    let path = report_dir.join("COMPARATOR_TRIAGE.md");
    fs::write(&path, render_triage_report(report))
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(path)
}

fn triage_symbol_from_comparator_only(index: usize, symbol: &ComparatorOnlySymbol) -> TriageSymbol {
    TriageSymbol {
        id: format!("TRIAGE-{index:04}"),
        source: TriageSymbolSource::ComparatorOnly,
        state: TriageState::Open,
        language: normalize_field(&symbol.language),
        kind: normalize_field(&symbol.kind),
        path: normalize_path(&symbol.path),
        line: symbol.line,
        name: symbol.name.clone(),
        detail: format!(
            "Comparator reported {}:{} {} {}",
            symbol.path, symbol.line, symbol.kind, symbol.name
        ),
    }
}

fn triage_symbol_from_thinindex_only(index: usize, symbol: &ThinindexOnlySymbol) -> TriageSymbol {
    TriageSymbol {
        id: format!("TRIAGE-{index:04}"),
        source: TriageSymbolSource::ThinindexOnly,
        state: TriageState::Open,
        language: normalize_field(&symbol.language),
        kind: normalize_field(&symbol.kind),
        path: normalize_path(&symbol.path),
        line: symbol.line,
        name: symbol.name.clone(),
        detail: format!(
            "Thinindex-only symbol {}:{} {} {}",
            symbol.path, symbol.line, symbol.kind, symbol.name
        ),
    }
}

fn render_groups(out: &mut String, report: &TriageReport, source: TriageSymbolSource) {
    let mut grouped: BTreeMap<(String, String, String), Vec<&TriageSymbol>> = BTreeMap::new();
    for item in report.items.iter().filter(|item| item.source == source) {
        grouped
            .entry((item.language.clone(), item.kind.clone(), item.path.clone()))
            .or_default()
            .push(item);
    }

    if grouped.is_empty() {
        out.push_str("- none\n");
        return;
    }

    for ((language, kind, path), mut items) in grouped {
        items.sort_by_key(|item| (item.line, item.name.as_str(), item.id.as_str()));
        let ids = items
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "- language={} kind={} path={} count={} items={}\n",
            language,
            kind,
            path,
            items.len(),
            ids
        ));
    }
}

fn render_item(out: &mut String, item: &TriageSymbol) {
    out.push_str(&format!("- id: {}\n", item.id));
    out.push_str(&format!("  source: {}\n", item.source.as_str()));
    out.push_str(&format!("  state: {}\n", item.state.as_str()));
    out.push_str(&format!("  language: {}\n", item.language));
    out.push_str(&format!("  kind: {}\n", item.kind));
    out.push_str(&format!("  path: {}\n", item.path));
    out.push_str(&format!("  line: {}\n", item.line));
    out.push_str(&format!("  name: {}\n", item.name));
    out.push_str(&format!("  promotion: {}\n", item.promotion_action()));
    out.push_str(&format!("  detail: {}\n", item.detail));
}

fn triage_symbol_sort_key(item: &TriageSymbol) -> (u8, String, String, String, usize, String) {
    (
        match item.source {
            TriageSymbolSource::ComparatorOnly => 0,
            TriageSymbolSource::ThinindexOnly => 1,
        },
        item.language.clone(),
        item.kind.clone(),
        item.path.clone(),
        item.line,
        item.name.clone(),
    )
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn normalize_field(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        "unknown".to_string()
    } else {
        value.to_ascii_lowercase()
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
