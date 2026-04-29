use std::path::Path;

use crate::{model::IndexRecord, parser::ParserBackend};

const NATIVE_SOURCE: &str = "native";

#[derive(Debug, Default, Clone, Copy)]
pub struct NativeParser;

impl ParserBackend for NativeParser {
    fn parse_file(&self, _path: &Path, rel_path: &str, text: &str) -> Vec<IndexRecord> {
        parse_file(rel_path, text)
    }
}

pub fn parse_file(rel_path: &str, text: &str) -> Vec<IndexRecord> {
    let lang = language_from_path(rel_path);

    if lang == "rs" {
        return parse_rust_file(rel_path, &lang, text);
    }

    if lang == "py" {
        return parse_python_file(rel_path, &lang, text);
    }

    let mut records = Vec::new();

    for (line_index, line) in text.lines().enumerate() {
        let line_no = line_index + 1;

        match lang.as_str() {
            "js" | "jsx" | "ts" | "tsx" => {
                parse_js_like_line(rel_path, &lang, line_no, line, &mut records);
            }
            "md" => parse_markdown_line(rel_path, &lang, line_no, line, &mut records),
            "make" => parse_make_line(rel_path, &lang, line_no, line, &mut records),
            _ => {}
        }
    }

    records
}

#[derive(Debug, Default)]
struct RustState {
    brace_depth: usize,
    impl_body_depth: Option<usize>,
}

#[derive(Debug, Default)]
struct PythonState {
    class_indents: Vec<usize>,
    function_indents: Vec<usize>,
}

fn parse_python_file(path: &str, lang: &str, text: &str) -> Vec<IndexRecord> {
    let mut records = Vec::new();
    let mut state = PythonState::default();

    for (line_index, line) in text.lines().enumerate() {
        let line_no = line_index + 1;
        let code = python_code_before_line_comment(line);

        if code.trim().is_empty() {
            continue;
        }

        let indent = leading_whitespace_width(line);
        state.update_for_line(indent);
        parse_python_line(
            path,
            lang,
            line_no,
            line,
            code,
            indent,
            &mut state,
            &mut records,
        );
    }

    records
}

impl PythonState {
    fn update_for_line(&mut self, indent: usize) {
        while self
            .class_indents
            .last()
            .is_some_and(|class_indent| indent <= *class_indent)
        {
            self.class_indents.pop();
        }

        while self
            .function_indents
            .last()
            .is_some_and(|function_indent| indent <= *function_indent)
        {
            self.function_indents.pop();
        }
    }

    fn in_class_body(&self, indent: usize) -> bool {
        self.class_indents
            .last()
            .is_some_and(|class_indent| indent > *class_indent)
    }

    fn enter_class(&mut self, indent: usize) {
        self.class_indents.push(indent);
    }

    fn in_function_body(&self, indent: usize) -> bool {
        self.function_indents
            .last()
            .is_some_and(|function_indent| indent > *function_indent)
    }

    fn enter_function(&mut self, indent: usize) {
        self.function_indents.push(indent);
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_python_line(
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    code: &str,
    indent: usize,
    state: &mut PythonState,
    records: &mut Vec<IndexRecord>,
) {
    let trimmed = code.trim_start();

    if trimmed.starts_with('#') {
        return;
    }

    if let Some((col, name)) = keyword_name(code, "class") {
        push_record(records, path, lang, line_no, col, "class", name, line);
        state.enter_class(indent);
        return;
    }

    if let Some((col, name)) = keyword_name(code, "def") {
        let kind = if state.in_class_body(indent) {
            "method"
        } else {
            "function"
        };
        push_record(records, path, lang, line_no, col, kind, name, line);
        state.enter_function(indent);
        return;
    }

    if trimmed.starts_with("from ") {
        push_python_from_import_records(records, path, lang, line_no, line, code);
        return;
    }

    if trimmed.starts_with("import ") {
        push_python_import_records(records, path, lang, line_no, line, code);
        return;
    }

    if let Some((col, name)) = python_constant_assignment(code)
        && (indent == 0 || (state.in_class_body(indent) && !state.in_function_body(indent)))
    {
        push_record(records, path, lang, line_no, col, "constant", name, line);
    }
}

fn push_python_import_records(
    records: &mut Vec<IndexRecord>,
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    code: &str,
) {
    let Some(import_index) = code.find("import ") else {
        return;
    };
    let body_start = import_index + "import ".len();
    let body = code[body_start..].trim();

    for part in body.split(',') {
        let import_text = part.trim();

        if import_text.is_empty() {
            continue;
        }

        let name = python_import_binding_name(import_text);

        if name.is_empty() {
            continue;
        }

        if let Some(relative_col) = code[body_start..].find(&name) {
            push_record(
                records,
                path,
                lang,
                line_no,
                body_start + relative_col + 1,
                "import",
                name,
                line,
            );
        }
    }
}

fn push_python_from_import_records(
    records: &mut Vec<IndexRecord>,
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    code: &str,
) {
    let Some(import_index) = code.find(" import ") else {
        return;
    };
    let body_start = import_index + " import ".len();
    let body = code[body_start..].trim();

    if body == "*" {
        return;
    }

    for part in body.split(',') {
        let import_text = part.trim();

        if import_text.is_empty() {
            continue;
        }

        let name = python_import_binding_name(import_text);

        if name.is_empty() {
            continue;
        }

        if let Some(relative_col) = code[body_start..].find(&name) {
            push_record(
                records,
                path,
                lang,
                line_no,
                body_start + relative_col + 1,
                "import",
                name,
                line,
            );
        }
    }
}

fn python_import_binding_name(import_text: &str) -> String {
    if let Some((_, alias)) = import_text.rsplit_once(" as ") {
        return take_identifier(alias.trim());
    }

    take_identifier(import_text.split('.').next().unwrap_or_default())
}

fn python_constant_assignment(code: &str) -> Option<(usize, String)> {
    let equals = code.find('=')?;

    if code[..equals].ends_with(['!', '<', '>', '=']) || code[equals + 1..].starts_with('=') {
        return None;
    }

    let before_equals = code[..equals].trim_end();
    let binding = before_equals
        .rsplit_once(':')
        .map(|(name, _)| name.trim())
        .unwrap_or(before_equals)
        .trim();

    let name = take_identifier(binding);

    if name.len() != binding.len() || !is_python_constant_name(&name) {
        return None;
    }

    let col = code.find(&name)? + 1;
    Some((col, name))
}

fn is_python_constant_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase() || ch == '_')
        && name.chars().any(|ch| ch.is_ascii_uppercase())
        && name
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

fn parse_rust_file(path: &str, lang: &str, text: &str) -> Vec<IndexRecord> {
    let mut records = Vec::new();
    let mut state = RustState::default();

    for (line_index, line) in text.lines().enumerate() {
        let line_no = line_index + 1;
        let code = rust_code_before_line_comment(line);
        let starts_impl = rust_has_keyword(code, "impl") && code.contains('{');
        let in_impl = state.impl_body_depth.is_some();

        parse_rust_line(path, lang, line_no, line, code, in_impl, &mut records);
        state.update(code, starts_impl);
    }

    records
}

impl RustState {
    fn update(&mut self, code: &str, starts_impl: bool) {
        let depth_before = self.brace_depth;
        let opens = code.chars().filter(|ch| *ch == '{').count();
        let closes = code.chars().filter(|ch| *ch == '}').count();

        self.brace_depth = self.brace_depth.saturating_add(opens);
        self.brace_depth = self.brace_depth.saturating_sub(closes);

        if starts_impl {
            self.impl_body_depth = Some(depth_before + 1);
        }

        if let Some(impl_body_depth) = self.impl_body_depth
            && self.brace_depth < impl_body_depth
        {
            self.impl_body_depth = None;
        }
    }
}

fn parse_rust_line(
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    code: &str,
    in_impl: bool,
    records: &mut Vec<IndexRecord>,
) {
    let trimmed = code.trim_start();

    if trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
        || trimmed.starts_with("#[")
        || trimmed.starts_with("#!")
    {
        return;
    }

    if let Some((col, name)) = keyword_name(code, "use") {
        push_rust_import_records(records, path, lang, line_no, line, col, &name, code);
        return;
    }

    if let Some((col, name)) = keyword_name(code, "fn") {
        let kind = if in_impl { "method" } else { "function" };
        push_record(records, path, lang, line_no, col, kind, name, line);
        return;
    }

    for (keyword, kind) in [
        ("struct", "struct"),
        ("enum", "enum"),
        ("trait", "trait"),
        ("mod", "module"),
        ("const", "constant"),
        ("static", "variable"),
        ("type", "type"),
    ] {
        if let Some((col, name)) = keyword_name(code, keyword) {
            push_record(records, path, lang, line_no, col, kind, name, line);
            return;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn push_rust_import_records(
    records: &mut Vec<IndexRecord>,
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    use_col: usize,
    first_name: &str,
    code: &str,
) {
    let use_body_start = use_col - 1 + "use".len();
    let use_body = code[use_body_start..].trim().trim_end_matches(';').trim();

    if use_body.is_empty() {
        push_record(
            records,
            path,
            lang,
            line_no,
            use_col,
            "import",
            first_name.to_string(),
            line,
        );
        return;
    }

    for name in rust_use_leaf_names(use_body) {
        if let Some(relative_col) = code[use_body_start..].find(&name) {
            push_record(
                records,
                path,
                lang,
                line_no,
                use_body_start + relative_col + 1,
                "import",
                name,
                line,
            );
        }
    }
}

fn rust_use_leaf_names(use_body: &str) -> Vec<String> {
    let mut names = Vec::new();

    for raw_segment in use_body.split(['{', '}', ',']) {
        let segment = raw_segment.trim();

        if segment.is_empty() || segment == "::" || segment.ends_with("::") {
            continue;
        }

        let aliased = segment
            .rsplit_once(" as ")
            .map(|(_, alias)| alias.trim())
            .unwrap_or(segment);

        let leaf = aliased
            .rsplit("::")
            .find(|part| !part.trim().is_empty())
            .unwrap_or("")
            .trim();

        if leaf.is_empty()
            || leaf == "*"
            || matches!(leaf, "self" | "super" | "crate" | "std" | "core" | "alloc")
        {
            continue;
        }

        let name = take_identifier(leaf);

        if !name.is_empty() && !names.contains(&name) {
            names.push(name);
        }
    }

    names
}

fn rust_code_before_line_comment(line: &str) -> &str {
    line.split_once("//").map(|(code, _)| code).unwrap_or(line)
}

fn python_code_before_line_comment(line: &str) -> &str {
    line.split_once('#').map(|(code, _)| code).unwrap_or(line)
}

fn rust_has_keyword(line: &str, keyword: &str) -> bool {
    let mut search_start = 0;

    while let Some(relative_index) = line[search_start..].find(keyword) {
        let keyword_start = search_start + relative_index;

        if is_identifier_boundary(line, keyword_start, keyword.len()) {
            return true;
        }

        search_start = keyword_start + keyword.len();
    }

    false
}

fn leading_whitespace_width(line: &str) -> usize {
    line.chars()
        .take_while(|ch| ch.is_whitespace())
        .map(|ch| if ch == '\t' { 4 } else { 1 })
        .sum()
}

fn parse_js_like_line(
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    records: &mut Vec<IndexRecord>,
) {
    let trimmed = line.trim_start();

    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
        return;
    }

    if let Some((col, name)) = keyword_name(line, "class") {
        push_record(records, path, lang, line_no, col, "class", name, line);
        return;
    }

    if let Some((col, name)) = keyword_name(line, "function") {
        push_record(records, path, lang, line_no, col, "function", name, line);
        return;
    }

    for keyword in ["const", "let", "var"] {
        if let Some((col, name)) = keyword_name(line, keyword) {
            let after_name = col - 1 + name.len();
            let rest = line.get(after_name..).unwrap_or_default();
            let kind = if rest.contains("=>") {
                "function"
            } else {
                "variable"
            };
            push_record(records, path, lang, line_no, col, kind, name, line);
            return;
        }
    }
}

fn parse_markdown_line(
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    records: &mut Vec<IndexRecord>,
) {
    let trimmed = line.trim_start();
    let leading_spaces = line.len() - trimmed.len();

    if !trimmed.starts_with('#') {
        return;
    }

    let marker_len = trimmed.chars().take_while(|ch| *ch == '#').count();

    if marker_len == 0 || marker_len > 6 {
        return;
    }

    let after_marker = &trimmed[marker_len..];

    if !after_marker.starts_with(char::is_whitespace) {
        return;
    }

    let name = after_marker.trim();

    if name.is_empty() {
        return;
    }

    push_record(
        records,
        path,
        lang,
        line_no,
        leading_spaces + marker_len + 1,
        "section",
        name.to_string(),
        line,
    );
}

fn parse_make_line(
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    records: &mut Vec<IndexRecord>,
) {
    let trimmed = line.trim_start();

    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || line.starts_with('\t')
        || trimmed.starts_with('.')
    {
        return;
    }

    let Some(colon) = line.find(':') else {
        return;
    };

    let target = line[..colon].trim();

    if target.is_empty()
        || target.contains(char::is_whitespace)
        || target.contains('$')
        || target.contains('=')
    {
        return;
    }

    let col = line.find(target).unwrap_or(0) + 1;
    push_record(
        records,
        path,
        lang,
        line_no,
        col,
        "make_target",
        target.to_string(),
        line,
    );
}

fn keyword_name(line: &str, keyword: &str) -> Option<(usize, String)> {
    let mut search_start = 0;

    while let Some(relative_index) = line[search_start..].find(keyword) {
        let keyword_start = search_start + relative_index;
        let keyword_end = keyword_start + keyword.len();

        if !is_identifier_boundary(line, keyword_start, keyword.len()) {
            search_start = keyword_end;
            continue;
        }

        let after_keyword = &line[keyword_end..];
        let whitespace_len = after_keyword
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();

        if whitespace_len == 0 {
            search_start = keyword_end;
            continue;
        }

        let name_start = keyword_end + whitespace_len;
        let name = take_identifier(&line[name_start..]);

        if !name.is_empty() {
            return Some((name_start + 1, name));
        }

        search_start = keyword_end;
    }

    None
}

fn is_identifier_boundary(line: &str, start: usize, len: usize) -> bool {
    let before = line[..start].chars().next_back();
    let after = line[start + len..].chars().next();

    let valid_before =
        before.is_none_or(|ch| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'));
    let valid_after =
        after.is_none_or(|ch| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'));

    valid_before && valid_after
}

fn take_identifier(value: &str) -> String {
    value
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '$')
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn push_record(
    records: &mut Vec<IndexRecord>,
    path: &str,
    lang: &str,
    line: usize,
    col: usize,
    kind: &str,
    name: String,
    text: &str,
) {
    records.push(IndexRecord::new(
        path,
        line,
        col,
        lang,
        kind,
        name,
        text.trim(),
        NATIVE_SOURCE,
    ));
}

fn language_from_path(path: &str) -> String {
    let filename = Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    if filename == "Makefile" || filename.ends_with(".mk") {
        return "make".to_string();
    }

    match Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "py" => "py".to_string(),
        "rs" => "rs".to_string(),
        "ts" => "ts".to_string(),
        "tsx" => "tsx".to_string(),
        "js" => "js".to_string(),
        "jsx" => "jsx".to_string(),
        "md" | "mdx" => "md".to_string(),
        other if !other.is_empty() => other.to_string(),
        _ => "text".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_native_landmarks() {
        let records = parse_file(
            "src/lib.rs",
            "pub const INDEX_SCHEMA_VERSION: u32 = 8;\npub fn build_index() {}\nstruct PromptService;\n",
        );

        assert!(records.iter().any(|record| record.name == "build_index"));
        assert!(records.iter().any(|record| record.name == "PromptService"));
        assert!(
            records
                .iter()
                .any(|record| record.name == "INDEX_SCHEMA_VERSION")
        );
    }

    #[test]
    fn parses_python_classes_functions_methods_imports_and_constants() {
        let records = parse_file(
            "app/services/prompt_service.py",
            r#"
import os
import pathlib as pl
from typing import Optional, TYPE_CHECKING as CHECKING

MAX_RETRIES = 3
local_value = 1

class PromptService:
    DEFAULT_MODEL: str = "base"

    def build_prompt(self):
        LOCAL_CACHE = "skip"
        pass

    async def fetch_prompt(self):
        pass

async def create_prompt_service():
    return PromptService()

def helper_function():
    return Optional
"#,
        );

        for (name, kind) in [
            ("os", "import"),
            ("pl", "import"),
            ("Optional", "import"),
            ("CHECKING", "import"),
            ("MAX_RETRIES", "constant"),
            ("DEFAULT_MODEL", "constant"),
            ("PromptService", "class"),
            ("build_prompt", "method"),
            ("fetch_prompt", "method"),
            ("create_prompt_service", "function"),
            ("helper_function", "function"),
        ] {
            assert!(
                records.iter().any(|record| record.name == name
                    && record.kind == kind
                    && record.source == NATIVE_SOURCE),
                "missing {kind} {name}, got:\n{records:#?}"
            );
        }

        assert!(
            !records.iter().any(|record| record.name == "local_value"),
            "lowercase assignment should not be indexed as a constant:\n{records:#?}"
        );
        assert!(
            !records.iter().any(|record| record.name == "LOCAL_CACHE"),
            "method-local uppercase assignment should not be indexed as a constant:\n{records:#?}"
        );

        let async_method = records
            .iter()
            .find(|record| record.name == "fetch_prompt")
            .expect("async method");
        assert_eq!(async_method.kind, "method");
        assert_eq!(async_method.line, 16);
        assert_eq!(async_method.col, 15);
    }

    #[test]
    fn parses_rust_items_methods_and_imports() {
        let records = parse_file(
            "src/lib.rs",
            r#"
use std::{fs, path::Path};
use crate::parser::ParserBackend as Backend;

pub mod parser;

pub struct PromptService;
pub enum SearchMode { Exact }
pub trait ParserBackend {
    fn parse_file(&self);
}
pub type RecordMap = std::collections::BTreeMap<String, String>;
pub static GLOBAL_COUNTER: usize = 0;

impl PromptService {
    pub const DEFAULT_LIMIT: usize = 30;

    pub fn new() -> Self {
        Self
    }
}
"#,
        );

        for (name, kind) in [
            ("fs", "import"),
            ("Path", "import"),
            ("Backend", "import"),
            ("parser", "module"),
            ("PromptService", "struct"),
            ("SearchMode", "enum"),
            ("ParserBackend", "trait"),
            ("parse_file", "function"),
            ("RecordMap", "type"),
            ("GLOBAL_COUNTER", "variable"),
            ("DEFAULT_LIMIT", "constant"),
            ("new", "method"),
        ] {
            assert!(
                records.iter().any(|record| record.name == name
                    && record.kind == kind
                    && record.source == NATIVE_SOURCE),
                "missing {kind} {name}, got:\n{records:#?}"
            );
        }

        let new_method = records
            .iter()
            .find(|record| record.name == "new" && record.kind == "method")
            .expect("new method");
        assert_eq!(new_method.line, 18);
        assert_eq!(new_method.col, 12);
    }

    #[test]
    fn parses_markdown_headings_as_sections() {
        let records = parse_file("README.md", "# Thinindex\n\n## Tests\n");

        assert!(
            records
                .iter()
                .any(|record| record.kind == "section" && record.name == "Tests")
        );
    }
}
