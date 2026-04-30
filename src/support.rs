#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SupportLevel {
    Supported,
    Experimental,
    Blocked,
    ExtrasBacked,
}

impl SupportLevel {
    pub const ALL: [SupportLevel; 4] = [
        SupportLevel::Supported,
        SupportLevel::Experimental,
        SupportLevel::Blocked,
        SupportLevel::ExtrasBacked,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Experimental => "experimental",
            Self::Blocked => "blocked",
            Self::ExtrasBacked => "extras-backed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SupportBackend {
    TreeSitter,
    Extras,
    None,
}

impl SupportBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TreeSitter => "tree_sitter",
            Self::Extras => "extras",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupportEntry {
    pub name: &'static str,
    pub language_id: Option<&'static str>,
    pub extensions: &'static [&'static str],
    pub support_level: SupportLevel,
    pub backend: SupportBackend,
    pub record_kinds: &'static [&'static str],
    pub known_gaps: &'static str,
    pub license_status: &'static str,
    pub grammar_package: Option<&'static str>,
    pub conformance_fixture_repo: Option<&'static str>,
    pub conformance_fixture_path: Option<&'static str>,
}

pub fn support_matrix() -> &'static [SupportEntry] {
    SUPPORT_MATRIX
}

pub fn support_entries_by_level(level: SupportLevel) -> Vec<&'static SupportEntry> {
    SUPPORT_MATRIX
        .iter()
        .filter(|entry| entry.support_level == level)
        .collect()
}

pub fn support_entry_for_name(name: &str) -> Option<&'static SupportEntry> {
    SUPPORT_MATRIX
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case(name))
}

pub fn support_level_definitions() -> &'static [(&'static str, &'static str)] {
    &[
        (
            "supported",
            "grammar/query/fixture/license/docs exist; conformance passes; real-repo checks pass where configured",
        ),
        (
            "experimental",
            "grammar/query exists, but conformance or real-repo coverage is incomplete",
        ),
        (
            "blocked",
            "missing permissive grammar, broken integration, unclear license, or unacceptable parser quality",
        ),
        (
            "extras-backed",
            "project-owned extras intentionally handle deterministic format landmarks instead of Tree-sitter",
        ),
    ]
}

const SUPPORT_MATRIX: &[SupportEntry] = &[
    tree_sitter_supported(
        "Rust",
        "rs",
        &[".rs"],
        &[
            "function", "struct", "enum", "trait", "type", "module", "constant", "variable",
        ],
        "tree-sitter-rust",
        "src/rust/widget.rs",
        "use records are deferred to deterministic refs; no macro expansion or type resolution.",
    ),
    tree_sitter_supported(
        "Python",
        "py",
        &[".py"],
        &["function", "method", "class", "variable", "import"],
        "tree-sitter-python",
        "src/python/widget.py",
        "Syntactic extraction only; decorators and complex assignment targets are not semantic analysis.",
    ),
    tree_sitter_supported(
        "JavaScript",
        "js",
        &[".js"],
        &[
            "function", "method", "class", "variable", "import", "export",
        ],
        "tree-sitter-javascript",
        "src/javascript/widget.js",
        "No runtime module, prototype, or bundler resolution.",
    ),
    tree_sitter_supported(
        "JSX",
        "jsx",
        &[".jsx"],
        &[
            "function", "method", "class", "variable", "import", "export",
        ],
        "tree-sitter-javascript",
        "src/javascript/widget.jsx",
        "Definition extraction is Tree-sitter-backed; element usage remains deterministic reference evidence.",
    ),
    tree_sitter_supported(
        "TypeScript",
        "ts",
        &[".ts"],
        &[
            "function",
            "method",
            "class",
            "interface",
            "type",
            "variable",
            "import",
            "export",
        ],
        "tree-sitter-typescript",
        "src/typescript/widget.ts",
        "No type alias, generic constraint, or project graph resolution.",
    ),
    tree_sitter_supported(
        "TSX",
        "tsx",
        &[".tsx"],
        &[
            "function",
            "method",
            "class",
            "interface",
            "type",
            "variable",
            "import",
            "export",
        ],
        "tree-sitter-typescript",
        "src/typescript/widget.tsx",
        "Definition extraction is Tree-sitter-backed; element usage remains deterministic reference evidence.",
    ),
    tree_sitter_supported(
        "Java",
        "java",
        &[".java"],
        &[
            "method",
            "class",
            "enum",
            "interface",
            "type",
            "variable",
            "import",
        ],
        "tree-sitter-java",
        "src/java/JavaWidget.java",
        "No package visibility, inherited member, or build-system classpath resolution.",
    ),
    tree_sitter_supported(
        "Go",
        "go",
        &[".go"],
        &[
            "function",
            "method",
            "struct",
            "interface",
            "type",
            "module",
            "variable",
            "constant",
            "import",
        ],
        "tree-sitter-go",
        "src/go/widget.go",
        "No semantic exported API set or module graph resolution.",
    ),
    tree_sitter_supported(
        "C",
        "c",
        &[".c", ".h"],
        &["function", "struct", "enum", "type", "variable", "import"],
        "tree-sitter-c",
        "src/c/widget.c",
        "No macro expansion, preprocessor configuration, or compile database semantics.",
    ),
    tree_sitter_supported(
        "C#",
        "cs",
        &[".cs"],
        &[
            "method",
            "class",
            "struct",
            "enum",
            "interface",
            "type",
            "module",
            "variable",
            "import",
        ],
        "tree-sitter-c-sharp",
        "src/csharp/Widget.cs",
        "No partial-type, assembly, or Roslyn-level resolution.",
    ),
    tree_sitter_supported(
        "C++",
        "cpp",
        &[".cc", ".cpp", ".cxx", ".hh", ".hpp", ".hxx"],
        &[
            "function", "method", "class", "struct", "enum", "type", "module", "variable", "import",
        ],
        "tree-sitter-cpp",
        "src/cpp/widget.cpp",
        "No template instantiation, macro expansion, or compile database semantics.",
    ),
    tree_sitter_supported(
        "Shell",
        "sh",
        &[".sh", ".bash"],
        &["function", "variable"],
        "tree-sitter-bash",
        "src/shell/widget.sh",
        "Sourced files and shell runtime expansion are not resolved.",
    ),
    tree_sitter_supported(
        "Ruby",
        "rb",
        &[".rb"],
        &["method", "class", "module", "constant"],
        "tree-sitter-ruby",
        "src/ruby/widget.rb",
        "No require/load target or metaprogramming resolution.",
    ),
    tree_sitter_supported(
        "PHP",
        "php",
        &[".php"],
        &[
            "function",
            "method",
            "class",
            "interface",
            "trait",
            "enum",
            "module",
            "variable",
            "constant",
            "import",
        ],
        "tree-sitter-php",
        "src/php/widget.php",
        "No dynamic include, autoload, or runtime namespace resolution.",
    ),
    tree_sitter_experimental(
        "Scala",
        "scala",
        &[".scala"],
        &[
            "function", "class", "enum", "trait", "type", "module", "variable", "constant",
            "import",
        ],
        "tree-sitter-scala",
        "src/scala/Widget.scala",
        "Conformance exists, but real-repo coverage and givens/implicits/extension handling remain incomplete.",
    ),
    tree_sitter_experimental(
        "Kotlin",
        "kt",
        &[".kt", ".kts"],
        &[
            "function", "class", "enum", "type", "module", "variable", "import",
        ],
        "tree-sitter-kotlin-ng",
        "src/kotlin/Widget.kt",
        "Conformance exists, but real-repo coverage and interface/enum-class/extension distinctions remain incomplete.",
    ),
    tree_sitter_experimental(
        "Swift",
        "swift",
        &[".swift"],
        &[
            "function",
            "method",
            "class",
            "struct",
            "enum",
            "interface",
            "type",
            "variable",
            "import",
        ],
        "tree-sitter-swift",
        "src/swift/Widget.swift",
        "Conformance exists, but real-repo coverage and extension/overload/module handling remain incomplete.",
    ),
    tree_sitter_experimental(
        "Dart",
        "dart",
        &[".dart"],
        &[
            "function", "method", "class", "enum", "type", "variable", "constant", "import",
            "export",
        ],
        "tree-sitter-dart",
        "src/dart/widget.dart",
        "Conformance exists, but real-repo coverage and package/extension/type-alias handling remain incomplete.",
    ),
    tree_sitter_experimental(
        "Nix",
        "nix",
        &[".nix"],
        &["function", "module", "import"],
        "tree-sitter-nix",
        "src/nix/default.nix",
        "Conformance exists, but real-repo coverage and exhaustive attr/scalar extraction remain incomplete by design.",
    ),
    extras_backed(
        "CSS",
        &[".css"],
        &["css_class", "css_id", "css_variable", "keyframes"],
        "sample_repo",
        "frontend/styles/header.css",
        "Selectors and keyframes only; no cascade or browser semantics.",
    ),
    extras_backed(
        "HTML",
        &[".html"],
        &["html_tag", "html_id", "html_class", "data_attribute"],
        "html_repo",
        "templates/base.html",
        "Tags and attributes only; no DOM or browser semantics.",
    ),
    extras_backed(
        "Markdown",
        &[".md", ".markdown"],
        &["section", "checklist", "link", "todo", "fixme"],
        "sample_repo",
        "docs/guide.md",
        "Useful landmarks only; not a full Markdown AST.",
    ),
    extras_backed(
        "JSON",
        &[".json"],
        &["key"],
        "sample_repo",
        "config/app.json",
        "Object keys only; scalar values are intentionally skipped.",
    ),
    extras_backed(
        "TOML",
        &[".toml"],
        &["key", "table"],
        "sample_repo",
        "config/thinindex.toml",
        "Keys and tables only; scalar values are intentionally skipped.",
    ),
    extras_backed(
        "YAML",
        &[".yaml", ".yml"],
        &["key", "section"],
        "sample_repo",
        "config/pipeline.yaml",
        "Mapping keys and sections only; scalar values are intentionally skipped.",
    ),
    blocked(
        "Vue/Svelte single-file components",
        &[".vue", ".svelte"],
        "No selected permissive grammar/query/fixture/notice path and no component section adapter.",
    ),
    blocked(
        "Objective-C/Objective-C++",
        &[".m", ".mm"],
        "No selected permissive grammar/query/fixture/notice path.",
    ),
    blocked(
        "SQL",
        &[".sql"],
        "No product-approved grammar/query policy for dialect differences.",
    ),
    blocked(
        "XML",
        &[".xml"],
        "No product-approved extras policy for non-noisy XML landmarks.",
    ),
    blocked(
        "Lua",
        &[".lua"],
        "No selected permissive grammar/query/fixture/notice path.",
    ),
    blocked(
        "Haskell",
        &[".hs"],
        "No selected permissive grammar/query/fixture/notice path.",
    ),
    blocked(
        "Elixir",
        &[".ex", ".exs"],
        "No selected permissive grammar/query/fixture/notice path.",
    ),
];

const fn tree_sitter_supported(
    name: &'static str,
    language_id: &'static str,
    extensions: &'static [&'static str],
    record_kinds: &'static [&'static str],
    grammar_package: &'static str,
    fixture_path: &'static str,
    known_gaps: &'static str,
) -> SupportEntry {
    tree_sitter_entry(
        name,
        language_id,
        extensions,
        SupportLevel::Supported,
        record_kinds,
        grammar_package,
        fixture_path,
        known_gaps,
    )
}

const fn tree_sitter_experimental(
    name: &'static str,
    language_id: &'static str,
    extensions: &'static [&'static str],
    record_kinds: &'static [&'static str],
    grammar_package: &'static str,
    fixture_path: &'static str,
    known_gaps: &'static str,
) -> SupportEntry {
    tree_sitter_entry(
        name,
        language_id,
        extensions,
        SupportLevel::Experimental,
        record_kinds,
        grammar_package,
        fixture_path,
        known_gaps,
    )
}

#[allow(clippy::too_many_arguments)]
const fn tree_sitter_entry(
    name: &'static str,
    language_id: &'static str,
    extensions: &'static [&'static str],
    support_level: SupportLevel,
    record_kinds: &'static [&'static str],
    grammar_package: &'static str,
    fixture_path: &'static str,
    known_gaps: &'static str,
) -> SupportEntry {
    SupportEntry {
        name,
        language_id: Some(language_id),
        extensions,
        support_level,
        backend: SupportBackend::TreeSitter,
        record_kinds,
        known_gaps,
        license_status: "MIT grammar notice",
        grammar_package: Some(grammar_package),
        conformance_fixture_repo: Some("language_pack"),
        conformance_fixture_path: Some(fixture_path),
    }
}

const fn extras_backed(
    name: &'static str,
    extensions: &'static [&'static str],
    record_kinds: &'static [&'static str],
    fixture_repo: &'static str,
    fixture_path: &'static str,
    known_gaps: &'static str,
) -> SupportEntry {
    SupportEntry {
        name,
        language_id: None,
        extensions,
        support_level: SupportLevel::ExtrasBacked,
        backend: SupportBackend::Extras,
        record_kinds,
        known_gaps,
        license_status: "project-owned extras; no third-party parser dependency",
        grammar_package: None,
        conformance_fixture_repo: Some(fixture_repo),
        conformance_fixture_path: Some(fixture_path),
    }
}

const fn blocked(
    name: &'static str,
    extensions: &'static [&'static str],
    known_gaps: &'static str,
) -> SupportEntry {
    SupportEntry {
        name,
        language_id: None,
        extensions,
        support_level: SupportLevel::Blocked,
        backend: SupportBackend::None,
        record_kinds: &[],
        known_gaps,
        license_status: "blocked: no approved parser/extras support path",
        grammar_package: None,
        conformance_fixture_repo: None,
        conformance_fixture_path: None,
    }
}
