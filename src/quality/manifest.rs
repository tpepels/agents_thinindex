use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::bench::{BenchmarkRepoSet, load_benchmark_repo_set};

pub type QualityRepoSet = BenchmarkRepoSet;

pub fn load_quality_repo_set(test_repos_root: &Path) -> Result<QualityRepoSet> {
    load_benchmark_repo_set(test_repos_root)
}

pub fn quality_report_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".dev_index").join("quality")
}
