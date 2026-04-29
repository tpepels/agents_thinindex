mod common;

use std::{fs, path::Path, time::Instant};

use common::{load_index_snapshot_from_sqlite, run_named_index_integrity_checks};
use thinindex::{
    bench::{
        BenchmarkRepo, BenchmarkRepoSet, BenchmarkRunOptions, load_benchmark_repo_set,
        render_benchmark_report, run_benchmark,
    },
    indexer::build_index,
};

#[test]
#[ignore = "rebuilds .dev_index and prints benchmark reports for repos under test_repos/; run with: cargo test --test bench_repos -- --ignored"]
fn benchmark_repos_under_test_repos() {
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
        "bench_repos benchmarking {} repo(s){}:",
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
        benchmark_repo(&repo);
    }
}

fn benchmark_repo(repo: &BenchmarkRepo) {
    let dev_index = repo.path.join(".dev_index");
    if dev_index.exists() {
        fs::remove_dir_all(&dev_index).unwrap_or_else(|error| {
            panic!(
                "failed to remove .dev_index for {}: {error}",
                dev_index.display()
            )
        });
    }

    let started = Instant::now();
    build_index(&repo.path).unwrap_or_else(|error| {
        panic!(
            "failed to build index for {}: {error:#}",
            repo.path.display()
        )
    });
    let build_duration = started.elapsed();

    let snapshot = load_index_snapshot_from_sqlite(&repo.path);
    let expected_paths: Vec<&str> = repo.expected_paths.iter().map(String::as_str).collect();
    run_named_index_integrity_checks(&repo.name, &snapshot, &expected_paths);

    let mut report = run_benchmark(
        &repo.path,
        BenchmarkRunOptions {
            queries: repo.queries.clone(),
            build_duration: Some(build_duration),
        },
    )
    .unwrap_or_else(|error| panic!("failed to benchmark {}: {error:#}", repo.path.display()));

    report.repo_name.clone_from(&repo.name);
    report.repo_path = repo.path.display().to_string();
    report.repo_kind.clone_from(&repo.kind);

    println!("{}", render_benchmark_report(&report));
}
