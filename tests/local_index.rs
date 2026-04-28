mod common;

use std::{fs, path::Path};

use common::run_named_index_integrity_checks;
use thinindex::indexer::build_index;

#[test]
#[ignore = "rebuilds the developer-local .dev_index for the thinindex repo; run with: cargo test --test local_index -- --ignored"]
fn local_index_passes_shared_integrity_checks() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dev_index = root.join(".dev_index");

    if dev_index.exists() {
        fs::remove_dir_all(&dev_index).unwrap_or_else(|error| {
            panic!(
                "failed to remove local .dev_index before rebuild: {}\nerror: {error}",
                dev_index.display()
            )
        });
    }

    build_index(root).unwrap_or_else(|error| {
        panic!(
            "failed to rebuild local thinindex index for {}\nerror: {error:#}",
            root.display()
        )
    });

    let index_path = dev_index.join("index.jsonl");
    let index = fs::read_to_string(&index_path).unwrap_or_else(|error| {
        panic!(
            "failed to read rebuilt local index at {}\nerror: {error}",
            index_path.display()
        )
    });

    run_named_index_integrity_checks(
        "thinindex local repo",
        &index,
        &[
            "src/indexer.rs",
            "src/search.rs",
            "src/bin/wi.rs",
            "src/bin/wi-init.rs",
            "src/wi_cli.rs",
        ],
    );
}
