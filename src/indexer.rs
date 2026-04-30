use std::{
    collections::BTreeSet,
    fs,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

use crate::{
    deps::extract_dependencies,
    extras::index_extras,
    model::{FileMeta, IndexRecord},
    refs::{extract_refs, finalize_refs},
    store::{
        load_manifest, load_records, prepare_for_build, remove_records_for_paths,
        save_index_snapshot, sort_records, sort_refs,
    },
    tree_sitter_extraction::TreeSitterExtractionEngine,
};

const IGNORE_DIRS: &[&str] = &[".git", ".dev_index"];
pub const MAX_RECORDS_PER_FILE: usize = 50_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStats {
    pub root: PathBuf,
    pub scanned_files: usize,
    pub changed_files: usize,
    pub deleted_files: usize,
    pub records: usize,
    /// Populated when an existing index was reset (schema bump or corrupted
    /// manifest). Callers may surface this; library code stays silent.
    pub reset_message: Option<&'static str>,
}

pub fn find_repo_root(start: &Path) -> Result<PathBuf> {
    let start = start
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", start.display()))?;

    for candidate in start.ancestors() {
        if candidate.join(".git").exists() {
            return Ok(candidate.to_path_buf());
        }
    }

    Ok(start)
}

pub fn build_index(start: &Path) -> Result<BuildStats> {
    let root = find_repo_root(start)?;

    let (mut manifest, reset_message) = prepare_for_build(&root)?;

    let existing_records = load_records(&root)?;

    let files = discover_files(&root)?;
    let current_paths: BTreeSet<String> = files
        .iter()
        .map(|path| relpath(&root, path))
        .collect::<Result<_>>()?;

    let deleted_paths: Vec<String> = manifest
        .files
        .keys()
        .filter(|path| !current_paths.contains(*path))
        .cloned()
        .collect();

    for path in &deleted_paths {
        manifest.files.remove(path);
    }

    let mut changed_paths = Vec::new();
    let mut changed_files = Vec::new();

    for path in &files {
        let rel = relpath(&root, path)?;
        let next_meta = file_meta(path)?;

        if manifest.files.get(&rel) != Some(&next_meta) {
            changed_paths.push(rel.clone());
            changed_files.push((path.clone(), rel, next_meta));
        }
    }

    let mut stale_paths = deleted_paths.clone();
    stale_paths.extend(changed_paths.clone());

    let mut records = remove_records_for_paths(existing_records, &stale_paths);

    if !changed_files.is_empty() {
        let parser = TreeSitterExtractionEngine::default();

        for (path, rel, meta) in changed_files {
            let Some(text) = read_utf8_source_file(&path, "failed to read source file")? else {
                manifest.files.insert(rel, meta);
                continue;
            };

            let mut file_records = parser
                .parse_file(&rel, &text)
                .with_context(|| format!("failed to parse source file: {rel}"))?;

            let mut extra_records = index_extras(&rel, &text);
            file_records.append(&mut extra_records);
            enforce_record_limit_for_file(&rel, file_records.len())?;
            records.append(&mut file_records);

            manifest.files.insert(rel, meta);
        }
    }

    dedupe_records_by_location(&mut records);
    debug_assert_unique_record_locations(&records);
    sort_records(&mut records);

    let dependencies = extract_dependencies(&root, &files, &records)?;

    let mut refs = extract_all_refs(&root, &files, &records)?;
    dedupe_refs_by_location(&mut refs);
    refs = finalize_refs(refs);
    sort_refs(&mut refs);

    save_index_snapshot(&root, &manifest, &records, &refs, &dependencies)?;

    Ok(BuildStats {
        root,
        scanned_files: current_paths.len(),
        changed_files: changed_paths.len(),
        deleted_files: deleted_paths.len(),
        records: records.len(),
        reset_message,
    })
}

fn enforce_record_limit_for_file(rel: &str, record_count: usize) -> Result<()> {
    if record_count > MAX_RECORDS_PER_FILE {
        anyhow::bail!(
            "record explosion guard tripped for {rel}: {record_count} records exceeds per-file cap {MAX_RECORDS_PER_FILE}"
        );
    }

    Ok(())
}

/// Cheap pre-check: compare manifest entries to current file mtimes/sizes
/// without parsing files or rewriting the index. Returns `Ok(false)` if any
/// indexed file has been touched, deleted, or added since the last build, or
/// if the manifest itself needs reset.
pub fn index_is_fresh(start: &Path) -> Result<bool> {
    let root = find_repo_root(start)?;
    let manifest = load_manifest(&root)?;

    let files = discover_files(&root)?;
    let mut current_paths: BTreeSet<String> = BTreeSet::new();

    for path in &files {
        let rel = relpath(&root, path)?;
        let next_meta = file_meta(path)?;

        if manifest.files.get(&rel) != Some(&next_meta) {
            return Ok(false);
        }

        current_paths.insert(rel);
    }

    if manifest.files.keys().any(|p| !current_paths.contains(p)) {
        return Ok(false);
    }

    Ok(true)
}

fn dedupe_records_by_location(records: &mut Vec<IndexRecord>) {
    records.sort_by_key(record_location_preference_key);

    let mut seen_locations = BTreeSet::new();

    records.retain(|record| seen_locations.insert((record.path.clone(), record.line, record.col)));
}

fn record_location_preference_key(record: &IndexRecord) -> (usize, String, usize, usize, String) {
    (
        record_location_preference_rank(record),
        record.path.clone(),
        record.line,
        record.col,
        record.name.clone(),
    )
}

fn record_location_preference_rank(record: &IndexRecord) -> usize {
    match (record.source.as_str(), record.kind.as_str()) {
        ("tree_sitter", "section") => 0,

        ("extras", "css_id")
        | ("extras", "css_class")
        | ("extras", "css_variable")
        | ("extras", "keyframes")
        | ("extras", "html_id")
        | ("extras", "html_class")
        | ("extras", "html_tag")
        | ("extras", "data_attribute")
        | ("extras", "jsx_class")
        | ("extras", "component_usage")
        | ("extras", "key")
        | ("extras", "table")
        | ("extras", "todo")
        | ("extras", "fixme")
        | ("extras", "checklist")
        | ("extras", "link") => 0,

        ("tree_sitter", _) => 1,

        ("extras", "section") => 2,

        ("extras", "heading")
        | ("extras", "markdown_heading")
        | ("extras", "heading_1")
        | ("extras", "heading_2")
        | ("extras", "heading_3")
        | ("extras", "heading_4")
        | ("extras", "heading_5")
        | ("extras", "heading_6") => 3,

        _ => 9,
    }
}

fn debug_assert_unique_record_locations(records: &[IndexRecord]) {
    let mut seen = BTreeSet::new();

    for record in records {
        let key = (record.path.clone(), record.line, record.col);

        debug_assert!(
            seen.insert(key),
            "duplicate index location: path={} line={} col={} kind={} name={} source={}",
            record.path,
            record.line,
            record.col,
            record.kind,
            record.name,
            record.source
        );
    }
}

fn extract_all_refs(
    root: &Path,
    files: &[PathBuf],
    records: &[IndexRecord],
) -> Result<Vec<crate::model::ReferenceRecord>> {
    let mut refs = Vec::new();

    for path in files {
        let rel = relpath(root, path)?;
        let Some(text) = read_utf8_source_file(path, "failed to read source file for refs")? else {
            continue;
        };

        let mut file_refs = extract_refs(&rel, &text, records);
        refs.append(&mut file_refs);
    }

    Ok(refs)
}

fn dedupe_refs_by_location(refs: &mut Vec<crate::model::ReferenceRecord>) {
    sort_refs(refs);

    let mut seen_locations = BTreeSet::new();

    refs.retain(|reference| {
        seen_locations.insert((
            reference.from_path.clone(),
            reference.from_line,
            reference.from_col,
            reference.to_name.clone(),
            reference.ref_kind.clone(),
        ))
    });
}

fn discover_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut builder = WalkBuilder::new(root);

    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .parents(true)
        .add_custom_ignore_filename(".thinindexignore");

    let mut files = Vec::new();

    for result in builder.build() {
        let entry = match result {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let path = entry.path();

        if path == root {
            continue;
        }

        let rel = match path.strip_prefix(root) {
            Ok(rel) => rel,
            Err(_) => continue,
        };

        if should_always_ignore_path(rel) {
            continue;
        }

        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            if is_likely_binary(path) {
                continue;
            }

            files.push(path.to_path_buf());
        }
    }

    files.sort();
    Ok(files)
}

const BINARY_SNIFF_BYTES: usize = 8192;

/// Heuristic used by git, ripgrep, and friends: read the leading chunk and
/// treat the file as binary if it contains a NUL byte. Cheap (one syscall,
/// at most 8KB) and avoids feeding non-text blobs to the parser or
/// `read_to_string`, which would otherwise abort the whole build.
fn is_likely_binary(path: &Path) -> bool {
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };

    let mut buf = [0u8; BINARY_SNIFF_BYTES];
    let n = match file.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return false,
    };

    buf[..n].contains(&0)
}

fn should_always_ignore_path(rel: &Path) -> bool {
    rel.components().any(|component| {
        let value = component.as_os_str().to_string_lossy();
        IGNORE_DIRS.contains(&value.as_ref())
    })
}

fn read_utf8_source_file(path: &Path, context: &str) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == ErrorKind::InvalidData => Ok(None),
        Err(error) => Err(error).with_context(|| format!("{context}: {}", path.display())),
    }
}

fn relpath(root: &Path, path: &Path) -> Result<String> {
    Ok(path
        .strip_prefix(root)
        .with_context(|| {
            format!(
                "failed to make path relative: root={} path={}",
                root.display(),
                path.display()
            )
        })?
        .to_string_lossy()
        .replace('\\', "/"))
}

fn file_meta(path: &Path) -> Result<FileMeta> {
    let metadata =
        fs::metadata(path).with_context(|| format!("failed to stat file: {}", path.display()))?;

    let mtime = metadata
        .modified()
        .with_context(|| format!("failed to read mtime: {}", path.display()))?;

    let mtime_ns = mtime
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    Ok(FileMeta {
        mtime_ns,
        size: metadata.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_explosion_guard_rejects_single_file_over_cap() {
        let error = enforce_record_limit_for_file("src/generated.rs", MAX_RECORDS_PER_FILE + 1)
            .expect_err("record cap should fail");
        let message = format!("{error:#}");

        assert!(message.contains("record explosion guard tripped"));
        assert!(message.contains("src/generated.rs"));
    }
}
