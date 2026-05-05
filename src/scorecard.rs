use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::Result;

use crate::{
    agent_instructions::{REPOSITORY_SEARCH_BLOCK, repository_search_block_is_current},
    context::{render_impact_command, render_pack_command, render_refs_command},
    doctor::run_doctor,
    indexer::{build_index, index_is_fresh},
    search::{SearchOptions, search},
    support::{SupportBackend, SupportLevel, support_entries_by_level},
};

const WARM_QUERY_PASS_MS: u128 = 150;
const WARM_QUERY_WARN_MS: u128 = 500;
const DEFAULT_CONTEXT_LIMIT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreStatus {
    Pass,
    Warn,
    Fail,
}

impl ScoreStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreDimension {
    pub name: &'static str,
    pub status: ScoreStatus,
    pub evidence: String,
    pub action: String,
}

impl ScoreDimension {
    pub fn pass(name: &'static str, evidence: impl Into<String>) -> Self {
        Self {
            name,
            status: ScoreStatus::Pass,
            evidence: evidence.into(),
            action: "none".to_string(),
        }
    }

    pub fn warn(
        name: &'static str,
        evidence: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            name,
            status: ScoreStatus::Warn,
            evidence: evidence.into(),
            action: action.into(),
        }
    }

    pub fn fail(
        name: &'static str,
        evidence: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            name,
            status: ScoreStatus::Fail,
            evidence: evidence.into(),
            action: action.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScorecardOptions {
    pub query: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScorecardReport {
    pub root: PathBuf,
    pub query: String,
    pub dimensions: Vec<ScoreDimension>,
}

impl ScorecardReport {
    pub fn pass_count(&self) -> usize {
        self.dimensions
            .iter()
            .filter(|dimension| dimension.status == ScoreStatus::Pass)
            .count()
    }

    pub fn warn_count(&self) -> usize {
        self.dimensions
            .iter()
            .filter(|dimension| dimension.status == ScoreStatus::Warn)
            .count()
    }

    pub fn fail_count(&self) -> usize {
        self.dimensions
            .iter()
            .filter(|dimension| dimension.status == ScoreStatus::Fail)
            .count()
    }
}

pub fn run_scorecard(root: &Path, options: &ScorecardOptions) -> Result<ScorecardReport> {
    let query = options.query.trim();
    let query = if query.is_empty() {
        "build_index"
    } else {
        query
    };
    let search_options = SearchOptions {
        limit: DEFAULT_CONTEXT_LIMIT,
        ..SearchOptions::default()
    };
    let mut dimensions = Vec::new();
    let index_ready = ensure_index_ready_for_scorecard(root, &mut dimensions);

    if index_ready {
        dimensions.extend(score_indexed_workflows(root, query, &search_options)?);
    } else {
        dimensions.extend(index_unavailable_dimensions());
    }

    dimensions.push(score_doctor(root));
    dimensions.push(score_agent_instructions(root));
    dimensions.push(score_instruction_behavior_alignment());
    dimensions.push(score_support_claims());

    Ok(ScorecardReport {
        root: root.to_path_buf(),
        query: query.to_string(),
        dimensions,
    })
}

pub fn render_scorecard(report: &ScorecardReport) -> String {
    let mut out = String::new();

    out.push_str("thinindex value scorecard\n");
    out.push_str(&format!("repo: {}\n", report.root.display()));
    out.push_str(&format!("query: {}\n", report.query));
    out.push_str(&format!(
        "summary: pass {} / warn {} / fail {}\n\n",
        report.pass_count(),
        report.warn_count(),
        report.fail_count()
    ));

    for dimension in &report.dimensions {
        out.push_str(&format!(
            "[{}] {}\n",
            dimension.status.as_str(),
            dimension.name
        ));
        out.push_str(&format!("  evidence: {}\n", dimension.evidence));
        out.push_str(&format!("  action: {}\n", dimension.action));
    }

    out
}

fn ensure_index_ready_for_scorecard(root: &Path, dimensions: &mut Vec<ScoreDimension>) -> bool {
    match index_is_fresh(root) {
        Ok(true) => {
            dimensions.push(ScoreDimension::warn(
                "stale/missing index auto-recovers",
                "index was already fresh, so this scorecard run did not need recovery",
                "run the scorecard on a missing or stale index to observe one-shot recovery",
            ));
            true
        }
        Ok(false) => rebuild_once(root, dimensions, "index was stale before scorecard checks"),
        Err(error) => rebuild_once(
            root,
            dimensions,
            format!("index was missing or unusable before scorecard checks: {error:#}"),
        ),
    }
}

fn rebuild_once(
    root: &Path,
    dimensions: &mut Vec<ScoreDimension>,
    reason: impl Into<String>,
) -> bool {
    let reason = reason.into();

    match build_index(root).and_then(|_| index_is_fresh(root)) {
        Ok(true) => {
            dimensions.push(ScoreDimension::pass(
                "stale/missing index auto-recovers",
                format!("{reason}; build_index ran once and the index is fresh"),
            ));
            true
        }
        Ok(false) => {
            dimensions.push(ScoreDimension::fail(
                "stale/missing index auto-recovers",
                format!("{reason}; index is still stale after one build"),
                "run `build_index` manually and inspect changed files or ignored paths",
            ));
            false
        }
        Err(error) => {
            dimensions.push(ScoreDimension::fail(
                "stale/missing index auto-recovers",
                format!("{reason}; one-shot rebuild failed: {error:#}"),
                "fix the indexing error, then rerun `wi-scorecard`",
            ));
            false
        }
    }
}

fn score_indexed_workflows(
    root: &Path,
    query: &str,
    options: &SearchOptions,
) -> Result<Vec<ScoreDimension>> {
    let mut dimensions = Vec::new();
    let started = Instant::now();
    let results = search(root, query, options)?;
    let warm_ms = started.elapsed().as_millis();

    dimensions.push(score_search_results(&results));
    dimensions.push(score_warm_latency(warm_ms));

    let refs = render_refs_command(root, query, options)?;
    dimensions.push(score_refs(refs.result_count, &refs.text));

    let pack = render_pack_command(root, query, options)?;
    dimensions.push(score_pack(pack.result_count, &pack.text));

    let impact = render_impact_command(root, query, options)?;
    dimensions.push(score_impact(impact.result_count, &impact.text));

    Ok(dimensions)
}

fn score_search_results(results: &[crate::search::SearchResult]) -> ScoreDimension {
    if let Some(first) = results.first() {
        ScoreDimension::pass(
            "wi <term> gives useful file:line results",
            format!(
                "{}:{} {} {}",
                first.record.path, first.record.line, first.record.kind, first.record.name
            ),
        )
    } else {
        ScoreDimension::fail(
            "wi <term> gives useful file:line results",
            "query returned no file:line results",
            "try a more specific symbol or inspect parser support for the repository language",
        )
    }
}

fn score_warm_latency(warm_ms: u128) -> ScoreDimension {
    if warm_ms <= WARM_QUERY_PASS_MS {
        ScoreDimension::pass(
            "warm query latency is acceptable",
            format!("{warm_ms} ms, budget <= {WARM_QUERY_PASS_MS} ms"),
        )
    } else if warm_ms <= WARM_QUERY_WARN_MS {
        ScoreDimension::warn(
            "warm query latency is acceptable",
            format!("{warm_ms} ms, warning budget <= {WARM_QUERY_WARN_MS} ms"),
            "inspect `docs/PERFORMANCE.md` and rerun `build_index --stats`",
        )
    } else {
        ScoreDimension::fail(
            "warm query latency is acceptable",
            format!("{warm_ms} ms exceeds {WARM_QUERY_WARN_MS} ms"),
            "profile search and SQLite load time before optimizing",
        )
    }
}

fn score_refs(result_count: usize, text: &str) -> ScoreDimension {
    if result_count > 1 && text.contains("References:\n- ") {
        ScoreDimension::pass(
            "wi refs <term> gives useful references",
            format!("{result_count} primary/reference rows with evidence"),
        )
    } else if result_count > 0 {
        ScoreDimension::warn(
            "wi refs <term> gives useful references",
            format!("{result_count} primary rows but limited direct reference evidence"),
            "try a non-entry-point symbol or add fixture coverage for richer reference evidence",
        )
    } else {
        ScoreDimension::fail(
            "wi refs <term> gives useful references",
            "refs returned no primary or reference rows",
            "choose a query with a concrete indexed definition",
        )
    }
}

fn score_pack(result_count: usize, text: &str) -> ScoreDimension {
    if result_count > 1 && text.contains("reason:") && text.contains("confidence:") {
        ScoreDimension::pass(
            "wi pack <term> gives a bounded useful read set",
            format!("{result_count} rows with reasons/confidence"),
        )
    } else if result_count > 0 {
        ScoreDimension::warn(
            "wi pack <term> gives a bounded useful read set",
            format!("{result_count} rows, but limited context beyond primary definitions"),
            "try a symbol with dependencies, tests, docs, or references",
        )
    } else {
        ScoreDimension::fail(
            "wi pack <term> gives a bounded useful read set",
            "pack returned no context",
            "choose a query with a concrete indexed definition",
        )
    }
}

fn score_impact(result_count: usize, text: &str) -> ScoreDimension {
    if result_count > 1 && text.contains("reason:") && text.contains("confidence:") {
        ScoreDimension::pass(
            "wi impact <term> gives plausible affected files with reasons",
            format!("{result_count} rows with reasons/confidence"),
        )
    } else if result_count > 0 {
        ScoreDimension::warn(
            "wi impact <term> gives plausible affected files with reasons",
            format!("{result_count} rows, but limited affected-file evidence"),
            "try a non-entry-point symbol with callers, tests, docs, or dependents",
        )
    } else {
        ScoreDimension::fail(
            "wi impact <term> gives plausible affected files with reasons",
            "impact returned no affected-file evidence",
            "choose a query with a concrete indexed definition",
        )
    }
}

fn score_doctor(root: &Path) -> ScoreDimension {
    let report = run_doctor(root);
    if report.has_failures() {
        ScoreDimension::fail(
            "wi doctor gives actionable state",
            "doctor reports one or more failing setup checks",
            "run `wi doctor` and fix the listed `next:` actions",
        )
    } else if report.has_warnings() {
        ScoreDimension::warn(
            "wi doctor gives actionable state",
            "doctor reports warnings but no failures",
            "run `wi doctor` and decide whether the warnings matter for this repo",
        )
    } else {
        ScoreDimension::pass("wi doctor gives actionable state", "doctor overall is ok")
    }
}

fn score_agent_instructions(root: &Path) -> ScoreDimension {
    let agents = root.join("AGENTS.md");
    let cursor = root.join(".cursor/rules/thinindex.mdc");
    let copilot = root.join(".github/copilot-instructions.md");
    let agents_ok = file_has_current_block(&agents, "# AGENTS\n\n");
    let cursor_ok = file_has_current_block(&cursor, "# thinindex\n\n");
    let copilot_ok = file_has_current_block(&copilot, "# GitHub Copilot instructions\n\n");

    if agents_ok && cursor_ok && copilot_ok {
        ScoreDimension::pass(
            "wi-init creates useful agent instructions",
            "AGENTS.md, Cursor rule, and Copilot instructions have the current block",
        )
    } else {
        let missing = [
            ("AGENTS.md", agents_ok),
            (".cursor/rules/thinindex.mdc", cursor_ok),
            (".github/copilot-instructions.md", copilot_ok),
        ]
        .into_iter()
        .filter_map(|(name, ok)| (!ok).then_some(name))
        .collect::<Vec<_>>()
        .join(", ");

        ScoreDimension::fail(
            "wi-init creates useful agent instructions",
            format!("missing or stale instruction surface(s): {missing}"),
            "run `wi-init` and review the generated local instruction files",
        )
    }
}

fn score_instruction_behavior_alignment() -> ScoreDimension {
    let required = [
        "Use `wi <term>` directly",
        "auto-builds or auto-rebuilds a missing/stale index once",
        "Use `wi refs <term>`",
        "Use `wi pack <term>`",
        "Use `wi impact <term>`",
        "Use `wi --help`",
    ];
    let missing = required
        .into_iter()
        .filter(|needle| !REPOSITORY_SEARCH_BLOCK.contains(needle))
        .collect::<Vec<_>>();

    if missing.is_empty() {
        ScoreDimension::pass(
            "generated instructions match actual behavior",
            "canonical block includes direct wi use, auto-rebuild, refs, pack, impact, and help",
        )
    } else {
        ScoreDimension::fail(
            "generated instructions match actual behavior",
            format!("canonical block is missing: {}", missing.join(", ")),
            "update `src/agent_instructions.rs` and generated docs together",
        )
    }
}

fn score_support_claims() -> ScoreDimension {
    let supported = support_entries_by_level(SupportLevel::Supported).len();
    let experimental = support_entries_by_level(SupportLevel::Experimental).len();
    let blocked = support_entries_by_level(SupportLevel::Blocked);
    let extras = support_entries_by_level(SupportLevel::ExtrasBacked).len();
    let blocked_with_backend = blocked
        .iter()
        .filter(|entry| entry.backend != SupportBackend::None)
        .count();

    if supported > 0
        && experimental > 0
        && !blocked.is_empty()
        && extras > 0
        && blocked_with_backend == 0
    {
        ScoreDimension::pass(
            "unsupported/experimental parser support is not overclaimed",
            format!(
                "{supported} supported, {experimental} experimental, {extras} extras-backed, {} blocked",
                blocked.len()
            ),
        )
    } else {
        ScoreDimension::fail(
            "unsupported/experimental parser support is not overclaimed",
            format!(
                "{supported} supported, {experimental} experimental, {extras} extras-backed, {} blocked, {blocked_with_backend} blocked with backend",
                blocked.len()
            ),
            "fix support matrix levels before making parser support claims",
        )
    }
}

fn file_has_current_block(path: &Path, empty_base_prefix: &str) -> bool {
    fs::read_to_string(path)
        .map(|contents| repository_search_block_is_current(&contents, empty_base_prefix))
        .unwrap_or(false)
}

fn index_unavailable_dimensions() -> Vec<ScoreDimension> {
    [
        "wi <term> gives useful file:line results",
        "warm query latency is acceptable",
        "wi refs <term> gives useful references",
        "wi pack <term> gives a bounded useful read set",
        "wi impact <term> gives plausible affected files with reasons",
    ]
    .into_iter()
    .map(|name| {
        ScoreDimension::fail(
            name,
            "index is not usable after one scorecard rebuild attempt",
            "fix indexing first, then rerun `wi-scorecard`",
        )
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{ScoreDimension, ScoreStatus, ScorecardReport, render_scorecard};

    #[test]
    fn render_scorecard_is_deterministic_for_fixed_report() {
        let report = ScorecardReport {
            root: "/repo".into(),
            query: "Needle".to_string(),
            dimensions: vec![
                ScoreDimension::pass("dimension a", "evidence a"),
                ScoreDimension::warn("dimension b", "evidence b", "action b"),
                ScoreDimension::fail("dimension c", "evidence c", "action c"),
            ],
        };

        assert_eq!(render_scorecard(&report), render_scorecard(&report));
        assert!(render_scorecard(&report).contains("summary: pass 1 / warn 1 / fail 1"));
    }

    #[test]
    fn status_strings_are_stable() {
        assert_eq!(ScoreStatus::Pass.as_str(), "pass");
        assert_eq!(ScoreStatus::Warn.as_str(), "warn");
        assert_eq!(ScoreStatus::Fail.as_str(), "fail");
    }
}
