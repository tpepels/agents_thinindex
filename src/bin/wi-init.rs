use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use thinindex::indexer::{build_index, find_repo_root};

const WI_TEMPLATE: &str = include_str!("../../templates/WI.md");
const THINDEXIGNORE_TEMPLATE: &str = include_str!("../../templates/.thinindexignore");
const AGENTS_MARKER: &str = "See WI.md for repository search/index usage.";

#[derive(Debug, Parser)]
#[command(
    name = "wi-init",
    version,
    about = "Initialize or remove thinindex/wi agent usage for one repository"
)]
struct Args {
    #[arg(
        default_value = ".",
        help = "Directory inside the repository to initialize"
    )]
    path: PathBuf,

    #[arg(long, alias = "disable", help = "Remove this repo's .dev_index")]
    remove: bool,

    #[arg(long, help = "With --remove, keep .dev_index")]
    keep_index: bool,

    #[arg(
        long,
        help = "Overwrite WI.md and .thinindexignore even if they already exist"
    )]
    force: bool,
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

    let root = find_repo_root(&start)?;

    if args.remove {
        remove_repo(&root, args.keep_index)?;
        return Ok(());
    }

    write_wi_md(&root, args.force)?;
    write_thinindexignore(&root, args.force)?;
    update_gitignore(&root)?;
    update_agents_md(&root)?;

    let stats = build_index(&root)?;

    println!("initialized: {}", root.display());
    println!("wrote: {}", root.join("WI.md").display());
    println!("wrote: {}", root.join(".thinindexignore").display());
    println!("updated: {}", root.join("AGENTS.md").display());
    println!("records: {}", stats.records);

    Ok(())
}

fn remove_repo(root: &Path, keep_index: bool) -> Result<()> {
    if !keep_index {
        let index_dir = root.join(".dev_index");

        if index_dir.exists() {
            fs::remove_dir_all(&index_dir)
                .with_context(|| format!("failed to remove {}", index_dir.display()))?;
        }

        println!("removed: {}", index_dir.display());
    } else {
        println!("kept: {}", root.join(".dev_index").display());
    }

    println!("removed thinindex setup from: {}", root.display());

    Ok(())
}

fn write_wi_md(root: &Path, force: bool) -> Result<()> {
    let path = root.join("WI.md");

    if path.exists() && !force {
        println!("exists: {} (use --force to overwrite)", path.display());
        return Ok(());
    }

    fs::write(&path, WI_TEMPLATE).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

fn write_thinindexignore(root: &Path, force: bool) -> Result<()> {
    let path = root.join(".thinindexignore");

    if path.exists() && !force {
        println!("exists: {} (use --force to overwrite)", path.display());
        return Ok(());
    }

    fs::write(&path, THINDEXIGNORE_TEMPLATE)
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

fn update_gitignore(root: &Path) -> Result<()> {
    let path = root.join(".gitignore");

    if !path.exists() {
        return Ok(());
    }

    let existing =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

    let already_ignored = existing.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == ".dev_index" || trimmed == ".dev_index/" || trimmed == "/.dev_index/"
    });

    if already_ignored {
        return Ok(());
    }

    let mut file = OpenOptions::new()
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to open {}", path.display()))?;

    if !existing.ends_with('\n') {
        writeln!(file)?;
    }

    writeln!(file)?;
    writeln!(file, "# thinindex")?;
    writeln!(file, ".dev_index/")?;

    println!("updated: {}", path.display());

    Ok(())
}

fn update_agents_md(root: &Path) -> Result<()> {
    let path = root.join("AGENTS.md");

    if path.exists() {
        let existing = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        if existing.contains(AGENTS_MARKER) {
            return Ok(());
        }

        let mut file = OpenOptions::new()
            .append(true)
            .open(&path)
            .with_context(|| format!("failed to open {}", path.display()))?;

        if !existing.ends_with('\n') {
            writeln!(file)?;
        }

        writeln!(file)?;
        writeln!(file, "## Repository search")?;
        writeln!(file)?;
        writeln!(file, "{AGENTS_MARKER}")?;

        return Ok(());
    }

    fs::write(&path, format!("# AGENTS\n\n{AGENTS_MARKER}\n"))
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}
