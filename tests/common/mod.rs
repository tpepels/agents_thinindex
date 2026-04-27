#![allow(dead_code)]

use std::{fs, path::Path, process::Command};

use assert_cmd::prelude::*;
use tempfile::TempDir;

pub fn has_ctags() -> bool {
    Command::new("ctags")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn temp_repo() -> TempDir {
    let temp = tempfile::tempdir().expect("create tempdir");
    fs::create_dir_all(temp.path().join(".git")).expect("create .git marker");
    temp
}

pub fn write_file(root: &Path, relpath: &str, text: &str) {
    let path = root.join(relpath);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }

    fs::write(path, text).expect("write test file");
}

pub fn build_index_bin() -> Command {
    Command::cargo_bin("build_index").expect("build_index binary")
}

pub fn wi_bin() -> Command {
    Command::cargo_bin("wi").expect("wi binary")
}

pub fn wi_init_bin() -> Command {
    Command::cargo_bin("wi-init").expect("wi-init binary")
}

pub fn wi_stats_bin() -> Command {
    Command::cargo_bin("wi-stats").expect("wi-stats binary")
}

pub fn run_build(root: &Path) -> String {
    let output = build_index_bin()
        .current_dir(root)
        .output()
        .expect("run build_index");

    assert!(
        output.status.success(),
        "build_index failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn run_wi(root: &Path, args: &[&str]) -> String {
    let output = wi_bin()
        .current_dir(root)
        .args(args)
        .output()
        .expect("run wi");

    assert!(
        output.status.success(),
        "wi failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn fixture_repo(name: &str) -> TempDir {
    let temp = tempfile::tempdir().expect("create tempdir");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);

    copy_dir_all(&source, temp.path()).expect("copy fixture repo");

    temp
}

fn copy_dir_all(source: &Path, target: &Path) -> std::io::Result<()> {
    fs::create_dir_all(target)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }

    Ok(())
}
