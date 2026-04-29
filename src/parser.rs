use std::path::Path;

use crate::model::IndexRecord;

pub trait ParserBackend {
    fn parse_file(&self, path: &Path, rel_path: &str, text: &str) -> Vec<IndexRecord>;
}
