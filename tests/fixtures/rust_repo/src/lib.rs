use std::{fs, path::Path};
use crate::parser::ParserBackend as Backend;

pub mod parser;

pub struct PromptService {
    name: String,
}

pub enum SearchMode {
    Exact,
    Prefix,
}

pub trait ParserBackend {
    fn parse_file(&self, path: &Path);
}

pub type RecordMap = std::collections::BTreeMap<String, String>;
pub static GLOBAL_COUNTER: usize = 0;
pub const INDEX_SCHEMA_VERSION: u32 = 7;

impl PromptService {
    pub const DEFAULT_LIMIT: usize = 30;

    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn build_index(&self) -> usize {
        self.name.len()
    }
}

pub fn standalone_function() -> Option<Backend> {
    let _path = Path::new("src/lib.rs");
    let _ = fs::read_to_string(_path);
    None
}
