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
    let mut records = Vec::new();

    for (line_index, line) in text.lines().enumerate() {
        let line_no = line_index + 1;

        match lang.as_str() {
            "py" => parse_python_line(rel_path, &lang, line_no, line, &mut records),
            "rs" => parse_rust_line(rel_path, &lang, line_no, line, &mut records),
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

fn parse_python_line(
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    records: &mut Vec<IndexRecord>,
) {
    let trimmed = line.trim_start();

    if trimmed.starts_with('#') {
        return;
    }

    if let Some((col, name)) = keyword_name(line, "class") {
        push_record(records, path, lang, line_no, col, "class", name, line);
        return;
    }

    if let Some((col, name)) = keyword_name(line, "def") {
        let kind = if line.len() > trimmed.len() {
            "method"
        } else {
            "function"
        };
        push_record(records, path, lang, line_no, col, kind, name, line);
    }
}

fn parse_rust_line(
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

    for (keyword, kind) in [
        ("fn", "function"),
        ("struct", "struct"),
        ("enum", "enum"),
        ("trait", "trait"),
        ("mod", "module"),
        ("const", "constant"),
        ("static", "variable"),
        ("type", "type"),
    ] {
        if let Some((col, name)) = keyword_name(line, keyword) {
            push_record(records, path, lang, line_no, col, kind, name, line);
            return;
        }
    }
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
            "pub const INDEX_SCHEMA_VERSION: u32 = 6;\npub fn build_index() {}\nstruct PromptService;\n",
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
    fn parses_markdown_headings_as_sections() {
        let records = parse_file("README.md", "# Thinindex\n\n## Tests\n");

        assert!(
            records
                .iter()
                .any(|record| record.kind == "section" && record.name == "Tests")
        );
    }
}
