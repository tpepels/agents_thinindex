use std::{
    collections::BTreeSet,
    fs,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
    time::{Duration, Instant, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

use crate::{
    deps::extract_dependencies,
    extras::index_extras,
    model::{FileMeta, IndexRecord},
    privacy::{SENSITIVE_PATH_WARNING_LIMIT, SensitivePathWarning, sensitive_path_reason},
    refs::{extract_refs, finalize_refs, refs_from_dependencies},
    semantic::SemanticAdapterRegistry,
    store::{
        load_manifest, load_records, prepare_for_build, remove_records_for_paths,
        save_index_snapshot, sort_records, sort_refs,
    },
    tree_sitter_extraction::TreeSitterExtractionEngine,
};

const IGNORE_DIRS: &[&str] = &[".git", ".dev_index"];
pub const MAX_RECORDS_PER_FILE: usize = 50_000;
pub const MAX_INDEXED_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const LARGE_FILE_WARNING_BYTES: u64 = 512 * 1024;
const MAX_SIZE_WARNINGS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileSizeAction {
    Skipped,
    Indexed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSizeWarning {
    pub path: String,
    pub size: u64,
    pub threshold: u64,
    pub action: FileSizeAction,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildTimings {
    pub discover: Duration,
    pub change_detection: Duration,
    pub parse: Duration,
    pub dependencies: Duration,
    pub refs: Duration,
    pub semantic: Duration,
    pub save: Duration,
    pub total: Duration,
}

#[derive(Debug, Clone)]
struct DiscoveredFile {
    path: PathBuf,
    rel: String,
    meta: FileMeta,
    indexable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStats {
    pub root: PathBuf,
    pub scanned_files: usize,
    pub changed_files: usize,
    pub unchanged_files: usize,
    pub deleted_files: usize,
    pub records: usize,
    pub refs: usize,
    pub dependencies: usize,
    pub semantic_facts: usize,
    pub total_file_bytes: u64,
    pub large_files: Vec<FileSizeWarning>,
    pub sensitive_paths: Vec<SensitivePathWarning>,
    pub timings: BuildTimings,
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
    build_index_with_semantic_adapters(start, &SemanticAdapterRegistry::disabled())
}

pub fn build_index_with_semantic_adapters(
    start: &Path,
    semantic_adapters: &SemanticAdapterRegistry,
) -> Result<BuildStats> {
    let total_start = Instant::now();
    let root = find_repo_root(start)?;

    let (mut manifest, reset_message) = prepare_for_build(&root)?;

    let existing_records = load_records(&root)?;

    let discover_start = Instant::now();
    let files = discover_files(&root)?;
    let discover_elapsed = discover_start.elapsed();

    let change_start = Instant::now();
    let mut discovered_files = Vec::new();
    let mut current_paths = BTreeSet::new();
    let mut total_file_bytes = 0u64;
    let mut large_files = Vec::new();
    let mut sensitive_paths = Vec::new();

    for path in files {
        let rel = relpath(&root, &path)?;
        let meta = file_meta(&path)?;
        let indexable = meta.size <= MAX_INDEXED_FILE_BYTES;

        total_file_bytes = total_file_bytes.saturating_add(meta.size);
        current_paths.insert(rel.clone());

        if let Some(reason) = sensitive_path_reason(&rel) {
            sensitive_paths.push(SensitivePathWarning {
                path: rel.clone(),
                reason,
            });
        }

        if meta.size >= LARGE_FILE_WARNING_BYTES {
            large_files.push(FileSizeWarning {
                path: rel.clone(),
                size: meta.size,
                threshold: if indexable {
                    LARGE_FILE_WARNING_BYTES
                } else {
                    MAX_INDEXED_FILE_BYTES
                },
                action: if indexable {
                    FileSizeAction::Indexed
                } else {
                    FileSizeAction::Skipped
                },
            });
        }

        discovered_files.push(DiscoveredFile {
            path,
            rel,
            meta,
            indexable,
        });
    }

    large_files.sort_by_key(|warning| {
        (
            warning.action,
            std::cmp::Reverse(warning.size),
            warning.path.clone(),
        )
    });
    large_files.truncate(MAX_SIZE_WARNINGS);
    sensitive_paths.sort_by(|a, b| a.path.cmp(&b.path).then(a.reason.cmp(b.reason)));
    sensitive_paths.truncate(SENSITIVE_PATH_WARNING_LIMIT);

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
    let mut indexable_files = Vec::new();
    let mut unchanged_files = 0usize;

    for discovered in discovered_files {
        let changed = manifest.files.get(&discovered.rel) != Some(&discovered.meta);

        if changed {
            changed_paths.push(discovered.rel.clone());
        } else {
            unchanged_files += 1;
        }

        if discovered.indexable {
            indexable_files.push(discovered.path.clone());

            if changed {
                changed_files.push((
                    discovered.path,
                    discovered.rel.clone(),
                    discovered.meta.clone(),
                ));
            }
        } else if changed {
            manifest.files.insert(discovered.rel, discovered.meta);
        }
    }
    let change_elapsed = change_start.elapsed();

    let mut stale_paths = deleted_paths.clone();
    stale_paths.extend(changed_paths.clone());

    let mut records = remove_records_for_paths(existing_records, &stale_paths);

    let parse_start = Instant::now();
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
    let parse_elapsed = parse_start.elapsed();

    dedupe_records_by_location(&mut records);
    debug_assert_unique_record_locations(&records);
    sort_records(&mut records);

    let dependencies_start = Instant::now();
    let dependencies = extract_dependencies(&root, &indexable_files, &records)?;
    let dependencies_elapsed = dependencies_start.elapsed();

    let refs_start = Instant::now();
    let mut refs = extract_all_refs(&root, &indexable_files, &records, &dependencies)?;
    dedupe_refs_by_location(&mut refs);
    refs = finalize_refs(refs);
    sort_refs(&mut refs);
    let refs_elapsed = refs_start.elapsed();

    let semantic_start = Instant::now();
    let semantic_facts = semantic_adapters.collect_facts(&root, &records, &dependencies);
    let semantic_elapsed = semantic_start.elapsed();

    let save_start = Instant::now();
    save_index_snapshot(
        &root,
        &manifest,
        &records,
        &refs,
        &dependencies,
        &semantic_facts,
    )?;
    let save_elapsed = save_start.elapsed();

    Ok(BuildStats {
        root,
        scanned_files: current_paths.len(),
        changed_files: changed_paths.len(),
        unchanged_files,
        deleted_files: deleted_paths.len(),
        records: records.len(),
        refs: refs.len(),
        dependencies: dependencies.len(),
        semantic_facts: semantic_facts.len(),
        total_file_bytes,
        large_files,
        sensitive_paths,
        timings: BuildTimings {
            discover: discover_elapsed,
            change_detection: change_elapsed,
            parse: parse_elapsed,
            dependencies: dependencies_elapsed,
            refs: refs_elapsed,
            semantic: semantic_elapsed,
            save: save_elapsed,
            total: total_start.elapsed(),
        },
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
    dependencies: &[crate::model::DependencyEdge],
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

    refs.extend(refs_from_dependencies(dependencies, records));

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
