use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use thinindex::indexer::{build_index, find_repo_root};
use thinindex::wi_cli::wi_help_text;

const THININDEXIGNORE_TEMPLATE: &str = include_str!("../../templates/.thinindexignore");
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

    #[arg(long, help = "Overwrite .thinindexignore even if it exist")]
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

    let mut rollback = InitRollback::capture(&root)?;

    write_wi_md(&root, args.force)?;
    write_thinindexignore(&root, args.force)?;
    update_gitignore(&root)?;
    update_agents_md(&root)?;

    if env::var_os("THININDEX_TEST_FAIL_WI_INIT_AFTER_WRITES").is_some() {
        anyhow::bail!("test failure after wi-init writes");
    }

    let stats = build_index(&root)?;

    rollback.commit();

    println!("initialized: {}", root.display());
    println!("wrote: {}", root.join("WI.md").display());
    println!("wrote: {}", root.join(".thinindexignore").display());
    println!("updated: {}", root.join("AGENTS.md").display());
    println!("records: {}", stats.records);

    Ok(())
}

#[derive(Debug)]
struct FileSnapshot {
    path: PathBuf,
    existed: bool,
    content: Option<Vec<u8>>,
}

impl FileSnapshot {
    fn capture(path: PathBuf) -> Result<Self> {
        if path.exists() {
            let content = fs::read(&path)
                .with_context(|| format!("failed to snapshot {}", path.display()))?;

            Ok(Self {
                path,
                existed: true,
                content: Some(content),
            })
        } else {
            Ok(Self {
                path,
                existed: false,
                content: None,
            })
        }
    }

    fn restore(&self) -> Result<()> {
        if self.existed {
            fs::write(
                &self.path,
                self.content
                    .as_ref()
                    .expect("existing snapshot should have content"),
            )
            .with_context(|| format!("failed to restore {}", self.path.display()))?;
        } else if self.path.exists() {
            fs::remove_file(&self.path)
                .with_context(|| format!("failed to remove {}", self.path.display()))?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct InitRollback {
    snapshots: Vec<FileSnapshot>,
    dev_index_existed: bool,
    dev_index_path: PathBuf,
    committed: bool,
}

impl InitRollback {
    fn capture(root: &Path) -> Result<Self> {
        let paths = ["WI.md", ".thinindexignore", "AGENTS.md", ".gitignore"]
            .iter()
            .map(|name| root.join(name))
            .collect::<Vec<_>>();

        let mut snapshots = Vec::new();

        for path in paths {
            snapshots.push(FileSnapshot::capture(path)?);
        }

        let dev_index_path = root.join(".dev_index");
        let dev_index_existed = dev_index_path.exists();

        Ok(Self {
            snapshots,
            dev_index_existed,
            dev_index_path,
            committed: false,
        })
    }

    fn commit(&mut self) {
        self.committed = true;
    }

    fn rollback(&self) {
        if self.committed {
            return;
        }

        for snapshot in self.snapshots.iter().rev() {
            if let Err(error) = snapshot.restore() {
                eprintln!(
                    "warning: rollback failed for {}: {error:#}",
                    snapshot.path.display()
                );
            }
        }

        if !self.dev_index_existed
            && self.dev_index_path.exists()
            && let Err(error) = fs::remove_dir_all(&self.dev_index_path)
        {
            eprintln!(
                "warning: rollback failed for {}: {error:#}",
                self.dev_index_path.display()
            );
        }
    }
}

impl Drop for InitRollback {
    fn drop(&mut self) {
        self.rollback();
    }
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

fn render_wi_md() -> String {
    wi_help_text()
}

fn write_wi_md(root: &Path, _force: bool) -> Result<()> {
    let path = root.join("WI.md");

    fs::write(&path, render_wi_md())
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

fn write_thinindexignore(root: &Path, force: bool) -> Result<()> {
    let path = root.join(".thinindexignore");

    if path.exists() && !force {
        println!("exists: {} (use --force to overwrite)", path.display());
        return Ok(());
    }

    fs::write(&path, THININDEXIGNORE_TEMPLATE)
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
