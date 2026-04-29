use std::{collections::BTreeSet, path::Path};

use anyhow::Result;

use crate::{
    model::ReferenceRecord,
    search::{SearchOptions, SearchResult, search},
    store::load_refs,
};

const DEFAULT_PRIMARY_LIMIT: usize = 3;
const DEFAULT_REFS_LIMIT: usize = 20;
const DEFAULT_PACK_LIMIT: usize = 10;
const PACK_TEST_LIMIT: usize = 3;
const PACK_CALLER_LIMIT: usize = 3;
const PACK_DOC_LIMIT: usize = 2;

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

fn primary_matches(root: &Path, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
    let mut primary_options = options.clone();
    primary_options.limit = DEFAULT_PRIMARY_LIMIT;
    search(root, query, &primary_options)
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
