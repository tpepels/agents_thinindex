use std::collections::{BTreeMap, BTreeSet};

use crate::model::{IndexRecord, ReferenceRecord};

pub const MIN_TEXT_REFERENCE_SYMBOL_LEN: usize = 3;
pub const MAX_REFS_PER_TARGET_NAME: usize = 20;
pub const MAX_REFS_PER_FILE: usize = 50;
pub const MAX_TOTAL_REFS_PER_BUILD: usize = 5_000;
pub const REFERENCE_STOPLIST: &[&str] = &[
    "test", "main", "new", "run", "app", "id", "name", "type", "value",
];

const SOURCE_TEXT: &str = "text";
const SOURCE_IMPORTS: &str = "imports";
const SOURCE_EXTRAS: &str = "extras";

#[derive(Debug, Clone)]
struct TargetSymbol {
    name: String,
    kind: String,
    definitions: BTreeSet<(String, usize, usize)>,
}

#[derive(Debug, Clone, Copy)]
struct LineContext<'a> {
    path: &'a str,
    lang: &'a str,
    line_no: usize,
    text: &'a str,
}

#[derive(Debug, Clone)]
struct RefSpec<'a> {
    to_kind: Option<&'a str>,
    ref_kind: &'a str,
    evidence: String,
    source: &'a str,
}

pub fn extract_refs(path: &str, text: &str, records: &[IndexRecord]) -> Vec<ReferenceRecord> {
    let targets = target_symbols(records);
    let lang = lang_from_path(path);
    let mut refs = Vec::new();

    for (line_index, line) in text.lines().enumerate() {
        let ctx = LineContext {
            path,
            lang,
            line_no: line_index + 1,
            text: line,
        };

        extract_imports(&ctx, &mut refs);
        extract_text_references(&ctx, &targets, &mut refs);

        match lang {
            "md" | "markdown" => extract_markdown_links(&ctx, &mut refs),
            "css" => extract_css_usages(&ctx, &mut refs),
            "html" => extract_html_usages(&ctx, &mut refs),
            "tsx" | "jsx" => extract_html_usages(&ctx, &mut refs),
            _ => {}
        }
    }

    finalize_refs(refs)
}

fn target_symbols(records: &[IndexRecord]) -> BTreeMap<String, TargetSymbol> {
    let mut targets = BTreeMap::new();

    for record in records {
        if !is_text_reference_target(&record.name) {
            continue;
        }

        let entry = targets
            .entry(record.name.clone())
            .or_insert_with(|| TargetSymbol {
                name: record.name.clone(),
                kind: record.kind.clone(),
                definitions: BTreeSet::new(),
            });

        entry
            .definitions
            .insert((record.path.clone(), record.line, record.col));
    }

    targets
}

fn is_text_reference_target(name: &str) -> bool {
    name.chars().count() >= MIN_TEXT_REFERENCE_SYMBOL_LEN
        && !is_stoplisted(name)
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        && name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
}

fn lang_from_path(path: &str) -> &str {
    path.rsplit_once('.')
        .map(|(_, ext)| ext)
        .unwrap_or("unknown")
}

fn push_ref(
    refs: &mut Vec<ReferenceRecord>,
    ctx: &LineContext<'_>,
    col: usize,
    to_name: impl Into<String>,
    spec: RefSpec<'_>,
) {
    let to_name = to_name.into();

    if to_name.trim().is_empty() || is_stoplisted(&to_name) {
        return;
    }

    refs.push(ReferenceRecord::new(
        ctx.path,
        ctx.line_no,
        col,
        to_name,
        spec.to_kind,
        spec.ref_kind,
        compact_evidence(spec.evidence),
        spec.source,
    ));
}

fn extract_text_references(
    ctx: &LineContext<'_>,
    targets: &BTreeMap<String, TargetSymbol>,
    refs: &mut Vec<ReferenceRecord>,
) {
    for target in targets.values() {
        let mut search_start = 0;

        while let Some(relative_index) = ctx.text[search_start..].find(&target.name) {
            let index = search_start + relative_index;
            let col = index + 1;

            if is_word_boundary_match(ctx.text, index, target.name.len())
                && !target
                    .definitions
                    .contains(&(ctx.path.to_string(), ctx.line_no, col))
            {
                let ref_kind = if is_test_path(ctx.path) {
                    "test_reference"
                } else {
                    "text_reference"
                };

                push_ref(
                    refs,
                    ctx,
                    col,
                    target.name.as_str(),
                    RefSpec {
                        to_kind: Some(target.kind.as_str()),
                        ref_kind,
                        evidence: ctx.text.trim().to_string(),
                        source: SOURCE_TEXT,
                    },
                );
            }

            search_start = index + target.name.len();
        }
    }
}

fn extract_imports(ctx: &LineContext<'_>, refs: &mut Vec<ReferenceRecord>) {
    match ctx.lang {
        "py" => extract_python_imports(ctx, refs),
        "rs" => extract_rust_imports(ctx, refs),
        "ts" | "tsx" | "js" | "jsx" => extract_js_imports(ctx, refs),
        _ => {}
    }
}

fn extract_python_imports(ctx: &LineContext<'_>, refs: &mut Vec<ReferenceRecord>) {
    let trimmed = ctx.text.trim_start();
    let leading_spaces = ctx.text.len() - trimmed.len();

    if let Some(rest) = trimmed.strip_prefix("from ") {
        let Some(import_index) = rest.find(" import ") else {
            return;
        };
        let imported = &rest[import_index + " import ".len()..];
        let imported_base = leading_spaces + "from ".len() + import_index + " import ".len();
        push_import_names(ctx, refs, imported, imported_base);
        return;
    }

    if let Some(imported) = trimmed.strip_prefix("import ") {
        push_import_names(ctx, refs, imported, leading_spaces + "import ".len());
    }
}

fn push_import_names(
    ctx: &LineContext<'_>,
    refs: &mut Vec<ReferenceRecord>,
    imported: &str,
    imported_base: usize,
) {
    let mut offset = 0;

    for raw_part in imported.split(',') {
        let leading = raw_part.len() - raw_part.trim_start().len();
        let part_start = imported_base + offset + leading;
        let part = raw_part.trim();
        let name = part
            .split_whitespace()
            .next()
            .unwrap_or("")
            .rsplit('.')
            .next()
            .unwrap_or("");

        if is_identifier(name) {
            push_ref(
                refs,
                ctx,
                part_start + 1,
                name,
                RefSpec {
                    to_kind: None,
                    ref_kind: "import",
                    evidence: import_evidence(ctx.text),
                    source: SOURCE_IMPORTS,
                },
            );
        }

        offset += raw_part.len() + 1;
    }
}

fn extract_rust_imports(ctx: &LineContext<'_>, refs: &mut Vec<ReferenceRecord>) {
    let trimmed = ctx.text.trim_start();
    let leading_spaces = ctx.text.len() - trimmed.len();

    if let Some(rest) = trimmed.strip_prefix("mod ") {
        let name = take_identifier(rest);

        if !name.is_empty() {
            push_ref(
                refs,
                ctx,
                leading_spaces + "mod ".len() + 1,
                name,
                RefSpec {
                    to_kind: None,
                    ref_kind: "import",
                    evidence: import_evidence(ctx.text),
                    source: SOURCE_IMPORTS,
                },
            );
        }

        return;
    }

    let Some(rest) = trimmed.strip_prefix("use ") else {
        return;
    };

    if !(rest.starts_with("crate::") || rest.starts_with("super::")) {
        return;
    }

    let path = rest.trim_end_matches(';').trim();
    let base = leading_spaces + "use ".len();

    for (offset, name) in rust_use_names(path) {
        push_ref(
            refs,
            ctx,
            base + offset + 1,
            name,
            RefSpec {
                to_kind: None,
                ref_kind: "import",
                evidence: import_evidence(ctx.text),
                source: SOURCE_IMPORTS,
            },
        );
    }
}

fn rust_use_names(path: &str) -> Vec<(usize, &str)> {
    if let Some(open) = path.find('{') {
        let close = path[open + 1..]
            .find('}')
            .map(|offset| open + 1 + offset)
            .unwrap_or(path.len());
        let inner = &path[open + 1..close];
        let mut names = Vec::new();
        let mut offset = 0;

        for raw_part in inner.split(',') {
            let leading = raw_part.len() - raw_part.trim_start().len();
            let name = take_identifier(raw_part.trim());

            if !name.is_empty() {
                names.push((open + 1 + offset + leading, name));
            }

            offset += raw_part.len() + 1;
        }

        return names;
    }

    let trimmed = path.trim_end_matches(';').trim();
    let Some((prefix, leaf)) = trimmed.rsplit_once("::") else {
        return Vec::new();
    };

    if !is_identifier(leaf) {
        return Vec::new();
    }

    vec![(prefix.len() + "::".len(), leaf)]
}

fn extract_js_imports(ctx: &LineContext<'_>, refs: &mut Vec<ReferenceRecord>) {
    let trimmed = ctx.text.trim_start();
    let leading_spaces = ctx.text.len() - trimmed.len();

    if trimmed.starts_with("import ")
        || (trimmed.starts_with("export ") && trimmed.contains(" from "))
    {
        extract_js_named_imports(ctx, refs, trimmed, leading_spaces);
    }
}

fn extract_js_named_imports(
    ctx: &LineContext<'_>,
    refs: &mut Vec<ReferenceRecord>,
    trimmed: &str,
    leading_spaces: usize,
) {
    let Some(from_index) = trimmed.find(" from ") else {
        return;
    };

    let spec = &trimmed[..from_index];

    if let Some(open) = spec.find('{') {
        let close = spec[open + 1..]
            .find('}')
            .map(|offset| open + 1 + offset)
            .unwrap_or(spec.len());
        let inner = &spec[open + 1..close];
        let mut offset = 0;

        for raw_part in inner.split(',') {
            let leading = raw_part.len() - raw_part.trim_start().len();
            let part = raw_part.trim();
            let name = part.split_whitespace().next().unwrap_or("");

            if is_identifier(name) {
                push_ref(
                    refs,
                    ctx,
                    leading_spaces + open + 1 + offset + leading + 1,
                    name,
                    RefSpec {
                        to_kind: None,
                        ref_kind: "import",
                        evidence: import_evidence(ctx.text),
                        source: SOURCE_IMPORTS,
                    },
                );
            }

            offset += raw_part.len() + 1;
        }

        return;
    }

    let spec = spec
        .trim_start_matches("import")
        .trim_start_matches("export")
        .trim();
    let name = spec.split(',').next().unwrap_or("").trim();

    if is_identifier(name) {
        let col = ctx.text.find(name).map(|index| index + 1).unwrap_or(1);
        push_ref(
            refs,
            ctx,
            col,
            name,
            RefSpec {
                to_kind: None,
                ref_kind: "import",
                evidence: import_evidence(ctx.text),
                source: SOURCE_IMPORTS,
            },
        );
    }
}

fn extract_markdown_links(ctx: &LineContext<'_>, refs: &mut Vec<ReferenceRecord>) {
    let mut search_start = 0;

    while let Some(relative_open) = ctx.text[search_start..].find('[') {
        let open = search_start + relative_open;

        let Some(close_relative) = ctx.text[open..].find(']') else {
            break;
        };
        let close = open + close_relative;

        if !ctx.text[close..].starts_with("](") {
            search_start = open + 1;
            continue;
        }

        let target_start = close + 2;
        let Some(target_close_relative) = ctx.text[target_start..].find(')') else {
            break;
        };
        let target_close = target_start + target_close_relative;
        let target = &ctx.text[target_start..target_close];

        if !target.is_empty() {
            push_ref(
                refs,
                ctx,
                target_start + 1,
                target,
                RefSpec {
                    to_kind: Some("link"),
                    ref_kind: "markdown_link",
                    evidence: markdown_link_evidence(ctx.text, open, target_close + 1),
                    source: SOURCE_EXTRAS,
                },
            );
        }

        search_start = target_close + 1;
    }
}

fn extract_css_usages(ctx: &LineContext<'_>, refs: &mut Vec<ReferenceRecord>) {
    for (index, token) in css_tokens(ctx.text) {
        let to_kind = if token.starts_with('.') {
            "css_class"
        } else if token.starts_with("--") {
            "css_variable"
        } else {
            continue;
        };

        push_ref(
            refs,
            ctx,
            index + 1,
            token.as_str(),
            RefSpec {
                to_kind: Some(to_kind),
                ref_kind: "css_usage",
                evidence: token.clone(),
                source: SOURCE_EXTRAS,
            },
        );
    }
}

fn css_tokens(line: &str) -> Vec<(usize, String)> {
    let mut tokens = Vec::new();
    let bytes = line.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let ch = bytes[index] as char;

        if ch == '.' && index + 1 < bytes.len() && is_ident_start(bytes[index + 1] as char) {
            let end = scan_identifier(line, index + 1);
            tokens.push((index, line[index..end].to_string()));
            index = end;
            continue;
        }

        if ch == '-' && index + 1 < bytes.len() && bytes[index + 1] as char == '-' {
            let end = scan_identifier(line, index + 2);

            if end > index + 2 {
                tokens.push((index, line[index..end].to_string()));
                index = end;
                continue;
            }
        }

        index += 1;
    }

    tokens
}

fn extract_html_usages(ctx: &LineContext<'_>, refs: &mut Vec<ReferenceRecord>) {
    let mut search_start = 0;

    while let Some(relative_open) = ctx.text[search_start..].find('<') {
        let open = search_start + relative_open;

        if ctx.text[open..].starts_with("</") || ctx.text[open..].starts_with("<!--") {
            search_start = open + 1;
            continue;
        }

        let tag_start = open + 1;
        let tag_end = ctx.text[open..]
            .find('>')
            .map(|offset| open + offset)
            .unwrap_or(ctx.text.len());

        let attrs = &ctx.text[tag_start..tag_end];
        extract_html_attributes(ctx, refs, attrs, tag_start);

        search_start = tag_end.saturating_add(1).min(ctx.text.len());
    }
}

fn extract_html_attributes(
    ctx: &LineContext<'_>,
    refs: &mut Vec<ReferenceRecord>,
    attrs: &str,
    attrs_base: usize,
) {
    for attr in ["id", "class", "className", "data-testid"] {
        let mut search_start = 0;

        while let Some(relative_index) = attrs[search_start..].find(attr) {
            let attr_index = search_start + relative_index;

            if !is_attribute_boundary(attrs, attr_index, attr.len()) {
                search_start = attr_index + attr.len();
                continue;
            }

            let absolute_attr_col = attrs_base + attr_index + 1;

            if attr.starts_with("data-") {
                push_ref(
                    refs,
                    ctx,
                    absolute_attr_col,
                    attr,
                    RefSpec {
                        to_kind: Some("data_attribute"),
                        ref_kind: "html_usage",
                        evidence: attr.to_string(),
                        source: SOURCE_EXTRAS,
                    },
                );
            }

            if let Some((value_start, value)) = attribute_value(attrs, attr_index + attr.len()) {
                let absolute_value_start = attrs_base + value_start;

                match attr {
                    "id" if !value.is_empty() => {
                        push_ref(
                            refs,
                            ctx,
                            absolute_value_start + 1,
                            format!("#{value}"),
                            RefSpec {
                                to_kind: Some("html_id"),
                                ref_kind: "html_usage",
                                evidence: format!("#{value}"),
                                source: SOURCE_EXTRAS,
                            },
                        );
                    }
                    "class" | "className" => {
                        push_class_usage_refs(refs, ctx, absolute_value_start, value);
                    }
                    _ => {}
                }
            }

            search_start = attr_index + attr.len();
        }
    }
}

fn push_class_usage_refs(
    refs: &mut Vec<ReferenceRecord>,
    ctx: &LineContext<'_>,
    value_start: usize,
    value: &str,
) {
    let mut offset = 0;

    for class_name in value.split_whitespace() {
        if let Some(relative_index) = value[offset..].find(class_name) {
            let class_start = value_start + offset + relative_index;
            let to_name = format!(".{class_name}");
            push_ref(
                refs,
                ctx,
                class_start + 1,
                to_name.as_str(),
                RefSpec {
                    to_kind: Some("css_class"),
                    ref_kind: "html_usage",
                    evidence: to_name.clone(),
                    source: SOURCE_EXTRAS,
                },
            );
            offset += relative_index + class_name.len();
        }
    }
}

pub fn finalize_refs(mut refs: Vec<ReferenceRecord>) -> Vec<ReferenceRecord> {
    refs.sort_by_key(ref_sort_key);

    let mut capped = Vec::new();
    let mut per_target: BTreeMap<String, usize> = BTreeMap::new();
    let mut per_file: BTreeMap<String, usize> = BTreeMap::new();

    for reference in refs {
        if capped.len() >= MAX_TOTAL_REFS_PER_BUILD {
            break;
        }

        let target_count = per_target.entry(reference.to_name.clone()).or_default();
        if *target_count >= MAX_REFS_PER_TARGET_NAME {
            continue;
        }

        let file_count = per_file.entry(reference.from_path.clone()).or_default();
        if *file_count >= MAX_REFS_PER_FILE {
            continue;
        }

        *target_count += 1;
        *file_count += 1;
        capped.push(reference);
    }

    capped
}

fn ref_sort_key(reference: &ReferenceRecord) -> (String, usize, usize, String, String, String) {
    (
        reference.from_path.clone(),
        reference.from_line,
        reference.from_col,
        reference.to_name.clone(),
        reference.ref_kind.clone(),
        reference.source.clone(),
    )
}

fn is_word_boundary_match(line: &str, start: usize, len: usize) -> bool {
    let before = line[..start].chars().next_back();
    let after = line[start + len..].chars().next();

    before.is_none_or(|ch| !is_word_char(ch)) && after.is_none_or(|ch| !is_word_char(ch))
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_identifier(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && value.chars().all(is_ident_char)
        && !is_stoplisted(value)
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '-'
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
}

fn scan_identifier(line: &str, start: usize) -> usize {
    let mut end = start;

    for (offset, ch) in line[start..].char_indices() {
        if !is_ident_char(ch) {
            break;
        }

        end = start + offset + ch.len_utf8();
    }

    end
}

fn take_identifier(value: &str) -> &str {
    let end = value
        .char_indices()
        .take_while(|(_, ch)| is_ident_char(*ch))
        .map(|(idx, ch)| idx + ch.len_utf8())
        .last()
        .unwrap_or(0);

    &value[..end]
}

fn attribute_value(attrs: &str, after_name: usize) -> Option<(usize, &str)> {
    let after_name_slice = &attrs[after_name..];
    let equals_relative = after_name_slice.find('=')?;
    let equals = after_name + equals_relative;

    let after_equals = attrs[equals + 1..].trim_start();
    let leading_spaces = attrs[equals + 1..].len() - after_equals.len();
    let value_open = equals + 1 + leading_spaces;

    let quote = attrs[value_open..].chars().next()?;

    if quote != '"' && quote != '\'' {
        return None;
    }

    let value_start = value_open + quote.len_utf8();
    let value_rest = &attrs[value_start..];
    let value_end_relative = value_rest.find(quote)?;
    let value_end = value_start + value_end_relative;

    Some((value_start, &attrs[value_start..value_end]))
}

fn is_attribute_boundary(attrs: &str, start: usize, len: usize) -> bool {
    let before = attrs[..start].chars().next_back();
    let after = attrs[start + len..].chars().next();

    let valid_before =
        before.is_none_or(|ch| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'));
    let valid_after =
        after.is_none_or(|ch| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'));

    valid_before && valid_after
}

fn import_evidence(line: &str) -> String {
    compact_evidence(line.trim().trim_end_matches(';').to_string())
}

fn markdown_link_evidence(line: &str, start: usize, end: usize) -> String {
    compact_evidence(line[start..end].to_string())
}

fn compact_evidence(value: String) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_test_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    let filename = normalized.rsplit('/').next().unwrap_or(&normalized);

    normalized.starts_with("tests/")
        || normalized.contains("/tests/")
        || normalized.starts_with("test/")
        || normalized.contains("/test/")
        || normalized.starts_with("__tests__/")
        || normalized.contains("/__tests__/")
        || filename.contains("_test")
        || filename.contains(".test.")
        || filename.contains(".spec.")
}

fn is_stoplisted(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    REFERENCE_STOPLIST.contains(&normalized.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(path: &str, line: usize, col: usize, kind: &str, name: &str) -> IndexRecord {
        IndexRecord::new(path, line, col, "rs", kind, name, name, "ctags")
    }

    #[test]
    fn extracts_python_import_reference() {
        let refs = extract_refs(
            "src/use_service.py",
            "from service import PromptService\n",
            &[],
        );

        assert!(refs.iter().any(|r| {
            r.to_name == "PromptService" && r.ref_kind == "import" && r.source == SOURCE_IMPORTS
        }));
    }

    #[test]
    fn extracts_rust_use_and_mod_references() {
        let refs = extract_refs(
            "src/lib.rs",
            "use crate::indexer::build_index;\nmod refs;\n",
            &[],
        );

        assert!(
            refs.iter()
                .any(|r| r.to_name == "build_index" && r.ref_kind == "import")
        );
        assert!(
            refs.iter()
                .any(|r| r.to_name == "refs" && r.ref_kind == "import")
        );
    }

    #[test]
    fn extracts_js_import_reference() {
        let refs = extract_refs(
            "frontend/app.tsx",
            "import { PromptService } from './prompt_service';\n",
            &[],
        );

        assert!(
            refs.iter()
                .any(|r| r.to_name == "PromptService" && r.ref_kind == "import")
        );
    }

    #[test]
    fn extracts_markdown_link_reference() {
        let refs = extract_refs("docs/guide.md", "[Guide](../src/service.py)\n", &[]);

        assert!(refs.iter().any(|r| {
            r.to_name == "../src/service.py"
                && r.ref_kind == "markdown_link"
                && r.to_kind.as_deref() == Some("link")
                && r.source == SOURCE_EXTRAS
        }));
    }

    #[test]
    fn extracts_css_and_html_usage_references() {
        let css_refs = extract_refs(
            "styles/app.css",
            ".headerNavigation { color: var(--paper-bg); }\n",
            &[],
        );
        let html_refs = extract_refs(
            "templates/base.html",
            r#"<div id="mainHeader" class="headerNavigation" data-testid="main-header"></div>"#,
            &[],
        );

        assert!(
            css_refs
                .iter()
                .any(|r| r.to_name == ".headerNavigation" && r.ref_kind == "css_usage")
        );
        assert!(
            css_refs
                .iter()
                .any(|r| r.to_name == "--paper-bg" && r.ref_kind == "css_usage")
        );
        assert!(
            html_refs
                .iter()
                .any(|r| r.to_name == "#mainHeader" && r.ref_kind == "html_usage")
        );
        assert!(
            html_refs
                .iter()
                .any(|r| r.to_name == ".headerNavigation" && r.ref_kind == "html_usage")
        );
        assert!(
            html_refs
                .iter()
                .any(|r| r.to_name == "data-testid" && r.ref_kind == "html_usage")
        );
    }

    #[test]
    fn extracts_text_reference_without_definition_or_subword_matches() {
        let records = vec![record("src/service.py", 1, 7, "class", "PromptService")];
        let refs = extract_refs(
            "src/use_service.py",
            "PromptService()\nOtherPromptService()\n",
            &records,
        );

        assert!(
            refs.iter()
                .any(|r| r.to_name == "PromptService" && r.from_line == 1)
        );
        assert!(!refs.iter().any(|r| r.from_line == 2));
    }

    #[test]
    fn classifies_text_refs_in_tests_as_test_reference() {
        let records = vec![record("src/service.py", 1, 7, "class", "PromptService")];
        let refs = extract_refs(
            "tests/service_test.py",
            "assert PromptService()\n",
            &records,
        );

        assert!(
            refs.iter()
                .any(|r| r.to_name == "PromptService" && r.ref_kind == "test_reference")
        );
    }

    #[test]
    fn stoplist_excludes_common_text_reference_targets() {
        let records = vec![record("src/app.py", 1, 5, "function", "run")];
        let refs = extract_refs("src/use_app.py", "run()\n", &records);

        assert!(refs.is_empty());
    }

    #[test]
    fn applies_per_file_and_per_target_caps() {
        let records = vec![record("src/service.py", 1, 7, "class", "PromptService")];
        let text = "PromptService()\n".repeat(MAX_REFS_PER_FILE + 10);
        let refs = extract_refs("src/use_service.py", &text, &records);

        assert!(refs.len() <= MAX_REFS_PER_FILE);
        assert!(refs.len() <= MAX_REFS_PER_TARGET_NAME);
    }
}
