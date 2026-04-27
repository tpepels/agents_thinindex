use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::model::IndexRecord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtagsStatus {
    pub available: bool,
    pub is_universal: bool,
    pub version_output: String,
}

pub fn check_ctags() -> Result<CtagsStatus> {
    let output = Command::new("ctags")
        .arg("--version")
        .output()
        .context("failed to execute `ctags --version`; install Universal Ctags")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let version_output = format!("{stdout}{stderr}");

    if !output.status.success() {
        return Err(anyhow!(
            "`ctags --version` failed; install Universal Ctags\n{}",
            version_output.trim()
        ));
    }

    let is_universal = version_output
        .to_ascii_lowercase()
        .contains("universal ctags");

    Ok(CtagsStatus {
        available: true,
        is_universal,
        version_output,
    })
}

pub fn index_with_ctags(root: &Path, files: &[PathBuf]) -> Result<Vec<IndexRecord>> {
    if files.is_empty() {
        return Ok(Vec::new());
    }

    let status = check_ctags()?;

    if !status.is_universal {
        eprintln!(
            "warning: `ctags` is available but does not appear to be Universal Ctags; JSON output may fail"
        );
    }

    let mut command = Command::new("ctags");
    command
        .current_dir(root)
        .arg("--output-format=json")
        .arg("--fields=+nK") // ensure line and column fields are present
        .arg("-f")
        .arg("-");

    for file in files {
        command.arg(file);
    }

    let output = command
        .output()
        .context("failed to execute `ctags --output-format=json`")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return Err(anyhow!(
            "ctags failed with status {}\n{}",
            output.status,
            stderr.trim()
        ));
    }

    let mut records = Vec::new();

    for (idx, line) in stdout.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let value: Value = serde_json::from_str(trimmed).with_context(|| {
            format!(
                "failed to parse ctags JSON output on line {}: {}",
                idx + 1,
                trimmed
            )
        })?;

        if let Some(record) = record_from_ctags_value(root, value)? {
            records.push(record);
        }
    }

    Ok(records)
}

fn record_from_ctags_value(root: &Path, value: Value) -> Result<Option<IndexRecord>> {
    let kind = value.get("_type").and_then(Value::as_str).unwrap_or("");

    if kind != "tag" {
        return Ok(None);
    }

    let name = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();

    if name.is_empty() {
        return Ok(None);
    }

    let path = value
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();

    if path.is_empty() {
        return Ok(None);
    }

    let line = value.get("line").and_then(Value::as_u64).unwrap_or(1) as usize;
    let col = value.get("column").and_then(Value::as_u64).unwrap_or(1) as usize;

    let language = value
        .get("language")
        .and_then(Value::as_str)
        .map(normalize_language)
        .unwrap_or_else(|| language_from_path(path));

    let ctags_kind = value
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("symbol");

    let normalized_kind = normalize_kind(ctags_kind);

    let pattern = value.get("pattern").and_then(Value::as_str).unwrap_or(name);

    let relpath = normalize_ctags_path(root, path);

    Ok(Some(IndexRecord::new(
        relpath,
        line,
        col,
        language,
        normalized_kind,
        name,
        clean_pattern(pattern),
        "ctags",
    )))
}

fn normalize_ctags_path(root: &Path, path: &str) -> String {
    let raw = PathBuf::from(path);

    let rel = if raw.is_absolute() {
        raw.strip_prefix(root).unwrap_or(&raw).to_path_buf()
    } else {
        raw
    };

    rel.to_string_lossy().replace('\\', "/")
}

fn normalize_language(value: &str) -> String {
    match value.to_ascii_lowercase().as_str() {
        "python" => "py".to_string(),
        "typescript" => "ts".to_string(),
        "typescriptreact" => "tsx".to_string(),
        "javascript" => "js".to_string(),
        "javascriptreact" => "jsx".to_string(),
        "css" => "css".to_string(),
        "html" => "html".to_string(),
        "markdown" => "md".to_string(),
        other => other.to_string(),
    }
}

fn language_from_path(path: &str) -> String {
    match Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "py" => "py".to_string(),
        "ts" => "ts".to_string(),
        "tsx" => "tsx".to_string(),
        "js" => "js".to_string(),
        "jsx" => "jsx".to_string(),
        "css" => "css".to_string(),
        "html" | "htm" => "html".to_string(),
        "md" | "mdx" => "md".to_string(),
        other if !other.is_empty() => other.to_string(),
        _ => "text".to_string(),
    }
}

fn normalize_kind(value: &str) -> String {
    match value.to_ascii_lowercase().as_str() {
        "class" | "classes" => "class".to_string(),
        "function" | "functions" => "function".to_string(),
        "method" | "methods" => "method".to_string(),
        "member" | "members" => "member".to_string(),
        "variable" | "variables" => "variable".to_string(),
        "constant" | "constants" => "constant".to_string(),
        "interface" | "interfaces" => "interface".to_string(),
        "type" | "types" | "typedef" | "typedefs" => "type".to_string(),
        "module" | "modules" => "module".to_string(),
        "namespace" | "namespaces" => "namespace".to_string(),
        "property" | "properties" => "property".to_string(),
        "field" | "fields" => "field".to_string(),
        "enum" | "enumeration" | "enums" => "enum".to_string(),
        "enumerator" | "enumerators" => "enum_member".to_string(),
        "import" | "imports" => "import".to_string(),
        other if !other.is_empty() => other.to_string(),
        _ => "symbol".to_string(),
    }
}

fn clean_pattern(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("/^")
        .trim_end_matches("$/")
        .trim_end_matches(";\"")
        .trim()
        .to_string()
}
