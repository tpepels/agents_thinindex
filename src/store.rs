/// Removes the entire .dev_index directory and recreates it.
pub fn reset_dev_index(root: &Path) -> Result<()> {
    let dir = index_dir(root);
    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| format!("failed to remove {}", dir.display()))?;
    }
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    Ok(())
}
use crate::model::INDEX_SCHEMA_VERSION;
pub fn is_manifest_stale(manifest: &Manifest) -> bool {
    manifest.schema_version != INDEX_SCHEMA_VERSION
}

pub fn remove_index_files(root: &Path) -> Result<()> {
    let index = index_path(root);
    let manifest = manifest_path(root);
    // Remove index.jsonl and manifest.json, but keep wi_usage.jsonl if present
    let _ = fs::remove_file(&index);
    let _ = fs::remove_file(&manifest);
    Ok(())
}
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::model::{IndexRecord, Manifest};

pub const DEV_INDEX_DIR: &str = ".dev_index";
pub const MANIFEST_FILE: &str = "manifest.json";
pub const INDEX_FILE: &str = "index.jsonl";

pub fn index_dir(root: &Path) -> PathBuf {
    root.join(DEV_INDEX_DIR)
}

pub fn manifest_path(root: &Path) -> PathBuf {
    index_dir(root).join(MANIFEST_FILE)
}

pub fn index_path(root: &Path) -> PathBuf {
    index_dir(root).join(INDEX_FILE)
}

pub fn ensure_index_dir(root: &Path) -> Result<()> {
    let path = index_dir(root);

    fs::create_dir_all(&path)
        .with_context(|| format!("failed to create index directory: {}", path.display()))?;

    Ok(())
}

pub fn load_manifest(root: &Path) -> Result<Manifest> {
    let path = manifest_path(root);
    if !path.exists() {
        return Ok(Manifest::new());
    }
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return Ok(Manifest::new()),
    };
    match serde_json::from_str::<Manifest>(&text) {
        Ok(manifest) => Ok(manifest),
        Err(_) => Ok(Manifest {
            schema_version: 0,
            files: Default::default(),
        }),
    }
}

pub fn save_manifest(root: &Path, manifest: &Manifest) -> Result<()> {
    ensure_index_dir(root)?;

    let path = manifest_path(root);
    let tmp = path.with_extension("json.tmp");

    let text = serde_json::to_string_pretty(manifest).context("failed to serialize manifest")?;

    fs::write(&tmp, text).with_context(|| format!("failed to write {}", tmp.display()))?;

    fs::rename(&tmp, &path)
        .with_context(|| format!("failed to replace manifest: {}", path.display()))?;

    Ok(())
}

pub fn load_records(root: &Path) -> Result<Vec<IndexRecord>> {
    let path = index_path(root);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let file =
        File::open(&path).with_context(|| format!("failed to open index: {}", path.display()))?;

    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (idx, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "failed to read line {} from index: {}",
                idx + 1,
                path.display()
            )
        })?;

        if line.trim().is_empty() {
            continue;
        }

        let record: IndexRecord = serde_json::from_str(&line).with_context(|| {
            format!(
                "failed to parse index JSON on line {}: {}",
                idx + 1,
                path.display()
            )
        })?;

        records.push(record);
    }

    Ok(records)
}

pub fn save_records(root: &Path, records: &[IndexRecord]) -> Result<()> {
    ensure_index_dir(root)?;

    let path = index_path(root);
    let tmp = path.with_extension("jsonl.tmp");

    let file = File::create(&tmp).with_context(|| format!("failed to create {}", tmp.display()))?;

    let mut writer = BufWriter::new(file);

    for record in records {
        let line = serde_json::to_string(record).context("failed to serialize index record")?;

        writer
            .write_all(line.as_bytes())
            .with_context(|| format!("failed to write {}", tmp.display()))?;

        writer
            .write_all(b"\n")
            .with_context(|| format!("failed to write newline to {}", tmp.display()))?;
    }

    writer
        .flush()
        .with_context(|| format!("failed to flush {}", tmp.display()))?;

    fs::rename(&tmp, &path)
        .with_context(|| format!("failed to replace index: {}", path.display()))?;

    Ok(())
}

pub fn remove_records_for_paths(records: Vec<IndexRecord>, paths: &[String]) -> Vec<IndexRecord> {
    records
        .into_iter()
        .filter(|record| !paths.contains(&record.path))
        .collect()
}

pub fn sort_records(records: &mut [IndexRecord]) {
    records.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.line.cmp(&b.line))
            .then(a.col.cmp(&b.col))
            .then(a.kind.cmp(&b.kind))
            .then(a.name.cmp(&b.name))
            .then(a.source.cmp(&b.source))
    });
}
