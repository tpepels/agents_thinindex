use std::{env, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use thinindex::indexer::build_index;

#[derive(Debug, Parser)]
#[command(
    name = "build_index",
    version,
    about = "Build or update a repo-local thin code index"
)]
struct Args {
    #[arg(
        default_value = ".",
        help = "Directory inside the repository to index from"
    )]
    path: PathBuf,

    #[arg(short, long, help = "Print only the .dev_index path")]
    quiet: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    let start = if args.path.is_absolute() {
        args.path
    } else {
        env::current_dir()?.join(args.path)
    };

    let stats = build_index(&start)?;

    if args.quiet {
        println!("{}", stats.root.join(".dev_index").display());
        return Ok(());
    }

    println!("indexed: {}", stats.root.display());
    println!("scanned files: {}", stats.scanned_files);
    println!("changed files: {}", stats.changed_files);
    println!("deleted files: {}", stats.deleted_files);
    println!("records: {}", stats.records);

    if !stats.ctags_universal {
        println!("warning: ctags does not appear to be Universal Ctags");
    }

    Ok(())
}
