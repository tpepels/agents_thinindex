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
pub const CONFIDENCE_AMBIGUOUS: &str = "ambiguous";
pub const CONFIDENCE_RESOLVED: &str = "resolved";
pub const CONFIDENCE_UNRESOLVED: &str = "unresolved";

const REASON_ABSOLUTE_PATH: &str = "absolute_path";
const REASON_AMBIGUOUS_MATCH: &str = "ambiguous_match";
const REASON_EXTERNAL_PACKAGE: &str = "external_package";
const REASON_SYSTEM_INCLUDE: &str = "system_include";
const REASON_TARGET_NOT_FOUND: &str = "target_not_found";
const REASON_UNSUPPORTED_LANGUAGE: &str = "unsupported_language";

const C_EXTENSIONS: &[&str] = &["h", "hpp", "hh", "hxx"];
const DART_EXTENSIONS: &[&str] = &["dart"];
const JS_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mjs", "cjs", "json"];
const KOTLIN_EXTENSIONS: &[&str] = &["kt", "kts"];
const NIX_EXTENSIONS: &[&str] = &["nix"];
const PHP_EXTENSIONS: &[&str] = &["php"];
const RUBY_EXTENSIONS: &[&str] = &["rb"];
const SHELL_EXTENSIONS: &[&str] = &["sh", "bash"];

const PYTHON_SOURCE_ROOTS: &[&str] = &["src", "lib", "app"];
const JVM_SOURCE_ROOTS: &[&str] = &[
    "src/main/java",
    "src/test/java",
    "src/main/kotlin",
    "src/test/kotlin",
    "src/main/scala",
    "src/test/scala",
    "src",
];
const DOTNET_SOURCE_ROOTS: &[&str] = &["src", "test", "tests"];

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

    let resolver = ResolverIndex::new(root, file_set)?;
    let mut edges = Vec::new();

    for record in records {
        if record.kind == "import"
            && record.source == "tree_sitter"
            && let Some(edge) = edge_from_import_record(record, &resolver)
        {
            edges.push(edge);
        }
    }

    for (path, rel) in rel_files {
        let Some(text) = read_utf8_source_file(&path)? else {
            continue;
        };

        extract_scanned_dependencies(&rel, &text, &resolver, &mut edges);
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
    resolver: &ResolverIndex,
) -> Option<DependencyEdge> {
    let import_path = clean_import_path(&record.name)?;
    let import_path = normalize_dependency_import_path(record, import_path);
    let dependency_kind = dependency_kind_for_record(record)?;
    let resolution = resolve_import(
        &record.path,
        &record.lang,
        dependency_kind,
        &import_path,
        resolver,
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
        "cs" => Some("cs_import"),
        "c" => Some("c_include"),
        "cpp" => Some("cpp_include"),
        "php" => Some("php_include"),
        "scala" => Some("scala_import"),
        "kt" => Some("kotlin_import"),
        "swift" => Some("swift_import"),
        "dart" => Some("dart_import"),
        _ => None,
    }
}

fn extract_scanned_dependencies(
    rel: &str,
    text: &str,
    resolver: &ResolverIndex,
    edges: &mut Vec<DependencyEdge>,
) {
    let lang = lang_from_path(rel);

    for (line_index, line) in text.lines().enumerate() {
        let line_no = line_index + 1;

        match lang {
            "rs" => extract_rust_dependencies(rel, line_no, line, resolver, edges),
            "rb" => extract_ruby_dependencies(rel, line_no, line, resolver, edges),
            "sh" | "bash" => extract_shell_dependencies(rel, line_no, line, resolver, edges),
            "nix" => extract_nix_dependencies(rel, line_no, line, resolver, edges),
            _ => {}
        }
    }
}

fn extract_rust_dependencies(
    rel: &str,
    line_no: usize,
    line: &str,
    resolver: &ResolverIndex,
    edges: &mut Vec<DependencyEdge>,
) {
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();

    if let Some((prefix, rest)) = rust_mod_rest(trimmed) {
        if rest.contains('{') {
            return;
        }

        let name = take_identifier(rest);
        if name.is_empty() {
            return;
        }

        let resolution = resolve_rust_module(rel, name, resolver);
        edges.push(edge(
            rel,
            line_no,
            leading + prefix.len() + 1,
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
    if import_path.is_empty() || import_path.contains('{') {
        return;
    }

    let resolution = resolve_rust_use(rel, import_path, resolver);
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
    resolver: &ResolverIndex,
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
        resolve_relative_module(rel, import_path, RUBY_EXTENSIONS, resolver)
    } else {
        resolve_root_module_or_external(import_path, RUBY_EXTENSIONS, resolver)
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
    resolver: &ResolverIndex,
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
        Resolution::unresolved(REASON_ABSOLUTE_PATH)
    } else {
        resolve_relative_or_root_path(rel, import_path, SHELL_EXTENSIONS, resolver)
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

fn extract_nix_dependencies(
    rel: &str,
    line_no: usize,
    line: &str,
    resolver: &ResolverIndex,
    edges: &mut Vec<DependencyEdge>,
) {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return;
    }

    let Some(import_offset) = trimmed.find("import ") else {
        return;
    };
    let rest = &trimmed[import_offset + "import ".len()..];
    let Some((offset, import_path)) = nix_path_value(rest) else {
        return;
    };
    if !(import_path.starts_with("./") || import_path.starts_with("../")) {
        return;
    }

    let leading = line.len() - trimmed.len();
    let resolution = resolve_relative_path(rel, import_path, NIX_EXTENSIONS, resolver);
    edges.push(edge(
        rel,
        line_no,
        leading + import_offset + "import ".len() + offset + 1,
        import_path,
        "nix_import",
        "nix",
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
    resolver: &ResolverIndex,
) -> Resolution {
    match dependency_kind {
        "python_import" => resolve_python_import(from_path, import_path, resolver),
        "js_import" => resolve_js_import(from_path, import_path, resolver),
        "go_import" => resolve_go_import(from_path, import_path, resolver),
        "java_import" => resolve_jvm_import(import_path, &["java"], resolver),
        "kotlin_import" => resolve_jvm_import(import_path, KOTLIN_EXTENSIONS, resolver),
        "scala_import" => resolve_jvm_import(import_path, &["scala"], resolver),
        "cs_import" => resolve_dotnet_import(import_path, resolver),
        "c_include" | "cpp_include" => resolve_c_include(from_path, import_path, resolver),
        "php_include" => {
            resolve_relative_or_root_path(from_path, import_path, PHP_EXTENSIONS, resolver)
        }
        "swift_import" => resolve_swift_import(import_path, resolver),
        "dart_import" => resolve_dart_import(from_path, import_path, resolver),
        _ if lang == "nix" => Resolution::unresolved(REASON_TARGET_NOT_FOUND),
        _ => Resolution::unresolved(REASON_UNSUPPORTED_LANGUAGE),
    }
}

fn resolve_python_import(
    from_path: &str,
    import_path: &str,
    resolver: &ResolverIndex,
) -> Resolution {
    let module = import_path.trim_start_matches('.');
    if module.is_empty() {
        return Resolution::unresolved(REASON_TARGET_NOT_FOUND);
    }

    let module_path = module.replace('.', "/");
    let parent = parent_dir(from_path);
    let mut tiers = Vec::new();

    if !parent.is_empty() {
        tiers.push(python_module_candidates(&join_rel(&parent, &module_path)));
    }
    for root in PYTHON_SOURCE_ROOTS {
        tiers.push(python_module_candidates(&join_rel(root, &module_path)));
    }
    tiers.push(python_module_candidates(&module_path));

    let not_found_reason = if !import_path.starts_with('.') && !module.contains('.') {
        REASON_EXTERNAL_PACKAGE
    } else {
        REASON_TARGET_NOT_FOUND
    };

    resolve_candidate_tiers(tiers, resolver, not_found_reason)
}

fn resolve_js_import(from_path: &str, import_path: &str, resolver: &ResolverIndex) -> Resolution {
    if import_path.starts_with("./") || import_path.starts_with("../") {
        return resolve_relative_path(from_path, import_path, JS_EXTENSIONS, resolver);
    }

    if let Some(package_name) = &resolver.js_package_name
        && let Some(rest) = import_path.strip_prefix(package_name)
        && (rest.is_empty() || rest.starts_with('/'))
    {
        let local_path = rest.trim_start_matches('/');
        if !local_path.is_empty() {
            return resolve_root_relative_path(local_path, JS_EXTENSIONS, resolver);
        }
    }

    Resolution::unresolved(REASON_EXTERNAL_PACKAGE)
}

fn resolve_go_import(from_path: &str, import_path: &str, resolver: &ResolverIndex) -> Resolution {
    if import_path.starts_with("./") || import_path.starts_with("../") {
        return resolve_relative_path(from_path, import_path, &["go"], resolver);
    }

    if let Some(module) = &resolver.go_module
        && let Some(rest) = import_path.strip_prefix(module)
        && (rest.is_empty() || rest.starts_with('/'))
    {
        let package_path = rest.trim_start_matches('/');
        if !package_path.is_empty() {
            let matches = resolver.matching_files_in_dir(package_path, &["go"]);
            return resolution_from_matches(matches, REASON_TARGET_NOT_FOUND);
        }
    }

    let suffix = import_path.trim_matches('/');
    let mut matches = BTreeSet::new();

    for candidate in resolver.files() {
        if !file_has_extension(candidate, &["go"]) {
            continue;
        }

        let parent = parent_dir(candidate);
        if parent == suffix || parent.ends_with(&format!("/{suffix}")) {
            matches.insert(candidate.clone());
            continue;
        }

        let parts: Vec<&str> = parent.split('/').collect();
        for index in 1..parts.len() {
            let local_suffix = parts[index..].join("/");
            if suffix.ends_with(&local_suffix) {
                matches.insert(candidate.clone());
            }
        }
    }

    if matches.is_empty() {
        Resolution::unresolved(REASON_EXTERNAL_PACKAGE)
    } else {
        resolution_from_matches(matches.into_iter().collect(), REASON_EXTERNAL_PACKAGE)
    }
}

fn resolve_jvm_import(
    import_path: &str,
    extensions: &[&str],
    resolver: &ResolverIndex,
) -> Resolution {
    if is_known_jvm_external(import_path) {
        return Resolution::unresolved(REASON_EXTERNAL_PACKAGE);
    }

    let Some(module_path) = normalized_jvm_import_path(import_path) else {
        return Resolution::unresolved(REASON_TARGET_NOT_FOUND);
    };
    let is_package_import = import_path.ends_with(".*") || import_path.ends_with("._");

    let mut tiers = Vec::new();
    if is_package_import {
        tiers.push(resolver.matching_files_in_dir(&module_path, extensions));
    } else {
        let mut source_root_candidates = Vec::new();
        for root in JVM_SOURCE_ROOTS {
            for extension in extensions {
                source_root_candidates.push(format!("{}/{module_path}.{extension}", root));
            }
        }
        tiers.push(source_root_candidates);

        let mut suffix_matches = Vec::new();
        for extension in extensions {
            suffix_matches
                .extend(resolver.matching_file_suffix(&format!("{module_path}.{extension}")));
        }
        tiers.push(suffix_matches);

        tiers.push(resolver.matching_files_in_dir(&module_path, extensions));
    }

    resolve_candidate_tiers(tiers, resolver, REASON_TARGET_NOT_FOUND)
}

fn resolve_dotnet_import(import_path: &str, resolver: &ResolverIndex) -> Resolution {
    let cleaned = import_path
        .trim()
        .strip_prefix("static ")
        .unwrap_or(import_path.trim())
        .trim();

    if cleaned.starts_with("System.") || cleaned == "System" || cleaned.starts_with("Microsoft.") {
        return Resolution::unresolved(REASON_EXTERNAL_PACKAGE);
    }

    let module_path = cleaned.replace('.', "/");
    let mut tiers = Vec::new();

    let mut source_root_candidates = Vec::new();
    for root in DOTNET_SOURCE_ROOTS {
        source_root_candidates.push(format!("{root}/{module_path}.cs"));
    }
    tiers.push(source_root_candidates);
    tiers.push(resolver.matching_file_suffix(&format!("{module_path}.cs")));
    tiers.push(resolver.matching_files_in_dir(&module_path, &["cs"]));

    resolve_candidate_tiers(tiers, resolver, REASON_TARGET_NOT_FOUND)
}

fn resolve_c_include(from_path: &str, import_path: &str, resolver: &ResolverIndex) -> Resolution {
    if is_system_include(import_path) {
        return Resolution::unresolved(REASON_SYSTEM_INCLUDE);
    }

    let relative = resolve_relative_path(from_path, import_path, C_EXTENSIONS, resolver);
    if relative.target_path.is_some() || relative.confidence == CONFIDENCE_AMBIGUOUS {
        return relative;
    }

    resolution_from_matches(
        resolver.matching_file_suffix(import_path),
        REASON_TARGET_NOT_FOUND,
    )
}

fn resolve_swift_import(import_path: &str, resolver: &ResolverIndex) -> Resolution {
    let module_path = import_path.replace('.', "/");
    let matches = resolver.matching_file_suffix(&format!("{module_path}.swift"));
    if matches.is_empty() {
        Resolution::unresolved(REASON_EXTERNAL_PACKAGE)
    } else {
        resolution_from_matches(matches, REASON_TARGET_NOT_FOUND)
    }
}

fn resolve_dart_import(from_path: &str, import_path: &str, resolver: &ResolverIndex) -> Resolution {
    if import_path.starts_with("dart:") {
        return Resolution::unresolved(REASON_EXTERNAL_PACKAGE);
    }

    if let Some(rest) = import_path.strip_prefix("package:") {
        let Some((package, package_path)) = rest.split_once('/') else {
            return Resolution::unresolved(REASON_EXTERNAL_PACKAGE);
        };
        if resolver.dart_package_name.as_deref() == Some(package) {
            return resolve_root_relative_path(
                &join_rel("lib", package_path),
                DART_EXTENSIONS,
                resolver,
            );
        }
        return Resolution::unresolved(REASON_EXTERNAL_PACKAGE);
    }

    if import_path.starts_with("./") || import_path.starts_with("../") {
        return resolve_relative_path(from_path, import_path, DART_EXTENSIONS, resolver);
    }

    Resolution::unresolved(REASON_TARGET_NOT_FOUND)
}

fn resolve_rust_module(from_path: &str, module: &str, resolver: &ResolverIndex) -> Resolution {
    let parent = parent_dir(from_path);
    let base = if parent.is_empty() {
        module.to_string()
    } else {
        join_rel(&parent, module)
    };

    resolve_candidates(
        vec![format!("{base}.rs"), format!("{base}/mod.rs")],
        resolver,
    )
}

fn resolve_rust_use(from_path: &str, import_path: &str, resolver: &ResolverIndex) -> Resolution {
    let module_path = rust_use_module_path(from_path, import_path);
    let Some(module_path) = module_path else {
        return Resolution::unresolved(REASON_EXTERNAL_PACKAGE);
    };

    resolve_candidates(
        vec![format!("{module_path}.rs"), format!("{module_path}/mod.rs")],
        resolver,
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
        "self" => {
            let parent = parent_dir(from_path);
            Some(join_rel(&parent, &joined))
        }
        _ => None,
    }
}

fn resolve_relative_module(
    from_path: &str,
    import_path: &str,
    extensions: &[&str],
    resolver: &ResolverIndex,
) -> Resolution {
    resolve_relative_path(from_path, import_path, extensions, resolver)
}

fn resolve_relative_or_root_path(
    from_path: &str,
    import_path: &str,
    extensions: &[&str],
    resolver: &ResolverIndex,
) -> Resolution {
    let relative = resolve_relative_path(from_path, import_path, extensions, resolver);
    if relative.target_path.is_some() || relative.confidence == CONFIDENCE_AMBIGUOUS {
        return relative;
    }

    resolve_root_relative_path(import_path, extensions, resolver)
}

fn resolve_root_module_or_external(
    import_path: &str,
    extensions: &[&str],
    resolver: &ResolverIndex,
) -> Resolution {
    let resolution = resolve_root_relative_path(import_path, extensions, resolver);
    if resolution.target_path.is_some() || resolution.confidence == CONFIDENCE_AMBIGUOUS {
        resolution
    } else {
        Resolution::unresolved(REASON_EXTERNAL_PACKAGE)
    }
}

fn resolve_relative_path(
    from_path: &str,
    import_path: &str,
    extensions: &[&str],
    resolver: &ResolverIndex,
) -> Resolution {
    let parent = parent_dir(from_path);
    let base = join_rel(&parent, import_path);
    resolve_candidates(path_candidates(&base, import_path, extensions), resolver)
}

fn resolve_root_relative_path(
    import_path: &str,
    extensions: &[&str],
    resolver: &ResolverIndex,
) -> Resolution {
    let base = normalize_rel_path(import_path);
    resolve_candidates(path_candidates(&base, import_path, extensions), resolver)
}

fn resolve_candidates(candidates: Vec<String>, resolver: &ResolverIndex) -> Resolution {
    resolution_from_matches(
        resolver.matching_candidates(candidates),
        REASON_TARGET_NOT_FOUND,
    )
}

fn resolve_candidate_tiers(
    tiers: Vec<Vec<String>>,
    resolver: &ResolverIndex,
    not_found_reason: &'static str,
) -> Resolution {
    for tier in tiers {
        let matches = resolver.matching_candidates(tier);
        if !matches.is_empty() {
            return resolution_from_matches(matches, not_found_reason);
        }
    }

    Resolution::unresolved(not_found_reason)
}

fn resolution_from_matches(matches: Vec<String>, not_found_reason: &'static str) -> Resolution {
    match matches.as_slice() {
        [] => Resolution::unresolved(not_found_reason),
        [target] => Resolution::resolved(target.clone()),
        _ => Resolution::ambiguous(),
    }
}

fn path_candidates(base: &str, import_path: &str, extensions: &[&str]) -> Vec<String> {
    let mut candidates = Vec::new();

    candidates.push(base.to_string());

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

    candidates
}

fn python_module_candidates(base: &str) -> Vec<String> {
    vec![
        format!("{base}.py"),
        format!("{base}/__init__.py"),
        format!("{base}/mod.py"),
    ]
}

#[derive(Debug, Clone)]
struct ResolverIndex {
    files: BTreeSet<String>,
    go_module: Option<String>,
    js_package_name: Option<String>,
    dart_package_name: Option<String>,
}

impl ResolverIndex {
    fn new(root: &Path, files: BTreeSet<String>) -> Result<Self> {
        Ok(Self {
            files,
            go_module: read_go_module(root)?,
            js_package_name: read_js_package_name(root)?,
            dart_package_name: read_pubspec_name(root)?,
        })
    }

    fn files(&self) -> &BTreeSet<String> {
        &self.files
    }

    fn matching_candidates(&self, candidates: Vec<String>) -> Vec<String> {
        let mut matches = BTreeSet::new();

        for candidate in candidates {
            let normalized = normalize_rel_path(&candidate);
            if self.files.contains(&normalized) {
                matches.insert(normalized);
            }
        }

        matches.into_iter().collect()
    }

    fn matching_file_suffix(&self, suffix: &str) -> Vec<String> {
        let suffix = normalize_rel_path(suffix);
        self.files
            .iter()
            .filter(|candidate| *candidate == &suffix || candidate.ends_with(&format!("/{suffix}")))
            .cloned()
            .collect()
    }

    fn matching_files_in_dir(&self, dir: &str, extensions: &[&str]) -> Vec<String> {
        let dir = normalize_rel_path(dir);
        self.files
            .iter()
            .filter(|candidate| file_has_extension(candidate, extensions))
            .filter(|candidate| {
                let parent = parent_dir(candidate);
                parent == dir || parent.ends_with(&format!("/{dir}"))
            })
            .cloned()
            .collect()
    }
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

    fn ambiguous() -> Self {
        Self {
            target_path: None,
            confidence: CONFIDENCE_AMBIGUOUS,
            unresolved_reason: Some(REASON_AMBIGUOUS_MATCH),
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

fn normalize_dependency_import_path(record: &IndexRecord, import_path: String) -> String {
    if record.lang == "scala"
        && !import_path.contains('.')
        && let Some(full_path) = import_path_from_evidence(&record.text)
    {
        return full_path;
    }

    import_path
}

fn import_path_from_evidence(evidence: &str) -> Option<String> {
    let trimmed = evidence.trim();
    let path = trimmed
        .strip_prefix("import ")
        .or_else(|| trimmed.strip_prefix("import\t"))?
        .trim()
        .trim_end_matches(';')
        .trim();

    if path.is_empty() || path.contains('{') || path.contains('}') || path.contains(',') {
        return None;
    }

    path.split_whitespace().next().map(str::to_string)
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

fn nix_path_value(value: &str) -> Option<(usize, &str)> {
    if let Some(quoted) = quoted_value(value) {
        return Some(quoted);
    }

    let trimmed = value.trim_start();
    let leading = value.len() - trimmed.len();
    let end = trimmed
        .find(|ch: char| ch.is_whitespace() || matches!(ch, ';' | ')' | ']' | '}'))
        .unwrap_or(trimmed.len());
    let path = trimmed[..end]
        .trim_end_matches(';')
        .trim_end_matches(')')
        .trim_end_matches(']')
        .trim_end_matches('}');

    if path.is_empty() {
        None
    } else {
        Some((leading, path))
    }
}

fn rust_mod_rest(trimmed: &str) -> Option<(&'static str, &str)> {
    for prefix in ["mod ", "pub mod ", "pub(crate) mod ", "pub(super) mod "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return Some((prefix, rest));
        }
    }

    None
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

fn normalized_jvm_import_path(import_path: &str) -> Option<String> {
    let path = import_path
        .trim()
        .strip_prefix("static ")
        .unwrap_or(import_path.trim())
        .trim()
        .trim_end_matches(".*")
        .trim_end_matches("._")
        .trim();

    if path.is_empty() || path.contains('{') || path.contains('}') {
        None
    } else {
        Some(path.replace('.', "/"))
    }
}

fn is_known_jvm_external(import_path: &str) -> bool {
    matches!(
        import_path.split('.').next().unwrap_or_default(),
        "java" | "javax" | "jdk" | "kotlin" | "scala" | "sbt"
    ) || import_path.starts_with("org.w3c.")
        || import_path.starts_with("org.xml.")
}

fn file_has_extension(path: &str, extensions: &[&str]) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extensions.contains(&extension))
}

fn read_go_module(root: &Path) -> Result<Option<String>> {
    let path = root.join("go.mod");
    let Some(text) = read_optional_utf8(&path)? else {
        return Ok(None);
    };

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(module) = trimmed.strip_prefix("module ") {
            let module = module.trim();
            if !module.is_empty() {
                return Ok(Some(module.to_string()));
            }
        }
    }

    Ok(None)
}

fn read_js_package_name(root: &Path) -> Result<Option<String>> {
    let path = root.join("package.json");
    let Some(text) = read_optional_utf8(&path)? else {
        return Ok(None);
    };

    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Ok(None);
    };

    Ok(json
        .get("name")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string))
}

fn read_pubspec_name(root: &Path) -> Result<Option<String>> {
    let path = root.join("pubspec.yaml");
    let Some(text) = read_optional_utf8(&path)? else {
        return Ok(None);
    };

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        let Some(name) = trimmed.strip_prefix("name:") else {
            continue;
        };
        let name = name.trim().trim_matches('"').trim_matches('\'');
        if !name.is_empty() {
            return Ok(Some(name.to_string()));
        }
    }

    Ok(None)
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

fn read_optional_utf8(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
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
