use std::{
    collections::BTreeSet,
    fs,
    path::Path,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};

use crate::{
    context::{render_impact_command, render_pack_command, render_refs_command},
    model::{IndexRecord, ReferenceRecord},
    search::{SearchOptions, search},
    store::{load_manifest, load_records, load_refs, sqlite_path},
};

const DEFAULT_QUERY_LIMIT: usize = 20;

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkReport {
    pub repo_name: String,
    pub build_duration: Option<Duration>,
    pub db_size_bytes: u64,
    pub indexed_file_count: usize,
    pub record_count: usize,
    pub ref_count: usize,
    pub query_count: usize,
    pub hit_count: usize,
    pub miss_count: usize,
    pub avg_wi_latency: Duration,
    pub avg_refs_latency: Duration,
    pub avg_pack_latency: Duration,
    pub avg_impact_latency: Duration,
    pub avg_result_count: f64,
    pub max_result_count: usize,
    pub avg_pack_files: f64,
    pub avg_impact_files: f64,
    pub duplicate_location_count: usize,
    pub malformed_record_count: usize,
    pub malformed_ref_count: usize,
    pub dev_index_path_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkRunOptions {
    pub queries: Option<Vec<String>>,
    pub build_duration: Option<Duration>,
}

pub fn run_benchmark(root: &Path, options: BenchmarkRunOptions) -> Result<BenchmarkReport> {
    let manifest = load_manifest(root)?;
    let records = load_records(root)?;
    let refs = load_refs(root)?;
    let queries = match options.queries {
        Some(queries) => sanitize_queries(queries),
        None => load_benchmark_queries(root, &records)?,
    };
    let repo_name = root
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.display().to_string());
    let db_size_bytes = fs::metadata(sqlite_path(root))
        .with_context(|| {
            format!(
                "failed to read SQLite index metadata for {}",
                root.display()
            )
        })?
        .len();

    let mut hit_count = 0usize;
    let mut result_sum = 0usize;
    let mut max_result_count = 0usize;
    let mut wi_total = Duration::ZERO;
    let mut refs_total = Duration::ZERO;
    let mut pack_total = Duration::ZERO;
    let mut impact_total = Duration::ZERO;
    let mut pack_file_sum = 0usize;
    let mut impact_file_sum = 0usize;

    for query in &queries {
        let search_options = SearchOptions {
            limit: 30,
            ..SearchOptions::default()
        };

        let start = Instant::now();
        let search_results = search(root, query, &search_options)?;
        wi_total += start.elapsed();

        if !search_results.is_empty() {
            hit_count += 1;
        }
        result_sum += search_results.len();
        max_result_count = max_result_count.max(search_results.len());

        let refs_options = SearchOptions {
            limit: 20,
            ..SearchOptions::default()
        };
        let start = Instant::now();
        let _ = render_refs_command(root, query, &refs_options)?;
        refs_total += start.elapsed();

        let pack_options = SearchOptions {
            limit: 10,
            ..SearchOptions::default()
        };
        let start = Instant::now();
        let pack_output = render_pack_command(root, query, &pack_options)?;
        pack_total += start.elapsed();
        pack_file_sum += count_output_files(&pack_output.text, false);

        let impact_options = SearchOptions {
            limit: 15,
            ..SearchOptions::default()
        };
        let start = Instant::now();
        let impact_output = render_impact_command(root, query, &impact_options)?;
        impact_total += start.elapsed();
        impact_file_sum += count_output_files(&impact_output.text, true);
    }

    let query_count = queries.len();
    let miss_count = query_count.saturating_sub(hit_count);
    let quality = quality_metrics(&records, &refs, manifest.files.keys());

    Ok(BenchmarkReport {
        repo_name,
        build_duration: options.build_duration,
        db_size_bytes,
        indexed_file_count: manifest.files.len(),
        record_count: records.len(),
        ref_count: refs.len(),
        query_count,
        hit_count,
        miss_count,
        avg_wi_latency: avg_duration(wi_total, query_count),
        avg_refs_latency: avg_duration(refs_total, query_count),
        avg_pack_latency: avg_duration(pack_total, query_count),
        avg_impact_latency: avg_duration(impact_total, query_count),
        avg_result_count: avg_usize(result_sum, query_count),
        max_result_count,
        avg_pack_files: avg_usize(pack_file_sum, query_count),
        avg_impact_files: avg_usize(impact_file_sum, query_count),
        duplicate_location_count: quality.duplicate_location_count,
        malformed_record_count: quality.malformed_record_count,
        malformed_ref_count: quality.malformed_ref_count,
        dev_index_path_count: quality.dev_index_path_count,
    })
}

pub fn render_benchmark_report(report: &BenchmarkReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("Repo: {}\n", report.repo_name));
    out.push_str(&format!(
        "- build: {}\n",
        report
            .build_duration
            .map(format_duration)
            .unwrap_or_else(|| "not measured".to_string())
    ));
    out.push_str(&format!("- db: {}\n", format_bytes(report.db_size_bytes)));
    out.push_str(&format!("- files: {}\n", report.indexed_file_count));
    out.push_str(&format!("- records: {}\n", report.record_count));
    out.push_str(&format!("- refs: {}\n", report.ref_count));
    out.push_str(&format!("- queries: {}\n", report.query_count));
    out.push_str(&format!(
        "- hits: {} misses: {}\n",
        report.hit_count, report.miss_count
    ));
    out.push_str(&format!(
        "- hit rate: {}%\n",
        percentage(report.hit_count, report.query_count)
    ));
    out.push_str(&format!(
        "- avg wi latency: {}\n",
        format_duration(report.avg_wi_latency)
    ));
    out.push_str(&format!(
        "- avg refs latency: {}\n",
        format_duration(report.avg_refs_latency)
    ));
    out.push_str(&format!(
        "- avg pack latency: {}\n",
        format_duration(report.avg_pack_latency)
    ));
    out.push_str(&format!(
        "- avg impact latency: {}\n",
        format_duration(report.avg_impact_latency)
    ));
    out.push_str(&format!("- avg results: {:.1}\n", report.avg_result_count));
    out.push_str(&format!("- max results: {}\n", report.max_result_count));
    out.push_str(&format!("- avg pack files: {:.1}\n", report.avg_pack_files));
    out.push_str(&format!(
        "- avg impact files: {:.1}\n",
        report.avg_impact_files
    ));
    out.push_str(&format!("- integrity: {}\n", integrity_status(report)));
    out
}

fn load_benchmark_queries(root: &Path, records: &[IndexRecord]) -> Result<Vec<String>> {
    let query_file = root.join(".thinindex-bench.txt");
    if query_file.exists() {
        let text = fs::read_to_string(&query_file)
            .with_context(|| format!("failed to read {}", query_file.display()))?;
        let queries = sanitize_queries(
            text.lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty() && !trimmed.starts_with('#')
                })
                .map(ToOwned::to_owned)
                .collect(),
        );
        if !queries.is_empty() {
            return Ok(queries);
        }
    }

    Ok(fallback_queries(records))
}

fn fallback_queries(records: &[IndexRecord]) -> Vec<String> {
    let mut names = BTreeSet::new();
    for record in records {
        let name = record.name.trim();
        if !name.is_empty() {
            names.insert(name.to_string());
        }
    }

    names.into_iter().take(DEFAULT_QUERY_LIMIT).collect()
}

fn sanitize_queries(queries: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut cleaned = Vec::new();

    for query in queries {
        let query = query.trim();
        if !query.is_empty() && seen.insert(query.to_string()) {
            cleaned.push(query.to_string());
        }
    }

    cleaned
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct QualityMetrics {
    duplicate_location_count: usize,
    malformed_record_count: usize,
    malformed_ref_count: usize,
    dev_index_path_count: usize,
}

fn quality_metrics<'a>(
    records: &[IndexRecord],
    refs: &[ReferenceRecord],
    manifest_paths: impl Iterator<Item = &'a String>,
) -> QualityMetrics {
    let mut record_locations = BTreeSet::new();
    let mut duplicate_location_count = 0usize;
    let mut malformed_record_count = 0usize;
    let mut malformed_ref_count = 0usize;
    let mut dev_index_path_count = 0usize;

    for record in records {
        if !record_locations.insert((
            record.path.clone(),
            record.line,
            record.col,
            record.kind.clone(),
            record.name.clone(),
        )) {
            duplicate_location_count += 1;
        }

        if record.path.is_empty()
            || record.line == 0
            || record.kind.is_empty()
            || record.name.is_empty()
            || record.source.is_empty()
        {
            malformed_record_count += 1;
        }

        if is_dev_index_path(&record.path) {
            dev_index_path_count += 1;
        }
    }

    for reference in refs {
        if reference.from_path.is_empty()
            || reference.from_line == 0
            || reference.to_name.is_empty()
            || reference.ref_kind.is_empty()
            || reference.source.is_empty()
        {
            malformed_ref_count += 1;
        }

        if is_dev_index_path(&reference.from_path) {
            dev_index_path_count += 1;
        }
    }

    for path in manifest_paths {
        if is_dev_index_path(path) {
            dev_index_path_count += 1;
        }
    }

    QualityMetrics {
        duplicate_location_count,
        malformed_record_count,
        malformed_ref_count,
        dev_index_path_count,
    }
}

fn count_output_files(text: &str, skip_primary: bool) -> usize {
    let mut paths = BTreeSet::new();
    let mut in_primary = false;

    for line in text.lines() {
        if line.ends_with(':') {
            in_primary = line == "Primary:";
            continue;
        }

        if skip_primary && in_primary {
            continue;
        }

        let Some(row) = line.strip_prefix("- ") else {
            continue;
        };

        if row == "none" {
            continue;
        }

        if let Some((path, _rest)) = row.split_once(':') {
            paths.insert(path.to_string());
        }
    }

    paths.len()
}

fn avg_duration(total: Duration, count: usize) -> Duration {
    if count == 0 {
        return Duration::ZERO;
    }

    Duration::from_nanos((total.as_nanos() / count as u128) as u64)
}

fn avg_usize(total: usize, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    }
}

fn percentage(value: usize, total: usize) -> usize {
    if total == 0 {
        0
    } else {
        ((value as f64 / total as f64) * 100.0).round() as usize
    }
}

fn format_duration(duration: Duration) -> String {
    if duration.as_millis() > 0 {
        format!("{}ms", duration.as_millis())
    } else {
        format!("{}us", duration.as_micros())
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{}KB", bytes.div_ceil(1024))
    } else {
        format!("{bytes}B")
    }
}

fn integrity_status(report: &BenchmarkReport) -> String {
    if report.duplicate_location_count == 0
        && report.malformed_record_count == 0
        && report.malformed_ref_count == 0
        && report.dev_index_path_count == 0
    {
        "ok".to_string()
    } else {
        format!(
            "duplicate_locations={} malformed_records={} malformed_refs={} dev_index_paths={}",
            report.duplicate_location_count,
            report.malformed_record_count,
            report.malformed_ref_count,
            report.dev_index_path_count
        )
    }
}

fn is_dev_index_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    normalized == ".dev_index" || normalized.starts_with(".dev_index/")
}
