use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const INDEX_SCHEMA_VERSION: u32 = 10;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexRecord {
    pub path: String,
    pub line: usize,
    pub col: usize,
    pub lang: String,
    pub kind: String,
    pub name: String,
    pub text: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceRecord {
    pub from_path: String,
    pub from_line: usize,
    pub from_col: usize,
    pub to_name: String,
    pub to_kind: Option<String>,
    pub ref_kind: String,
    pub confidence: String,
    pub reason: Option<String>,
    pub evidence: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyEdge {
    pub from_path: String,
    pub from_line: usize,
    pub from_col: usize,
    pub import_path: String,
    pub target_path: Option<String>,
    pub dependency_kind: String,
    pub lang: String,
    pub confidence: String,
    pub unresolved_reason: Option<String>,
    pub evidence: String,
    pub source: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticFactKind {
    ResolvedDefinition,
    ResolvedReference,
    TypeReference,
    CallTarget,
    Implementation,
    Diagnostic,
}

impl SemanticFactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SemanticFactKind::ResolvedDefinition => "resolved_definition",
            SemanticFactKind::ResolvedReference => "resolved_reference",
            SemanticFactKind::TypeReference => "type_reference",
            SemanticFactKind::CallTarget => "call_target",
            SemanticFactKind::Implementation => "implementation",
            SemanticFactKind::Diagnostic => "diagnostic",
        }
    }
}

impl std::str::FromStr for SemanticFactKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "resolved_definition" => Ok(SemanticFactKind::ResolvedDefinition),
            "resolved_reference" => Ok(SemanticFactKind::ResolvedReference),
            "type_reference" => Ok(SemanticFactKind::TypeReference),
            "call_target" => Ok(SemanticFactKind::CallTarget),
            "implementation" => Ok(SemanticFactKind::Implementation),
            "diagnostic" => Ok(SemanticFactKind::Diagnostic),
            _ => Err(format!("unknown semantic fact kind: {value}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticFact {
    pub source_path: String,
    pub source_line: usize,
    pub source_col: usize,
    pub kind: SemanticFactKind,
    pub symbol: String,
    pub target_path: Option<String>,
    pub target_line: Option<usize>,
    pub target_col: Option<usize>,
    pub detail: Option<String>,
    pub confidence: String,
    pub adapter: String,
}

impl ReferenceRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        from_path: impl Into<String>,
        from_line: usize,
        from_col: usize,
        to_name: impl Into<String>,
        to_kind: Option<impl Into<String>>,
        ref_kind: impl Into<String>,
        evidence: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        let source = source.into();
        let ref_kind = ref_kind.into();
        let (confidence, reason) = default_ref_confidence_and_reason(&source, &ref_kind);

        Self::new_with_confidence(
            from_path,
            from_line,
            from_col,
            to_name,
            to_kind,
            ref_kind,
            confidence,
            Some(reason),
            evidence,
            source,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_confidence(
        from_path: impl Into<String>,
        from_line: usize,
        from_col: usize,
        to_name: impl Into<String>,
        to_kind: Option<impl Into<String>>,
        ref_kind: impl Into<String>,
        confidence: impl Into<String>,
        reason: Option<impl Into<String>>,
        evidence: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            from_path: from_path.into(),
            from_line,
            from_col,
            to_name: to_name.into(),
            to_kind: to_kind.map(Into::into),
            ref_kind: ref_kind.into(),
            confidence: confidence.into(),
            reason: reason.map(|value| truncate(value.into(), 120)),
            evidence: truncate(evidence.into(), 120),
            source: source.into(),
        }
    }
}

fn default_ref_confidence_and_reason(source: &str, ref_kind: &str) -> (&'static str, &'static str) {
    match source {
        "tree_sitter" => ("syntax", "tree_sitter_reference_capture"),
        "dependency_graph" => ("dependency", "dependency_graph_edge"),
        "imports" => ("syntax", "line_import_syntax"),
        "extras" => ("syntax", "structured_format_reference"),
        "text" if ref_kind == "test_reference" => ("heuristic", "test_text_reference"),
        "text" => ("heuristic", "broad_text_fallback"),
        _ => ("heuristic", "legacy_reference"),
    }
}

impl DependencyEdge {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        from_path: impl Into<String>,
        from_line: usize,
        from_col: usize,
        import_path: impl Into<String>,
        target_path: Option<impl Into<String>>,
        dependency_kind: impl Into<String>,
        lang: impl Into<String>,
        confidence: impl Into<String>,
        unresolved_reason: Option<impl Into<String>>,
        evidence: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            from_path: from_path.into(),
            from_line,
            from_col,
            import_path: import_path.into(),
            target_path: target_path.map(Into::into),
            dependency_kind: dependency_kind.into(),
            lang: lang.into(),
            confidence: confidence.into(),
            unresolved_reason: unresolved_reason.map(Into::into),
            evidence: truncate(evidence.into(), 120),
            source: source.into(),
        }
    }
}

impl SemanticFact {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_path: impl Into<String>,
        source_line: usize,
        source_col: usize,
        kind: SemanticFactKind,
        symbol: impl Into<String>,
        target_path: Option<impl Into<String>>,
        target_line: Option<usize>,
        target_col: Option<usize>,
        detail: Option<impl Into<String>>,
        confidence: impl Into<String>,
        adapter: impl Into<String>,
    ) -> Self {
        Self {
            source_path: source_path.into(),
            source_line,
            source_col,
            kind,
            symbol: symbol.into(),
            target_path: target_path.map(Into::into),
            target_line,
            target_col,
            detail: detail.map(|value| truncate(value.into(), 120)),
            confidence: confidence.into(),
            adapter: adapter.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileMeta {
    pub mtime_ns: u128,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Manifest {
    #[serde(default, rename = "schema_version")]
    pub schema_version: u32,
    pub files: BTreeMap<String, FileMeta>,
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            schema_version: INDEX_SCHEMA_VERSION,
            files: BTreeMap::new(),
        }
    }
}

impl IndexRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: impl Into<String>,
        line: usize,
        col: usize,
        lang: impl Into<String>,
        kind: impl Into<String>,
        name: impl Into<String>,
        text: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            line,
            col,
            lang: lang.into(),
            kind: kind.into(),
            name: name.into(),
            text: truncate(text.into(), 120),
            source: source.into(),
        }
    }
}

pub fn truncate(mut value: String, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value;
    }

    value = value.chars().take(max_chars.saturating_sub(1)).collect();
    value.push('…');
    value
}
