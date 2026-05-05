use std::{env, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use thinindex::{
    indexer::find_repo_root,
    stats::{
        compute_agent_workflow_audit, compute_windows, current_unix_seconds, read_usage_events,
        render_agent_workflow_audit, render_hit_miss_graph, render_usage_table,
    },
};

#[derive(Debug, Parser)]
#[command(
    name = "wi-stats",
    version,
    about = "Show repo-local wi usage stats",
    after_help = "\
Notes:
  Reads local usage events from .dev_index/index.sqlite.
  Run `wi <query>`, `wi refs <query>`, `wi pack <query>`, or `wi impact <query>` first to record events.
  This is an advisory local report; it cannot detect external grep/find/ls/Read usage.
"
)]
struct Args {
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

    let events = read_usage_events(&root)?;

    if events.is_empty() {
        println!("No wi usage recorded yet for {}", root.display());
        println!("Run `wi <query>` to generate usage data.");
        return Ok(());
    }

    let now = current_unix_seconds();
    let windows = compute_windows(&events, now);

    println!("WI usage");
    println!();
    print!("{}", render_usage_table(&windows));
    println!();
    print!("{}", render_hit_miss_graph(&windows));
    println!();
    let audit = compute_agent_workflow_audit(&events);
    print!("{}", render_agent_workflow_audit(&audit));
    println!();
    println!("Recent misses");

    let misses: Vec<&str> = events
        .iter()
        .rev()
        .filter(|e| e.result_count == 0)
        .take(10)
        .map(|e| e.query.as_str())
        .collect();

    if misses.is_empty() {
        println!("None");
    } else {
        for query in misses {
            println!("- {query}");
        }
    }

    Ok(())
}
