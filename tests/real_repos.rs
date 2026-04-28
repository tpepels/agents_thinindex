mod common;

use std::{
    fs,
    path::{Path, PathBuf},
};

use common::run_named_index_integrity_checks;
use thinindex::indexer::build_index;

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
#[ignore = "rebuilds .dev_index for every repo under test_repos/; run with: cargo test --test real_repos -- --ignored"]
fn real_repos_pass_shared_integrity_checks() {
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

    let names: Vec<String> = repos
        .iter()
        .map(|path| {
            path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string())
        })
        .collect();

    println!("real_repos testing {} repo(s):", repos.len());
    for name in &names {
        println!("  - {name}");
    }

    for (path, name) in repos.iter().zip(names.iter()) {
        let dev_index = path.join(".dev_index");

        if dev_index.exists() {
            fs::remove_dir_all(&dev_index).unwrap_or_else(|error| {
                panic!(
                    "failed to remove .dev_index for {}: {error}",
                    dev_index.display()
                )
            });
        }

        build_index(path).unwrap_or_else(|error| {
            panic!("failed to build index for {}: {error:#}", path.display())
        });

        let index_path = dev_index.join("index.jsonl");
        let index = fs::read_to_string(&index_path).unwrap_or_else(|error| {
            panic!(
                "failed to read rebuilt index at {}: {error}",
                index_path.display()
            )
        });

        run_named_index_integrity_checks(name, &index, &[]);
    }
}

fn is_repo_root(path: &Path) -> bool {
    PROJECT_MARKERS
        .iter()
        .any(|marker| path.join(marker).exists())
}
