mod common;

use std::fs;

use assert_cmd::prelude::*;
use common::*;

#[test]
fn wi_init_writes_wi_and_agents_files() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "src/main.py",
        r#"
class PromptService:
    pass
"#,
    );

    wi_init_bin().current_dir(root).assert().success();

    let wi = fs::read_to_string(root.join("WI.md")).expect("read WI.md");
    assert!(
        wi.contains("Run `build_index` before discovery"),
        "unexpected WI.md content:\n{wi}"
    );

    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md");
    assert!(agents.contains("See WI.md for repository search/index usage."));

    assert!(root.join(".dev_index/index.jsonl").exists());
}

#[test]
fn wi_init_writes_thinindexignore() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    wi_init_bin().current_dir(root).assert().success();

    let ignore_path = root.join(".thinindexignore");
    assert!(
        ignore_path.exists(),
        ".thinindexignore should exist after wi-init"
    );

    let contents = fs::read_to_string(&ignore_path).expect("read .thinindexignore");
    assert!(
        contents.contains("node_modules/"),
        "expected bundled template content in .thinindexignore, got:\n{contents}"
    );
}

#[test]
fn wi_init_force_overwrites_thinindexignore() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    wi_init_bin().current_dir(root).assert().success();

    let ignore_path = root.join(".thinindexignore");
    fs::write(&ignore_path, "garbage_only\n").expect("overwrite with garbage");

    wi_init_bin()
        .current_dir(root)
        .arg("--force")
        .assert()
        .success();

    let contents = fs::read_to_string(&ignore_path).expect("read .thinindexignore");
    assert!(
        contents.contains("node_modules/"),
        "expected bundled template content after --force, got:\n{contents}"
    );
}

#[test]
fn wi_init_force_overwrites_wi_md() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "WI.md", "garbage\n");
    wi_init_bin()
        .current_dir(root)
        .arg("--force")
        .assert()
        .success();
    let wi = fs::read_to_string(root.join("WI.md")).unwrap();
    assert!(
        wi.contains("Run `build_index` before discovery"),
        "WI.md should be overwritten with template, got:\n{wi}"
    );
}

#[test]
fn wi_init_updates_gitignore_once() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(root, ".gitignore", "target/\n");

    wi_init_bin().current_dir(root).assert().success();
    wi_init_bin().current_dir(root).assert().success();

    let gitignore = fs::read_to_string(root.join(".gitignore")).unwrap();
    let count = gitignore
        .lines()
        .filter(|line| line.trim() == ".dev_index/")
        .count();

    assert_eq!(count, 1, ".dev_index/ should be added exactly once");
}

#[test]
fn wi_init_gitignore_existing_variants_are_not_duplicated() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    for existing in [".dev_index\n", ".dev_index/\n", "/.dev_index/\n"] {
        let repo = temp_repo();
        let root = repo.path();

        write_file(root, ".gitignore", existing);

        wi_init_bin().current_dir(root).assert().success();

        let gitignore = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert_eq!(
            gitignore, existing,
            "wi-init should not append .dev_index/ when variant already exists"
        );
    }
}

#[test]
fn wi_init_rolls_back_existing_files_on_failure() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "WI.md", "original WI\n");
    write_file(root, ".thinindexignore", "original ignore\n");
    write_file(root, "AGENTS.md", "original AGENTS\n");
    write_file(root, ".gitignore", "original .gitignore\n");

    let dev_index = root.join(".dev_index");
    if dev_index.exists() {
        fs::remove_dir_all(&dev_index).unwrap();
    }

    let output = wi_init_bin()
        .current_dir(root)
        .env("THININDEX_TEST_FAIL_WI_INIT_AFTER_WRITES", "1")
        .output()
        .expect("run wi-init with failure injection");

    assert!(!output.status.success(), "wi-init should fail");

    assert_eq!(
        fs::read_to_string(root.join("WI.md")).unwrap(),
        "original WI\n"
    );
    assert_eq!(
        fs::read_to_string(root.join(".thinindexignore")).unwrap(),
        "original ignore\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("AGENTS.md")).unwrap(),
        "original AGENTS\n"
    );
    assert_eq!(
        fs::read_to_string(root.join(".gitignore")).unwrap(),
        "original .gitignore\n"
    );

    assert!(
        !dev_index.exists(),
        ".dev_index should be removed on rollback when it did not exist before"
    );
}

#[test]
fn wi_init_rolls_back_new_files_on_failure() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    for name in ["WI.md", ".thinindexignore", "AGENTS.md", ".gitignore"] {
        let path = root.join(name);
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
    }

    let dev_index = root.join(".dev_index");
    if dev_index.exists() {
        fs::remove_dir_all(&dev_index).unwrap();
    }

    let output = wi_init_bin()
        .current_dir(root)
        .env("THININDEX_TEST_FAIL_WI_INIT_AFTER_WRITES", "1")
        .output()
        .expect("run wi-init with failure injection");

    assert!(!output.status.success(), "wi-init should fail");

    assert!(!root.join("WI.md").exists());
    assert!(!root.join(".thinindexignore").exists());
    assert!(!root.join("AGENTS.md").exists());
    assert!(!root.join(".gitignore").exists());
    assert!(!dev_index.exists());
}

#[test]
fn wi_init_preserves_existing_dev_index_on_failure() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    let dev_index = root.join(".dev_index");
    fs::create_dir_all(&dev_index).unwrap();
    fs::write(dev_index.join("dummy"), "keep").unwrap();

    let output = wi_init_bin()
        .current_dir(root)
        .env("THININDEX_TEST_FAIL_WI_INIT_AFTER_WRITES", "1")
        .output()
        .expect("run wi-init with failure injection");

    assert!(!output.status.success(), "wi-init should fail");
    assert!(dev_index.exists());
    assert_eq!(fs::read_to_string(dev_index.join("dummy")).unwrap(), "keep");
}

#[test]
fn wi_init_remove_leaves_repo_files_alone() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "WI.md", "original WI\n");
    write_file(root, ".thinindexignore", "original ignore\n");
    write_file(root, "AGENTS.md", "original AGENTS\n");
    write_file(root, ".gitignore", "original .gitignore\n");
    wi_init_bin()
        .current_dir(root)
        .arg("--remove")
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(root.join("WI.md")).unwrap(),
        "original WI\n"
    );
    assert_eq!(
        fs::read_to_string(root.join(".thinindexignore")).unwrap(),
        "original ignore\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("AGENTS.md")).unwrap(),
        "original AGENTS\n"
    );
    assert_eq!(
        fs::read_to_string(root.join(".gitignore")).unwrap(),
        "original .gitignore\n"
    );
}

#[test]
fn fixture_repo_remove_command_removes_index() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    wi_init_bin().current_dir(root).assert().success();

    assert!(root.join(".dev_index/index.jsonl").exists());

    wi_init_bin()
        .current_dir(root)
        .arg("--remove")
        .assert()
        .success();

    assert!(!root.join(".dev_index").exists());
    assert!(root.join("WI.md").exists());
    assert!(root.join("AGENTS.md").exists());
}

#[test]
fn fixture_keep_index_remove_preserves_dev_index() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = fixture_repo("sample_repo");
    let root = repo.path();

    wi_init_bin().current_dir(root).assert().success();

    assert!(root.join(".dev_index/index.jsonl").exists());

    wi_init_bin()
        .current_dir(root)
        .args(["--remove", "--keep-index"])
        .assert()
        .success();

    assert!(root.join(".dev_index/index.jsonl").exists());
}
