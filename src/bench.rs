use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};

use crate::{
    context::{render_impact_command, render_pack_command, render_refs_command},
    model::{IndexRecord, ReferenceRecord},
    search::{SearchOptions, search},
    store::{load_manifest, load_records, load_refs, sqlite_path},
};

const DEFAULT_QUERY_LIMIT: usize = 20;
const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    ".gitignore",
    "src",
];

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkReport {
    pub repo_name: String,
    pub repo_path: String,
    pub repo_kind: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkRepo {
    pub name: String,
    pub path: PathBuf,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub queries: Option<Vec<String>>,
    pub expected_paths: Vec<String>,
    pub expected_symbols: Vec<String>,
    pub expected_symbol_patterns: Vec<String>,
    pub from_manifest: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BenchmarkRepoSet {
    MissingRoot,
    Empty,
    Repos {
        manifest_used: bool,
        repos: Vec<BenchmarkRepo>,
    },
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
        repo_path: root.display().to_string(),
        repo_kind: None,
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
    out.push_str(&format!("- path: {}\n", report.repo_path));
    if let Some(kind) = &report.repo_kind {
        out.push_str(&format!("- kind: {kind}\n"));
    }
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

pub fn load_benchmark_repo_set(test_repos_root: &Path) -> Result<BenchmarkRepoSet> {
    if !test_repos_root.exists() {
        return Ok(BenchmarkRepoSet::MissingRoot);
    }

    let manifest_path = test_repos_root.join("MANIFEST.toml");
    if manifest_path.exists() {
        let text = fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read {}", manifest_path.display()))?;
        let repos = parse_benchmark_manifest(&text, test_repos_root)?;

        if repos.is_empty() {
            return Ok(BenchmarkRepoSet::Empty);
        }

        return Ok(BenchmarkRepoSet::Repos {
            manifest_used: true,
            repos,
        });
    }

    let mut repos = Vec::new();
    for entry in fs::read_dir(test_repos_root)
        .with_context(|| format!("failed to read {}", test_repos_root.display()))?
    {
        let path = entry
            .with_context(|| format!("failed to read entry under {}", test_repos_root.display()))?
            .path();

        if !path.is_dir() || !is_repo_root(&path) {
            continue;
        }

        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());

        repos.push(BenchmarkRepo {
            name,
            path,
            kind: None,
            description: None,
            queries: None,
            expected_paths: Vec::new(),
            expected_symbols: Vec::new(),
            expected_symbol_patterns: Vec::new(),
            from_manifest: false,
        });
    }

    repos.sort_by(|a, b| a.name.cmp(&b.name).then(a.path.cmp(&b.path)));

    if repos.is_empty() {
        Ok(BenchmarkRepoSet::Empty)
    } else {
        Ok(BenchmarkRepoSet::Repos {
            manifest_used: false,
            repos,
        })
    }
}

pub fn parse_benchmark_manifest(text: &str, test_repos_root: &Path) -> Result<Vec<BenchmarkRepo>> {
    let mut repos = Vec::new();
    let mut current: Option<ManifestRepoBuilder> = None;

    for (index, raw_line) in text.lines().enumerate() {
        let line_no = index + 1;
        let line = strip_toml_comment(raw_line).trim().to_string();

        if line.is_empty() {
            continue;
        }

        if line == "[[repo]]" {
            if let Some(builder) = current.take()
                && let Some(repo) = finish_manifest_repo(builder, test_repos_root)?
            {
                repos.push(repo);
            }
            current = Some(ManifestRepoBuilder::default());
            continue;
        }

        let Some(builder) = current.as_mut() else {
            bail!("MANIFEST.toml line {line_no}: expected [[repo]] before fields");
        };
        let Some((raw_key, raw_value)) = line.split_once('=') else {
            bail!("MANIFEST.toml line {line_no}: expected key = value");
        };
        let key = raw_key.trim();
        let value = raw_value.trim();

        match key {
            "name" => builder.name = Some(parse_toml_string(value, line_no)?),
            "path" => builder.path = Some(parse_toml_string(value, line_no)?),
            "kind" => builder.kind = Some(parse_toml_string(value, line_no)?),
            "description" => builder.description = Some(parse_toml_string(value, line_no)?),
            "queries" => builder.queries = Some(parse_toml_string_array(value, line_no)?),
            "expected_paths" => {
                builder.expected_paths = Some(parse_toml_string_array(value, line_no)?)
            }
            "expected_symbols" => {
                builder.expected_symbols = Some(parse_toml_string_array(value, line_no)?)
            }
            "expected_symbol_patterns" => {
                builder.expected_symbol_patterns = Some(parse_toml_string_array(value, line_no)?)
            }
            "skip" => builder.skip = parse_toml_bool(value, line_no)?,
            other => bail!("MANIFEST.toml line {line_no}: unknown repo field `{other}`"),
        }
    }

    if let Some(builder) = current.take()
        && let Some(repo) = finish_manifest_repo(builder, test_repos_root)?
    {
        repos.push(repo);
    }

    repos.sort_by(|a, b| a.name.cmp(&b.name).then(a.path.cmp(&b.path)));
    Ok(repos)
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

#[derive(Debug, Default)]
struct ManifestRepoBuilder {
    name: Option<String>,
    path: Option<String>,
    kind: Option<String>,
    description: Option<String>,
    queries: Option<Vec<String>>,
    expected_paths: Option<Vec<String>>,
    expected_symbols: Option<Vec<String>>,
    expected_symbol_patterns: Option<Vec<String>>,
    skip: bool,
}

fn finish_manifest_repo(
    builder: ManifestRepoBuilder,
    test_repos_root: &Path,
) -> Result<Option<BenchmarkRepo>> {
    if builder.skip {
        return Ok(None);
    }

    let Some(name) = builder.name else {
        bail!("MANIFEST.toml repo entry missing required field `name`");
    };
    let Some(raw_path) = builder.path else {
        bail!("MANIFEST.toml repo `{name}` missing required field `path`");
    };
    let queries = sanitize_queries(builder.queries.ok_or_else(|| {
        anyhow::anyhow!("MANIFEST.toml repo `{name}` missing required field `queries`")
    })?)
    .into_iter()
    .take(DEFAULT_QUERY_LIMIT)
    .collect::<Vec<_>>();

    if queries.is_empty() {
        bail!("MANIFEST.toml repo `{name}` must define at least one query");
    }

    let path = resolve_manifest_repo_path(test_repos_root, &raw_path);
    if !path.exists() {
        bail!(
            "MANIFEST.toml repo `{name}` path does not exist: {}",
            path.display()
        );
    }

    Ok(Some(BenchmarkRepo {
        name,
        path,
        kind: builder.kind,
        description: builder.description,
        queries: Some(queries),
        expected_paths: builder.expected_paths.unwrap_or_default(),
        expected_symbols: builder.expected_symbols.unwrap_or_default(),
        expected_symbol_patterns: builder.expected_symbol_patterns.unwrap_or_default(),
        from_manifest: true,
    }))
}

fn resolve_manifest_repo_path(test_repos_root: &Path, raw_path: &str) -> PathBuf {
    if raw_path == "." {
        return test_repos_root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| test_repos_root.to_path_buf());
    }

    test_repos_root.join(raw_path)
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

fn strip_toml_comment(line: &str) -> String {
    let mut out = String::new();
    let mut in_string = false;
    let mut escaped = false;

    for ch in line.chars() {
        if escaped {
            out.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => {
                out.push(ch);
                escaped = true;
            }
            '"' => {
                out.push(ch);
                in_string = !in_string;
            }
            '#' if !in_string => break,
            _ => out.push(ch),
        }
    }

    out
}

fn parse_toml_string(value: &str, line_no: usize) -> Result<String> {
    let value = value.trim();
    if !value.starts_with('"') || !value.ends_with('"') || value.len() < 2 {
        bail!("MANIFEST.toml line {line_no}: expected quoted string");
    }

    unescape_toml_string(&value[1..value.len() - 1], line_no)
}

fn parse_toml_string_array(value: &str, line_no: usize) -> Result<Vec<String>> {
    let value = value.trim();
    if !value.starts_with('[') || !value.ends_with(']') {
        bail!("MANIFEST.toml line {line_no}: expected string array");
    }

    let mut items = Vec::new();
    let mut chars = value[1..value.len() - 1].chars().peekable();

    loop {
        while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
            chars.next();
        }

        if chars.peek().is_none() {
            break;
        }

        if chars.next() != Some('"') {
            bail!("MANIFEST.toml line {line_no}: expected quoted array item");
        }

        let mut raw = String::new();
        let mut escaped = false;
        let mut closed = false;
        for ch in chars.by_ref() {
            if escaped {
                raw.push('\\');
                raw.push(ch);
                escaped = false;
                continue;
            }

            match ch {
                '\\' => escaped = true,
                '"' => {
                    closed = true;
                    break;
                }
                _ => raw.push(ch),
            }
        }

        if !closed {
            bail!("MANIFEST.toml line {line_no}: unterminated array string");
        }

        items.push(unescape_toml_string(&raw, line_no)?);

        while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
            chars.next();
        }

        match chars.peek() {
            Some(',') => {
                chars.next();
            }
            Some(_) => bail!("MANIFEST.toml line {line_no}: expected comma between array items"),
            None => break,
        }
    }

    Ok(items)
}

fn parse_toml_bool(value: &str, line_no: usize) -> Result<bool> {
    match value.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => bail!("MANIFEST.toml line {line_no}: expected boolean"),
    }
}

fn unescape_toml_string(value: &str, line_no: usize) -> Result<String> {
    let mut out = String::new();
    let mut chars = value.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }

        match chars.next() {
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some(other) => {
                bail!("MANIFEST.toml line {line_no}: unsupported escape sequence \\{other}")
            }
            None => bail!("MANIFEST.toml line {line_no}: trailing escape in string"),
        }
    }

    Ok(out)
}

fn is_repo_root(path: &Path) -> bool {
    PROJECT_MARKERS
        .iter()
        .any(|marker| path.join(marker).exists())
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
