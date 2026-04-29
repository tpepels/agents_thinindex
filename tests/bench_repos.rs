use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use thinindex::{
    bench::{BenchmarkRunOptions, render_benchmark_report, run_benchmark},
    indexer::build_index,
};

const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    ".gitignore",
    "src",
];

#[test]
#[ignore = "rebuilds .dev_index and prints benchmark reports for repos under test_repos/; run with: cargo test --test bench_repos -- --ignored"]
fn benchmark_repos_under_test_repos() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_repos");

    if !root.exists() {
        println!("skipped: test_repos/ missing");
        return;
    }

    let mut repos: Vec<PathBuf> = fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && is_repo_root(path))
        .collect();

    repos.sort();

    if repos.is_empty() {
        println!("skipped: test_repos/ has no repo directories");
        return;
    }

    println!("bench_repos benchmarking {} repo(s):", repos.len());

    for repo in repos {
        let name = repo
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| repo.display().to_string());
        println!("  - {name}");

        let dev_index = repo.join(".dev_index");
        if dev_index.exists() {
            fs::remove_dir_all(&dev_index).unwrap_or_else(|error| {
                panic!(
                    "failed to remove .dev_index for {}: {error}",
                    dev_index.display()
                )
            });
        }

        let started = Instant::now();
        build_index(&repo).unwrap_or_else(|error| {
            panic!("failed to build index for {}: {error:#}", repo.display())
        });
        let build_duration = started.elapsed();

        let report = run_benchmark(
            &repo,
            BenchmarkRunOptions {
                queries: None,
                build_duration: Some(build_duration),
            },
        )
        .unwrap_or_else(|error| panic!("failed to benchmark {}: {error:#}", repo.display()));

        println!("{}", render_benchmark_report(&report));
    }
}

fn is_repo_root(path: &Path) -> bool {
    PROJECT_MARKERS
        .iter()
        .any(|marker| path.join(marker).exists())
}
