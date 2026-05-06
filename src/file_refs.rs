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
pub const MAX_FILE_REFERENCES_PER_FILE: usize = 96;
pub const MAX_FILE_REFERENCES_PER_FILE_KIND: usize = 64;

const REASON_ABSOLUTE_PATH: &str = "absolute_path";
const REASON_AMBIGUOUS_MATCH: &str = "ambiguous_match";
const REASON_EXTERNAL_PACKAGE: &str = "external_package";
const REASON_EXTERNAL_URL: &str = "external_url";
const REASON_SYSTEM_INCLUDE: &str = "system_include";
const REASON_TARGET_NOT_FOUND: &str = "target_not_found";

const COMMON_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "jsx", "ts", "tsx", "mjs", "cjs", "json", "toml", "yaml", "yml", "md",
    "markdown", "html", "htm", "css", "scss", "sass", "png", "jpg", "jpeg", "svg", "gif", "webp",
    "ico", "sh", "bash", "rb", "php", "go", "h", "hpp", "hh", "c", "cc", "cpp", "cs", "java", "kt",
    "swift", "dart", "resx", "config", "settings", "sln", "csproj", "props", "targets", "xml",
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileReferenceExtraction {
    pub references: Vec<FileReference>,
    pub warnings: Vec<FileReferenceCapWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileReferenceCapWarning {
    pub source_path: String,
    pub reference_kind: Option<String>,
    pub kept: usize,
    pub dropped: usize,
    pub cap: usize,
}

pub fn extract_file_references(
    root: &Path,
    files: &[PathBuf],
    dependencies: &[DependencyEdge],
) -> Result<FileReferenceExtraction> {
    let resolver = FileResolver::new(root, files)?;
    let mut references = Vec::new();

    references.extend(
        dependencies
            .iter()
            .filter_map(file_reference_from_dependency),
    );

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

pub fn finalize_file_references(mut references: Vec<FileReference>) -> FileReferenceExtraction {
    references.sort_by(|a, b| {
        a.source_path
            .cmp(&b.source_path)
            .then(a.source_line.cmp(&b.source_line))
            .then(a.source_col.cmp(&b.source_col))
            .then(a.raw_target.cmp(&b.raw_target))
            .then(a.reference_kind.cmp(&b.reference_kind))
            .then(
                file_reference_confidence_rank(&a.confidence)
                    .cmp(&file_reference_confidence_rank(&b.confidence)),
            )
            .then(a.source.cmp(&b.source))
            .then(a.target_path.cmp(&b.target_path))
    });

    let mut seen = BTreeSet::new();
    let mut deduped: Vec<FileReference> = references
        .into_iter()
        .filter(|reference| {
            seen.insert((
                reference.source_path.clone(),
                reference.raw_target.clone(),
                reference.reference_kind.clone(),
            ))
        })
        .collect();

    let warnings = apply_reference_caps(&mut deduped);

    FileReferenceExtraction {
        references: deduped,
        warnings,
    }
}

fn file_reference_confidence_rank(confidence: &str) -> usize {
    match confidence {
        "resolved" => 0,
        "ambiguous" => 1,
        "unresolved" => 2,
        _ => 3,
    }
}

fn file_reference_from_dependency(dependency: &DependencyEdge) -> Option<FileReference> {
    if matches!(
        dependency.unresolved_reason.as_deref(),
        Some(REASON_EXTERNAL_PACKAGE | REASON_SYSTEM_INCLUDE)
    ) {
        return None;
    }

    Some(FileReference::new(
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
    ))
}

fn apply_reference_caps(references: &mut Vec<FileReference>) -> Vec<FileReferenceCapWarning> {
    let mut kept_by_file: BTreeMap<String, usize> = BTreeMap::new();
    let mut kept_by_file_kind: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut dropped_by_file: BTreeMap<String, usize> = BTreeMap::new();
    let mut dropped_by_file_kind: BTreeMap<(String, String), usize> = BTreeMap::new();

    references.retain(|reference| {
        let file_count = kept_by_file
            .entry(reference.source_path.clone())
            .or_insert(0usize);
        let kind_key = (
            reference.source_path.clone(),
            reference.reference_kind.clone(),
        );
        let kind_count = kept_by_file_kind.entry(kind_key.clone()).or_insert(0usize);

        if *file_count >= MAX_FILE_REFERENCES_PER_FILE {
            *dropped_by_file
                .entry(reference.source_path.clone())
                .or_insert(0usize) += 1;
            return false;
        }

        if *kind_count >= MAX_FILE_REFERENCES_PER_FILE_KIND {
            *dropped_by_file_kind.entry(kind_key).or_insert(0usize) += 1;
            return false;
        }

        *file_count += 1;
        *kind_count += 1;
        true
    });

    let mut warnings = Vec::new();
    for (source_path, dropped) in dropped_by_file {
        warnings.push(FileReferenceCapWarning {
            kept: MAX_FILE_REFERENCES_PER_FILE,
            source_path,
            reference_kind: None,
            dropped,
            cap: MAX_FILE_REFERENCES_PER_FILE,
        });
    }
    for ((source_path, reference_kind), dropped) in dropped_by_file_kind {
        warnings.push(FileReferenceCapWarning {
            kept: MAX_FILE_REFERENCES_PER_FILE_KIND,
            source_path,
            reference_kind: Some(reference_kind),
            dropped,
            cap: MAX_FILE_REFERENCES_PER_FILE_KIND,
        });
    }

    warnings.sort_by(|a, b| {
        a.source_path
            .cmp(&b.source_path)
            .then(a.reference_kind.cmp(&b.reference_kind))
    });
    warnings
}

fn reference_kind_from_dependency(kind: &str) -> &'static str {
    if kind.contains("include") {
        "include"
    } else if kind.contains("export") {
        "export"
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
        "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" => scan_js_import_exports(text),
        "py" => scan_python_imports(text),
        "md" | "markdown" => scan_markdown(text),
        "html" | "htm" => scan_html(text),
        "css" | "scss" | "sass" => scan_css(text),
        "json" | "toml" | "yaml" | "yml" => scan_config(rel, text),
        _ if is_project_file(rel) => scan_project_file(text),
        _ if is_package_or_build_file(rel) => scan_config(rel, text),
        _ => Vec::new(),
    }
}

fn scan_js_import_exports(text: &str) -> Vec<Candidate> {
    let import_re = Regex::new(r#"\bimport\s+(?:[^"'`;\n]*?\s+from\s*)?["'`]([^"'`]+)["'`]"#)
        .expect("valid regex");
    let export_re = Regex::new(
        r#"\bexport\s+(?:type\s+)?(?:\*(?:\s+as\s+\w+)?|\{[^}\n]*\})\s+from\s*["'`]([^"'`]+)["'`]"#,
    )
    .expect("valid regex");
    let dynamic_re =
        Regex::new(r#"\bimport\s*\(\s*["'`]([^"'`]+)["'`]\s*\)"#).expect("valid regex");
    let mut candidates = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        push_regex_path_candidates(&mut candidates, line_no + 1, line, &import_re, "import");
        push_regex_path_candidates(&mut candidates, line_no + 1, line, &export_re, "export");
        push_regex_path_candidates(&mut candidates, line_no + 1, line, &dynamic_re, "import");
    }

    candidates
}

fn scan_python_imports(text: &str) -> Vec<Candidate> {
    let from_module_re =
        Regex::new(r#"^\s*from\s+([.]+[A-Za-z_][A-Za-z0-9_.]*)\s+import\s+"#).expect("valid regex");
    let from_current_re = Regex::new(r#"^\s*from\s+([.]+)\s+import\s+([A-Za-z_][A-Za-z0-9_]*)"#)
        .expect("valid regex");
    let mut candidates = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        if let Some(caps) = from_module_re.captures(line) {
            let target = caps.get(1).expect("target match");
            let raw = python_relative_module_to_path(target.as_str());
            candidates.push(Candidate {
                raw,
                line: line_no + 1,
                col: target.start() + 1,
                kind: "import",
                evidence: line.trim().to_string(),
            });
            continue;
        }

        if let Some(caps) = from_current_re.captures(line) {
            let dots = caps.get(1).expect("dots match");
            let target = caps.get(2).expect("target match");
            let raw =
                python_relative_module_to_path(&format!("{}{}", dots.as_str(), target.as_str()));
            candidates.push(Candidate {
                raw,
                line: line_no + 1,
                col: target.start() + 1,
                kind: "import",
                evidence: line.trim().to_string(),
            });
        }
    }

    candidates
}

fn push_regex_path_candidates(
    candidates: &mut Vec<Candidate>,
    line_no: usize,
    line: &str,
    regex: &Regex,
    kind: &'static str,
) {
    for caps in regex.captures_iter(line) {
        let target = caps.get(1).expect("target match");
        let raw = target.as_str().trim();
        if !is_js_local_module_target(raw) {
            continue;
        }

        candidates.push(Candidate {
            raw: raw.to_string(),
            line: line_no,
            col: target.start() + 1,
            kind,
            evidence: line.trim().to_string(),
        });
    }
}

fn is_js_local_module_target(raw: &str) -> bool {
    (raw.starts_with("./") || raw.starts_with("../") || raw.starts_with('/'))
        && !is_template_or_variable_target(raw)
}

fn python_relative_module_to_path(module: &str) -> String {
    let dots = module.chars().take_while(|ch| *ch == '.').count();
    let rest = module[dots..].replace('.', "/");
    let mut prefix = if dots <= 1 {
        "./".to_string()
    } else {
        "../".repeat(dots - 1)
    };
    prefix.push_str(&rest);
    prefix
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
    let attr_re = Regex::new(r#"(?i)(href|src|srcset|poster|action)\s*=\s*["']([^"']+)["']"#)
        .expect("valid regex");
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
            if attr == "srcset" {
                for srcset_target in split_srcset_targets(raw) {
                    if !is_path_like(srcset_target) {
                        continue;
                    }

                    candidates.push(Candidate {
                        raw: srcset_target.to_string(),
                        line: line_no + 1,
                        col: target.start() + raw.find(srcset_target).unwrap_or(0) + 1,
                        kind: "asset",
                        evidence: line.trim().to_string(),
                    });
                }
            } else {
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
    let import_re = Regex::new(r#"@import\s+(?:url\(\s*)?["']?([^"')\s;]+)"#).expect("valid regex");
    let mut candidates = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        for caps in import_re.captures_iter(line) {
            let target = caps.get(1).expect("target match");
            let raw = target.as_str().trim();
            if !is_path_like(raw) {
                continue;
            }

            candidates.push(Candidate {
                raw: raw.to_string(),
                line: line_no + 1,
                col: target.start() + 1,
                kind: "stylesheet",
                evidence: line.trim().to_string(),
            });
        }

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

fn scan_project_file(text: &str) -> Vec<Candidate> {
    let attr_re =
        Regex::new(r#"(?i)\b(include|update|remove|project|hintpath)\s*=\s*["']([^"']+)["']"#)
            .expect("valid regex");
    let mut candidates = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        for caps in attr_re.captures_iter(line) {
            let target = caps.get(2).expect("target match");
            let raw = target.as_str().trim();
            if !is_path_like(raw) {
                continue;
            }

            candidates.push(Candidate {
                raw: raw.to_string(),
                line: line_no + 1,
                col: target.start() + 1,
                kind: config_reference_kind("project.csproj", raw),
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
    } else if is_package_or_build_file(rel) || is_project_file(rel) {
        "package_entry"
    } else {
        "config_path"
    }
}

fn split_srcset_targets(raw: &str) -> impl Iterator<Item = &str> {
    raw.split(',')
        .filter_map(|entry| entry.split_whitespace().next())
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

        candidates.extend(stylesheet_partial_candidates(base, raw));

        for index in DIRECTORY_INDEXES {
            candidates.push(format!("{base}/{index}"));
        }
    }

    candidates
}

fn stylesheet_partial_candidates(base: &str, raw: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let base_path = Path::new(base);
    let Some(name) = base_path.file_name().and_then(|name| name.to_str()) else {
        return candidates;
    };

    let parent = base_path
        .parent()
        .map(|path| normalize_rel_path(&path.to_string_lossy()))
        .unwrap_or_default();
    for extension in ["scss", "sass", "css"] {
        if parent.is_empty() {
            candidates.push(format!("_{name}.{extension}"));
        } else {
            candidates.push(format!("{parent}/_{name}.{extension}"));
        }
    }

    if !raw.starts_with("./") && !raw.starts_with("../") && !raw.starts_with('/') {
        for extension in ["scss", "sass", "css"] {
            candidates.push(format!("_sass/{raw}.{extension}"));
            if let Some((dir, name)) = raw.rsplit_once('/') {
                candidates.push(format!("_sass/{dir}/_{name}.{extension}"));
            } else {
                candidates.push(format!("_sass/_{raw}.{extension}"));
            }
        }
    }

    candidates
}

fn clean_local_target(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if is_external_or_fragment(trimmed) {
        return None;
    }

    if is_template_or_variable_target(trimmed) {
        return None;
    }

    let mut end = trimmed.len();
    for delimiter in ['#', '?'] {
        if let Some(index) = trimmed.find(delimiter) {
            end = end.min(index);
        }
    }

    let cleaned = trimmed[..end].replace('\\', "/");
    let without_query = cleaned.trim();

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
        || lower.starts_with("package:")
        || lower.starts_with("tel:")
        || lower.starts_with("data:")
        || lower.starts_with("javascript:")
        || lower.starts_with("//")
        || has_non_file_uri_scheme(&lower)
}

fn is_path_like(raw: &str) -> bool {
    if is_external_or_fragment(raw) {
        return false;
    }

    let cleaned = clean_local_target(raw).unwrap_or_default();
    if cleaned.is_empty()
        || cleaned.contains(':')
        || is_template_or_variable_target(&cleaned)
        || looks_like_version(&cleaned)
    {
        return false;
    }

    cleaned.starts_with("./")
        || cleaned.starts_with("../")
        || cleaned.starts_with('/')
        || cleaned.contains('/')
        || raw.contains('\\')
        || has_known_file_extension(&cleaned)
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

fn is_project_file(rel: &str) -> bool {
    matches!(
        Path::new(rel)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "csproj" | "vbproj" | "fsproj" | "props" | "targets"
    )
}

fn has_known_file_extension(path: &str) -> bool {
    let extension = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    COMMON_EXTENSIONS.contains(&extension.as_str())
}

fn has_non_file_uri_scheme(lower: &str) -> bool {
    let Some((scheme, _)) = lower.split_once(':') else {
        return false;
    };

    !scheme.is_empty()
        && scheme
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
        && scheme
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic())
        && scheme.len() > 1
}

fn is_template_or_variable_target(raw: &str) -> bool {
    raw.contains("{{")
        || raw.contains("}}")
        || raw.contains("{%")
        || raw.contains("%}")
        || raw.contains("$(")
        || raw.contains("${")
        || raw.starts_with('$')
        || raw.contains("{ ")
        || raw.contains(" }")
}

fn looks_like_version(raw: &str) -> bool {
    let mut saw_dot = false;
    let mut saw_digit = false;

    for ch in raw.chars() {
        if ch.is_ascii_digit() {
            saw_digit = true;
        } else if ch == '.' {
            saw_dot = true;
        } else {
            return false;
        }
    }

    saw_dot && saw_digit
}

fn should_resolve_bare_name_relative(from_path: &str, target: &str) -> bool {
    !target.contains('/')
        && Path::new(target).extension().is_some()
        && (is_project_file(from_path)
            || matches!(
                lang_from_path(from_path),
                "html" | "css" | "scss" | "sass" | "md"
            ))
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
