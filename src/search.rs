use std::path::Path;

use anyhow::Result;

use crate::{model::IndexRecord, privacy::redact_sensitive_text, store::load_records};

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub kind: Option<String>,
    pub lang: Option<String>,
    pub path: Option<String>,
    pub source: Option<String>,
    pub limit: usize,
    pub verbose: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub record: IndexRecord,
    pub score: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RankKey {
    exact_rank: usize,
    path_penalty: usize,
    match_rank: usize,
    kind_rank: usize,
    path_depth: usize,
    text_len: usize,
    path: String,
    line: usize,
    col: usize,
    kind: String,
    name: String,
    source: String,
}

pub fn search(root: &Path, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
    let records = load_records(root)?;
    let query = query.trim();

    if query.is_empty() {
        return Ok(Vec::new());
    }

    let mut results: Vec<SearchResult> = records
        .into_iter()
        .filter(|record| matches_filters(record, options))
        .filter_map(|record| score_record(record, query))
        .collect();

    results.sort_by_key(rank_key);

    let limit = if options.limit == 0 {
        30
    } else {
        options.limit
    };

    results.truncate(limit);

    Ok(results)
}

fn matches_filters(record: &IndexRecord, options: &SearchOptions) -> bool {
    if options
        .kind
        .as_ref()
        .is_some_and(|kind| record.kind != *kind)
    {
        return false;
    }

    if options
        .lang
        .as_ref()
        .is_some_and(|lang| record.lang != *lang)
    {
        return false;
    }

    if options
        .path
        .as_ref()
        .is_some_and(|path| !record.path.contains(path))
    {
        return false;
    }

    if options
        .source
        .as_ref()
        .is_some_and(|source| record.source != *source)
    {
        return false;
    }

    true
}

fn score_record(record: IndexRecord, query: &str) -> Option<SearchResult> {
    let score = match_rank(query, &record);

    if score == usize::MAX {
        return None;
    }

    Some(SearchResult { record, score })
}

fn rank_key(result: &SearchResult) -> RankKey {
    let record = &result.record;

    RankKey {
        exact_rank: exact_rank(result.score),
        path_penalty: path_penalty(&record.path),
        match_rank: result.score,
        kind_rank: kind_rank(&record.kind),
        path_depth: path_depth(&record.path),
        text_len: record.text.chars().count(),
        path: record.path.clone(),
        line: record.line,
        col: record.col,
        kind: record.kind.clone(),
        name: record.name.clone(),
        source: record.source.clone(),
    }
}

fn exact_rank(match_rank: usize) -> usize {
    match match_rank {
        0 | 1 => 0,
        _ => 1,
    }
}

fn match_rank(query: &str, record: &IndexRecord) -> usize {
    let query_lower = query.to_ascii_lowercase();
    let name_lower = record.name.to_ascii_lowercase();

    if record.name == query {
        return 0;
    }

    if name_lower == query_lower {
        return 1;
    }

    if name_lower.starts_with(&query_lower) {
        return 2;
    }

    if word_boundary_match(&name_lower, &query_lower) || camel_case_match(&record.name, query) {
        return 3;
    }

    if name_lower.contains(&query_lower) {
        return 4;
    }

    if text_is_searchable(record) {
        let text_lower = record.text.to_ascii_lowercase();

        if text_lower == query_lower {
            return 20;
        }

        if word_boundary_match(&text_lower, &query_lower) {
            return 21;
        }

        if text_lower.contains(&query_lower) {
            return 30;
        }
    }

    usize::MAX
}

fn text_is_searchable(record: &IndexRecord) -> bool {
    matches!(
        record.kind.as_str(),
        "todo" | "fixme" | "checklist" | "link" | "section" | "heading" | "markdown_heading"
    )
}

fn kind_rank(kind: &str) -> usize {
    match kind {
        "function" | "class" | "method" | "component" | "component_def" => 0,
        "interface" | "type" | "struct" | "enum" | "trait" => 1,
        "const" | "constant" | "variable" | "module" | "import" | "export" => 2,
        "target" | "make_target" => 3,
        "css_id" | "css_class" | "css_variable" | "keyframes" => 4,
        "jsx_class" | "component_usage" => 5,
        "html_id" | "html_class" | "html_tag" | "data_attribute" | "id" => 6,
        "key" | "table" => 6,
        "todo" | "fixme" => 7,
        "heading" | "markdown_heading" | "section" => 8,
        "checklist" | "link" => 9,
        _ => 99,
    }
}

fn path_penalty(path: &str) -> usize {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();

    if normalized.contains("/fixtures/")
        || normalized.starts_with("fixtures/")
        || normalized.contains("/fixture/")
        || normalized.starts_with("fixture/")
        || normalized.contains("/examples/")
        || normalized.starts_with("examples/")
        || normalized.contains("/example/")
        || normalized.starts_with("example/")
    {
        return 25;
    }

    if normalized.contains("/tests/")
        || normalized.starts_with("tests/")
        || normalized.contains("/test/")
        || normalized.starts_with("test/")
    {
        return 10;
    }

    if normalized.contains("/docs/")
        || normalized.starts_with("docs/")
        || normalized.ends_with(".md")
        || normalized.ends_with(".mdx")
    {
        return 15;
    }

    0
}

fn path_depth(path: &str) -> usize {
    path.replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty())
        .count()
}

fn word_boundary_match(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }

    haystack
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .any(|word| word == needle)
}

fn camel_case_match(value: &str, query: &str) -> bool {
    if query.is_empty() {
        return false;
    }

    let abbreviation: String = value
        .chars()
        .filter(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
        .collect();

    !abbreviation.is_empty()
        && abbreviation
            .to_ascii_lowercase()
            .starts_with(&query.to_ascii_lowercase())
}

pub fn format_result(result: &SearchResult, verbose: bool) -> String {
    let record = &result.record;
    let path = redact_sensitive_text(&record.path);
    let kind = redact_sensitive_text(&record.kind);
    let name = redact_sensitive_text(&record.name);

    if verbose {
        format!(
            "{}:{}:{}\n  kind: {}\n  lang: {}\n  source: {}\n  text: {}",
            path,
            record.line,
            record.col,
            kind,
            record.lang,
            record.source,
            redact_sensitive_text(&record.text)
        )
    } else {
        format!("{}:{} {} {}", path, record.line, kind, name)
    }
}
