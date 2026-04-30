use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const INDEX_SCHEMA_VERSION: u32 = 8;

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
        Self {
            from_path: from_path.into(),
            from_line,
            from_col,
            to_name: to_name.into(),
            to_kind: to_kind.map(Into::into),
            ref_kind: ref_kind.into(),
            evidence: truncate(evidence.into(), 120),
            source: source.into(),
        }
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
