use std::{env, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use thinindex::{
    indexer::find_repo_root,
    search::{format_result, search, SearchOptions},
    stats::{self, UsageEvent},
};

#[derive(Debug, Parser)]
#[command(name = "wi", version, about = "Search the repo-local thin code index")]
struct Args {
    #[arg(help = "Search query")]
    query: String,

    #[arg(
        short = 't',
        value_name = "KIND",
        help = "Filter by record kind, e.g. function, class, css_class"
    )]
    kind: Option<String>,

    #[arg(
        short = 'l',
        value_name = "LANG",
        help = "Filter by language, e.g. py, ts, tsx, js, jsx, css, html, md"
    )]
    lang: Option<String>,

    #[arg(short = 'p', value_name = "PATH", help = "Filter by path substring")]
    path: Option<String>,

    #[arg(
        short = 's',
        value_name = "SOURCE",
        help = "Filter by source, e.g. ctags or extras"
    )]
    source: Option<String>,

    #[arg(short = 'n', value_name = "N", help = "Maximum number of results")]
    limit: Option<usize>,

    #[arg(short = 'v', help = "Print verbose results")]
    verbose: bool,

    #[arg(
        short = 'r',
        value_name = "REPO",
        default_value = ".",
        help = "Directory inside the repository"
    )]
    repo: PathBuf,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    let start = if args.repo.is_absolute() {
        args.repo
    } else {
        env::current_dir()?.join(args.repo)
    };

    let root = find_repo_root(&start)?;

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

    let results = search(&root, &args.query, &options)?;
    let result_count = results.len();

    for result in &results {
        println!("{}", format_result(result, options.verbose));
    }

    let event = UsageEvent {
        ts: stats::current_unix_seconds(),
        query: args.query.clone(),
        query_len: args.query.chars().count(),
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
