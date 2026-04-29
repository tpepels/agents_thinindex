use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

use crate::{
    ctags::{check_ctags, index_with_ctags},
    extras::index_extras,
    model::{FileMeta, IndexRecord},
    refs::extract_refs,
    store::{
        load_manifest, load_records, load_refs, prepare_for_build, remove_records_for_paths,
        remove_refs_for_paths, save_index_snapshot, sort_records, sort_refs,
    },
};

const IGNORE_DIRS: &[&str] = &[".git", ".dev_index"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStats {
    pub root: PathBuf,
    pub scanned_files: usize,
    pub changed_files: usize,
    pub deleted_files: usize,
    pub records: usize,
    pub ctags_universal: bool,
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
    let ctags_status = check_ctags()?;

    let (mut manifest, reset_message) = prepare_for_build(&root)?;

    let existing_records = load_records(&root)?;
    let existing_refs = load_refs(&root)?;

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

    for path in files {
        let rel = relpath(&root, &path)?;
        let next_meta = file_meta(&path)?;

        if manifest.files.get(&rel) != Some(&next_meta) {
            changed_paths.push(rel.clone());
            changed_files.push((path, rel, next_meta));
        }
    }

    let mut stale_paths = deleted_paths.clone();
    stale_paths.extend(changed_paths.clone());

    let mut records = remove_records_for_paths(existing_records, &stale_paths);
    let mut refs = remove_refs_for_paths(existing_refs, &stale_paths);

    if !changed_files.is_empty() {
        let changed_abs_paths: Vec<PathBuf> = changed_files
            .iter()
            .map(|(path, _, _)| path.clone())
            .collect();

        let mut ctags_records = index_with_ctags(&root, &changed_abs_paths)?;
        records.append(&mut ctags_records);

        for (path, rel, meta) in changed_files {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("failed to read source file: {}", path.display()))?;

            let mut extra_records = index_extras(&rel, &text);
            records.append(&mut extra_records);

            let mut file_refs = extract_refs(&rel, &text);
            refs.append(&mut file_refs);

            manifest.files.insert(rel, meta);
        }
    }

    dedupe_records_by_location(&mut records);
    dedupe_refs_by_location(&mut refs);
    debug_assert_unique_record_locations(&records);
    sort_records(&mut records);
    sort_refs(&mut refs);

    save_index_snapshot(&root, &manifest, &records, &refs)?;

    Ok(BuildStats {
        root,
        scanned_files: current_paths.len(),
        changed_files: changed_paths.len(),
        deleted_files: deleted_paths.len(),
        records: records.len(),
        ctags_universal: ctags_status.is_universal,
        reset_message,
    })
}

/// Cheap pre-check: compare manifest entries to current file mtimes/sizes
/// without invoking ctags or rewriting the index. Returns `Ok(false)` if
/// any indexed file has been touched, deleted, or added since the last
/// build, or if the manifest itself needs reset.
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
        ("ctags", "section") => 0,

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
        | ("extras", "todo")
        | ("extras", "fixme")
        | ("extras", "checklist")
        | ("extras", "link") => 0,

        ("ctags", _) => 1,

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
/// at most 8KB) and avoids feeding non-text blobs to ctags or
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
