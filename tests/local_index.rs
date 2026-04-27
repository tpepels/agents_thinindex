use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use thinindex::indexer::build_index;

static LOCAL_INDEX: OnceLock<String> = OnceLock::new();

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn local_index_path() -> PathBuf {
    repo_root().join(".dev_index/index.jsonl")
}

fn rebuilt_local_index() -> &'static str {
    LOCAL_INDEX.get_or_init(|| {
        let dev_index = repo_root().join(".dev_index");

        if dev_index.exists() {
            fs::remove_dir_all(&dev_index).unwrap_or_else(|error| {
                panic!(
                    "failed to remove local index before rebuild: {}\nerror: {error}",
                    dev_index.display()
                )
            });
        }

        build_index(repo_root()).unwrap_or_else(|error| {
            panic!(
                "failed to rebuild local thinindex index for {}\nerror: {error:#}",
                repo_root().display()
            )
        });

        let path = local_index_path();

        fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read rebuilt local thinindex index at {}\nerror: {error}",
                path.display()
            )
        })
    })
}

#[test]
#[ignore = "checks the developer-local .dev_index for the thinindex repo; run with: cargo test --test local_index -- --ignored"]
fn local_index_has_no_duplicate_locations() {
    let index = rebuilt_local_index();

    let mut seen = BTreeSet::new();
    let mut duplicates = Vec::new();

    for line in index.lines().filter(|line| !line.trim().is_empty()) {
        let record: serde_json::Value =
            serde_json::from_str(line).expect("parse index record JSON");

        let path = record
            .get("path")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();

        let line_no = record
            .get("line")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);

        let col = record
            .get("col")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);

        let key = (path, line_no, col);

        if !seen.insert(key) {
            duplicates.push(line.to_string());
        }
    }

    assert!(
        duplicates.is_empty(),
        "rebuilt local .dev_index should not contain duplicate path+line+col records:\n{}",
        duplicates.join("\n")
    );
}

#[test]
#[ignore = "checks the developer-local .dev_index for the thinindex repo; run with: cargo test --test local_index -- --ignored"]
fn local_index_has_required_fields_for_every_record() {
    let index = rebuilt_local_index();

    let mut invalid = Vec::new();

    for line in index.lines().filter(|line| !line.trim().is_empty()) {
        let record: serde_json::Value =
            serde_json::from_str(line).expect("parse index record JSON");

        for field in [
            "path", "line", "col", "lang", "kind", "name", "text", "source",
        ] {
            if record.get(field).is_none() {
                invalid.push(format!("missing {field}: {line}"));
            }
        }
    }

    assert!(
        invalid.is_empty(),
        "rebuilt local .dev_index contains malformed records:\n{}",
        invalid.join("\n")
    );
}

#[test]
#[ignore = "checks the developer-local .dev_index for the thinindex repo; run with: cargo test --test local_index -- --ignored"]
fn local_index_does_not_include_dev_index_itself() {
    let index = rebuilt_local_index();

    let bad: Vec<_> = index
        .lines()
        .filter(|line| line.contains(".dev_index/"))
        .collect();

    assert!(
        bad.is_empty(),
        "rebuilt local index should not index .dev_index itself:\n{}",
        bad.join("\n")
    );
}

#[test]
#[ignore = "checks the developer-local .dev_index for the thinindex repo; run with: cargo test --test local_index -- --ignored"]
fn local_index_has_expected_thinindex_landmarks() {
    let index = rebuilt_local_index();

    for required in [
        "src/indexer.rs",
        "src/search.rs",
        "src/bin/wi.rs",
        "src/bin/wi-init.rs",
        "src/wi_cli.rs",
    ] {
        assert!(
            index.contains(required),
            "rebuilt local .dev_index should contain {required}"
        );
    }
}
