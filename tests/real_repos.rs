mod common;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use common::{load_index_snapshot_from_sqlite, run_named_index_integrity_checks};
use thinindex::{
    bench::{BenchmarkRepo, BenchmarkRepoSet, load_benchmark_repo_set},
    indexer::build_index,
    store::load_manifest,
    tree_sitter_extraction::{LanguageRegistry, TREE_SITTER_SOURCE, TreeSitterExtractionEngine},
};

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

    for repo in repos {
        check_repo(&repo);
    }
}

fn check_repo(repo: &BenchmarkRepo) {
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
    print_parser_coverage_report(repo, &snapshot.records);
}

#[derive(Debug, Default)]
struct ParserCoverage {
    files_seen_by_language: BTreeMap<String, usize>,
    records_by_language: BTreeMap<String, usize>,
    parse_errors_by_language: BTreeMap<String, usize>,
    unsupported_extensions: BTreeMap<String, usize>,
}

fn print_parser_coverage_report(repo: &BenchmarkRepo, records: &[thinindex::model::IndexRecord]) {
    let coverage = collect_parser_coverage(repo, records);

    println!("parser coverage for {}:", repo.name);
    println!(
        "  files seen by language: {}",
        render_counts(&coverage.files_seen_by_language)
    );
    println!(
        "  records emitted by language: {}",
        render_counts(&coverage.records_by_language)
    );
    println!(
        "  parse errors by language: {}",
        render_counts(&coverage.parse_errors_by_language)
    );
    println!(
        "  unsupported extension gaps: {}",
        render_top_gaps(&coverage.unsupported_extensions)
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
            *coverage
                .unsupported_extensions
                .entry(extension)
                .or_default() += 1;
        }
    }

    coverage
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
