use std::{env, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use thinindex::{
    binary_state::{ensure_binary_matches_source, print_version_if_requested},
    indexer::find_repo_root,
    scorecard::{ScorecardOptions, render_scorecard, run_scorecard},
};

#[derive(Debug, Parser)]
#[command(
    name = "wi-scorecard",
    version,
    about = "Measure the local thinindex product value loop"
)]
struct Args {
    #[arg(
        long = "repo",
        default_value = ".",
        help = "Directory inside the repository"
    )]
    repo: PathBuf,

    #[arg(
        long,
        default_value = "build_index",
        help = "Query used for search, refs, pack, and impact checks"
    )]
    query: String,
}

fn main() {
    if print_version_if_requested("wi-scorecard") {
        return;
    }

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
    ensure_binary_matches_source(&root, "wi-scorecard")?;
    let report = run_scorecard(&root, &ScorecardOptions { query: args.query })?;

    print!("{}", render_scorecard(&report));
    Ok(())
}
