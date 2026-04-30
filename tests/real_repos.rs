mod common;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use common::{load_index_snapshot_from_sqlite, run_named_index_integrity_checks};
use regex::Regex;
use thinindex::{
    bench::{BenchmarkRepo, BenchmarkRepoSet, load_benchmark_repo_set},
    context::{render_impact_command, render_pack_command, render_refs_command},
    indexer::build_index,
    search::{SearchOptions, search},
    store::load_manifest,
    tree_sitter_extraction::{LanguageRegistry, TREE_SITTER_SOURCE, TreeSitterExtractionEngine},
};

const EXTRAS_SOURCE: &str = "extras";
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

    let coverage = collect_parser_coverage(repo, &snapshot.records);
    let zero_record_languages = supported_languages_without_records(&coverage);

    let symbol_coverage = check_expected_symbols(repo, &snapshot.records);
    assert!(
        symbol_coverage.missing.is_empty(),
        "expected symbols missing for {}: {:?}",
        repo.name,
        symbol_coverage.missing,
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
    files_seen_by_extras_format: BTreeMap<String, usize>,
    records_by_extras_format: BTreeMap<String, usize>,
    parse_errors_by_language: BTreeMap<String, usize>,
    unsupported_extensions: BTreeMap<String, usize>,
}

#[derive(Debug, Default)]
struct SymbolCoverage {
    checked: usize,
    missing: Vec<String>,
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
        "  expected symbols: checked={} missing={}",
        report.symbol_coverage.checked,
        report.symbol_coverage.missing.len()
    );
    println!(
        "  manifest query smoke: checked={} misses={}",
        report.query_smoke.checked, report.query_smoke.misses
    );
}

fn collect_parser_coverage(
    repo: &BenchmarkRepo,
    records: &[thinindex::model::IndexRecord],
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
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            let parsed = engine
                .parse_file_with_diagnostics(relpath, &text)
                .unwrap_or_else(|error| {
                    panic!("failed to parse {relpath} for coverage: {error:#}")
                });

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
            } else {
                *coverage
                    .unsupported_extensions
                    .entry(extension)
                    .or_default() += 1;
            }
        }
    }

    coverage
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
        coverage.checked += 1;

        if !records.iter().any(|record| record.name == *expected) {
            coverage.missing.push(expected.clone());
        }
    }

    for pattern in &repo.expected_symbol_patterns {
        coverage.checked += 1;
        let regex = Regex::new(pattern).unwrap_or_else(|error| {
            panic!(
                "invalid expected_symbol_patterns entry `{pattern}` for {}: {error}",
                repo.name
            )
        });

        if !records.iter().any(|record| regex.is_match(&record.name)) {
            coverage.missing.push(format!("pattern:{pattern}"));
        }
    }

    coverage
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

        self.expected_symbols_checked += report.symbol_coverage.checked;
        self.expected_symbols_missing += report.symbol_coverage.missing.len();
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
