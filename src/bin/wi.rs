use std::{env, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use thinindex::{
    indexer::find_repo_root,
    search::{format_result, search, SearchOptions},
};

#[derive(Debug, Parser)]
#[command(name = "wi", version, about = "Search the repo-local thin code index")]
struct Args {
    #[arg(help = "Search query")]
    query: String,

    #[arg(
        long = "type",
        help = "Filter by record kind, e.g. function, class, css_class"
    )]
    kind: Option<String>,

    #[arg(
        long,
        help = "Filter by language, e.g. py, ts, tsx, js, jsx, css, html, md"
    )]
    lang: Option<String>,

    #[arg(long, help = "Filter by path substring")]
    path: Option<String>,

    #[arg(long, help = "Filter by source, e.g. ctags or extras")]
    source: Option<String>,

    #[arg(long, default_value_t = 30, help = "Maximum number of results")]
    limit: usize,

    #[arg(short, long, help = "Print verbose results")]
    verbose: bool,

    #[arg(
        long = "repo",
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

    let options = SearchOptions {
        kind: args.kind,
        lang: args.lang,
        path: args.path,
        source: args.source,
        limit: args.limit,
        verbose: args.verbose,
    };

    let results = search(&root, &args.query, &options)?;

    for result in results {
        println!("{}", format_result(&result, options.verbose));
    }

    Ok(())
}
