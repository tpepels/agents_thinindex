use std::env;

use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use thinindex::wi_cli::WiArgs;
use thinindex::{
    indexer::{find_repo_root, index_is_fresh},
    search::{SearchOptions, format_result, search},
    stats::{self, UsageEvent},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = WiArgs::parse();
    let query = args.query.clone();
    let repo = env::current_dir()?;
    let root = find_repo_root(&repo)?;

    match index_is_fresh(&root) {
        Ok(false) => bail!("index is stale; run `build_index`"),
        Ok(true) => {}
        Err(error) => bail!("{error:#}"),
    }

    let used_type = args.kind.is_some();
    let used_lang = args.lang.is_some();
    let used_path = args.path.is_some();
    let used_limit = args.limit.is_some();
    let limit = args.limit.unwrap_or(30);

    let options = SearchOptions {
        kind: args.kind,
        lang: args.lang,
        path: args.path,
        source: args.source,
        limit,
        verbose: args.verbose,
    };

    let results = search(&root, &query, &options)?;
    let result_count = results.len();

    for result in &results {
        println!("{}", format_result(result, options.verbose));
    }

    let event = UsageEvent {
        ts: stats::current_unix_seconds(),
        query: query.clone(),
        query_len: query.chars().count(),
        result_count,
        hit: result_count > 0,
        used_type,
        used_lang,
        used_path,
        used_limit,
        repo: root.display().to_string(),
        indexed_files: stats::manifest_indexed_files(&root),
    };

    if let Err(error) = stats::append_usage_event(&root, &event) {
        eprintln!("warning: failed to log wi usage: {error:#}");
    }

    Ok(())
}
