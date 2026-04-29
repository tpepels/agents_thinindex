mod common;

use std::{fs, path::Path};

use common::{load_index_snapshot_from_sqlite, run_named_index_integrity_checks};
use thinindex::{
    bench::{BenchmarkRepo, BenchmarkRepoSet, load_benchmark_repo_set},
    indexer::build_index,
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
}
