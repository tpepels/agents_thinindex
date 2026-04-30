use std::{env, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use thinindex::indexer::{FileSizeAction, build_index};

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

    #[arg(long, help = "Print build performance and scale diagnostics")]
    stats: bool,
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

    if let Some(message) = stats.reset_message {
        println!("{message}");
    }

    if args.quiet {
        println!("{}", stats.root.join(".dev_index").display());
        return Ok(());
    }

    println!("indexed: {}", stats.root.display());
    println!("scanned files: {}", stats.scanned_files);
    println!("changed files: {}", stats.changed_files);
    println!("deleted files: {}", stats.deleted_files);
    println!("records: {}", stats.records);
    print_large_file_warnings(&stats.large_files);

    if args.stats {
        print_stats(&stats);
    }

    Ok(())
}

fn print_large_file_warnings(warnings: &[thinindex::indexer::FileSizeWarning]) {
    let skipped: Vec<_> = warnings
        .iter()
        .filter(|warning| warning.action == FileSizeAction::Skipped)
        .collect();

    if skipped.is_empty() {
        return;
    }

    println!("skipped large files: {}", skipped.len());
    for warning in skipped.iter().take(5) {
        println!(
            "warning: skipped large file {} ({} bytes > {} byte cap)",
            warning.path, warning.size, warning.threshold
        );
    }
}

fn print_stats(stats: &thinindex::indexer::BuildStats) {
    println!("refs: {}", stats.refs);
    println!("dependencies: {}", stats.dependencies);
    println!("unchanged files: {}", stats.unchanged_files);
    println!("total file bytes: {}", stats.total_file_bytes);
    println!(
        "max indexed file bytes: {}",
        thinindex::indexer::MAX_INDEXED_FILE_BYTES
    );
    println!(
        "large file warning bytes: {}",
        thinindex::indexer::LARGE_FILE_WARNING_BYTES
    );

    println!("performance:");
    println!("  discover ms: {}", stats.timings.discover.as_millis());
    println!(
        "  change detection ms: {}",
        stats.timings.change_detection.as_millis()
    );
    println!("  parse ms: {}", stats.timings.parse.as_millis());
    println!(
        "  dependencies ms: {}",
        stats.timings.dependencies.as_millis()
    );
    println!("  refs ms: {}", stats.timings.refs.as_millis());
    println!("  save ms: {}", stats.timings.save.as_millis());
    println!("  total ms: {}", stats.timings.total.as_millis());

    println!("large files:");
    if stats.large_files.is_empty() {
        println!("  none");
    } else {
        for warning in &stats.large_files {
            println!(
                "  {} {} bytes action={:?} threshold={}",
                warning.path, warning.size, warning.action, warning.threshold
            );
        }
    }

    println!(
        "sqlite tuning: journal_mode=WAL synchronous=NORMAL temp_store=MEMORY cache_size=-20000"
    );
}
