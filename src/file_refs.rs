use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result};
use regex::Regex;

use crate::model::{DependencyEdge, FileReference};

const SOURCE_DEPENDENCY_GRAPH: &str = "dependency_graph";
const SOURCE_FILE_SCAN: &str = "file_reference_scan";

const REASON_ABSOLUTE_PATH: &str = "absolute_path";
const REASON_AMBIGUOUS_MATCH: &str = "ambiguous_match";
const REASON_EXTERNAL_URL: &str = "external_url";
const REASON_TARGET_NOT_FOUND: &str = "target_not_found";

const COMMON_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "jsx", "ts", "tsx", "mjs", "cjs", "json", "toml", "yaml", "yml", "md",
    "markdown", "html", "htm", "css", "scss", "sass", "png", "jpg", "jpeg", "svg", "gif", "webp",
    "ico", "sh", "bash", "rb", "php", "go", "h", "hpp", "hh", "c", "cc", "cpp",
];

const DIRECTORY_INDEXES: &[&str] = &[
    "index.rs",
    "mod.rs",
    "__init__.py",
    "index.py",
    "index.js",
    "index.jsx",
    "index.ts",
    "index.tsx",
    "index.mjs",
    "index.cjs",
    "index.html",
    "README.md",
    "readme.md",
];

#[derive(Debug, Clone)]
struct Candidate {
    raw: String,
    line: usize,
    col: usize,
    kind: &'static str,
    evidence: String,
}

#[derive(Debug, Clone)]
struct Resolution {
    target_path: Option<String>,
    confidence: &'static str,
    unresolved_reason: Option<&'static str>,
}

pub fn extract_file_references(
    root: &Path,
    files: &[PathBuf],
    dependencies: &[DependencyEdge],
) -> Result<Vec<FileReference>> {
    let resolver = FileResolver::new(root, files)?;
    let mut references = Vec::new();

    references.extend(dependencies.iter().map(file_reference_from_dependency));

    for path in files {
        let rel = relpath(root, path)?;
        let Some(text) = read_utf8_file(path)? else {
            continue;
        };

        for candidate in scan_file_candidates(&rel, &text) {
            if is_external_or_fragment(&candidate.raw) {
                continue;
            }

            let resolution = resolver.resolve(&rel, &candidate.raw);
            references.push(FileReference::new(
                rel.clone(),
                candidate.line,
                candidate.col,
                candidate.raw,
                resolution.target_path,
                candidate.kind,
                lang_from_path(&rel),
                resolution.confidence,
                resolution.unresolved_reason,
                candidate.evidence,
                SOURCE_FILE_SCAN,
            ));
        }
    }

    Ok(finalize_file_references(references))
}

pub fn finalize_file_references(mut references: Vec<FileReference>) -> Vec<FileReference> {
    references.sort_by(|a, b| {
        a.source_path
            .cmp(&b.source_path)
            .then(a.source_line.cmp(&b.source_line))
            .then(a.source_col.cmp(&b.source_col))
            .then(a.raw_target.cmp(&b.raw_target))
            .then(a.reference_kind.cmp(&b.reference_kind))
            .then(a.target_path.cmp(&b.target_path))
            .then(a.source.cmp(&b.source))
    });

    let mut seen = BTreeSet::new();
    references.retain(|reference| {
        seen.insert((
            reference.source_path.clone(),
            reference.raw_target.clone(),
            reference.target_path.clone(),
            reference.reference_kind.clone(),
        ))
    });

    references
}

fn file_reference_from_dependency(dependency: &DependencyEdge) -> FileReference {
    FileReference::new(
        dependency.from_path.clone(),
        dependency.from_line,
        dependency.from_col,
        dependency.import_path.clone(),
        dependency.target_path.clone(),
        reference_kind_from_dependency(&dependency.dependency_kind),
        dependency.lang.clone(),
        dependency.confidence.clone(),
        dependency.unresolved_reason.clone(),
        dependency.evidence.clone(),
        SOURCE_DEPENDENCY_GRAPH,
    )
}

fn reference_kind_from_dependency(kind: &str) -> &'static str {
    if kind.contains("include") {
        "include"
    } else if kind.contains("require") {
        "require"
    } else if kind.contains("source") {
        "source"
    } else {
        "import"
    }
}

fn scan_file_candidates(rel: &str, text: &str) -> Vec<Candidate> {
    match lang_from_path(rel) {
        "md" | "markdown" => scan_markdown(text),
        "html" | "htm" => scan_html(text),
        "css" | "scss" | "sass" => scan_css(text),
        "json" | "toml" | "yaml" | "yml" => scan_config(rel, text),
        _ if is_package_or_build_file(rel) => scan_config(rel, text),
        _ => Vec::new(),
    }
}

fn scan_markdown(text: &str) -> Vec<Candidate> {
    let link_re = Regex::new(r"!?\[[^\]\n]*\]\(([^)\s]+)(?:\s+[^)]*)?\)").expect("valid regex");
    let mut candidates = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        for caps in link_re.captures_iter(line) {
            let whole = caps.get(0).expect("whole match");
            let target = caps.get(1).expect("target match");
            let raw = target.as_str().trim();
            if !is_path_like(raw) {
                continue;
            }

            candidates.push(Candidate {
                raw: raw.to_string(),
                line: line_no + 1,
                col: target.start() + 1,
                kind: if whole.as_str().starts_with("!") || is_asset_path(raw) {
                    "asset"
                } else {
                    "link"
                },
                evidence: line.trim().to_string(),
            });
        }
    }

    candidates
}

fn scan_html(text: &str) -> Vec<Candidate> {
    let attr_re =
        Regex::new(r#"(?i)(href|src|poster|action)\s*=\s*["']([^"']+)["']"#).expect("valid regex");
    let mut candidates = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        let lower = line.to_ascii_lowercase();
        for caps in attr_re.captures_iter(line) {
            let attr = caps
                .get(1)
                .expect("attr match")
                .as_str()
                .to_ascii_lowercase();
            let target = caps.get(2).expect("target match");
            let raw = target.as_str().trim();
            if !is_path_like(raw) {
                continue;
            }

            candidates.push(Candidate {
                raw: raw.to_string(),
                line: line_no + 1,
                col: target.start() + 1,
                kind: html_reference_kind(&lower, &attr, raw),
                evidence: line.trim().to_string(),
            });
        }
    }

    candidates
}

fn html_reference_kind(line: &str, attr: &str, raw: &str) -> &'static str {
    if attr == "src" && line.contains("<script") {
        "script"
    } else if attr == "href" && (line.contains("stylesheet") || css_like_path(raw)) {
        "stylesheet"
    } else if attr == "src" || attr == "poster" || is_asset_path(raw) {
        "asset"
    } else {
        "link"
    }
}

fn scan_css(text: &str) -> Vec<Candidate> {
    let url_re = Regex::new(r#"url\(\s*["']?([^"')]+)["']?\s*\)"#).expect("valid regex");
    let mut candidates = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        for caps in url_re.captures_iter(line) {
            let target = caps.get(1).expect("target match");
            let raw = target.as_str().trim();
            if !is_path_like(raw) {
                continue;
            }

            candidates.push(Candidate {
                raw: raw.to_string(),
                line: line_no + 1,
                col: target.start() + 1,
                kind: "asset",
                evidence: line.trim().to_string(),
            });
        }
    }

    candidates
}

fn scan_config(rel: &str, text: &str) -> Vec<Candidate> {
    let key_re = Regex::new(r#"(?i)["']?([a-z0-9_.-]*?(?:path|file|files|dir|dirs|main|module|types|typings|bin|browser|style|styles|script|scripts|source|sources|include|includes|exclude|extends|schema|template|templates|fixture|fixtures|config|entry|entrypoint|root|rootdir|outdir|workflows?))["']?\s*[:=]\s*(.+)"#)
        .expect("valid regex");
    let quoted_re = Regex::new(r#""([^"]+)"|'([^']+)'"#).expect("valid regex");
    let mut candidates = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        let Some(caps) = key_re.captures(line) else {
            continue;
        };
        let Some(values) = caps.get(2) else {
            continue;
        };

        for value_caps in quoted_re.captures_iter(values.as_str()) {
            let target = value_caps
                .get(1)
                .or_else(|| value_caps.get(2))
                .expect("quoted capture");
            let raw = target.as_str().trim();
            if !is_path_like(raw) {
                continue;
            }

            let col = values.start() + target.start() + 1;
            candidates.push(Candidate {
                raw: raw.to_string(),
                line: line_no + 1,
                col,
                kind: config_reference_kind(rel, raw),
                evidence: line.trim().to_string(),
            });
        }
    }

    candidates
}

fn config_reference_kind(rel: &str, raw: &str) -> &'static str {
    if is_testish_path(rel) || raw.contains("fixture") || raw.contains("fixtures/") {
        "fixture"
    } else if is_package_or_build_file(rel) {
        "package_entry"
    } else {
        "config_path"
    }
}

#[derive(Debug, Clone)]
struct FileResolver {
    files: BTreeSet<String>,
    lower_to_canonical: BTreeMap<String, String>,
}

impl FileResolver {
    fn new(root: &Path, files: &[PathBuf]) -> Result<Self> {
        let mut file_set = BTreeSet::new();
        let mut lower_to_canonical = BTreeMap::new();

        for path in files {
            let rel = relpath(root, path)?;
            lower_to_canonical.insert(rel.to_ascii_lowercase(), rel.clone());
            file_set.insert(rel);
        }

        Ok(Self {
            files: file_set,
            lower_to_canonical,
        })
    }

    fn resolve(&self, from_path: &str, raw_target: &str) -> Resolution {
        let Some(cleaned) = clean_local_target(raw_target) else {
            return Resolution::unresolved(REASON_EXTERNAL_URL);
        };

        if cleaned.starts_with('/') {
            return Resolution::unresolved(REASON_ABSOLUTE_PATH);
        }

        let base = if cleaned.starts_with("./")
            || cleaned.starts_with("../")
            || should_resolve_bare_name_relative(from_path, &cleaned)
        {
            join_rel(&parent_dir(from_path), &cleaned)
        } else {
            normalize_rel_path(&cleaned)
        };

        let matches = self.match_candidates(path_candidates(&base, &cleaned));
        match matches.as_slice() {
            [] => Resolution::unresolved(REASON_TARGET_NOT_FOUND),
            [target] => Resolution::resolved(target.clone()),
            _ => Resolution::ambiguous(),
        }
    }

    fn match_candidates(&self, candidates: Vec<String>) -> Vec<String> {
        let mut matches = BTreeSet::new();

        for candidate in candidates {
            let normalized = normalize_rel_path(&candidate);
            if self.files.contains(&normalized) {
                matches.insert(normalized);
                continue;
            }

            if let Some(canonical) = self
                .lower_to_canonical
                .get(&normalized.to_ascii_lowercase())
            {
                matches.insert(canonical.clone());
            }
        }

        matches.into_iter().collect()
    }
}

fn path_candidates(base: &str, raw: &str) -> Vec<String> {
    let mut candidates = vec![base.to_string()];
    let path = Path::new(raw);

    if path.extension().is_none() {
        for extension in COMMON_EXTENSIONS {
            candidates.push(format!("{base}.{extension}"));
        }

        for index in DIRECTORY_INDEXES {
            candidates.push(format!("{base}/{index}"));
        }
    }

    candidates
}

fn clean_local_target(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if is_external_or_fragment(trimmed) {
        return None;
    }

    let without_query = trimmed
        .split_once('#')
        .map(|(path, _)| path)
        .unwrap_or(trimmed)
        .split_once('?')
        .map(|(path, _)| path)
        .unwrap_or_else(|| {
            trimmed
                .split_once('?')
                .map(|(path, _)| path)
                .unwrap_or(trimmed)
        })
        .trim();

    if without_query.is_empty() {
        None
    } else {
        Some(without_query.to_string())
    }
}

fn is_external_or_fragment(raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return true;
    }

    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
        || lower.starts_with("data:")
        || lower.starts_with("javascript:")
        || lower.starts_with("//")
}

fn is_path_like(raw: &str) -> bool {
    if is_external_or_fragment(raw) {
        return false;
    }

    let cleaned = clean_local_target(raw).unwrap_or_default();
    cleaned.starts_with("./")
        || cleaned.starts_with("../")
        || cleaned.starts_with('/')
        || cleaned.contains('/')
        || Path::new(&cleaned).extension().is_some()
}

fn is_asset_path(path: &str) -> bool {
    let extension = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    matches!(
        extension.as_str(),
        "png"
            | "jpg"
            | "jpeg"
            | "svg"
            | "gif"
            | "webp"
            | "ico"
            | "avif"
            | "bmp"
            | "woff"
            | "woff2"
            | "ttf"
            | "otf"
            | "mp4"
            | "webm"
            | "mp3"
            | "wav"
    )
}

fn css_like_path(path: &str) -> bool {
    matches!(
        Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "css" | "scss" | "sass"
    )
}

fn is_package_or_build_file(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    matches!(
        name,
        "package.json"
            | "Cargo.toml"
            | "pyproject.toml"
            | "tsconfig.json"
            | "composer.json"
            | "pubspec.yaml"
            | "pubspec.yml"
            | "Makefile"
            | "makefile"
            | "Justfile"
            | "justfile"
    ) || rel.starts_with(".github/workflows/")
}

fn should_resolve_bare_name_relative(from_path: &str, target: &str) -> bool {
    !target.contains('/')
        && Path::new(target).extension().is_some()
        && matches!(
            lang_from_path(from_path),
            "html" | "css" | "scss" | "sass" | "md"
        )
}

fn is_testish_path(rel: &str) -> bool {
    let normalized = rel.to_ascii_lowercase();
    normalized.starts_with("test/")
        || normalized.starts_with("tests/")
        || normalized.contains("/test/")
        || normalized.contains("/tests/")
        || normalized.contains("fixture")
}

fn relpath(root: &Path, path: &Path) -> Result<String> {
    let rel = path
        .strip_prefix(root)
        .with_context(|| format!("{} is not under {}", path.display(), root.display()))?;
    Ok(normalize_rel_path(&rel.to_string_lossy()))
}

fn read_utf8_file(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == ErrorKind::InvalidData => Ok(None),
        Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
    }
}

fn parent_dir(path: &str) -> String {
    Path::new(path)
        .parent()
        .map(|path| normalize_rel_path(&path.to_string_lossy()))
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
    for component in Path::new(path).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop();
            }
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::RootDir | Component::Prefix(_) => {}
        }
    }

    parts.join("/")
}

fn lang_from_path(path: &str) -> &'static str {
    let extension = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();

    match extension {
        "bash" => "sh",
        "htm" => "html",
        "markdown" => "md",
        "rs" => "rs",
        "py" => "py",
        "js" => "js",
        "jsx" => "jsx",
        "ts" => "ts",
        "tsx" => "tsx",
        "mjs" => "mjs",
        "cjs" => "cjs",
        "json" => "json",
        "toml" => "toml",
        "yaml" => "yaml",
        "yml" => "yml",
        "md" => "md",
        "html" => "html",
        "css" => "css",
        "scss" => "scss",
        "sass" => "sass",
        "rb" => "rb",
        "php" => "php",
        "go" => "go",
        "c" => "c",
        "cc" => "cc",
        "cpp" => "cpp",
        "h" => "h",
        "hpp" => "hpp",
        "sh" => "sh",
        _ => "text",
    }
}

impl Resolution {
    fn resolved(target_path: String) -> Self {
        Self {
            target_path: Some(target_path),
            confidence: "resolved",
            unresolved_reason: None,
        }
    }

    fn ambiguous() -> Self {
        Self {
            target_path: None,
            confidence: "ambiguous",
            unresolved_reason: Some(REASON_AMBIGUOUS_MATCH),
        }
    }

    fn unresolved(reason: &'static str) -> Self {
        Self {
            target_path: None,
            confidence: "unresolved",
            unresolved_reason: Some(reason),
        }
    }
}
