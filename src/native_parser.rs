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

    if matches!(lang.as_str(), "js" | "jsx" | "ts" | "tsx") {
        return parse_js_like_file(rel_path, &lang, text);
    }

    let mut records = Vec::new();

    for (line_index, line) in text.lines().enumerate() {
        let line_no = line_index + 1;

        match lang.as_str() {
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

#[derive(Debug, Default)]
struct JsState {
    brace_depth: usize,
    class_body_depths: Vec<usize>,
    pending_class_body: bool,
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

fn parse_js_like_file(path: &str, lang: &str, text: &str) -> Vec<IndexRecord> {
    let mut records = Vec::new();
    let mut state = JsState::default();

    for (line_index, line) in text.lines().enumerate() {
        let line_no = line_index + 1;
        let code = js_code_before_line_comment(line);

        if code.trim().is_empty() {
            continue;
        }

        let in_class = state.in_class_body();
        parse_js_like_line(path, lang, line_no, line, code, in_class, &mut records);
        state.update(code, js_line_enters_class_body(code));
    }

    records
}

impl JsState {
    fn in_class_body(&self) -> bool {
        self.class_body_depths
            .last()
            .is_some_and(|class_depth| self.brace_depth >= *class_depth)
    }

    fn update(&mut self, code: &str, enters_class: bool) {
        let depth_before = self.brace_depth;
        let opens = code.chars().filter(|ch| *ch == '{').count();
        let closes = code.chars().filter(|ch| *ch == '}').count();

        if enters_class {
            self.pending_class_body = true;
        }

        if self.pending_class_body && opens > 0 {
            self.class_body_depths.push(depth_before + 1);
            self.pending_class_body = false;
        }

        self.brace_depth = self.brace_depth.saturating_add(opens);
        self.brace_depth = self.brace_depth.saturating_sub(closes);

        while self
            .class_body_depths
            .last()
            .is_some_and(|class_depth| self.brace_depth < *class_depth)
        {
            self.class_body_depths.pop();
        }
    }
}

fn parse_js_like_line(
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    code: &str,
    in_class: bool,
    records: &mut Vec<IndexRecord>,
) {
    let trimmed = code.trim_start();

    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
        return;
    }

    push_js_import_records(records, path, lang, line_no, line, code);
    push_js_export_records(records, path, lang, line_no, line, code);

    if let Some((col, name)) = keyword_name(code, "class") {
        push_record(records, path, lang, line_no, col, "class", name, line);
        return;
    }

    if in_class && let Some((col, name)) = js_class_method_name(code) {
        push_record(records, path, lang, line_no, col, "method", name, line);
        return;
    }

    if let Some((col, name)) = keyword_name(code, "function") {
        push_record(records, path, lang, line_no, col, "function", name, line);
        return;
    }

    for (keyword, kind) in [("interface", "interface"), ("type", "type")] {
        if let Some((col, name)) = keyword_name(code, keyword) {
            push_record(records, path, lang, line_no, col, kind, name, line);
            return;
        }
    }

    if let Some((col, name, rest)) = js_variable_declaration(code) {
        let kind = if js_assignment_is_function(rest) {
            "function"
        } else {
            "variable"
        };
        push_record(records, path, lang, line_no, col, kind, name, line);
    }
}

fn js_code_before_line_comment(line: &str) -> &str {
    line.split_once("//").map(|(code, _)| code).unwrap_or(line)
}

fn js_line_enters_class_body(code: &str) -> bool {
    keyword_name(code, "class").is_some() && code.contains('{')
}

fn push_js_import_records(
    records: &mut Vec<IndexRecord>,
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    code: &str,
) {
    let trimmed = code.trim_start();
    let leading_spaces = code.len() - trimmed.len();
    let Some(rest) = trimmed.strip_prefix("import ") else {
        return;
    };

    let mut spec_start = leading_spaces + "import ".len();
    let mut spec = rest.trim_start();
    spec_start += rest.len() - spec.len();

    if let Some(after_type) = strip_js_keyword_prefix(spec, "type") {
        spec_start += spec.len() - after_type.len();
        spec = after_type.trim_start();
        spec_start += after_type.len() - spec.len();
    }

    if spec.starts_with(['"', '\'']) {
        return;
    }

    let spec_end = spec
        .find(" from ")
        .or_else(|| spec.find('='))
        .unwrap_or(spec.len());
    let raw_import_spec = &spec[..spec_end];
    let import_spec = raw_import_spec.trim();
    let import_spec_base = spec_start + raw_import_spec.len() - raw_import_spec.trim_start().len();

    push_js_binding_spec_records(
        records,
        path,
        lang,
        line_no,
        line,
        code,
        import_spec,
        import_spec_base,
        "import",
    );
}

fn push_js_export_records(
    records: &mut Vec<IndexRecord>,
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    code: &str,
) {
    let trimmed = code.trim_start();
    let leading_spaces = code.len() - trimmed.len();
    let Some(rest) = trimmed.strip_prefix("export ") else {
        return;
    };

    let export_col = leading_spaces + 1;
    let rest = rest.trim_start();
    let rest_base =
        leading_spaces + "export ".len() + (trimmed["export ".len()..].len() - rest.len());

    if let Some(after_open) = rest.strip_prefix('{') {
        let close = after_open.find('}').unwrap_or(after_open.len());
        let inner = &after_open[..close];
        push_js_named_binding_records(
            records,
            JsPushContext {
                path,
                lang,
                line_no,
                line,
                code,
                kind: "export",
            },
            inner,
            rest_base + 1,
        );
        return;
    }

    if let Some(default_rest) = rest.strip_prefix("default ") {
        let default_rest = default_rest.trim_start();
        let default_base =
            rest_base + "default ".len() + (rest["default ".len()..].len() - default_rest.len());

        if let Some((relative_col, name)) = js_default_export_name(default_rest) {
            let col = if default_rest.starts_with("function ") || default_rest.starts_with("class ")
            {
                export_col
            } else {
                default_base + relative_col
            };
            push_record(records, path, lang, line_no, col, "export", name, line);
        }

        return;
    }

    for keyword in [
        "function",
        "class",
        "interface",
        "type",
        "const",
        "let",
        "var",
    ] {
        if let Some((_col, name)) = keyword_name(code, keyword) {
            push_record(
                records, path, lang, line_no, export_col, "export", name, line,
            );
            return;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn push_js_binding_spec_records(
    records: &mut Vec<IndexRecord>,
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    code: &str,
    spec: &str,
    spec_base: usize,
    kind: &str,
) {
    if spec.is_empty() {
        return;
    }

    if let Some(open) = spec.find('{') {
        let default_part = spec[..open].trim().trim_end_matches(',').trim();
        push_js_single_binding_record(
            records,
            path,
            lang,
            line_no,
            line,
            code,
            default_part,
            spec_base,
            kind,
        );

        let close = spec[open + 1..]
            .find('}')
            .map(|offset| open + 1 + offset)
            .unwrap_or(spec.len());
        let inner = &spec[open + 1..close];
        push_js_named_binding_records(
            records,
            JsPushContext {
                path,
                lang,
                line_no,
                line,
                code,
                kind,
            },
            inner,
            spec_base + open + 1,
        );
        return;
    }

    if spec.starts_with('*') {
        if let Some((_, alias)) = spec.split_once(" as ") {
            push_js_single_binding_record(
                records,
                path,
                lang,
                line_no,
                line,
                code,
                alias.trim(),
                spec_base,
                kind,
            );
        }
        return;
    }

    let default_part = spec.split(',').next().unwrap_or_default().trim();
    push_js_single_binding_record(
        records,
        path,
        lang,
        line_no,
        line,
        code,
        default_part,
        spec_base,
        kind,
    );
}

#[derive(Debug, Clone, Copy)]
struct JsPushContext<'a> {
    path: &'a str,
    lang: &'a str,
    line_no: usize,
    line: &'a str,
    code: &'a str,
    kind: &'a str,
}

fn push_js_named_binding_records(
    records: &mut Vec<IndexRecord>,
    ctx: JsPushContext<'_>,
    inner: &str,
    inner_base: usize,
) {
    let mut offset = 0;

    for raw_part in inner.split(',') {
        let leading = raw_part.len() - raw_part.trim_start().len();
        let part = raw_part.trim();

        if let Some(name) = js_named_binding_name(part) {
            let binding_search_start = inner_base + offset + leading;
            let binding_slice = &ctx.code[binding_search_start..];
            let col = binding_slice
                .find(&name)
                .map(|relative| binding_search_start + relative + 1)
                .unwrap_or(binding_search_start + 1);

            push_record(
                records,
                ctx.path,
                ctx.lang,
                ctx.line_no,
                col,
                ctx.kind,
                name,
                ctx.line,
            );
        }

        offset += raw_part.len() + 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn push_js_single_binding_record(
    records: &mut Vec<IndexRecord>,
    path: &str,
    lang: &str,
    line_no: usize,
    line: &str,
    code: &str,
    binding: &str,
    spec_base: usize,
    kind: &str,
) {
    let name = js_single_binding_name(binding);

    if name.is_empty() {
        return;
    }

    let binding_slice = &code[spec_base.min(code.len())..];
    let col = binding_slice
        .find(&name)
        .map(|relative| spec_base + relative + 1)
        .unwrap_or(spec_base + 1);

    push_record(records, path, lang, line_no, col, kind, name, line);
}

fn js_named_binding_name(part: &str) -> Option<String> {
    let part = part.trim();

    if part.is_empty() {
        return None;
    }

    let part = part.strip_prefix("type ").unwrap_or(part).trim_start();

    if let Some((_, alias)) = part.rsplit_once(" as ") {
        return Some(take_identifier(alias.trim()));
    }

    let name = take_identifier(part);

    if name.is_empty() || name == "type" {
        None
    } else {
        Some(name)
    }
}

fn js_single_binding_name(binding: &str) -> String {
    let binding = binding
        .trim()
        .strip_prefix("type ")
        .unwrap_or(binding.trim())
        .trim();
    let name = take_identifier(binding);

    if is_js_reserved_word(&name) {
        String::new()
    } else {
        name
    }
}

fn js_default_export_name(default_rest: &str) -> Option<(usize, String)> {
    for keyword in ["function", "class"] {
        if let Some((col, name)) = keyword_name(default_rest, keyword) {
            return Some((col, name));
        }
    }

    let name = take_identifier(default_rest);

    if name.is_empty() || is_js_reserved_word(&name) {
        None
    } else {
        Some((1, name))
    }
}

fn js_variable_declaration(code: &str) -> Option<(usize, String, &str)> {
    let trimmed = code.trim_start();
    let mut offset = code.len() - trimmed.len();
    let mut rest = trimmed;

    for prefix in ["export", "declare"] {
        if let Some(after_prefix) = strip_js_keyword_prefix(rest, prefix) {
            offset += rest.len() - after_prefix.len();
            rest = after_prefix.trim_start();
            offset += after_prefix.len() - rest.len();
        }
    }

    for keyword in ["const", "let", "var"] {
        if let Some(after_keyword) = strip_js_keyword_prefix(rest, keyword) {
            let name_start = offset + rest.len() - after_keyword.len();
            let after_space = after_keyword.trim_start();
            let name_start = name_start + after_keyword.len() - after_space.len();
            let name = take_identifier(after_space);

            if name.is_empty() || is_js_reserved_word(&name) {
                return None;
            }

            let after_name = &after_space[name.len()..];
            return Some((name_start + 1, name, after_name));
        }
    }

    None
}

fn js_assignment_is_function(rest: &str) -> bool {
    let before_semicolon = rest.split(';').next().unwrap_or(rest);

    before_semicolon.contains("=>")
        || before_semicolon.contains("= function")
        || before_semicolon.contains("= async function")
}

fn js_class_method_name(code: &str) -> Option<(usize, String)> {
    let trimmed = code.trim_start();
    let mut offset = code.len() - trimmed.len();
    let mut rest = trimmed;

    if rest.starts_with('}') || rest.starts_with('{') || rest.starts_with(';') {
        return None;
    }

    loop {
        let mut changed = false;

        for prefix in [
            "public",
            "private",
            "protected",
            "static",
            "async",
            "readonly",
            "override",
            "abstract",
            "accessor",
            "declare",
            "get",
            "set",
        ] {
            if let Some(after_prefix) = strip_js_keyword_prefix(rest, prefix) {
                offset += rest.len() - after_prefix.len();
                rest = after_prefix.trim_start();
                offset += after_prefix.len() - rest.len();
                changed = true;
                break;
            }
        }

        if !changed {
            break;
        }
    }

    if rest.starts_with('[') {
        return None;
    }

    if rest.starts_with('#') {
        offset += 1;
        rest = &rest[1..];
    }

    let name = take_identifier(rest);

    if name.is_empty() || is_js_reserved_word(&name) {
        return None;
    }

    let mut after_name = &rest[name.len()..];

    if after_name.starts_with('?') {
        after_name = &after_name[1..];
    }

    after_name = strip_js_type_parameters(after_name.trim_start()).trim_start();

    if after_name.starts_with('(')
        || (after_name.starts_with('=')
            && after_name.split(';').next().unwrap_or("").contains("=>"))
    {
        Some((offset + 1, name))
    } else {
        None
    }
}

fn strip_js_keyword_prefix<'a>(value: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = value.strip_prefix(keyword)?;
    let next = rest.chars().next();

    if next.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$') {
        return None;
    }

    Some(rest)
}

fn strip_js_type_parameters(value: &str) -> &str {
    let trimmed = value.trim_start();

    if !trimmed.starts_with('<') {
        return value;
    }

    let mut depth = 0usize;

    for (index, ch) in trimmed.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth = depth.saturating_sub(1);

                if depth == 0 {
                    return &trimmed[index + 1..];
                }
            }
            _ => {}
        }
    }

    value
}

fn is_js_reserved_word(name: &str) -> bool {
    matches!(
        name,
        "" | "as"
            | "async"
            | "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "constructor"
            | "continue"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "export"
            | "extends"
            | "finally"
            | "for"
            | "from"
            | "function"
            | "get"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "let"
            | "new"
            | "return"
            | "set"
            | "static"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "try"
            | "type"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
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
            "pub const INDEX_SCHEMA_VERSION: u32 = 9;\npub fn build_index() {}\nstruct PromptService;\n",
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
    fn parses_js_ts_jsx_tsx_symbols() {
        let js_records = parse_file(
            "frontend/navigation.js",
            r#"
import React, { useMemo as useHeaderMemo } from "react";
import * as Metrics from "./metrics";

export class HeaderController {
  buildUrl(path) {
    return path;
  }

  handleClick = () => {};
}

export function createHeaderController() {}
export const useHeaderState = () => true;
export { useHeaderState as exportedHeaderState };
"#,
        );

        for (name, kind) in [
            ("React", "import"),
            ("useHeaderMemo", "import"),
            ("Metrics", "import"),
            ("HeaderController", "export"),
            ("HeaderController", "class"),
            ("buildUrl", "method"),
            ("handleClick", "method"),
            ("createHeaderController", "export"),
            ("createHeaderController", "function"),
            ("useHeaderState", "function"),
            ("exportedHeaderState", "export"),
        ] {
            assert!(
                js_records.iter().any(|record| record.name == name
                    && record.kind == kind
                    && record.source == NATIVE_SOURCE),
                "missing {kind} {name}, got:\n{js_records:#?}"
            );
        }

        let ts_records = parse_file(
            "frontend/navigation.ts",
            r#"
import type { RemoteNavigationProps as RemoteProps } from "./remote";

export interface HeaderNavigationProps {
  title: string;
}

export type HeaderNavigationMode = "compact" | "full";

export const createHeaderConfig = function () {
  return {};
};
"#,
        );

        for (name, kind) in [
            ("RemoteProps", "import"),
            ("HeaderNavigationProps", "export"),
            ("HeaderNavigationProps", "interface"),
            ("HeaderNavigationMode", "type"),
            ("createHeaderConfig", "function"),
        ] {
            assert!(
                ts_records.iter().any(|record| record.name == name
                    && record.kind == kind
                    && record.source == NATIVE_SOURCE),
                "missing {kind} {name}, got:\n{ts_records:#?}"
            );
        }

        let tsx_records = parse_file(
            "frontend/HeaderNavigation.tsx",
            r#"
import { HeaderButton } from "./HeaderButton";

export function HeaderNavigation(props: HeaderNavigationProps) {
  return <HeaderButton label={props.title} />;
}

export const HeaderShell = () => <HeaderNavigation title="Home" />;
"#,
        );

        for (name, kind) in [
            ("HeaderButton", "import"),
            ("HeaderNavigation", "export"),
            ("HeaderNavigation", "function"),
            ("HeaderShell", "function"),
        ] {
            assert!(
                tsx_records.iter().any(|record| record.name == name
                    && record.kind == kind
                    && record.source == NATIVE_SOURCE),
                "missing {kind} {name}, got:\n{tsx_records:#?}"
            );
        }
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
