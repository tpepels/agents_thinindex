mod common;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use common::{load_index_snapshot_from_sqlite, run_named_index_integrity_checks};
use regex::Regex;
use thinindex::{
    bench::{
        BenchmarkRepo, BenchmarkRepoSet, ExpectedAbsentSymbol, ExpectedSymbol,
        ExpectedSymbolPattern, load_benchmark_repo_set,
    },
    context::{render_impact_command, render_pack_command, render_refs_command},
    indexer,
    indexer::build_index,
    refs,
    search::{SearchOptions, search},
    store::load_manifest,
    tree_sitter_extraction::{LanguageRegistry, TREE_SITTER_SOURCE, TreeSitterExtractionEngine},
};

const EXTRAS_SOURCE: &str = "extras";
const SLOW_PARSE_WARNING_THRESHOLD: Duration = Duration::from_millis(250);
const NOISY_RECORD_WARNING_THRESHOLD: usize = 1_000;
const NOISY_REF_WARNING_THRESHOLD: usize = refs::MAX_REFS_PER_FILE;
const LARGE_FILE_WARNING_BYTES: u64 = 1_000_000;
const SUPPORTED_EXTRAS_EXTENSIONS: &[&str] = &[
    ".css",
    ".html",
    ".md",
    ".markdown",
    ".json",
    ".toml",
    ".yaml",
    ".yml",
];

#[test]
#[ignore = "rebuilds .dev_index for every repo under test_repos/; run with: cargo test --test real_repos -- --ignored"]
fn real_repos_pass_shared_integrity_checks() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_repos");

    let repo_set = load_benchmark_repo_set(&root)
        .unwrap_or_else(|error| panic!("failed to load benchmark repo set: {error:#}"));

    let BenchmarkRepoSet::Repos {
        manifest_used,
        repos,
    } = repo_set
    else {
        match repo_set {
            BenchmarkRepoSet::MissingRoot => println!("skipped: test_repos/ missing"),
            BenchmarkRepoSet::Empty => println!("skipped: test_repos/ has no repo directories"),
            BenchmarkRepoSet::Repos { .. } => unreachable!(),
        }
        return;
    };

    println!(
        "real_repos testing {} repo(s){}:",
        repos.len(),
        if manifest_used {
            " from MANIFEST.toml"
        } else {
            ""
        }
    );
    for repo in &repos {
        println!("  - {} ({})", repo.name, repo.path.display());
    }

    let mut aggregate = AggregateCoverage::default();

    for repo in &repos {
        let report = check_repo(repo);
        aggregate.add(&report);
    }

    print_aggregate_coverage_report(&aggregate);
}

fn check_repo(repo: &BenchmarkRepo) -> RepoHardeningReport {
    let dev_index = repo.path.join(".dev_index");

    if dev_index.exists() {
        fs::remove_dir_all(&dev_index).unwrap_or_else(|error| {
            panic!(
                "failed to remove .dev_index for {}: {error}",
                dev_index.display()
            )
        });
    }

    build_index(&repo.path).unwrap_or_else(|error| {
        panic!(
            "failed to build index for {}: {error:#}",
            repo.path.display()
        )
    });

    let snapshot = load_index_snapshot_from_sqlite(&repo.path);
    let expected_paths: Vec<&str> = repo.expected_paths.iter().map(String::as_str).collect();

    run_named_index_integrity_checks(&repo.name, &snapshot, &expected_paths);

    let coverage = collect_parser_coverage(repo, &snapshot.records, &snapshot.refs);
    let zero_record_languages = supported_languages_without_records(&coverage);

    let symbol_coverage = check_expected_symbols(repo, &snapshot.records);
    assert!(
        symbol_coverage.symbols_missing.is_empty()
            && symbol_coverage.patterns_missing.is_empty()
            && symbol_coverage.absent_symbols_found.is_empty(),
        "expected symbol manifest failures for {}: symbols={:?} patterns={:?} absent={:?}",
        repo.name,
        symbol_coverage.symbols_missing,
        symbol_coverage.patterns_missing,
        symbol_coverage.absent_symbols_found,
    );

    let query_smoke = run_manifest_query_smokes(repo);
    let manifest = load_manifest(&repo.path)
        .unwrap_or_else(|error| panic!("failed to load manifest for {}: {error:#}", repo.name));
    let report = RepoHardeningReport {
        repo_name: repo.name.clone(),
        repo_path: repo.path.display().to_string(),
        indexed_files: manifest.files.len(),
        record_count: snapshot.records.len(),
        ref_count: snapshot.refs.len(),
        coverage,
        zero_record_languages,
        symbol_coverage,
        query_smoke,
    };

    print_parser_coverage_report(&report);
    report
}

#[derive(Debug, Default)]
struct ParserCoverage {
    files_seen_by_language: BTreeMap<String, usize>,
    records_by_language: BTreeMap<String, usize>,
    refs_by_language: BTreeMap<String, usize>,
    files_seen_by_extras_format: BTreeMap<String, usize>,
    records_by_extras_format: BTreeMap<String, usize>,
    parse_errors_by_language: BTreeMap<String, usize>,
    parse_time_by_language: BTreeMap<String, Duration>,
    unsupported_extensions: BTreeMap<String, usize>,
    slow_files: Vec<FileDurationWarning>,
    noisy_record_files: Vec<FileCountWarning>,
    noisy_ref_files: Vec<FileCountWarning>,
    large_files: Vec<FileSizeWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileDurationWarning {
    path: String,
    language: String,
    duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileCountWarning {
    path: String,
    language: String,
    count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileSizeWarning {
    path: String,
    size_bytes: u64,
}

#[derive(Debug, Default)]
struct SymbolCoverage {
    symbols_checked: usize,
    symbols_missing: Vec<String>,
    patterns_checked: usize,
    patterns_missing: Vec<String>,
    absent_symbols_checked: usize,
    absent_symbols_found: Vec<String>,
}

#[derive(Debug, Default)]
struct QuerySmoke {
    checked: usize,
    misses: usize,
}

#[derive(Debug, Default)]
struct RepoHardeningReport {
    repo_name: String,
    repo_path: String,
    indexed_files: usize,
    record_count: usize,
    ref_count: usize,
    coverage: ParserCoverage,
    zero_record_languages: Vec<String>,
    symbol_coverage: SymbolCoverage,
    query_smoke: QuerySmoke,
}

#[derive(Debug, Default)]
struct AggregateCoverage {
    repo_count: usize,
    supported_languages_seen: BTreeSet<String>,
    supported_languages_with_failures: BTreeSet<String>,
    unsupported_extensions: BTreeMap<String, usize>,
    expected_symbols_checked: usize,
    expected_symbols_missing: usize,
    expected_symbol_patterns_checked: usize,
    expected_symbol_patterns_missing: usize,
    expected_absent_symbols_checked: usize,
    expected_absent_symbols_found: usize,
    zero_record_languages: BTreeSet<String>,
}

fn print_parser_coverage_report(report: &RepoHardeningReport) {
    let coverage = &report.coverage;

    println!("parser coverage for {}:", report.repo_name);
    println!("  path: {}", report.repo_path);
    println!("  files indexed: {}", report.indexed_files);
    println!("  records emitted: {}", report.record_count);
    println!("  refs emitted: {}", report.ref_count);
    println!(
        "  files seen by language: {}",
        render_counts(&coverage.files_seen_by_language)
    );
    println!(
        "  records emitted by language: {}",
        render_counts(&coverage.records_by_language)
    );
    println!(
        "  refs emitted by language: {}",
        render_counts(&coverage.refs_by_language)
    );
    println!(
        "  parse time by language: {}",
        render_duration_counts(&coverage.parse_time_by_language)
    );
    println!(
        "  extras-backed files seen: {}",
        render_counts(&coverage.files_seen_by_extras_format)
    );
    println!(
        "  extras-backed records emitted: {}",
        render_counts(&coverage.records_by_extras_format)
    );
    println!(
        "  parse errors by language: {}",
        render_counts(&coverage.parse_errors_by_language)
    );
    println!(
        "  unsupported extension gaps: {}",
        render_top_gaps(&coverage.unsupported_extensions)
    );
    println!(
        "  supported languages with zero records: {}",
        render_slice(&report.zero_record_languages)
    );
    println!(
        "  slowest files: {}",
        render_duration_warnings(&coverage.slow_files)
    );
    println!(
        "  noisiest record files: {}",
        render_count_warnings(&coverage.noisy_record_files)
    );
    println!(
        "  noisiest ref files: {}",
        render_count_warnings(&coverage.noisy_ref_files)
    );
    println!(
        "  large files: {}",
        render_size_warnings(&coverage.large_files)
    );
    println!(
        "  expected symbols: checked={} missing={}",
        report.symbol_coverage.symbols_checked,
        report.symbol_coverage.symbols_missing.len()
    );
    println!(
        "  expected symbol patterns: checked={} missing={}",
        report.symbol_coverage.patterns_checked,
        report.symbol_coverage.patterns_missing.len()
    );
    println!(
        "  expected absent symbols: checked={} found={}",
        report.symbol_coverage.absent_symbols_checked,
        report.symbol_coverage.absent_symbols_found.len()
    );
    println!(
        "  manifest query smoke: checked={} misses={}",
        report.query_smoke.checked, report.query_smoke.misses
    );
}

fn collect_parser_coverage(
    repo: &BenchmarkRepo,
    records: &[thinindex::model::IndexRecord],
    refs: &[thinindex::model::ReferenceRecord],
) -> ParserCoverage {
    let mut coverage = ParserCoverage::default();
    let manifest = load_manifest(&repo.path)
        .unwrap_or_else(|error| panic!("failed to load manifest for {}: {error:#}", repo.name));
    let registry = LanguageRegistry::default();
    let engine = TreeSitterExtractionEngine::default();

    for record in records
        .iter()
        .filter(|record| record.source == TREE_SITTER_SOURCE)
    {
        *coverage
            .records_by_language
            .entry(record.lang.clone())
            .or_default() += 1;
    }

    let records_by_path = counts_by_path(records.iter().map(|record| record.path.as_str()));
    let refs_by_path = counts_by_path(refs.iter().map(|reference| reference.from_path.as_str()));

    for reference in refs {
        let language = language_or_format_for_path(&registry, &reference.from_path);
        *coverage.refs_by_language.entry(language).or_default() += 1;
    }

    for record in records
        .iter()
        .filter(|record| record.source == EXTRAS_SOURCE)
    {
        *coverage
            .records_by_extras_format
            .entry(record.lang.clone())
            .or_default() += 1;
    }

    for relpath in manifest.files.keys() {
        if let Some(adapter) = registry.adapter_for_path(relpath) {
            *coverage
                .files_seen_by_language
                .entry(adapter.id.to_string())
                .or_default() += 1;

            let path = repo.path.join(relpath);
            let size_bytes = fs::metadata(&path)
                .unwrap_or_else(|error| panic!("failed to stat {}: {error}", path.display()))
                .len();
            if size_bytes > LARGE_FILE_WARNING_BYTES {
                coverage.large_files.push(FileSizeWarning {
                    path: relpath.clone(),
                    size_bytes,
                });
            }

            let text = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            let parsed = engine
                .parse_file_with_diagnostics(relpath, &text)
                .unwrap_or_else(|error| {
                    panic!("failed to parse {relpath} for coverage: {error:#}")
                });
            *coverage
                .parse_time_by_language
                .entry(adapter.id.to_string())
                .or_default() += parsed.parse_duration;

            if parsed.parse_duration > SLOW_PARSE_WARNING_THRESHOLD {
                coverage.slow_files.push(FileDurationWarning {
                    path: relpath.clone(),
                    language: adapter.id.to_string(),
                    duration: parsed.parse_duration,
                });
            }

            let record_count = records_by_path.get(relpath.as_str()).copied().unwrap_or(0);
            if record_count > NOISY_RECORD_WARNING_THRESHOLD {
                coverage.noisy_record_files.push(FileCountWarning {
                    path: relpath.clone(),
                    language: adapter.id.to_string(),
                    count: record_count,
                });
            }

            if parsed.had_error {
                *coverage
                    .parse_errors_by_language
                    .entry(adapter.id.to_string())
                    .or_default() += 1;
            }
        } else if let Some(extension) = extension_gap(relpath) {
            if SUPPORTED_EXTRAS_EXTENSIONS.contains(&extension.as_str()) {
                *coverage
                    .files_seen_by_extras_format
                    .entry(extension.trim_start_matches('.').to_string())
                    .or_default() += 1;
                let record_count = records_by_path.get(relpath.as_str()).copied().unwrap_or(0);
                if record_count > NOISY_RECORD_WARNING_THRESHOLD {
                    coverage.noisy_record_files.push(FileCountWarning {
                        path: relpath.clone(),
                        language: extension.trim_start_matches('.').to_string(),
                        count: record_count,
                    });
                }
            } else {
                *coverage
                    .unsupported_extensions
                    .entry(extension)
                    .or_default() += 1;
            }
        }

        let ref_count = refs_by_path.get(relpath.as_str()).copied().unwrap_or(0);
        if ref_count >= NOISY_REF_WARNING_THRESHOLD {
            coverage.noisy_ref_files.push(FileCountWarning {
                path: relpath.clone(),
                language: language_or_format_for_path(&registry, relpath),
                count: ref_count,
            });
        }
    }

    coverage
        .slow_files
        .sort_by(|a, b| b.duration.cmp(&a.duration).then(a.path.cmp(&b.path)));
    coverage
        .noisy_record_files
        .sort_by(|a, b| b.count.cmp(&a.count).then(a.path.cmp(&b.path)));
    coverage
        .noisy_ref_files
        .sort_by(|a, b| b.count.cmp(&a.count).then(a.path.cmp(&b.path)));
    coverage
        .large_files
        .sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes).then(a.path.cmp(&b.path)));

    coverage
}

fn counts_by_path<'a>(paths: impl Iterator<Item = &'a str>) -> BTreeMap<&'a str, usize> {
    let mut counts = BTreeMap::new();

    for path in paths {
        *counts.entry(path).or_default() += 1;
    }

    counts
}

fn language_or_format_for_path(registry: &LanguageRegistry, relpath: &str) -> String {
    if let Some(adapter) = registry.adapter_for_path(relpath) {
        return adapter.id.to_string();
    }

    extension_gap(relpath)
        .map(|extension| extension.trim_start_matches('.').to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn supported_languages_without_records(coverage: &ParserCoverage) -> Vec<String> {
    coverage
        .files_seen_by_language
        .keys()
        .filter(|language| {
            coverage
                .records_by_language
                .get(*language)
                .copied()
                .unwrap_or(0)
                == 0
        })
        .cloned()
        .collect()
}

fn check_expected_symbols(
    repo: &BenchmarkRepo,
    records: &[thinindex::model::IndexRecord],
) -> SymbolCoverage {
    let mut coverage = SymbolCoverage::default();

    for expected in &repo.expected_symbols {
        coverage.symbols_checked += 1;

        if !records.iter().any(|record| record.name == *expected) {
            coverage
                .symbols_missing
                .push(format!("repo={} name={expected}", repo.name));
        }
    }

    for expected in &repo.expected_symbol_specs {
        coverage.symbols_checked += 1;

        if !records
            .iter()
            .any(|record| record_matches_expected_symbol(record, expected))
        {
            coverage.symbols_missing.push(format!(
                "repo={} language={:?} path={:?} kind={:?} name={} nearby={}",
                repo.name,
                expected.language,
                expected.path,
                expected.kind,
                expected.name,
                render_nearby_records(records, expected)
            ));
        }
    }

    for pattern in &repo.expected_symbol_patterns {
        coverage.patterns_checked += 1;
        let regex = Regex::new(pattern).unwrap_or_else(|error| {
            panic!(
                "invalid expected_symbol_patterns entry `{pattern}` for {}: {error}",
                repo.name
            )
        });

        if !records.iter().any(|record| regex.is_match(&record.name)) {
            coverage.patterns_missing.push(format!(
                "repo={} name_regex={pattern} min_count=1 actual=0",
                repo.name
            ));
        }
    }

    for pattern in &repo.expected_symbol_pattern_specs {
        coverage.patterns_checked += 1;
        let regex = Regex::new(&pattern.name_regex).unwrap_or_else(|error| {
            panic!(
                "invalid expected_symbol_pattern name_regex `{}` for {}: {error}",
                pattern.name_regex, repo.name
            )
        });
        let count = records
            .iter()
            .filter(|record| record_matches_expected_symbol_pattern(record, pattern, &regex))
            .count();

        if count < pattern.min_count {
            coverage.patterns_missing.push(format!(
                "repo={} language={:?} path_glob={:?} kind={:?} name_regex={} min_count={} actual={count} nearby={}",
                repo.name,
                pattern.language,
                pattern.path_glob,
                pattern.kind,
                pattern.name_regex,
                pattern.min_count,
                render_pattern_candidates(records, pattern)
            ));
        }
    }

    for expected in &repo.expected_absent_symbol_specs {
        coverage.absent_symbols_checked += 1;
        let matches = records
            .iter()
            .filter(|record| record_matches_expected_absent_symbol(record, expected))
            .collect::<Vec<_>>();

        if !matches.is_empty() {
            coverage.absent_symbols_found.push(format!(
                "repo={} language={:?} path={:?} kind={:?} name={} matches={}",
                repo.name,
                expected.language,
                expected.path,
                expected.kind,
                expected.name,
                render_records(matches.into_iter())
            ));
        }
    }

    coverage.symbols_missing.sort();
    coverage.patterns_missing.sort();
    coverage.absent_symbols_found.sort();
    coverage
}

fn record_matches_expected_symbol(
    record: &thinindex::model::IndexRecord,
    expected: &ExpectedSymbol,
) -> bool {
    record.name == expected.name
        && expected
            .language
            .as_ref()
            .is_none_or(|language| record.lang == *language)
        && expected
            .path
            .as_ref()
            .is_none_or(|path| record.path == *path)
        && expected
            .kind
            .as_ref()
            .is_none_or(|kind| record.kind == *kind)
}

fn record_matches_expected_absent_symbol(
    record: &thinindex::model::IndexRecord,
    expected: &ExpectedAbsentSymbol,
) -> bool {
    record.name == expected.name
        && expected
            .language
            .as_ref()
            .is_none_or(|language| record.lang == *language)
        && expected
            .path
            .as_ref()
            .is_none_or(|path| record.path == *path)
        && expected
            .kind
            .as_ref()
            .is_none_or(|kind| record.kind == *kind)
}

fn record_matches_expected_symbol_pattern(
    record: &thinindex::model::IndexRecord,
    pattern: &ExpectedSymbolPattern,
    name_regex: &Regex,
) -> bool {
    pattern
        .language
        .as_ref()
        .is_none_or(|language| record.lang == *language)
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

fn render_nearby_records(
    records: &[thinindex::model::IndexRecord],
    expected: &ExpectedSymbol,
) -> String {
    let mut candidates = records
        .iter()
        .filter(|record| {
            expected
                .path
                .as_ref()
                .is_some_and(|path| record.path == *path)
                || expected
                    .language
                    .as_ref()
                    .is_some_and(|language| record.lang == *language)
                || expected
                    .kind
                    .as_ref()
                    .is_some_and(|kind| record.kind == *kind)
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        candidates = records.iter().take(3).collect();
    }

    render_records(candidates.into_iter())
}

fn render_pattern_candidates(
    records: &[thinindex::model::IndexRecord],
    pattern: &ExpectedSymbolPattern,
) -> String {
    let candidates = records
        .iter()
        .filter(|record| {
            pattern
                .language
                .as_ref()
                .is_none_or(|language| record.lang == *language)
                && pattern
                    .kind
                    .as_ref()
                    .is_none_or(|kind| record.kind == *kind)
                && pattern
                    .path_glob
                    .as_ref()
                    .is_none_or(|path_glob| glob_matches(path_glob, &record.path))
        })
        .collect::<Vec<_>>();

    render_records(candidates.into_iter())
}

fn render_records<'a>(records: impl Iterator<Item = &'a thinindex::model::IndexRecord>) -> String {
    let mut records = records.collect::<Vec<_>>();
    records.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.line.cmp(&right.line))
            .then(left.col.cmp(&right.col))
            .then(left.kind.cmp(&right.kind))
            .then(left.name.cmp(&right.name))
    });
    let rendered = records
        .into_iter()
        .take(3)
        .map(|record| {
            format!(
                "{}:{}:{} {} {} ({})",
                record.path, record.line, record.col, record.kind, record.name, record.lang
            )
        })
        .collect::<Vec<_>>();

    if rendered.is_empty() {
        "none".to_string()
    } else {
        rendered.join("; ")
    }
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

fn run_manifest_query_smokes(repo: &BenchmarkRepo) -> QuerySmoke {
    let mut smoke = QuerySmoke::default();
    let Some(queries) = &repo.queries else {
        return smoke;
    };

    for query in queries {
        smoke.checked += 1;

        let search_options = SearchOptions {
            limit: 30,
            ..SearchOptions::default()
        };
        let results = search(&repo.path, query, &search_options).unwrap_or_else(|error| {
            panic!("wi search failed for {} `{query}`: {error:#}", repo.name)
        });

        if results.is_empty() {
            smoke.misses += 1;
        }

        let refs_options = SearchOptions {
            limit: 20,
            ..SearchOptions::default()
        };
        render_refs_command(&repo.path, query, &refs_options).unwrap_or_else(|error| {
            panic!("wi refs failed for {} `{query}`: {error:#}", repo.name)
        });

        let pack_options = SearchOptions {
            limit: 10,
            ..SearchOptions::default()
        };
        render_pack_command(&repo.path, query, &pack_options).unwrap_or_else(|error| {
            panic!("wi pack failed for {} `{query}`: {error:#}", repo.name)
        });

        let impact_options = SearchOptions {
            limit: 15,
            ..SearchOptions::default()
        };
        render_impact_command(&repo.path, query, &impact_options).unwrap_or_else(|error| {
            panic!("wi impact failed for {} `{query}`: {error:#}", repo.name)
        });
    }

    smoke
}

impl AggregateCoverage {
    fn add(&mut self, report: &RepoHardeningReport) {
        self.repo_count += 1;

        for language in report.coverage.files_seen_by_language.keys() {
            self.supported_languages_seen.insert(language.clone());

            if report
                .coverage
                .records_by_language
                .get(language)
                .copied()
                .unwrap_or(0)
                == 0
            {
                self.supported_languages_with_failures
                    .insert(language.clone());
            }
        }

        for (extension, count) in &report.coverage.unsupported_extensions {
            *self
                .unsupported_extensions
                .entry(extension.clone())
                .or_default() += count;
        }

        for language in &report.zero_record_languages {
            self.zero_record_languages.insert(language.clone());
        }

        self.expected_symbols_checked += report.symbol_coverage.symbols_checked;
        self.expected_symbols_missing += report.symbol_coverage.symbols_missing.len();
        self.expected_symbol_patterns_checked += report.symbol_coverage.patterns_checked;
        self.expected_symbol_patterns_missing += report.symbol_coverage.patterns_missing.len();
        self.expected_absent_symbols_checked += report.symbol_coverage.absent_symbols_checked;
        self.expected_absent_symbols_found += report.symbol_coverage.absent_symbols_found.len();
    }
}

fn print_aggregate_coverage_report(aggregate: &AggregateCoverage) {
    println!("aggregate parser coverage:");
    println!("  repos checked: {}", aggregate.repo_count);
    println!(
        "  supported languages seen: {}",
        render_set(&aggregate.supported_languages_seen)
    );
    println!(
        "  supported languages with failures: {}",
        render_set(&aggregate.supported_languages_with_failures)
    );
    println!(
        "  unsupported extension gaps: {}",
        render_top_gaps(&aggregate.unsupported_extensions)
    );
    println!(
        "  supported languages with zero records: {}",
        render_set(&aggregate.zero_record_languages)
    );
    println!(
        "  expected symbols: checked={} missing={}",
        aggregate.expected_symbols_checked, aggregate.expected_symbols_missing
    );
    println!(
        "  expected symbol patterns: checked={} missing={}",
        aggregate.expected_symbol_patterns_checked, aggregate.expected_symbol_patterns_missing
    );
    println!(
        "  expected absent symbols: checked={} found={}",
        aggregate.expected_absent_symbols_checked, aggregate.expected_absent_symbols_found
    );
}

fn extension_gap(relpath: &str) -> Option<String> {
    PathBuf::from(relpath)
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map(|value| format!(".{}", value.to_ascii_lowercase()))
}

fn render_counts(counts: &BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }

    counts
        .iter()
        .map(|(name, count)| format!("{name}={count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_duration_counts(counts: &BTreeMap<String, Duration>) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }

    counts
        .iter()
        .map(|(name, duration)| format!("{name}={}", format_duration(*duration)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_set(values: &BTreeSet<String>) -> String {
    if values.is_empty() {
        return "none".to_string();
    }

    values.iter().cloned().collect::<Vec<_>>().join(", ")
}

fn render_slice(values: &[String]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }

    values.join(", ")
}

fn render_duration_warnings(values: &[FileDurationWarning]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }

    values
        .iter()
        .take(5)
        .map(|warning| {
            format!(
                "{}:{}:{}",
                warning.path,
                warning.language,
                format_duration(warning.duration)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_count_warnings(values: &[FileCountWarning]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }

    values
        .iter()
        .take(5)
        .map(|warning| format!("{}:{}:{}", warning.path, warning.language, warning.count))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_size_warnings(values: &[FileSizeWarning]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }

    values
        .iter()
        .take(5)
        .map(|warning| format!("{}:{} bytes", warning.path, warning.size_bytes))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_duration(duration: Duration) -> String {
    if duration.as_millis() >= 1 {
        format!("{}ms", duration.as_millis())
    } else {
        format!("{}us", duration.as_micros())
    }
}

#[test]
fn expected_symbol_specs_match_records_with_filters() {
    let repo = BenchmarkRepo {
        name: "fixture".to_string(),
        path: PathBuf::from("."),
        kind: None,
        languages: vec!["rs".to_string()],
        description: None,
        skip_reason: None,
        notes: None,
        ignore_guidance: None,
        queries: None,
        expected_paths: Vec::new(),
        expected_symbols: Vec::new(),
        expected_symbol_patterns: Vec::new(),
        expected_symbol_specs: vec![ExpectedSymbol {
            language: Some("rs".to_string()),
            path: Some("src/indexer.rs".to_string()),
            kind: Some("function".to_string()),
            name: "build_index".to_string(),
        }],
        expected_symbol_pattern_specs: vec![ExpectedSymbolPattern {
            language: Some("rs".to_string()),
            path_glob: Some("src/**/*.rs".to_string()),
            kind: Some("function".to_string()),
            name_regex: "^build_.*".to_string(),
            min_count: 1,
        }],
        expected_absent_symbol_specs: vec![ExpectedAbsentSymbol {
            language: Some("rs".to_string()),
            path: Some("src/indexer.rs".to_string()),
            kind: Some("function".to_string()),
            name: "NotARealBuildIndexCommentSymbol".to_string(),
        }],
        quality_thresholds: Vec::new(),
        from_manifest: false,
    };
    let records = vec![thinindex::model::IndexRecord::new(
        "src/indexer.rs",
        51,
        8,
        "rs",
        "function",
        "build_index",
        "pub fn build_index(start: &Path) -> Result<BuildStats> {",
        TREE_SITTER_SOURCE,
    )];

    let coverage = check_expected_symbols(&repo, &records);

    assert_eq!(coverage.symbols_checked, 1);
    assert!(coverage.symbols_missing.is_empty());
    assert_eq!(coverage.patterns_checked, 1);
    assert!(coverage.patterns_missing.is_empty());
    assert_eq!(coverage.absent_symbols_checked, 1);
    assert!(coverage.absent_symbols_found.is_empty());
}

#[test]
fn parser_report_helpers_are_deterministic() {
    let mut durations = BTreeMap::new();
    durations.insert("rs".to_string(), Duration::from_millis(2));
    durations.insert("py".to_string(), Duration::from_millis(1));

    assert_eq!(render_duration_counts(&durations), "py=1ms, rs=2ms");

    let warnings = vec![
        FileCountWarning {
            path: "b.rs".to_string(),
            language: "rs".to_string(),
            count: indexer::MAX_RECORDS_PER_FILE,
        },
        FileCountWarning {
            path: "a.rs".to_string(),
            language: "rs".to_string(),
            count: NOISY_RECORD_WARNING_THRESHOLD,
        },
    ];

    assert_eq!(
        render_count_warnings(&warnings),
        format!(
            "b.rs:rs:{}, a.rs:rs:{}",
            indexer::MAX_RECORDS_PER_FILE,
            NOISY_RECORD_WARNING_THRESHOLD
        )
    );
}

fn render_top_gaps(counts: &BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }

    let mut entries: Vec<(&String, &usize)> = counts.iter().collect();
    entries.sort_by(|(left_ext, left_count), (right_ext, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_ext.cmp(right_ext))
    });

    entries
        .into_iter()
        .take(8)
        .map(|(name, count)| format!("{name}={count}"))
        .collect::<Vec<_>>()
        .join(", ")
}
