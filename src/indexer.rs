use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

use crate::{
    ctags::{check_ctags, index_with_ctags},
    extras::index_extras,
    model::FileMeta,
    store::{
        load_manifest, load_records, remove_records_for_paths, save_manifest, save_records,
        sort_records,
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

    let mut manifest = load_manifest(&root)?;
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

            manifest.files.insert(rel, meta);
        }
    }

    sort_records(&mut records);

    save_records(&root, &records)?;
    save_manifest(&root, &manifest)?;

    Ok(BuildStats {
        root,
        scanned_files: current_paths.len(),
        changed_files: changed_paths.len(),
        deleted_files: deleted_paths.len(),
        records: records.len(),
        ctags_universal: ctags_status.is_universal,
    })
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
            files.push(path.to_path_buf());
        }
    }

    files.sort();
    Ok(files)
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
