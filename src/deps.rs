use std::{
    collections::BTreeSet,
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::model::{DependencyEdge, IndexRecord};

pub const DEPENDENCY_SOURCE_TREE_SITTER: &str = "tree_sitter";
pub const DEPENDENCY_SOURCE_SCAN: &str = "dependency_scan";
pub const CONFIDENCE_RESOLVED: &str = "resolved";
pub const CONFIDENCE_UNRESOLVED: &str = "unresolved";

pub fn extract_dependencies(
    root: &Path,
    files: &[PathBuf],
    records: &[IndexRecord],
) -> Result<Vec<DependencyEdge>> {
    let mut file_set = BTreeSet::new();
    let mut rel_files = Vec::new();

    for path in files {
        let rel = relpath(root, path)?;
        file_set.insert(rel.clone());
        rel_files.push((path.clone(), rel));
    }

    let mut edges = Vec::new();

    for record in records {
        if record.kind == "import"
            && record.source == "tree_sitter"
            && let Some(edge) = edge_from_import_record(record, &file_set)
        {
            edges.push(edge);
        }
    }

    for (path, rel) in rel_files {
        let Some(text) = read_utf8_source_file(&path)? else {
            continue;
        };

        extract_scanned_dependencies(&rel, &text, &file_set, &mut edges);
    }

    Ok(finalize_dependencies(edges))
}

pub fn finalize_dependencies(mut edges: Vec<DependencyEdge>) -> Vec<DependencyEdge> {
    sort_dependencies(&mut edges);

    let mut seen = BTreeSet::new();
    edges.retain(|edge| {
        seen.insert((
            edge.from_path.clone(),
            edge.import_path.clone(),
            edge.target_path.clone(),
            edge.dependency_kind.clone(),
        ))
    });

    edges
}

pub fn sort_dependencies(edges: &mut [DependencyEdge]) {
    edges.sort_by(|a, b| {
        a.from_path
            .cmp(&b.from_path)
            .then(a.from_line.cmp(&b.from_line))
            .then(a.from_col.cmp(&b.from_col))
            .then(a.import_path.cmp(&b.import_path))
            .then(a.dependency_kind.cmp(&b.dependency_kind))
            .then(a.target_path.cmp(&b.target_path))
    });
}

fn edge_from_import_record(
    record: &IndexRecord,
    file_set: &BTreeSet<String>,
) -> Option<DependencyEdge> {
    let import_path = clean_import_path(&record.name)?;
    let dependency_kind = dependency_kind_for_record(record)?;
    let resolution = resolve_import(
        &record.path,
        &record.lang,
        dependency_kind,
        &import_path,
        file_set,
    );

    Some(edge(
        &record.path,
        record.line,
        record.col,
        &import_path,
        dependency_kind,
        &record.lang,
        resolution,
        &record.text,
        DEPENDENCY_SOURCE_TREE_SITTER,
    ))
}

fn dependency_kind_for_record(record: &IndexRecord) -> Option<&'static str> {
    match record.lang.as_str() {
        "py" => Some("python_import"),
        "js" | "jsx" | "ts" | "tsx" => Some("js_import"),
        "go" => Some("go_import"),
        "java" => Some("java_import"),
        "c" => Some("c_include"),
        "cpp" => Some("cpp_include"),
        "php" => Some("php_include"),
        "scala" | "kt" | "swift" | "dart" => Some("import"),
        _ => None,
    }
}

fn extract_scanned_dependencies(
    rel: &str,
    text: &str,
    file_set: &BTreeSet<String>,
    edges: &mut Vec<DependencyEdge>,
) {
    let lang = lang_from_path(rel);

    for (line_index, line) in text.lines().enumerate() {
        let line_no = line_index + 1;

        match lang {
            "rs" => extract_rust_dependencies(rel, line_no, line, file_set, edges),
            "rb" => extract_ruby_dependencies(rel, line_no, line, file_set, edges),
            "sh" | "bash" => extract_shell_dependencies(rel, line_no, line, file_set, edges),
            _ => {}
        }
    }
}

fn extract_rust_dependencies(
    rel: &str,
    line_no: usize,
    line: &str,
    file_set: &BTreeSet<String>,
    edges: &mut Vec<DependencyEdge>,
) {
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();

    if let Some(rest) = trimmed.strip_prefix("mod ") {
        if rest.contains('{') {
            return;
        }

        let name = take_identifier(rest);
        if name.is_empty() {
            return;
        }

        let resolution = resolve_rust_module(rel, name, file_set);
        edges.push(edge(
            rel,
            line_no,
            leading + "mod ".len() + 1,
            name,
            "rust_module",
            "rs",
            resolution,
            line.trim(),
            DEPENDENCY_SOURCE_SCAN,
        ));
        return;
    }

    let Some(rest) = trimmed.strip_prefix("use ") else {
        return;
    };

    let import_path = rest.trim_end_matches(';').trim();
    if !(import_path.starts_with("crate::") || import_path.starts_with("super::")) {
        return;
    }

    let resolution = resolve_rust_use(rel, import_path, file_set);
    edges.push(edge(
        rel,
        line_no,
        leading + "use ".len() + 1,
        import_path,
        "rust_use",
        "rs",
        resolution,
        line.trim(),
        DEPENDENCY_SOURCE_SCAN,
    ));
}

fn extract_ruby_dependencies(
    rel: &str,
    line_no: usize,
    line: &str,
    file_set: &BTreeSet<String>,
    edges: &mut Vec<DependencyEdge>,
) {
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();
    let Some((keyword, rest)) = trimmed
        .strip_prefix("require_relative ")
        .map(|rest| ("require_relative", rest))
        .or_else(|| {
            trimmed
                .strip_prefix("require ")
                .map(|rest| ("require", rest))
        })
    else {
        return;
    };

    let Some((offset, import_path)) = quoted_value(rest) else {
        return;
    };

    let resolution = if keyword == "require_relative" || import_path.starts_with('.') {
        resolve_relative_module(rel, import_path, &["rb"], file_set)
    } else {
        Resolution::unresolved("external_package")
    };

    edges.push(edge(
        rel,
        line_no,
        leading + keyword.len() + 1 + offset + 1,
        import_path,
        "ruby_require",
        "rb",
        resolution,
        line.trim(),
        DEPENDENCY_SOURCE_SCAN,
    ));
}

fn extract_shell_dependencies(
    rel: &str,
    line_no: usize,
    line: &str,
    file_set: &BTreeSet<String>,
    edges: &mut Vec<DependencyEdge>,
) {
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();
    let Some((keyword, rest)) = trimmed
        .strip_prefix("source ")
        .map(|rest| ("source", rest))
        .or_else(|| trimmed.strip_prefix(". ").map(|rest| (".", rest)))
    else {
        return;
    };

    let Some((offset, import_path)) = shell_path_value(rest) else {
        return;
    };

    if import_path.starts_with('$') || import_path.starts_with('~') {
        return;
    }

    let resolution = if import_path.starts_with('/') {
        Resolution::unresolved("absolute_path")
    } else {
        resolve_relative_path(rel, import_path, &["sh", "bash"], file_set)
    };

    edges.push(edge(
        rel,
        line_no,
        leading + keyword.len() + 1 + offset + 1,
        import_path,
        "shell_source",
        lang_from_path(rel),
        resolution,
        line.trim(),
        DEPENDENCY_SOURCE_SCAN,
    ));
}

fn resolve_import(
    from_path: &str,
    lang: &str,
    dependency_kind: &str,
    import_path: &str,
    file_set: &BTreeSet<String>,
) -> Resolution {
    match dependency_kind {
        "python_import" => resolve_python_import(from_path, import_path, file_set),
        "js_import" => resolve_js_import(from_path, import_path, file_set),
        "go_import" => resolve_go_import(from_path, import_path, file_set),
        "java_import" => resolve_java_import(import_path, file_set),
        "c_include" | "cpp_include" => resolve_c_include(from_path, import_path, file_set),
        "php_include" => resolve_relative_path(from_path, import_path, &["php"], file_set),
        _ if lang == "scala" || lang == "kt" || lang == "swift" || lang == "dart" => {
            Resolution::unresolved("target_not_found")
        }
        _ => Resolution::unresolved("unsupported_language"),
    }
}

fn resolve_python_import(
    from_path: &str,
    import_path: &str,
    file_set: &BTreeSet<String>,
) -> Resolution {
    let module = import_path.trim_start_matches('.');
    if module.is_empty() {
        return Resolution::unresolved("target_not_found");
    }

    let module_path = module.replace('.', "/");
    let parent = parent_dir(from_path);
    let mut bases = Vec::new();

    if !parent.is_empty() {
        bases.push(join_rel(&parent, &module_path));
    }
    bases.push(module_path);

    for base in bases {
        for candidate in [
            format!("{base}.py"),
            format!("{base}/__init__.py"),
            format!("{base}/mod.py"),
        ] {
            if file_set.contains(&candidate) {
                return Resolution::resolved(candidate);
            }
        }
    }

    Resolution::unresolved("target_not_found")
}

fn resolve_js_import(
    from_path: &str,
    import_path: &str,
    file_set: &BTreeSet<String>,
) -> Resolution {
    if !(import_path.starts_with("./") || import_path.starts_with("../")) {
        return Resolution::unresolved("external_package");
    }

    resolve_relative_path(
        from_path,
        import_path,
        &["ts", "tsx", "js", "jsx", "mjs", "cjs", "json"],
        file_set,
    )
}

fn resolve_go_import(
    from_path: &str,
    import_path: &str,
    file_set: &BTreeSet<String>,
) -> Resolution {
    if import_path.starts_with("./") || import_path.starts_with("../") {
        return resolve_relative_path(from_path, import_path, &["go"], file_set);
    }

    let suffix = import_path.trim_matches('/');
    for candidate in file_set {
        if !candidate.ends_with(".go") {
            continue;
        }

        let parent = parent_dir(candidate);
        if parent == suffix || parent.ends_with(&format!("/{suffix}")) {
            return Resolution::resolved(candidate.clone());
        }

        let parts: Vec<&str> = parent.split('/').collect();
        for index in 1..parts.len() {
            let local_suffix = parts[index..].join("/");
            if suffix.ends_with(&local_suffix) {
                return Resolution::resolved(candidate.clone());
            }
        }
    }

    Resolution::unresolved("external_package")
}

fn resolve_java_import(import_path: &str, file_set: &BTreeSet<String>) -> Resolution {
    if import_path.starts_with("java.") || import_path.starts_with("javax.") {
        return Resolution::unresolved("external_package");
    }

    let java_path = import_path.trim_end_matches(".*").replace('.', "/");
    let candidates = if import_path.ends_with(".*") {
        vec![format!("{java_path}/package-info.java")]
    } else {
        vec![format!("{java_path}.java")]
    };

    for wanted in candidates {
        for candidate in file_set {
            if candidate == &wanted || candidate.ends_with(&format!("/{wanted}")) {
                return Resolution::resolved(candidate.clone());
            }
        }
    }

    Resolution::unresolved("target_not_found")
}

fn resolve_c_include(
    from_path: &str,
    import_path: &str,
    file_set: &BTreeSet<String>,
) -> Resolution {
    if is_system_include(import_path) {
        return Resolution::unresolved("system_include");
    }

    resolve_relative_path(from_path, import_path, &["h", "hpp", "hh", "hxx"], file_set)
}

fn resolve_rust_module(from_path: &str, module: &str, file_set: &BTreeSet<String>) -> Resolution {
    let parent = parent_dir(from_path);
    let base = if parent.is_empty() {
        module.to_string()
    } else {
        join_rel(&parent, module)
    };

    resolve_candidates(
        vec![format!("{base}.rs"), format!("{base}/mod.rs")],
        file_set,
    )
}

fn resolve_rust_use(from_path: &str, import_path: &str, file_set: &BTreeSet<String>) -> Resolution {
    let module_path = rust_use_module_path(from_path, import_path);
    let Some(module_path) = module_path else {
        return Resolution::unresolved("target_not_found");
    };

    resolve_candidates(
        vec![format!("{module_path}.rs"), format!("{module_path}/mod.rs")],
        file_set,
    )
}

fn rust_use_module_path(from_path: &str, import_path: &str) -> Option<String> {
    let path = import_path.trim_end_matches(';').trim();
    let (root, rest) = path.split_once("::")?;
    let mut segments: Vec<&str> = rest
        .split("::")
        .filter(|segment| !segment.is_empty() && !segment.contains('{'))
        .collect();

    if segments.len() > 1 {
        segments.pop();
    }

    if segments.is_empty() {
        return None;
    }

    let joined = segments.join("/");
    match root {
        "crate" => Some(join_rel("src", &joined)),
        "super" => {
            let parent = parent_dir(from_path);
            let base = parent_dir(&parent);
            Some(if base.is_empty() {
                joined
            } else {
                join_rel(&base, &joined)
            })
        }
        _ => None,
    }
}

fn resolve_relative_module(
    from_path: &str,
    import_path: &str,
    extensions: &[&str],
    file_set: &BTreeSet<String>,
) -> Resolution {
    resolve_relative_path(from_path, import_path, extensions, file_set)
}

fn resolve_relative_path(
    from_path: &str,
    import_path: &str,
    extensions: &[&str],
    file_set: &BTreeSet<String>,
) -> Resolution {
    let parent = parent_dir(from_path);
    let base = join_rel(&parent, import_path);
    let mut candidates = Vec::new();

    candidates.push(base.clone());

    if Path::new(import_path).extension().is_none() {
        for extension in extensions {
            candidates.push(format!("{base}.{extension}"));
        }

        for extension in extensions {
            candidates.push(format!("{base}/index.{extension}"));
        }

        if extensions.contains(&"py") {
            candidates.push(format!("{base}/__init__.py"));
        }

        if extensions.contains(&"rs") {
            candidates.push(format!("{base}/mod.rs"));
        }
    }

    resolve_candidates(candidates, file_set)
}

fn resolve_candidates(candidates: Vec<String>, file_set: &BTreeSet<String>) -> Resolution {
    for candidate in candidates {
        let normalized = normalize_rel_path(&candidate);
        if file_set.contains(&normalized) {
            return Resolution::resolved(normalized);
        }
    }

    Resolution::unresolved("target_not_found")
}

#[allow(clippy::too_many_arguments)]
fn edge(
    from_path: &str,
    from_line: usize,
    from_col: usize,
    import_path: &str,
    dependency_kind: &str,
    lang: &str,
    resolution: Resolution,
    evidence: &str,
    source: &str,
) -> DependencyEdge {
    DependencyEdge::new(
        from_path,
        from_line,
        from_col,
        import_path,
        resolution.target_path,
        dependency_kind,
        lang,
        resolution.confidence,
        resolution.unresolved_reason,
        evidence,
        source,
    )
}

#[derive(Debug, Clone)]
struct Resolution {
    target_path: Option<String>,
    confidence: &'static str,
    unresolved_reason: Option<&'static str>,
}

impl Resolution {
    fn resolved(target_path: String) -> Self {
        Self {
            target_path: Some(target_path),
            confidence: CONFIDENCE_RESOLVED,
            unresolved_reason: None,
        }
    }

    fn unresolved(reason: &'static str) -> Self {
        Self {
            target_path: None,
            confidence: CONFIDENCE_UNRESOLVED,
            unresolved_reason: Some(reason),
        }
    }
}

fn clean_import_path(value: &str) -> Option<String> {
    let mut cleaned = value.trim().trim_end_matches(';').trim();

    if cleaned.is_empty() {
        return None;
    }

    if let Some((_, quoted)) = quoted_value(cleaned) {
        return Some(quoted.to_string());
    }

    cleaned = cleaned
        .trim_matches('"')
        .trim_matches('\'')
        .trim_matches('`')
        .trim();

    cleaned = cleaned
        .strip_prefix('<')
        .unwrap_or(cleaned)
        .strip_suffix('>')
        .unwrap_or(cleaned)
        .trim();

    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn quoted_value(value: &str) -> Option<(usize, &str)> {
    let quote_start = value.find(['"', '\'', '`'])?;
    let quote = value[quote_start..].chars().next()?;
    let value_start = quote_start + quote.len_utf8();
    let rest = &value[value_start..];
    let value_end = rest.find(quote)?;

    Some((value_start, &rest[..value_end]))
}

fn shell_path_value(value: &str) -> Option<(usize, &str)> {
    if let Some(quoted) = quoted_value(value) {
        return Some(quoted);
    }

    let trimmed = value.trim_start();
    let leading = value.len() - trimmed.len();
    let end = trimmed
        .find(|ch: char| ch.is_whitespace() || ch == ';')
        .unwrap_or(trimmed.len());
    let path = &trimmed[..end];

    if path.is_empty() {
        None
    } else {
        Some((leading, path))
    }
}

fn is_system_include(import_path: &str) -> bool {
    matches!(
        import_path,
        "assert.h"
            | "ctype.h"
            | "errno.h"
            | "float.h"
            | "limits.h"
            | "math.h"
            | "stddef.h"
            | "stdint.h"
            | "stdio.h"
            | "stdlib.h"
            | "string.h"
            | "time.h"
            | "unistd.h"
            | "iostream"
            | "memory"
            | "string"
            | "vector"
    )
}

fn lang_from_path(path: &str) -> &str {
    path.rsplit_once('.')
        .map(|(_, ext)| ext)
        .unwrap_or("unknown")
}

fn take_identifier(value: &str) -> &str {
    let end = value
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_alphanumeric() || *ch == '_')
        .map(|(idx, ch)| idx + ch.len_utf8())
        .last()
        .unwrap_or(0);

    &value[..end]
}

fn parent_dir(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default()
}

fn join_rel(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        normalize_rel_path(child)
    } else {
        normalize_rel_path(&format!("{parent}/{child}"))
    }
}

fn normalize_rel_path(path: &str) -> String {
    let mut parts = Vec::new();
    let normalized = path.replace('\\', "/");

    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }

    parts.join("/")
}

fn read_utf8_source_file(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == ErrorKind::InvalidData => Ok(None),
        Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
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
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_path_normalization_handles_parent_segments() {
        assert_eq!(
            normalize_rel_path("src/../lib/service.py"),
            "lib/service.py"
        );
        assert_eq!(join_rel("src/app", "../lib/service"), "src/lib/service");
    }
}
