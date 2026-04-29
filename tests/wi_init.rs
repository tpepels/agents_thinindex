mod common;

use std::fs;

use assert_cmd::prelude::*;
use common::*;

const AGENTS_REPOSITORY_SEARCH_HEADING: &str = "## Repository search";
const AGENTS_REPOSITORY_SEARCH_BLOCK: &str = "\
## Repository search

- Before broad repository discovery, run `build_index`.
- Run `wi --help` if you need search filters or examples.
- Use `wi <term>` before grep/find/ls/Read to locate code.
- Read only files returned by `wi` unless the result is insufficient.
- If `wi` returns no useful result, rerun `build_index` once and retry.
- Fall back to grep/find/Read only after that retry fails.";

// WI.md and related assertions removed as WI.md is no longer generated or required.

fn assert_agents_has_canonical_repository_search_block(agents: &str) {
    assert!(
        agents.contains(AGENTS_REPOSITORY_SEARCH_BLOCK),
        "AGENTS.md should contain canonical repository search block, got:\n{agents}"
    );

    let heading_count = agents
        .lines()
        .filter(|line| line.trim() == AGENTS_REPOSITORY_SEARCH_HEADING)
        .count();

    assert_eq!(
        heading_count, 1,
        "AGENTS.md should contain exactly one repository search heading, got:\n{agents}"
    );

    assert!(
        !agents.contains("See WI.md for repository search/index usage."),
        "AGENTS.md should not contain old unbackticked weak marker, got:\n{agents}"
    );

    assert!(
        !agents.contains("See `WI.md` for repository search/index usage."),
        "AGENTS.md should not contain old backticked weak marker, got:\n{agents}"
    );
}

// WI.md generation and related tests removed.

#[test]
fn wi_init_does_not_create_wi_md() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    wi_init_bin().current_dir(root).assert().success();

    assert!(
        !root.join("WI.md").exists(),
        "wi-init must not create WI.md"
    );
}

#[test]
fn wi_init_creates_agents_md_when_absent() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    wi_init_bin().current_dir(root).assert().success();

    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md");
    assert_agents_has_canonical_repository_search_block(&agents);
}

#[test]
fn wi_init_appends_canonical_agents_block_to_existing_file() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "AGENTS.md", "# AGENTS\n\n- Existing rule.\n");

    wi_init_bin().current_dir(root).assert().success();

    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md");

    assert!(
        agents.contains("- Existing rule."),
        "existing AGENTS.md content should be preserved, got:\n{agents}"
    );
    assert_agents_has_canonical_repository_search_block(&agents);
}

#[test]
fn wi_init_normalizes_legacy_agents_marker() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "AGENTS.md",
        "# AGENTS\n\n## Repository search\n\nSee WI.md for repository search/index usage.\n",
    );

    wi_init_bin().current_dir(root).assert().success();

    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md");
    assert_agents_has_canonical_repository_search_block(&agents);
}

#[test]
fn wi_init_normalizes_backticked_legacy_agents_marker() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "AGENTS.md",
        "# AGENTS\n\n## Repository search\n\nSee `WI.md` for repository search/index usage.\n",
    );

    wi_init_bin().current_dir(root).assert().success();

    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md");
    assert_agents_has_canonical_repository_search_block(&agents);
}

#[test]
fn wi_init_normalizes_at_wi_marker_in_agents_md() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "AGENTS.md", "# AGENTS\n\n@WI.md\n");

    wi_init_bin().current_dir(root).assert().success();

    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md");

    assert!(
        !agents.lines().any(|line| line.trim() == "@WI.md"),
        "AGENTS.md should not rely on @WI.md as the only repository-search instruction, got:\n{agents}"
    );
    assert_agents_has_canonical_repository_search_block(&agents);
}

#[test]
fn wi_init_does_not_duplicate_agents_repository_search_block() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(
        root,
        "AGENTS.md",
        &format!("# AGENTS\n\n{AGENTS_REPOSITORY_SEARCH_BLOCK}\n"),
    );

    wi_init_bin().current_dir(root).assert().success();
    wi_init_bin().current_dir(root).assert().success();

    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md");
    assert_agents_has_canonical_repository_search_block(&agents);

    let block_count = agents.matches(AGENTS_REPOSITORY_SEARCH_BLOCK).count();
    assert_eq!(
        block_count, 1,
        "canonical repository search block should appear exactly once, got:\n{agents}"
    );
}

#[test]
fn wi_init_preserves_at_agents_md_import_in_claude_md() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "CLAUDE.md", "@AGENTS.md\n");

    wi_init_bin().current_dir(root).assert().success();

    let claude = fs::read_to_string(root.join("CLAUDE.md")).expect("read CLAUDE.md");

    assert!(
        claude.contains("@AGENTS.md"),
        "@AGENTS.md import directive should be preserved, got:\n{claude}"
    );
    assert!(
        !claude.starts_with("# AGENTS"),
        "CLAUDE.md should not be rewritten with a `# AGENTS` H1, got:\n{claude}"
    );
    assert_agents_has_canonical_repository_search_block(&claude);
}

#[test]
fn wi_init_does_not_emit_agents_h1_when_claude_md_filters_to_empty() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    // @WI.md is a legacy marker that gets filtered out, leaving an empty base.
    write_file(root, "CLAUDE.md", "@WI.md\n");

    wi_init_bin().current_dir(root).assert().success();

    let claude = fs::read_to_string(root.join("CLAUDE.md")).expect("read CLAUDE.md");

    assert_agents_has_canonical_repository_search_block(&claude);
    assert!(
        !claude.contains("# AGENTS"),
        "CLAUDE.md must never contain a `# AGENTS` heading, got:\n{claude}"
    );
    assert_eq!(
        claude,
        format!("{AGENTS_REPOSITORY_SEARCH_BLOCK}\n"),
        "CLAUDE.md with empty base should be exactly the canonical block plus trailing newline, got:\n{claude}"
    );
}

#[test]
fn wi_init_does_not_create_claude_md() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    wi_init_bin().current_dir(root).assert().success();

    assert!(
        !root.join("CLAUDE.md").exists(),
        "wi-init must not create CLAUDE.md when it does not already exist"
    );
}

#[test]
fn wi_init_does_not_duplicate_claude_md_marker() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "CLAUDE.md", "@AGENTS.md\n@WI.md\n");

    wi_init_bin().current_dir(root).assert().success();
    wi_init_bin().current_dir(root).assert().success();

    let claude = fs::read_to_string(root.join("CLAUDE.md")).expect("read CLAUDE.md");
    assert_agents_has_canonical_repository_search_block(&claude);
}

#[test]
fn wi_init_rolls_back_existing_claude_md_on_failure() {
    if !has_ctags() {
        eprintln!("skipping: ctags unavailable");
        return;
    }

    let repo = temp_repo();
    let root = repo.path();

    write_file(root, "CLAUDE.md", "@AGENTS.md\n");

    let output = wi_init_bin()
        .current_dir(root)
        .env("THININDEX_TEST_FAIL_WI_INIT_AFTER_WRITES", "1")
        .output()
        .expect("run wi-init with failure injection");

    assert!(!output.status.success(), "wi-init should fail");

    assert_eq!(
        fs::read_to_string(root.join("CLAUDE.md")).unwrap(),
        "@AGENTS.md\n",
        "CLAUDE.md should be restored to its pre-init contents on rollback"
    );
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

    // WI.md removed
    write_file(root, ".thinindexignore", "original ignore\n");
    write_file(root, "AGENTS.md", "original AGENTS\n");
    write_file(root, "CLAUDE.md", "original CLAUDE\n");
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

    // WI.md removed
    assert_eq!(
        fs::read_to_string(root.join(".thinindexignore")).unwrap(),
        "original ignore\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("AGENTS.md")).unwrap(),
        "original AGENTS\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("CLAUDE.md")).unwrap(),
        "original CLAUDE\n"
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

    for name in [
        "WI.md",
        ".thinindexignore",
        "AGENTS.md",
        "CLAUDE.md",
        ".gitignore",
    ] {
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

    // WI.md removed
    assert!(!root.join(".thinindexignore").exists());
    assert!(!root.join("AGENTS.md").exists());
    assert!(!root.join("CLAUDE.md").exists());
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

    // WI.md removed
    write_file(root, ".thinindexignore", "original ignore\n");
    write_file(root, "AGENTS.md", "original AGENTS\n");
    write_file(root, "CLAUDE.md", "original CLAUDE\n");
    write_file(root, ".gitignore", "original .gitignore\n");

    wi_init_bin()
        .current_dir(root)
        .arg("--remove")
        .assert()
        .success();

    // WI.md removed
    assert_eq!(
        fs::read_to_string(root.join(".thinindexignore")).unwrap(),
        "original ignore\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("AGENTS.md")).unwrap(),
        "original AGENTS\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("CLAUDE.md")).unwrap(),
        "original CLAUDE\n"
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
    // WI.md removed
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
