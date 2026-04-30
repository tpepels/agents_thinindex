use crate::model::IndexRecord;

const EXTRA_SOURCE: &str = "extras";

#[derive(Debug, Clone, Copy)]
struct LineContext<'a> {
    path: &'a str,
    lang: &'a str,
    line_no: usize,
    text: &'a str,
}

pub fn index_extras(path: &str, text: &str) -> Vec<IndexRecord> {
    let lang = lang_from_path(path);
    let mut records = Vec::new();

    for (line_index, line) in text.lines().enumerate() {
        let ctx = LineContext {
            path,
            lang,
            line_no: line_index + 1,
            text: line,
        };

        index_todos(&ctx, &mut records);

        match lang {
            "css" => index_css_line(&ctx, &mut records),
            "html" => index_html_line(&ctx, &mut records),
            "json" => index_json_line(&ctx, &mut records),
            "tsx" | "jsx" => {
                index_html_line(&ctx, &mut records);
                index_jsx_line(&ctx, &mut records);
            }
            "md" | "markdown" => index_markdown_line(&ctx, &mut records),
            "toml" => index_toml_line(&ctx, &mut records),
            "yaml" | "yml" => index_yaml_line(&ctx, &mut records),
            _ => {}
        }
    }

    records
}

fn lang_from_path(path: &str) -> &str {
    path.rsplit_once('.')
        .map(|(_, ext)| ext)
        .unwrap_or("unknown")
}

fn push_record(
    records: &mut Vec<IndexRecord>,
    ctx: &LineContext<'_>,
    col: usize,
    kind: &str,
    name: impl Into<String>,
) {
    records.push(IndexRecord {
        path: ctx.path.to_string(),
        line: ctx.line_no,
        col,
        lang: ctx.lang.to_string(),
        kind: kind.to_string(),
        name: name.into(),
        text: ctx.text.trim().to_string(),
        source: EXTRA_SOURCE.to_string(),
    });
}

fn index_todos(ctx: &LineContext<'_>, records: &mut Vec<IndexRecord>) {
    for marker in ["TODO", "FIXME"] {
        if let Some(index) = ctx.text.find(marker) {
            push_record(
                records,
                ctx,
                index + 1,
                &marker.to_ascii_lowercase(),
                marker,
            );
        }
    }
}

fn index_css_line(ctx: &LineContext<'_>, records: &mut Vec<IndexRecord>) {
    for (index, token) in css_tokens(ctx.text) {
        let kind = if token.starts_with('.') {
            "css_class"
        } else if token.starts_with('#') {
            "css_id"
        } else if token.starts_with("--") {
            "css_variable"
        } else {
            continue;
        };

        push_record(records, ctx, index + 1, kind, token);
    }

    if let Some(keyframes_index) = ctx.text.find("@keyframes") {
        let after = keyframes_index + "@keyframes".len();
        let rest = &ctx.text[after..];

        if let Some(offset) = rest.find(|ch: char| !ch.is_whitespace()) {
            let name_start = after + offset;
            let name = take_identifier(&ctx.text[name_start..]);

            if !name.is_empty() {
                push_record(records, ctx, name_start + 1, "keyframes", name);
            }
        }
    }
}

fn css_tokens(line: &str) -> Vec<(usize, String)> {
    let mut tokens = Vec::new();
    let bytes = line.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let ch = bytes[index] as char;

        if (ch == '.' || ch == '#')
            && index + 1 < bytes.len()
            && is_ident_start(bytes[index + 1] as char)
        {
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

fn index_html_line(ctx: &LineContext<'_>, records: &mut Vec<IndexRecord>) {
    let mut search_start = 0;

    while let Some(relative_open) = ctx.text[search_start..].find('<') {
        let open = search_start + relative_open;

        if ctx.text[open..].starts_with("</") || ctx.text[open..].starts_with("<!--") {
            search_start = open + 1;
            continue;
        }

        let tag_start = open + 1;
        let tag_name = take_identifier(&ctx.text[tag_start..]);

        if !tag_name.is_empty() && is_html_tag_name(&tag_name) {
            push_record(records, ctx, tag_start + 1, "html_tag", tag_name);
        }

        let tag_end = ctx.text[open..]
            .find('>')
            .map(|offset| open + offset)
            .unwrap_or(ctx.text.len());

        let attrs = &ctx.text[tag_start..tag_end];
        let attrs_base = tag_start;

        index_html_attributes(ctx, attrs, attrs_base, records);

        // When no closing `>` is on this line (JSX often splits a tag across
        // lines), `tag_end == ctx.text.len()`. Clamp so the next iteration's
        // slice stays in bounds; `find` on an empty slice ends the loop.
        search_start = tag_end.saturating_add(1).min(ctx.text.len());
    }
}

fn index_html_attributes(
    ctx: &LineContext<'_>,
    attrs: &str,
    attrs_base: usize,
    records: &mut Vec<IndexRecord>,
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
                push_record(records, ctx, absolute_attr_col, "data_attribute", attr);
            }

            if let Some((value_start, value)) = attribute_value(attrs, attr_index + attr.len()) {
                let absolute_value_start = attrs_base + value_start;

                match attr {
                    "id" if !value.is_empty() => {
                        push_record(
                            records,
                            ctx,
                            absolute_value_start + 1,
                            "html_id",
                            format!("#{value}"),
                        );
                    }
                    "class" => {
                        push_class_records(records, ctx, "html_class", absolute_value_start, value);
                    }
                    "className" => {
                        push_class_records(records, ctx, "jsx_class", absolute_value_start, value);
                    }
                    _ => {}
                }
            }

            search_start = attr_index + attr.len();
        }
    }
}

fn index_jsx_line(ctx: &LineContext<'_>, records: &mut Vec<IndexRecord>) {
    let mut search_start = 0;

    while let Some(relative_open) = ctx.text[search_start..].find('<') {
        let open = search_start + relative_open;

        if ctx.text[open..].starts_with("</") {
            search_start = open + 1;
            continue;
        }

        let component_start = open + 1;
        let component = take_identifier(&ctx.text[component_start..]);

        if component
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        {
            push_record(
                records,
                ctx,
                component_start + 1,
                "component_usage",
                component,
            );
        }

        search_start = open + 1;
    }
}

fn index_markdown_line(ctx: &LineContext<'_>, records: &mut Vec<IndexRecord>) {
    let trimmed = ctx.text.trim_start();
    let leading_spaces = ctx.text.len() - trimmed.len();

    if trimmed.starts_with('#') {
        let marker_len = trimmed.chars().take_while(|ch| *ch == '#').count();

        if (1..=6).contains(&marker_len) {
            let after_marker = &trimmed[marker_len..];

            if after_marker.starts_with(char::is_whitespace) {
                let name = after_marker.trim();

                if !name.is_empty() {
                    let title_offset = after_marker.len() - after_marker.trim_start().len();
                    push_record(
                        records,
                        ctx,
                        leading_spaces + marker_len + title_offset + 1,
                        "section",
                        name,
                    );
                }
            }
        }
    }

    if trimmed.starts_with("- [ ] ")
        || trimmed.starts_with("- [x] ")
        || trimmed.starts_with("- [X] ")
    {
        let item_start = leading_spaces + 6;
        let name = ctx.text[item_start..].trim();

        if !name.is_empty() {
            push_record(records, ctx, item_start + 1, "checklist", name);
        }
    }

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
            push_record(records, ctx, target_start + 1, "link", target);
        }

        search_start = target_close + 1;
    }
}

fn index_json_line(ctx: &LineContext<'_>, records: &mut Vec<IndexRecord>) {
    let mut search_start = 0;

    while let Some((open, close)) = quoted_span(ctx.text, search_start, '"') {
        let mut after = close + 1;

        while after < ctx.text.len()
            && ctx.text[after..]
                .chars()
                .next()
                .is_some_and(char::is_whitespace)
        {
            after += ctx.text[after..].chars().next().unwrap().len_utf8();
        }

        if ctx.text[after..].starts_with(':')
            && let Some(name) = normalize_config_key(&ctx.text[open + 1..close])
        {
            push_record(records, ctx, open + 2, "key", name);
        }

        search_start = close + 1;
    }
}

fn index_toml_line(ctx: &LineContext<'_>, records: &mut Vec<IndexRecord>) {
    let comment_start = comment_start_outside_quotes(ctx.text);
    let content = &ctx.text[..comment_start];
    let trimmed = content.trim_start();
    let leading_spaces = content.len() - trimmed.len();

    if trimmed.is_empty() {
        return;
    }

    if let Some((name, col)) = toml_table_name(trimmed, leading_spaces) {
        push_record(records, ctx, col, "table", name);
        return;
    }

    if let Some(equals) = find_unquoted_char(content, '=') {
        let raw_key = content[..equals].trim();

        if let Some(name) = normalize_config_key(raw_key) {
            let key_start = content[..equals]
                .find(raw_key)
                .map(|index| index + 1)
                .unwrap_or(1);
            push_record(records, ctx, key_start, "key", name);
        }
    }
}

fn index_yaml_line(ctx: &LineContext<'_>, records: &mut Vec<IndexRecord>) {
    let comment_start = comment_start_outside_quotes(ctx.text);
    let content = &ctx.text[..comment_start];
    let trimmed = content.trim_start();
    let mut leading_spaces = content.len() - trimmed.len();
    let mut candidate = trimmed;

    if candidate.is_empty() || candidate == "---" || candidate == "..." {
        return;
    }

    if let Some(rest) = candidate.strip_prefix("- ") {
        leading_spaces += 2;
        candidate = rest.trim_start();
        leading_spaces += rest.len() - candidate.len();
    }

    let Some(colon) = find_unquoted_char(candidate, ':') else {
        return;
    };

    let raw_key = candidate[..colon].trim();
    let Some(name) = normalize_config_key(raw_key) else {
        return;
    };

    let key_start = leading_spaces
        + candidate[..colon]
            .find(raw_key)
            .map(|index| index + 1)
            .unwrap_or(1);
    let value = candidate[colon + 1..].trim();
    let kind = if value.is_empty() { "section" } else { "key" };

    push_record(records, ctx, key_start, kind, name);
}

fn push_class_records(
    records: &mut Vec<IndexRecord>,
    ctx: &LineContext<'_>,
    kind: &str,
    value_start: usize,
    value: &str,
) {
    let mut offset = 0;

    for class_name in value.split_whitespace() {
        if class_name.is_empty() {
            continue;
        }

        if let Some(relative_index) = value[offset..].find(class_name) {
            let class_start = value_start + offset + relative_index;

            push_record(
                records,
                ctx,
                class_start + 1,
                kind,
                format!(".{class_name}"),
            );

            offset += relative_index + class_name.len();
        }
    }
}

fn toml_table_name(trimmed: &str, leading_spaces: usize) -> Option<(String, usize)> {
    if let Some(rest) = trimmed.strip_prefix("[[") {
        let close = rest.find("]]")?;
        let raw_name = rest[..close].trim();
        let name_offset = rest[..close].find(raw_name).unwrap_or_default();
        return normalize_config_key(raw_name).map(|name| (name, leading_spaces + 3 + name_offset));
    }

    if let Some(rest) = trimmed.strip_prefix('[') {
        let close = rest.find(']')?;
        let raw_name = rest[..close].trim();
        let name_offset = rest[..close].find(raw_name).unwrap_or_default();
        return normalize_config_key(raw_name).map(|name| (name, leading_spaces + 2 + name_offset));
    }

    None
}

fn normalize_config_key(raw: &str) -> Option<String> {
    let key = raw.trim();

    if key.is_empty() || key.starts_with('#') || key.starts_with('?') {
        return None;
    }

    let key = key
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            key.strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(key)
        .trim();

    if key.is_empty()
        || !key.chars().any(|ch| ch.is_ascii_alphanumeric())
        || !key.chars().all(is_config_key_char)
    {
        None
    } else {
        Some(key.to_string())
    }
}

fn is_config_key_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '$' | '@' | ':')
}

fn quoted_span(line: &str, start: usize, quote: char) -> Option<(usize, usize)> {
    let quote_byte = quote as u8;
    let bytes = line.as_bytes();
    let mut open = start;

    while open < bytes.len() && bytes[open] != quote_byte {
        open += 1;
    }

    if open >= bytes.len() {
        return None;
    }

    let mut index = open + 1;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];

        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote_byte {
            return Some((open, index));
        }

        index += 1;
    }

    None
}

fn comment_start_outside_quotes(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut index = 0;
    let mut quote: Option<u8> = None;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];

        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if let Some(active_quote) = quote {
            if byte == active_quote {
                quote = None;
            }
        } else if byte == b'\'' || byte == b'"' {
            quote = Some(byte);
        } else if byte == b'#' {
            return index;
        }

        index += 1;
    }

    line.len()
}

fn find_unquoted_char(line: &str, needle: char) -> Option<usize> {
    let needle = needle as u8;
    let bytes = line.as_bytes();
    let mut index = 0;
    let mut quote: Option<u8> = None;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];

        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if let Some(active_quote) = quote {
            if byte == active_quote {
                quote = None;
            }
        } else if byte == b'\'' || byte == b'"' {
            quote = Some(byte);
        } else if byte == needle {
            return Some(index);
        }

        index += 1;
    }

    None
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

fn take_identifier(value: &str) -> String {
    value.chars().take_while(|ch| is_ident_char(*ch)).collect()
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

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '-'
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
}

fn is_html_tag_name(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression: a `<` with no matching `>` on the same line (a JSX tag split
    // across lines, e.g. `<SpecArchiveScreen\n   props />`) used to panic with
    // `start byte index N is out of bounds` on the next loop iteration.
    #[test]
    fn jsx_tag_without_closing_bracket_does_not_panic() {
        let line = "            <SpecArchiveScreen";
        let records = index_extras("Component.tsx", line);
        assert!(records.iter().any(|r| r.name == "SpecArchiveScreen"));
    }

    #[test]
    fn html_open_bracket_at_end_of_line_does_not_panic() {
        // `<` is the last byte; tag_end == text.len(); next iteration must
        // not slice past the end.
        let _ = index_extras("page.html", "abc<");
        let _ = index_extras("page.html", "<");
    }

    #[test]
    fn config_extras_index_keys_without_scalar_values() {
        let records = index_extras(
            "config/app.json",
            r#"{ "serviceName": "JsonStringFake", "nested": { "enabled": true } }"#,
        );
        assert!(records.iter().any(|record| record.name == "serviceName"));
        assert!(records.iter().any(|record| record.name == "enabled"));
        assert!(!records.iter().any(|record| record.name == "JsonStringFake"));

        let records = index_extras(
            "settings.json",
            r#"{ "./target/debug/build_index": true, "parserConfigEnabled": true }"#,
        );
        assert!(
            !records
                .iter()
                .any(|record| record.name == "./target/debug/build_index")
        );
        assert!(
            records
                .iter()
                .any(|record| record.name == "parserConfigEnabled")
        );

        let records = index_extras(
            "thinindex.toml",
            "[tool.thinindex]\nprofile = \"TomlStringFake\"\n",
        );
        assert!(records.iter().any(|record| {
            record.kind == "table" && record.name == "tool.thinindex" && record.line == 1
        }));
        assert!(records.iter().any(|record| record.name == "profile"));
        assert!(!records.iter().any(|record| record.name == "TomlStringFake"));

        let records = index_extras(
            "pipeline.yaml",
            "pipeline:\n  name: parser-config\n  note: \"YamlStringFake: no symbol\"\n",
        );
        assert!(records.iter().any(|record| {
            record.kind == "section" && record.name == "pipeline" && record.line == 1
        }));
        assert!(records.iter().any(|record| record.name == "name"));
        assert!(!records.iter().any(|record| record.name == "YamlStringFake"));
    }
}
