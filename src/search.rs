use std::path::Path;

use anyhow::Result;

use crate::{model::IndexRecord, store::load_records};

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

    results.sort_by(|a, b| {
        a.score
            .cmp(&b.score)
            .then(a.record.path.cmp(&b.record.path))
            .then(a.record.line.cmp(&b.record.line))
            .then(a.record.col.cmp(&b.record.col))
            .then(a.record.kind.cmp(&b.record.kind))
            .then(a.record.name.cmp(&b.record.name))
            .then(a.record.source.cmp(&b.record.source))
    });

    let limit = if options.limit == 0 {
        30
    } else {
        options.limit
    };
    results.truncate(limit);

    Ok(results)
}

fn matches_filters(record: &IndexRecord, options: &SearchOptions) -> bool {
    if let Some(kind) = &options.kind {
        if record.kind != *kind {
            return false;
        }
    }

    if let Some(lang) = &options.lang {
        if record.lang != *lang {
            return false;
        }
    }

    if let Some(path) = &options.path {
        if !record.path.contains(path) {
            return false;
        }
    }

    if let Some(source) = &options.source {
        if record.source != *source {
            return false;
        }
    }

    true
}

fn score_record(record: IndexRecord, query: &str) -> Option<SearchResult> {
    let query_lower = query.to_ascii_lowercase();
    let name_lower = record.name.to_ascii_lowercase();
    let text_lower = record.text.to_ascii_lowercase();
    let path_lower = record.path.to_ascii_lowercase();
    let kind_lower = record.kind.to_ascii_lowercase();
    let source_lower = record.source.to_ascii_lowercase();

    let score = if record.name == query {
        0
    } else if name_lower == query_lower {
        1
    } else if name_lower.contains(&query_lower) {
        2
    } else if text_lower.contains(&query_lower) {
        3
    } else if path_lower.contains(&query_lower) {
        4
    } else if kind_lower.contains(&query_lower) {
        5
    } else if source_lower.contains(&query_lower) {
        6
    } else {
        return None;
    };

    Some(SearchResult { record, score })
}

pub fn format_result(result: &SearchResult, verbose: bool) -> String {
    let record = &result.record;

    if verbose {
        format!(
            "{}:{}:{}\n  kind: {}\n  lang: {}\n  source: {}\n  text: {}",
            record.path,
            record.line,
            record.col,
            record.kind,
            record.lang,
            record.source,
            record.text
        )
    } else {
        format!(
            "{}:{} {} {}",
            record.path, record.line, record.kind, record.name
        )
    }
}
