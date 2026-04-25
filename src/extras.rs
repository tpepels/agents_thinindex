use regex::Regex;

use crate::model::IndexRecord;

pub fn index_extras(relpath: &str, text: &str) -> Vec<IndexRecord> {
    let lang = language_from_path(relpath);

    let mut records = Vec::new();

    records.extend(index_todos(relpath, &lang, text));

    match lang.as_str() {
        "css" => records.extend(index_css(relpath, text)),
        "html" => records.extend(index_html(relpath, text)),
        "md" | "mdx" => records.extend(index_markdown(relpath, text)),
        "tsx" | "jsx" => records.extend(index_jsx_usage(relpath, &lang, text)),
        _ => {}
    }

    records
}

fn index_todos(relpath: &str, lang: &str, text: &str) -> Vec<IndexRecord> {
    let mut records = Vec::new();

    for (idx, line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let upper = line.to_ascii_uppercase();

        if upper.contains("TODO") {
            records.push(IndexRecord::new(
                relpath,
                line_no,
                first_col(line, "TODO"),
                lang,
                "todo",
                "TODO",
                line.trim(),
                "extras",
            ));
        }

        if upper.contains("FIXME") {
            records.push(IndexRecord::new(
                relpath,
                line_no,
                first_col(line, "FIXME"),
                lang,
                "fixme",
                "FIXME",
                line.trim(),
                "extras",
            ));
        }
    }

    records
}

fn index_css(relpath: &str, text: &str) -> Vec<IndexRecord> {
    let class_re = Regex::new(r"\.(-?[_a-zA-Z]+[_a-zA-Z0-9-]*)").unwrap();
    let id_re = Regex::new(r"#(-?[_a-zA-Z]+[_a-zA-Z0-9-]*)").unwrap();
    let var_re = Regex::new(r"(--[_a-zA-Z]+[_a-zA-Z0-9-]*)\s*:").unwrap();
    let keyframes_re = Regex::new(r"@keyframes\s+([_a-zA-Z][_a-zA-Z0-9-]*)").unwrap();

    let mut records = Vec::new();

    for (idx, line) in text.lines().enumerate() {
        let line_no = idx + 1;

        for caps in class_re.captures_iter(line) {
            let full = caps.get(0).unwrap().as_str();
            records.push(IndexRecord::new(
                relpath,
                line_no,
                first_col(line, full),
                "css",
                "css_class",
                full,
                line.trim(),
                "extras",
            ));
        }

        for caps in id_re.captures_iter(line) {
            let full = caps.get(0).unwrap().as_str();
            records.push(IndexRecord::new(
                relpath,
                line_no,
                first_col(line, full),
                "css",
                "css_id",
                full,
                line.trim(),
                "extras",
            ));
        }

        for caps in var_re.captures_iter(line) {
            let name = caps.get(1).unwrap().as_str();
            records.push(IndexRecord::new(
                relpath,
                line_no,
                first_col(line, name),
                "css",
                "css_variable",
                name,
                line.trim(),
                "extras",
            ));
        }

        for caps in keyframes_re.captures_iter(line) {
            let name = caps.get(1).unwrap().as_str();
            records.push(IndexRecord::new(
                relpath,
                line_no,
                first_col(line, name),
                "css",
                "keyframes",
                name,
                line.trim(),
                "extras",
            ));
        }
    }

    records
}

fn index_html(relpath: &str, text: &str) -> Vec<IndexRecord> {
    let id_re = Regex::new(r#"id=["']([^"']+)["']"#).unwrap();
    let class_re = Regex::new(r#"class=["']([^"']+)["']"#).unwrap();
    let data_re = Regex::new(r#"\b(data-[a-zA-Z0-9_-]+)(?:=["'][^"']*["'])?"#).unwrap();
    let tag_re = Regex::new(r"</?\s*([a-zA-Z][a-zA-Z0-9-]*)").unwrap();

    let mut records = Vec::new();

    for (idx, line) in text.lines().enumerate() {
        let line_no = idx + 1;

        for caps in id_re.captures_iter(line) {
            let raw = caps.get(1).unwrap().as_str();

            for id in raw.split_whitespace() {
                records.push(IndexRecord::new(
                    relpath,
                    line_no,
                    first_col(line, id),
                    "html",
                    "html_id",
                    format!("#{id}"),
                    line.trim(),
                    "extras",
                ));
            }
        }

        for caps in class_re.captures_iter(line) {
            let raw = caps.get(1).unwrap().as_str();

            for class_name in raw.split_whitespace() {
                records.push(IndexRecord::new(
                    relpath,
                    line_no,
                    first_col(line, class_name),
                    "html",
                    "html_class",
                    format!(".{class_name}"),
                    line.trim(),
                    "extras",
                ));
            }
        }

        for caps in data_re.captures_iter(line) {
            let name = caps.get(1).unwrap().as_str();

            records.push(IndexRecord::new(
                relpath,
                line_no,
                first_col(line, name),
                "html",
                "data_attribute",
                name,
                line.trim(),
                "extras",
            ));
        }

        for caps in tag_re.captures_iter(line) {
            let name = caps.get(1).unwrap().as_str();

            if is_landmark_tag(name) {
                records.push(IndexRecord::new(
                    relpath,
                    line_no,
                    first_col(line, name),
                    "html",
                    "html_tag",
                    name,
                    line.trim(),
                    "extras",
                ));
            }
        }
    }

    records
}

fn index_markdown(relpath: &str, text: &str) -> Vec<IndexRecord> {
    let heading_re = Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();
    let checklist_re = Regex::new(r"^\s*[-*]\s+\[[ xX]\]\s+(.+)$").unwrap();
    let link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    let fence_re = Regex::new(r"^```([A-Za-z0-9_-]+)?").unwrap();

    let mut records = Vec::new();

    for (idx, line) in text.lines().enumerate() {
        let line_no = idx + 1;

        if let Some(caps) = heading_re.captures(line) {
            let level = caps.get(1).unwrap().as_str().len();
            let name = caps.get(2).unwrap().as_str().trim();

            records.push(IndexRecord::new(
                relpath,
                line_no,
                1,
                "md",
                format!("heading_{level}"),
                name,
                line.trim(),
                "extras",
            ));
        }

        if let Some(caps) = checklist_re.captures(line) {
            let name = caps.get(1).unwrap().as_str().trim();

            records.push(IndexRecord::new(
                relpath,
                line_no,
                first_col(line, name),
                "md",
                "checklist",
                name,
                line.trim(),
                "extras",
            ));
        }

        for caps in link_re.captures_iter(line) {
            let name = caps.get(1).unwrap().as_str().trim();
            let target = caps.get(2).unwrap().as_str().trim();

            records.push(IndexRecord::new(
                relpath,
                line_no,
                first_col(line, name),
                "md",
                "link",
                name,
                target,
                "extras",
            ));
        }

        if let Some(caps) = fence_re.captures(line) {
            let name = caps
                .get(1)
                .map(|value| value.as_str())
                .unwrap_or("code")
                .trim();

            records.push(IndexRecord::new(
                relpath,
                line_no,
                1,
                "md",
                "code_fence",
                name,
                line.trim(),
                "extras",
            ));
        }
    }

    records
}

fn index_jsx_usage(relpath: &str, lang: &str, text: &str) -> Vec<IndexRecord> {
    let component_tag_re = Regex::new(r"</?\s*([A-Z][A-Za-z0-9_.]*)").unwrap();
    let class_name_re = Regex::new(r#"className=["']([^"']+)["']"#).unwrap();
    let data_re = Regex::new(r#"\b(data-[a-zA-Z0-9_-]+)(?:=["'][^"']*["'])?"#).unwrap();

    let mut records = Vec::new();

    for (idx, line) in text.lines().enumerate() {
        let line_no = idx + 1;

        for caps in component_tag_re.captures_iter(line) {
            let name = caps.get(1).unwrap().as_str();

            records.push(IndexRecord::new(
                relpath,
                line_no,
                first_col(line, name),
                lang,
                "jsx_component_usage",
                name,
                line.trim(),
                "extras",
            ));
        }

        for caps in class_name_re.captures_iter(line) {
            let raw = caps.get(1).unwrap().as_str();

            for class_name in raw.split_whitespace() {
                records.push(IndexRecord::new(
                    relpath,
                    line_no,
                    first_col(line, class_name),
                    lang,
                    "jsx_class",
                    format!(".{class_name}"),
                    line.trim(),
                    "extras",
                ));
            }
        }

        for caps in data_re.captures_iter(line) {
            let name = caps.get(1).unwrap().as_str();

            records.push(IndexRecord::new(
                relpath,
                line_no,
                first_col(line, name),
                lang,
                "data_attribute",
                name,
                line.trim(),
                "extras",
            ));
        }
    }

    records
}

fn first_col(line: &str, needle: &str) -> usize {
    line.find(needle).map(|idx| idx + 1).unwrap_or(1)
}

fn language_from_path(path: &str) -> String {
    match path
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "py" => "py".to_string(),
        "ts" => "ts".to_string(),
        "tsx" => "tsx".to_string(),
        "js" => "js".to_string(),
        "jsx" => "jsx".to_string(),
        "css" => "css".to_string(),
        "html" | "htm" => "html".to_string(),
        "md" => "md".to_string(),
        "mdx" => "mdx".to_string(),
        other if !other.is_empty() => other.to_string(),
        _ => "text".to_string(),
    }
}

fn is_landmark_tag(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "html"
            | "head"
            | "body"
            | "main"
            | "header"
            | "footer"
            | "nav"
            | "section"
            | "article"
            | "aside"
            | "form"
            | "button"
            | "input"
            | "textarea"
            | "select"
            | "dialog"
            | "template"
    )
}
