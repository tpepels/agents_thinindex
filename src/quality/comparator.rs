use std::{
    io::ErrorKind,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparatorRecord {
    pub path: String,
    pub line: usize,
    pub column: Option<usize>,
    pub kind: String,
    pub name: String,
    pub language: Option<String>,
    pub comparator: String,
}

impl ComparatorRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: impl Into<String>,
        line: usize,
        column: Option<usize>,
        kind: impl Into<String>,
        name: impl Into<String>,
        language: Option<impl Into<String>>,
        comparator: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            line,
            column,
            kind: kind.into(),
            name: name.into(),
            language: language.map(Into::into),
            comparator: comparator.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparatorStatus {
    Completed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparatorRun {
    pub comparator: String,
    pub status: ComparatorStatus,
    pub records: Vec<ComparatorRecord>,
    pub message: Option<String>,
}

impl ComparatorRun {
    pub fn completed(comparator: impl Into<String>, records: Vec<ComparatorRecord>) -> Self {
        Self {
            comparator: comparator.into(),
            status: ComparatorStatus::Completed,
            records,
            message: None,
        }
    }

    pub fn skipped(comparator: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            comparator: comparator.into(),
            status: ComparatorStatus::Skipped,
            records: Vec::new(),
            message: Some(message.into()),
        }
    }

    pub fn is_skipped(&self) -> bool {
        self.status == ComparatorStatus::Skipped
    }
}

pub trait QualityComparator {
    fn name(&self) -> &str;
    fn run(&self, repo_root: &Path) -> Result<ComparatorRun>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniversalCtagsComparator {
    command: String,
}

impl Default for UniversalCtagsComparator {
    fn default() -> Self {
        Self::new("ctags")
    }
}

impl UniversalCtagsComparator {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }

    fn command_available(&self) -> Result<bool> {
        match Command::new(&self.command)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) => Ok(status.success()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error).with_context(|| {
                format!(
                    "failed to probe optional Universal Ctags comparator `{}`",
                    self.command
                )
            }),
        }
    }
}

impl QualityComparator for UniversalCtagsComparator {
    fn name(&self) -> &str {
        "universal-ctags"
    }

    fn run(&self, repo_root: &Path) -> Result<ComparatorRun> {
        if !self.command_available()? {
            return Ok(ComparatorRun::skipped(
                self.name(),
                "comparator not found: optional Universal Ctags command is not installed",
            ));
        }

        let output = Command::new(&self.command)
            .args(["--output-format=json", "--fields=+nK", "-R", "."])
            .current_dir(repo_root)
            .output()
            .with_context(|| {
                format!(
                    "failed to run optional Universal Ctags comparator in {}",
                    repo_root.display()
                )
            })?;

        if !output.status.success() {
            bail!(
                "optional Universal Ctags comparator failed in {}\nstdout:\n{}\nstderr:\n{}",
                repo_root.display(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }

        let mut records = Vec::new();
        for (index, line) in String::from_utf8_lossy(&output.stdout).lines().enumerate() {
            if let Some(record) = parse_ctags_json_record(line, self.name())
                .with_context(|| format!("failed to parse ctags JSON output line {}", index + 1))?
            {
                records.push(record);
            }
        }

        records.sort_by(|left, right| {
            (
                left.path.as_str(),
                left.line,
                left.column.unwrap_or(0),
                left.kind.as_str(),
                left.name.as_str(),
            )
                .cmp(&(
                    right.path.as_str(),
                    right.line,
                    right.column.unwrap_or(0),
                    right.kind.as_str(),
                    right.name.as_str(),
                ))
        });

        Ok(ComparatorRun::completed(self.name(), records))
    }
}

pub fn parse_ctags_json_record(
    line: &str,
    comparator_name: impl Into<String>,
) -> Result<Option<ComparatorRecord>> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }

    if let Some(record_type) = json_string_field(line, "_type")?
        && record_type != "tag"
    {
        return Ok(None);
    }

    let Some(path) = json_string_field(line, "path")? else {
        return Ok(None);
    };
    let Some(name) = json_string_field(line, "name")? else {
        return Ok(None);
    };
    let Some(kind) = json_string_field(line, "kind")? else {
        return Ok(None);
    };
    let line_no = json_usize_field(line, "line")?.unwrap_or(0);
    let column = json_usize_field(line, "column")?;
    let language = json_string_field(line, "language")?;

    Ok(Some(ComparatorRecord::new(
        normalize_external_path(&path),
        line_no,
        column,
        kind,
        name,
        language,
        comparator_name,
    )))
}

fn normalize_external_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    normalized
        .strip_prefix("./")
        .unwrap_or(&normalized)
        .to_string()
}

fn json_string_field(line: &str, key: &str) -> Result<Option<String>> {
    let Some(value_start) = json_value_start(line, key) else {
        return Ok(None);
    };
    let value = &line[value_start..].trim_start();
    if !value.starts_with('"') {
        return Ok(None);
    }

    let mut out = String::new();
    let mut escaped = false;
    for ch in value[1..].chars() {
        if escaped {
            match ch {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'b' => out.push('\u{0008}'),
                'f' => out.push('\u{000c}'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                other => {
                    out.push('\\');
                    out.push(other);
                }
            }
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return Ok(Some(out)),
            _ => out.push(ch),
        }
    }

    bail!("unterminated JSON string field `{key}`")
}

fn json_usize_field(line: &str, key: &str) -> Result<Option<usize>> {
    let Some(value_start) = json_value_start(line, key) else {
        return Ok(None);
    };
    let value = line[value_start..].trim_start();
    let number = value
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();

    if number.is_empty() {
        return Ok(None);
    }

    number
        .parse::<usize>()
        .map(Some)
        .with_context(|| format!("invalid JSON integer field `{key}`"))
}

fn json_value_start(line: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{key}\"");
    let key_start = line.find(&needle)?;
    let after_key = key_start + needle.len();
    let colon_offset = line[after_key..].find(':')?;
    Some(after_key + colon_offset + 1)
}
