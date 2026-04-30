use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::Result;

use crate::{
    model::{DependencyEdge, IndexRecord, ReferenceRecord},
    search::{SearchOptions, SearchResult, search},
    store::{load_dependencies, load_records, load_refs},
};

const DEFAULT_PRIMARY_LIMIT: usize = 3;
const DEFAULT_REFS_LIMIT: usize = 20;
const DEFAULT_PACK_LIMIT: usize = 10;
const DEFAULT_IMPACT_LIMIT: usize = 15;
const PACK_TEST_LIMIT: usize = 3;
const PACK_CALLER_LIMIT: usize = 3;
const PACK_DOC_LIMIT: usize = 2;
const IMPACT_TEST_LIMIT: usize = 5;
const IMPACT_CALLER_LIMIT: usize = 5;
const IMPACT_DEPENDENT_LIMIT: usize = 5;
const IMPACT_DOC_LIMIT: usize = 3;
const IMPACT_CONFIG_LIMIT: usize = 5;
const IMPACT_UNKNOWN_LIMIT: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextOutput {
    pub text: String,
    pub result_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RefRow {
    reference: ReferenceRecord,
    rank: RefRank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RefRank {
    Production,
    Import,
    Test,
    Docs,
    Ui,
    Fixture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackGroup {
    Tests,
    Callers,
    Docs,
    Related,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ImpactGroup {
    References,
    Dependents,
    Tests,
    Docs,
    Config,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImpactRow {
    path: String,
    line: usize,
    col: usize,
    kind: String,
    target: String,
    confidence: &'static str,
    reason: String,
    priority: usize,
}

pub fn render_refs_command(
    root: &Path,
    query: &str,
    options: &SearchOptions,
) -> Result<ContextOutput> {
    let primary = primary_matches(root, query, options)?;
    if primary.is_empty() {
        return Ok(ContextOutput {
            text: String::new(),
            result_count: 0,
        });
    }

    let refs_limit = command_limit(options, DEFAULT_REFS_LIMIT);
    let refs = matching_refs(root, query, &primary, refs_limit)?;
    let mut out = String::new();

    out.push_str("Primary:\n");
    for result in &primary {
        out.push_str(&format!("- {}\n", primary_line(result)));
    }

    out.push('\n');
    out.push_str("References:\n");
    if refs.is_empty() {
        out.push_str(&format!("no references found for {query}\n"));
    } else {
        for row in &refs {
            out.push_str(&format!("- {}\n", ref_line(&row.reference)));
            out.push_str(&format!("  reason: {}\n", row.reference.evidence));
        }
    }

    Ok(ContextOutput {
        text: out,
        result_count: primary.len() + refs.len(),
    })
}

pub fn render_pack_command(
    root: &Path,
    query: &str,
    options: &SearchOptions,
) -> Result<ContextOutput> {
    let primary = primary_matches(root, query, options)?;
    if primary.is_empty() {
        return Ok(ContextOutput {
            text: String::new(),
            result_count: 0,
        });
    }

    let total_limit = command_limit(options, DEFAULT_PACK_LIMIT);
    let refs = matching_refs(root, query, &primary, DEFAULT_REFS_LIMIT)?;
    let mut out = String::new();
    let mut count = primary.len();

    out.push_str("Primary:\n");
    for result in &primary {
        out.push_str(&format!("- {}\n", primary_line(result)));
        out.push_str("  reason: exact symbol match\n");
    }

    let remaining = total_limit.saturating_sub(primary.len());
    let mut used = BTreeSet::new();
    for result in &primary {
        used.insert(result.record.path.clone());
    }

    let tests = pack_group_rows(
        &refs,
        PackGroup::Tests,
        PACK_TEST_LIMIT,
        remaining,
        &mut used,
    );
    count += tests.len();
    append_pack_group(&mut out, "Tests", &tests, "test_reference");

    let remaining = total_limit.saturating_sub(count);
    let callers = pack_group_rows(
        &refs,
        PackGroup::Callers,
        PACK_CALLER_LIMIT,
        remaining,
        &mut used,
    );
    count += callers.len();
    append_pack_group(&mut out, "Callers/importers", &callers, "import reference");

    let remaining = total_limit.saturating_sub(count);
    let docs = pack_group_rows(&refs, PackGroup::Docs, PACK_DOC_LIMIT, remaining, &mut used);
    count += docs.len();
    append_pack_group(&mut out, "Docs", &docs, "markdown link/reference");

    let remaining = total_limit.saturating_sub(count);
    let related = pack_group_rows(&refs, PackGroup::Related, remaining, remaining, &mut used);
    count += related.len();
    append_pack_group(
        &mut out,
        "Related UI/style/config",
        &related,
        "css/html usage",
    );

    if refs.is_empty() {
        out.push('\n');
        out.push_str(&format!("no references found for {query}\n"));
    }

    Ok(ContextOutput {
        text: out,
        result_count: count,
    })
}

pub fn render_impact_command(
    root: &Path,
    query: &str,
    options: &SearchOptions,
) -> Result<ContextOutput> {
    let primary = direct_primary_matches(primary_matches(root, query, options)?);
    if primary.is_empty() {
        return Ok(ContextOutput {
            text: String::new(),
            result_count: 0,
        });
    }

    let total_limit = command_limit(options, DEFAULT_IMPACT_LIMIT);
    let refs = matching_refs(root, query, &primary, usize::MAX)?;
    let records = load_records(root)?;
    let dependencies = load_dependencies(root)?;
    let primary_paths = primary_paths(&primary);
    let primary_names = primary_names(query, &primary);
    let mut groups = build_impact_groups(
        &refs,
        &dependencies,
        &records,
        &primary_paths,
        &primary_names,
    );
    let mut out = String::new();
    let mut non_primary_count = 0usize;
    let mut used_paths = primary_paths.clone();

    out.push_str("Direct definitions:\n");
    for result in &primary {
        out.push_str(&format!("- {}\n", primary_line(result)));
        out.push_str("  reason: exact symbol match\n");
        out.push_str("  confidence: direct\n");
    }

    let references = take_impact_rows(
        groups.remove(&ImpactGroup::References).unwrap_or_default(),
        IMPACT_CALLER_LIMIT,
        total_limit.saturating_sub(non_primary_count),
        &mut used_paths,
    );
    non_primary_count += references.len();
    append_impact_rows(&mut out, "References", &references);

    let dependents = take_impact_rows(
        groups.remove(&ImpactGroup::Dependents).unwrap_or_default(),
        IMPACT_DEPENDENT_LIMIT,
        total_limit.saturating_sub(non_primary_count),
        &mut used_paths,
    );
    non_primary_count += dependents.len();
    append_impact_rows(&mut out, "Dependent files", &dependents);

    let tests = take_impact_rows(
        groups.remove(&ImpactGroup::Tests).unwrap_or_default(),
        IMPACT_TEST_LIMIT,
        total_limit.saturating_sub(non_primary_count),
        &mut used_paths,
    );
    non_primary_count += tests.len();
    append_impact_rows(&mut out, "Likely tests", &tests);

    let docs = take_impact_rows(
        groups.remove(&ImpactGroup::Docs).unwrap_or_default(),
        IMPACT_DOC_LIMIT,
        total_limit.saturating_sub(non_primary_count),
        &mut used_paths,
    );
    non_primary_count += docs.len();
    append_impact_rows(&mut out, "Related docs", &docs);

    let config = take_impact_rows(
        groups.remove(&ImpactGroup::Config).unwrap_or_default(),
        IMPACT_CONFIG_LIMIT,
        total_limit.saturating_sub(non_primary_count),
        &mut used_paths,
    );
    non_primary_count += config.len();
    append_impact_rows(&mut out, "Build/config files", &config);

    let unknown = take_impact_rows(
        groups.remove(&ImpactGroup::Unknown).unwrap_or_default(),
        IMPACT_UNKNOWN_LIMIT,
        total_limit.saturating_sub(non_primary_count),
        &mut used_paths,
    );
    non_primary_count += unknown.len();
    append_impact_rows(&mut out, "Unresolved/unknown areas", &unknown);

    if non_primary_count == 0 {
        out.push('\n');
        out.push_str(&format!("no impact references found for {query}\n"));
    }

    Ok(ContextOutput {
        text: out,
        result_count: primary.len() + non_primary_count,
    })
}

fn primary_matches(root: &Path, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
    let mut primary_options = options.clone();
    primary_options.limit = DEFAULT_PRIMARY_LIMIT;
    search(root, query, &primary_options)
}

fn direct_primary_matches(primary: Vec<SearchResult>) -> Vec<SearchResult> {
    let filtered: Vec<SearchResult> = primary
        .iter()
        .filter(|result| is_direct_definition_record(&result.record))
        .cloned()
        .collect();

    if filtered.is_empty() {
        primary
    } else {
        filtered
    }
}

fn is_direct_definition_record(record: &IndexRecord) -> bool {
    matches!(
        record.kind.as_str(),
        "class"
            | "constant"
            | "css_class"
            | "css_id"
            | "css_variable"
            | "data_attribute"
            | "enum"
            | "function"
            | "html_class"
            | "html_id"
            | "html_tag"
            | "interface"
            | "key"
            | "keyframes"
            | "method"
            | "module"
            | "section"
            | "struct"
            | "table"
            | "trait"
            | "type"
            | "variable"
    )
}

fn matching_refs(
    root: &Path,
    query: &str,
    primary: &[SearchResult],
    limit: usize,
) -> Result<Vec<RefRow>> {
    let names = primary_names(query, primary);
    let primary_locations = primary_locations(primary);
    let mut refs: Vec<RefRow> = load_refs(root)?
        .into_iter()
        .filter(|reference| names.contains(&reference.to_name))
        .filter(|reference| {
            reference.ref_kind != "text_reference"
                || !primary_locations.contains(&(
                    reference.from_path.clone(),
                    reference.from_line,
                    reference.to_name.clone(),
                ))
        })
        .map(|reference| {
            let rank = ref_rank(&reference);
            RefRow { reference, rank }
        })
        .collect();

    refs.sort_by_key(ref_row_key);
    refs.truncate(limit);
    Ok(refs)
}

fn primary_locations(primary: &[SearchResult]) -> BTreeSet<(String, usize, String)> {
    primary
        .iter()
        .map(|result| {
            (
                result.record.path.clone(),
                result.record.line,
                result.record.name.clone(),
            )
        })
        .collect()
}

fn primary_paths(primary: &[SearchResult]) -> BTreeSet<String> {
    primary
        .iter()
        .map(|result| result.record.path.clone())
        .collect()
}

fn primary_names(query: &str, primary: &[SearchResult]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    names.insert(query.to_string());

    for result in primary {
        names.insert(result.record.name.clone());
    }

    names
}

fn primary_line(result: &SearchResult) -> String {
    let record = &result.record;
    format!(
        "{}:{} {} {}",
        record.path, record.line, record.kind, record.name
    )
}

fn ref_line(reference: &ReferenceRecord) -> String {
    format!(
        "{}:{} {} {}",
        reference.from_path, reference.from_line, reference.ref_kind, reference.to_name
    )
}

fn ref_row_key(row: &RefRow) -> (RefRank, usize, String, usize, usize, String, String) {
    (
        row.rank,
        path_penalty(&row.reference.from_path),
        row.reference.from_path.clone(),
        row.reference.from_line,
        row.reference.from_col,
        row.reference.ref_kind.clone(),
        row.reference.to_name.clone(),
    )
}

fn ref_rank(reference: &ReferenceRecord) -> RefRank {
    if is_fixture_path(&reference.from_path) {
        return RefRank::Fixture;
    }

    match reference.ref_kind.as_str() {
        "test_reference" => RefRank::Test,
        "import" => RefRank::Import,
        "markdown_link" => RefRank::Docs,
        "css_usage" | "html_usage" => RefRank::Ui,
        _ if is_test_path(&reference.from_path) => RefRank::Test,
        _ => RefRank::Production,
    }
}

fn pack_group_rows(
    refs: &[RefRow],
    group: PackGroup,
    group_limit: usize,
    total_remaining: usize,
    used: &mut BTreeSet<String>,
) -> Vec<ReferenceRecord> {
    if total_remaining == 0 {
        return Vec::new();
    }

    let mut rows = Vec::new();
    let limit = group_limit.min(total_remaining);

    for row in refs.iter().filter(|row| pack_group(row) == group) {
        if rows.len() >= limit {
            break;
        }

        if used.insert(row.reference.from_path.clone()) {
            rows.push(row.reference.clone());
        }
    }

    rows
}

fn pack_group(row: &RefRow) -> PackGroup {
    match row.reference.ref_kind.as_str() {
        "test_reference" => PackGroup::Tests,
        "markdown_link" => PackGroup::Docs,
        "css_usage" | "html_usage" => PackGroup::Related,
        _ if is_test_path(&row.reference.from_path) => PackGroup::Tests,
        _ if is_doc_path(&row.reference.from_path) => PackGroup::Docs,
        _ if is_ui_style_config_path(&row.reference.from_path) => PackGroup::Related,
        _ => PackGroup::Callers,
    }
}

fn append_pack_group(
    out: &mut String,
    heading: &str,
    refs: &[ReferenceRecord],
    default_reason: &str,
) {
    out.push('\n');
    out.push_str(heading);
    out.push_str(":\n");

    if refs.is_empty() {
        out.push_str("- none\n");
        return;
    }

    for reference in refs {
        out.push_str(&format!("- {}\n", ref_line(reference)));
        let reason = if reference.ref_kind == "test_reference" {
            format!("test_reference to {}", reference.to_name)
        } else if reference.ref_kind == "import" {
            "import reference".to_string()
        } else if reference.ref_kind == "markdown_link" {
            "markdown link/reference".to_string()
        } else if matches!(reference.ref_kind.as_str(), "css_usage" | "html_usage") {
            "css/html usage".to_string()
        } else if reference.ref_kind == "text_reference" {
            format!("text_reference to {}", reference.to_name)
        } else {
            default_reason.to_string()
        };
        out.push_str(&format!("  reason: {reason}\n"));
    }
}

fn build_impact_groups(
    refs: &[RefRow],
    dependencies: &[DependencyEdge],
    records: &[IndexRecord],
    primary_paths: &BTreeSet<String>,
    primary_names: &BTreeSet<String>,
) -> BTreeMap<ImpactGroup, Vec<ImpactRow>> {
    let mut groups: BTreeMap<ImpactGroup, Vec<ImpactRow>> = BTreeMap::new();

    for row in refs {
        let group = impact_group_for_ref(&row.reference);
        groups.entry(group).or_default().push(ImpactRow {
            path: row.reference.from_path.clone(),
            line: row.reference.from_line,
            col: row.reference.from_col,
            kind: row.reference.ref_kind.clone(),
            target: row.reference.to_name.clone(),
            confidence: impact_confidence_for_ref(&row.reference, group),
            reason: impact_reason_for_ref(&row.reference, group),
            priority: impact_ref_priority(&row.reference, group),
        });
    }

    add_dependency_impact_rows(&mut groups, dependencies, primary_paths);
    add_test_mapping_rows(&mut groups, records, primary_paths, primary_names);

    for rows in groups.values_mut() {
        rows.sort_by_key(impact_row_key);
    }

    groups
}

fn add_dependency_impact_rows(
    groups: &mut BTreeMap<ImpactGroup, Vec<ImpactRow>>,
    dependencies: &[DependencyEdge],
    primary_paths: &BTreeSet<String>,
) {
    let dependency_sources: BTreeSet<String> = dependencies
        .iter()
        .filter(|dependency| {
            dependency
                .target_path
                .as_ref()
                .is_some_and(|target| primary_paths.contains(target))
        })
        .map(|dependency| dependency.from_path.clone())
        .collect();

    for dependency in dependencies {
        if dependency
            .target_path
            .as_ref()
            .is_some_and(|target| primary_paths.contains(target))
        {
            let target = dependency.target_path.as_deref().unwrap_or_default();
            let group = if is_test_path(&dependency.from_path) {
                ImpactGroup::Tests
            } else if is_fixture_path(&dependency.from_path) {
                ImpactGroup::Unknown
            } else {
                ImpactGroup::Dependents
            };
            let confidence = match group {
                ImpactGroup::Tests => "test-related",
                ImpactGroup::Unknown => "heuristic",
                _ => "dependency",
            };
            let reason = match group {
                ImpactGroup::Tests => format!("test dependency imports {target}"),
                ImpactGroup::Unknown => format!("fixture/example dependency imports {target}"),
                _ => format!("dependency imports {target}"),
            };
            let priority = if group == ImpactGroup::Unknown { 20 } else { 0 };
            groups.entry(group).or_default().push(ImpactRow {
                path: dependency.from_path.clone(),
                line: dependency.from_line,
                col: dependency.from_col,
                kind: dependency.dependency_kind.clone(),
                target: target.to_string(),
                confidence,
                reason,
                priority,
            });
        }

        if primary_paths.contains(&dependency.from_path)
            && dependency.target_path.is_none()
            && dependency.unresolved_reason.is_some()
        {
            groups
                .entry(ImpactGroup::Unknown)
                .or_default()
                .push(ImpactRow {
                    path: dependency.from_path.clone(),
                    line: dependency.from_line,
                    col: dependency.from_col,
                    kind: dependency.dependency_kind.clone(),
                    target: dependency.import_path.clone(),
                    confidence: "heuristic",
                    reason: format!(
                        "unresolved dependency: {}",
                        dependency.unresolved_reason.as_deref().unwrap_or("unknown")
                    ),
                    priority: 10,
                });
        }
    }

    for dependency in dependencies {
        if dependency_sources.contains(&dependency.from_path)
            && dependency.target_path.is_none()
            && dependency.unresolved_reason.is_some()
        {
            groups
                .entry(ImpactGroup::Unknown)
                .or_default()
                .push(ImpactRow {
                    path: dependency.from_path.clone(),
                    line: dependency.from_line,
                    col: dependency.from_col,
                    kind: dependency.dependency_kind.clone(),
                    target: dependency.import_path.clone(),
                    confidence: "heuristic",
                    reason: format!(
                        "dependent file has unresolved dependency: {}",
                        dependency.unresolved_reason.as_deref().unwrap_or("unknown")
                    ),
                    priority: 20,
                });
        }
    }
}

fn add_test_mapping_rows(
    groups: &mut BTreeMap<ImpactGroup, Vec<ImpactRow>>,
    records: &[IndexRecord],
    primary_paths: &BTreeSet<String>,
    primary_names: &BTreeSet<String>,
) {
    let primary_stems: BTreeSet<String> = primary_paths
        .iter()
        .filter_map(|path| file_stem(path))
        .map(normalize_test_name)
        .collect();
    let primary_symbol_stems: BTreeSet<String> = primary_names
        .iter()
        .map(|name| normalize_test_name(name))
        .collect();

    let mut test_paths = BTreeSet::new();
    for record in records {
        if !is_test_path(&record.path) || !test_paths.insert(record.path.clone()) {
            continue;
        }

        let normalized_path = normalize_test_name(&record.path);
        let matches_file = primary_stems
            .iter()
            .any(|stem| !stem.is_empty() && normalized_path.contains(stem));
        let matches_symbol = primary_symbol_stems
            .iter()
            .any(|stem| !stem.is_empty() && normalized_path.contains(stem));

        if matches_file || matches_symbol {
            groups
                .entry(ImpactGroup::Tests)
                .or_default()
                .push(ImpactRow {
                    path: record.path.clone(),
                    line: record.line,
                    col: record.col,
                    kind: "test_mapping".to_string(),
                    target: record.name.clone(),
                    confidence: "test-related",
                    reason: "same-name test convention".to_string(),
                    priority: 5,
                });
        }
    }
}

fn take_impact_rows(
    mut rows: Vec<ImpactRow>,
    group_limit: usize,
    total_remaining: usize,
    used_paths: &mut BTreeSet<String>,
) -> Vec<ImpactRow> {
    if total_remaining == 0 {
        return Vec::new();
    }

    rows.sort_by_key(impact_row_key);
    let limit = group_limit.min(total_remaining);
    let mut selected = Vec::new();

    for row in rows {
        if selected.len() >= limit {
            break;
        }

        if used_paths.insert(row.path.clone()) {
            selected.push(row);
        }
    }

    selected
}

fn impact_group_for_ref(reference: &ReferenceRecord) -> ImpactGroup {
    let path = &reference.from_path;

    if reference.ref_kind == "test_reference" || is_test_path(path) {
        return ImpactGroup::Tests;
    }

    if reference.ref_kind == "module_dependency" {
        return if reference.confidence == "unresolved" {
            ImpactGroup::Unknown
        } else {
            ImpactGroup::Dependents
        };
    }

    if reference.ref_kind == "markdown_link" || is_doc_path(path) {
        return ImpactGroup::Docs;
    }

    if is_config_route_schema_path(path) {
        return ImpactGroup::Config;
    }

    if is_fixture_path(path) {
        return ImpactGroup::Unknown;
    }

    ImpactGroup::References
}

fn impact_confidence_for_ref(reference: &ReferenceRecord, group: ImpactGroup) -> &'static str {
    if reference.confidence == "semantic" {
        return "semantic";
    }

    match group {
        ImpactGroup::Tests => "test-related",
        ImpactGroup::Dependents => "dependency",
        ImpactGroup::Unknown => "heuristic",
        ImpactGroup::References if reference.confidence == "exact_local" => "direct",
        _ => "heuristic",
    }
}

fn impact_reason_for_ref(reference: &ReferenceRecord, group: ImpactGroup) -> String {
    match group {
        ImpactGroup::Tests => format!("test references {}", reference.to_name),
        ImpactGroup::Dependents => format!("dependency reference to {}", reference.to_name),
        ImpactGroup::Docs => format!("docs reference {}", reference.to_name),
        ImpactGroup::Config => format!("build/config reference {}", reference.to_name),
        ImpactGroup::Unknown if reference.confidence == "unresolved" => {
            format!("unresolved dependency {}", reference.to_name)
        }
        ImpactGroup::Unknown => format!("unknown impact from {}", reference.to_name),
        ImpactGroup::References => match reference.ref_kind.as_str() {
            "import" => format!("imports {}", reference.to_name),
            "call" => format!("calls {}", reference.to_name),
            "export" => format!("exports {}", reference.to_name),
            "type_reference" => format!("type reference to {}", reference.to_name),
            "text_reference" => format!("references {}", reference.to_name),
            other => format!("{other} to {}", reference.to_name),
        },
    }
}

fn impact_ref_priority(reference: &ReferenceRecord, group: ImpactGroup) -> usize {
    match group {
        ImpactGroup::Tests => match reference.ref_kind.as_str() {
            "test_reference" => 0,
            "call" | "import" => 1,
            _ => 2,
        },
        ImpactGroup::References => match reference.ref_kind.as_str() {
            "import" => 0,
            "call" => 1,
            "type_reference" => 2,
            "text_reference" => 5,
            _ => 8,
        },
        ImpactGroup::Dependents => 0,
        ImpactGroup::Docs => {
            if reference.ref_kind == "markdown_link" {
                0
            } else {
                2
            }
        }
        ImpactGroup::Config => 0,
        ImpactGroup::Unknown => {
            if is_fixture_path(&reference.from_path) {
                20
            } else {
                10
            }
        }
    }
}

fn impact_row_key(row: &ImpactRow) -> (usize, usize, String, usize, usize, String, String) {
    (
        row.priority,
        path_penalty(&row.path),
        row.path.clone(),
        row.line,
        row.col,
        row.kind.clone(),
        row.target.clone(),
    )
}

fn append_impact_rows(out: &mut String, heading: &str, rows: &[ImpactRow]) {
    out.push('\n');
    out.push_str(heading);
    out.push_str(":\n");

    if rows.is_empty() {
        out.push_str("- none\n");
        return;
    }

    for row in rows {
        out.push_str(&format!(
            "- {}:{} {} {}\n",
            row.path, row.line, row.kind, row.target
        ));
        out.push_str(&format!("  reason: {}\n", row.reason));
        out.push_str(&format!("  confidence: {}\n", row.confidence));
    }
}

fn command_limit(options: &SearchOptions, default_limit: usize) -> usize {
    if options.limit == 0 {
        default_limit
    } else {
        options.limit
    }
}

fn path_penalty(path: &str) -> usize {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();

    if is_fixture_path(&normalized) {
        return 25;
    }

    if is_test_path(&normalized) {
        return 10;
    }

    if is_doc_path(&normalized) {
        return 15;
    }

    0
}

fn is_doc_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();

    normalized.contains("/docs/")
        || normalized.starts_with("docs/")
        || normalized.ends_with(".md")
        || normalized.ends_with(".mdx")
}

fn is_ui_style_config_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();

    normalized.contains("/frontend/")
        || normalized.starts_with("frontend/")
        || normalized.contains("/styles/")
        || normalized.starts_with("styles/")
        || normalized.ends_with(".css")
        || normalized.ends_with(".html")
        || normalized.ends_with(".jsx")
        || normalized.ends_with(".tsx")
        || normalized.ends_with(".json")
        || normalized.ends_with(".toml")
        || normalized.ends_with(".yaml")
        || normalized.ends_with(".yml")
}

fn is_config_route_schema_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    let filename = normalized.rsplit('/').next().unwrap_or(&normalized);

    normalized.contains("/config/")
        || normalized.starts_with("config/")
        || normalized.contains("/configs/")
        || normalized.starts_with("configs/")
        || normalized.contains("/routes/")
        || normalized.starts_with("routes/")
        || normalized.contains("/schemas/")
        || normalized.starts_with("schemas/")
        || filename.contains("config")
        || filename.contains("settings")
        || filename.contains("route")
        || filename.contains("router")
        || filename.contains("schema")
        || normalized.ends_with(".json")
        || normalized.ends_with(".toml")
        || normalized.ends_with(".yaml")
        || normalized.ends_with(".yml")
}

fn file_stem(path: &str) -> Option<&str> {
    let file_name = path.rsplit('/').next().unwrap_or(path);
    file_name
        .rsplit_once('.')
        .map_or(Some(file_name), |(stem, _)| {
            if stem.is_empty() { None } else { Some(stem) }
        })
}

fn normalize_test_name(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

fn is_fixture_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();

    normalized.contains("/fixtures/")
        || normalized.starts_with("fixtures/")
        || normalized.contains("/fixture/")
        || normalized.starts_with("fixture/")
        || normalized.contains("/examples/")
        || normalized.starts_with("examples/")
        || normalized.contains("/example/")
        || normalized.starts_with("example/")
}

fn is_test_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    let filename = normalized.rsplit('/').next().unwrap_or(&normalized);

    normalized.contains("/tests/")
        || normalized.starts_with("tests/")
        || normalized.contains("/test/")
        || normalized.starts_with("test/")
        || normalized.contains("/__tests__/")
        || normalized.starts_with("__tests__/")
        || filename.contains("_test")
        || filename.contains(".test.")
        || filename.contains(".spec.")
}
