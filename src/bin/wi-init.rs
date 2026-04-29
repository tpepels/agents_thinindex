use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use thinindex::indexer::{build_index, find_repo_root};

const THININDEXIGNORE_TEMPLATE: &str = include_str!("../../templates/.thinindexignore");
const REPOSITORY_SEARCH_HEADING: &str = "## Repository search";
const REPOSITORY_SEARCH_BLOCK_VERSION: u32 = 2;
const REPOSITORY_SEARCH_BLOCK_VERSION_PREFIX: &str = "<!-- thinindex-repo-search-block: v";
const REPOSITORY_SEARCH_BLOCK: &str = "## Repository search

<!-- thinindex-repo-search-block: v2 -->

`wi` (\"where is\") is an index of *named* things in this repo — functions, classes, methods, CSS classes/variables, HTML ids, section headings, TODO/FIXME — not full text or paths. Use `wi <name>` whenever you'd grep for a name; use grep/rg/find directly for free text or paths.

- Run `wi --help` before your first repository search and treat its output as part of these instructions.
- Run `build_index` before broad discovery and after structural changes.
- Read only files returned by `wi` unless the result is insufficient.
- If `wi` misses a name you expect to exist, rerun `build_index` once and retry before grepping.";

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

    #[arg(long, help = "Overwrite .thinindexignore even if it exists")]
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

    write_thinindexignore(&root, args.force)?;
    update_gitignore(&root)?;
    update_agents_md(&root)?;
    update_claude_md(&root)?;

    if env::var_os("THININDEX_TEST_FAIL_WI_INIT_AFTER_WRITES").is_some() {
        anyhow::bail!("test failure after wi-init writes");
    }

    let stats = build_index(&root)?;

    rollback.commit();

    println!("initialized: {}", root.display());
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
    dev_index_path: PathBuf,
    dev_index_existed: bool,
    committed: bool,
}

impl InitRollback {
    fn capture(root: &Path) -> Result<Self> {
        let paths = [".thinindexignore", "AGENTS.md", "CLAUDE.md", ".gitignore"]
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
            dev_index_path,
            dev_index_existed,
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
        let updated = normalize_repository_search_block(&existing, "# AGENTS\n\n");

        if updated == existing {
            return Ok(());
        }

        fs::write(&path, updated).with_context(|| format!("failed to write {}", path.display()))?;
        println!("updated: {}", path.display());

        return Ok(());
    }

    fs::write(&path, format!("# AGENTS\n\n{REPOSITORY_SEARCH_BLOCK}\n"))
        .with_context(|| format!("failed to write {}", path.display()))?;

    println!("updated: {}", path.display());

    Ok(())
}

fn normalize_repository_search_block(existing: &str, empty_base_prefix: &str) -> String {
    // The version marker line is the contract: same-or-newer version means
    // we leave the file untouched, so users (or future wi-init releases) can
    // pin a block by writing a higher version.
    if let Some(existing_version) = parse_block_version(existing)
        && existing_version >= REPOSITORY_SEARCH_BLOCK_VERSION
    {
        return existing.to_string();
    }

    let without_sections = remove_repository_search_sections(existing);
    let without_legacy_markers = remove_legacy_repository_search_lines(&without_sections);
    let base = without_legacy_markers.trim_end();

    if base.is_empty() {
        format!("{empty_base_prefix}{REPOSITORY_SEARCH_BLOCK}\n")
    } else {
        format!("{base}\n\n{REPOSITORY_SEARCH_BLOCK}\n")
    }
}

fn parse_block_version(existing: &str) -> Option<u32> {
    for line in existing.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix(REPOSITORY_SEARCH_BLOCK_VERSION_PREFIX)
            && let Some(num) = rest.strip_suffix(" -->")
            && let Ok(version) = num.parse::<u32>()
        {
            return Some(version);
        }
    }

    None
}

fn remove_repository_search_sections(existing: &str) -> String {
    let mut kept = Vec::new();
    let mut lines = existing.lines().peekable();

    while let Some(line) = lines.next() {
        if line.trim() == REPOSITORY_SEARCH_HEADING {
            while let Some(next_line) = lines.peek() {
                if is_markdown_h1_or_h2(next_line) {
                    break;
                }

                lines.next();
            }
        } else {
            kept.push(line);
        }
    }

    kept.join("\n")
}

fn remove_legacy_repository_search_lines(existing: &str) -> String {
    existing
        .lines()
        .filter(|line| !is_legacy_repository_search_line(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_legacy_repository_search_line(line: &str) -> bool {
    let trimmed = line.trim();

    trimmed == "@WI.md"
        || trimmed.contains("See WI.md for repository search/index usage.")
        || trimmed.contains("See `WI.md` for repository search/index usage.")
        || trimmed
            .contains("Before broad repository discovery, run `build_index`, then use `wi <term>`")
}

fn is_markdown_h1_or_h2(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("# ") || trimmed.starts_with("## ")
}

fn update_claude_md(root: &Path) -> Result<()> {
    let path = root.join("CLAUDE.md");

    if !path.exists() {
        return Ok(());
    }

    let existing =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

    // If CLAUDE.md is just `@AGENTS.md`, the canonical block is already
    // reachable through the import — don't duplicate it here.
    // TODO - Make this more tolerant (could be \n, lower-case whatever, only if there's @AGENTS.md in the file don't update)
    if existing.trim() == "@AGENTS.md" {
        return Ok(());
    }

    let updated = normalize_repository_search_block(&existing, "");
    if updated == existing {
        return Ok(());
    }

    fs::write(&path, updated).with_context(|| format!("failed to write {}", path.display()))?;
    println!("updated: {}", path.display());

    Ok(())
}
