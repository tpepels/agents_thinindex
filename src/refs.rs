use crate::model::ReferenceRecord;

const REF_SOURCE: &str = "extras";

#[derive(Debug, Clone, Copy)]
struct LineContext<'a> {
    path: &'a str,
    line_no: usize,
    text: &'a str,
}

pub fn extract_refs(path: &str, text: &str) -> Vec<ReferenceRecord> {
    if !matches!(lang_from_path(path), "md" | "markdown") {
        return Vec::new();
    }

    let mut refs = Vec::new();

    for (line_index, line) in text.lines().enumerate() {
        let ctx = LineContext {
            path,
            line_no: line_index + 1,
            text: line,
        };

        extract_markdown_links(&ctx, &mut refs);
    }

    refs
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
) {
    refs.push(ReferenceRecord::new(
        ctx.path,
        ctx.line_no,
        col,
        to_name,
        Some("link"),
        "markdown_link",
        ctx.text.trim(),
        REF_SOURCE,
    ));
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
            push_ref(refs, ctx, target_start + 1, target);
        }

        search_start = target_close + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_markdown_link_reference() {
        let refs = extract_refs("docs/guide.md", "[Guide](../src/service.py)\n");

        assert!(refs.iter().any(|r| {
            r.to_name == "../src/service.py"
                && r.ref_kind == "markdown_link"
                && r.to_kind.as_deref() == Some("link")
                && r.source == "extras"
        }));
    }

    #[test]
    fn ignores_non_markdown_files() {
        let refs = extract_refs("src/service.py", "from service import PromptService\n");

        assert!(refs.is_empty());
    }
}
