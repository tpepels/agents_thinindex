use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const INDEX_SCHEMA_VERSION: u32 = 4;

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
