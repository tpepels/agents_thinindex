use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};

use crate::{
    agent_instructions::repository_search_block_is_current,
    indexer::index_is_fresh,
    licensing,
    model::INDEX_SCHEMA_VERSION,
    store::{DEV_INDEX_DIR, SQLITE_FILE, sqlite_path},
    support::{SupportLevel, support_entries_by_level, support_matrix},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorStatus {
    Ok,
    Warn,
    Fail,
}

impl DoctorStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCheck {
    pub status: DoctorStatus,
    pub name: &'static str,
    pub message: String,
    pub next: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    pub root: PathBuf,
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn has_failures(&self) -> bool {
        self.checks
            .iter()
            .any(|check| check.status == DoctorStatus::Fail)
    }

    pub fn has_warnings(&self) -> bool {
        self.checks
            .iter()
            .any(|check| check.status == DoctorStatus::Warn)
    }
}

pub fn run_doctor(root: &Path) -> DoctorReport {
    DoctorReport {
        root: root.to_path_buf(),
        checks: vec![
            check_index_exists(root),
            check_schema_current(root),
            check_index_fresh(root),
            check_parser_support(),
            check_agents_md(root),
            check_claude_md(root),
            check_dev_index_ignored(root),
            check_quality_state(root),
            check_license_state(),
            check_package_install_state(),
        ],
    }
}

pub fn render_doctor_report(report: &DoctorReport) -> String {
    let mut out = String::new();
    out.push_str("thinindex doctor\n");
    out.push_str(&format!("repo: {}\n", report.root.display()));
    out.push_str(&format!(
        "overall: {}\n\n",
        if report.has_failures() {
            "issues found"
        } else if report.has_warnings() {
            "ok with warnings"
        } else {
            "ok"
        }
    ));

    for check in &report.checks {
        out.push_str(&format!(
            "[{}] {}: {}\n",
            check.status.as_str(),
            check.name,
            check.message
        ));

        if let Some(next) = &check.next {
            out.push_str(&format!("  next: {next}\n"));
        }
    }

    out.push_str("\nNext steps:\n");
    if report.has_failures() || report.has_warnings() {
        out.push_str("- Fix any `fail` rows first, then rerun `wi doctor`.\n");
        out.push_str("- Run `build_index` after changing files or repository setup.\n");
        out.push_str("- Run `wi --help` for filters, examples, and subcommands.\n");
    } else {
        out.push_str("- Search with `wi <term>`.\n");
        out.push_str("- Use `wi pack <term>` before implementation work.\n");
        out.push_str("- Use `wi impact <term>` before editing a symbol or feature area.\n");
    }

    out
}

fn check_index_exists(root: &Path) -> DoctorCheck {
    let path = sqlite_path(root);
    if path.exists() {
        DoctorCheck::ok("index", format!("{DEV_INDEX_DIR}/{SQLITE_FILE} exists"))
    } else {
        DoctorCheck::fail(
            "index",
            format!("{DEV_INDEX_DIR}/{SQLITE_FILE} is missing"),
            "run `build_index` from the repository root",
        )
    }
}

fn check_schema_current(root: &Path) -> DoctorCheck {
    let path = sqlite_path(root);
    if !path.exists() {
        return DoctorCheck::fail(
            "schema",
            "schema cannot be checked because the SQLite index is missing",
            "run `build_index` to create the current schema",
        );
    }

    match read_schema_version(&path) {
        Ok(Some(version)) if version == INDEX_SCHEMA_VERSION => {
            DoctorCheck::ok("schema", format!("schema version {version} is current"))
        }
        Ok(Some(version)) => DoctorCheck::fail(
            "schema",
            format!("schema version {version} does not match {INDEX_SCHEMA_VERSION}"),
            "run `build_index` to rebuild the disposable local index",
        ),
        Ok(None) => DoctorCheck::fail(
            "schema",
            "schema version is missing from the SQLite meta table",
            "run `build_index` to rebuild the disposable local index",
        ),
        Err(error) => DoctorCheck::fail(
            "schema",
            format!("failed to read SQLite schema version: {error}"),
            "run `build_index`; if it still fails, remove `.dev_index/` and rebuild",
        ),
    }
}

fn check_index_fresh(root: &Path) -> DoctorCheck {
    match index_is_fresh(root) {
        Ok(true) => DoctorCheck::ok("freshness", "index is fresh"),
        Ok(false) => DoctorCheck::fail(
            "freshness",
            "index is stale; repository files changed since the last build",
            "run `build_index`, then retry the `wi` command",
        ),
        Err(error) => DoctorCheck::fail(
            "freshness",
            format!("freshness check could not run: {error:#}"),
            "run `build_index`, then rerun `wi doctor`",
        ),
    }
}

fn check_parser_support() -> DoctorCheck {
    DoctorCheck::ok(
        "parser support",
        format!(
            "built-in support matrix loaded: {} entries ({} supported, {} experimental, {} extras-backed, {} blocked); unsupported languages are skipped by design",
            support_matrix().len(),
            support_entries_by_level(SupportLevel::Supported).len(),
            support_entries_by_level(SupportLevel::Experimental).len(),
            support_entries_by_level(SupportLevel::ExtrasBacked).len(),
            support_entries_by_level(SupportLevel::Blocked).len(),
        ),
    )
}

fn check_agents_md(root: &Path) -> DoctorCheck {
    let path = root.join("AGENTS.md");
    if !path.exists() {
        return DoctorCheck::fail(
            "AGENTS.md",
            "AGENTS.md is missing",
            "run `wi-init` to write the canonical Repository search block",
        );
    }

    match fs::read_to_string(&path) {
        Ok(contents) if repository_search_block_is_current(&contents, "# AGENTS\n\n") => {
            DoctorCheck::ok(
                "AGENTS.md",
                "AGENTS.md has the current Repository search block",
            )
        }
        Ok(_) => DoctorCheck::fail(
            "AGENTS.md",
            "AGENTS.md is stale or missing the current Repository search block",
            "run `wi-init` to normalize AGENTS.md",
        ),
        Err(error) => DoctorCheck::fail(
            "AGENTS.md",
            format!("AGENTS.md could not be read: {error}"),
            "check file permissions, then rerun `wi doctor`",
        ),
    }
}

fn check_claude_md(root: &Path) -> DoctorCheck {
    let path = root.join("CLAUDE.md");
    if !path.exists() {
        return DoctorCheck::ok(
            "CLAUDE.md",
            "CLAUDE.md is absent; wi-init only normalizes it when present",
        );
    }

    match fs::read_to_string(&path) {
        Ok(contents) if repository_search_block_is_current(&contents, "") => DoctorCheck::ok(
            "CLAUDE.md",
            "CLAUDE.md has the current Repository search block",
        ),
        Ok(_) => DoctorCheck::fail(
            "CLAUDE.md",
            "CLAUDE.md is stale or missing the current Repository search block",
            "run `wi-init` to normalize existing CLAUDE.md",
        ),
        Err(error) => DoctorCheck::fail(
            "CLAUDE.md",
            format!("CLAUDE.md could not be read: {error}"),
            "check file permissions, then rerun `wi doctor`",
        ),
    }
}

fn check_dev_index_ignored(root: &Path) -> DoctorCheck {
    if dev_index_is_ignored(root) {
        DoctorCheck::ok("ignore", ".dev_index/ is ignored locally")
    } else {
        DoctorCheck::warn(
            "ignore",
            ".dev_index/ is not listed in .gitignore or .git/info/exclude",
            "add `.dev_index/` to .gitignore or run `wi-init` in repos that have a .gitignore",
        )
    }
}

fn check_quality_state(root: &Path) -> DoctorCheck {
    let quality_dir = root.join(".dev_index").join("quality");
    if quality_dir.exists() {
        DoctorCheck::ok(
            "quality",
            ".dev_index/quality/ exists for optional local quality reports",
        )
    } else {
        DoctorCheck::ok(
            "quality",
            "optional local quality reports are not present; normal wi commands do not require them",
        )
    }
}

fn check_license_state() -> DoctorCheck {
    let status = licensing::read_configured_license_status();
    DoctorCheck::ok(
        "license",
        format!(
            "edition={} state={} ({})",
            status.edition.as_str(),
            status.state.as_str(),
            status.reason
        ),
    )
}

fn check_package_install_state() -> DoctorCheck {
    match env::current_exe() {
        Ok(path) => DoctorCheck::ok("binary", format!("running wi from {}", path.display())),
        Err(error) => DoctorCheck::warn(
            "binary",
            format!("could not determine current executable path: {error}"),
            "verify `wi --version` works from your shell",
        ),
    }
}

fn read_schema_version(path: &Path) -> Result<Option<u32>> {
    let conn = Connection::open(path)?;
    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .optional()?;

    value
        .map(|value| value.parse::<u32>())
        .transpose()
        .map_err(Into::into)
}

fn dev_index_is_ignored(root: &Path) -> bool {
    [root.join(".gitignore"), root.join(".git/info/exclude")]
        .into_iter()
        .any(|path| file_ignores_dev_index(&path))
}

fn file_ignores_dev_index(path: &Path) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };

    contents.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == ".dev_index" || trimmed == ".dev_index/" || trimmed == "/.dev_index/"
    })
}

impl DoctorCheck {
    fn ok(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: DoctorStatus::Ok,
            name,
            message: message.into(),
            next: None,
        }
    }

    fn warn(name: &'static str, message: impl Into<String>, next: impl Into<String>) -> Self {
        Self {
            status: DoctorStatus::Warn,
            name,
            message: message.into(),
            next: Some(next.into()),
        }
    }

    fn fail(name: &'static str, message: impl Into<String>, next: impl Into<String>) -> Self {
        Self {
            status: DoctorStatus::Fail,
            name,
            message: message.into(),
            next: Some(next.into()),
        }
    }
}
