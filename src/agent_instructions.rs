pub const REPOSITORY_SEARCH_HEADING: &str = "## Repository search";
pub const REPOSITORY_SEARCH_BLOCK: &str = "## Repository search

- Use `wi <term>` before grep/find/ls/Read to locate code; `wi` auto-builds or auto-rebuilds a missing/stale index once before searching.
- Run `wi --help` if you need search filters, examples, or subcommands.
- For implementation work, prefer `wi pack <term>` to get a compact read set.
- Before editing a symbol or feature area, run `wi impact <term>` to find related tests/docs/callers.
- Read only files returned by `wi` unless the result is insufficient.
- Run `build_index` manually only when you want an explicit rebuild or when `wi` reports that auto-build failed.
- Fall back to grep/find/Read only after that retry fails.";

pub fn normalize_repository_search_block(existing: &str, empty_base_prefix: &str) -> String {
    let without_sections = remove_repository_search_sections(existing);
    let without_legacy_markers = remove_legacy_repository_search_lines(&without_sections);
    let base = without_legacy_markers.trim_end();

    if base.is_empty() {
        format!("{empty_base_prefix}{REPOSITORY_SEARCH_BLOCK}\n")
    } else {
        format!("{base}\n\n{REPOSITORY_SEARCH_BLOCK}\n")
    }
}

pub fn repository_search_block_is_current(existing: &str, empty_base_prefix: &str) -> bool {
    normalize_repository_search_block(existing, empty_base_prefix) == existing
}

fn remove_repository_search_sections(existing: &str) -> String {
    let mut kept = Vec::new();
    let mut lines = existing.lines().peekable();

    while let Some(line) = lines.next() {
        if line.trim() == REPOSITORY_SEARCH_HEADING {
            while let Some(next_line) = lines.peek() {
                if is_markdown_h1_or_h2(next_line) {
                    break;
                }

                lines.next();
            }
        } else {
            kept.push(line);
        }
    }

    kept.join("\n")
}

fn remove_legacy_repository_search_lines(existing: &str) -> String {
    existing
        .lines()
        .filter(|line| !is_legacy_repository_search_line(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_legacy_repository_search_line(line: &str) -> bool {
    let trimmed = line.trim();

    trimmed == "@WI.md"
        || trimmed == "@AGENTS.md"
        || trimmed.contains("See WI.md for repository search/index usage.")
        || trimmed.contains("See `WI.md` for repository search/index usage.")
        || trimmed.starts_with("<!-- thinindex-repo-search-block:")
        || trimmed
            .contains("Before broad repository discovery, run `build_index`, then use `wi <term>`")
        || trimmed.contains("Run `wi --help` before your first repository search")
        || trimmed.contains("If `wi` misses a name you expect to exist")
        || trimmed.contains("If results look stale, run `build_index`.")
        || trimmed.contains("Before broad repository discovery, run `build_index`.")
        || trimmed.contains("If `wi` returns no useful result, rerun `build_index` once and retry.")
}

fn is_markdown_h1_or_h2(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("# ") || trimmed.starts_with("## ")
}

#[cfg(test)]
mod tests {
    use super::{REPOSITORY_SEARCH_BLOCK, normalize_repository_search_block};

    #[test]
    fn current_block_is_stable() {
        let existing = format!("# AGENTS\n\n{REPOSITORY_SEARCH_BLOCK}\n");
        assert_eq!(
            normalize_repository_search_block(&existing, "# AGENTS\n\n"),
            existing
        );
    }

    #[test]
    fn legacy_markers_are_removed() {
        let normalized = normalize_repository_search_block("@WI.md\n", "# AGENTS\n\n");
        assert!(!normalized.contains("@WI.md"));
        assert!(normalized.contains(REPOSITORY_SEARCH_BLOCK));
    }
}
