mod common;

use std::{collections::BTreeSet, path::Path};

use common::{
    fixture_repo, load_index_snapshot_from_sqlite, run_build, run_named_index_integrity_checks,
    run_wi,
};
use thinindex::model::IndexRecord;

const EXTRAS_SOURCE: &str = "extras";

#[derive(Debug, Clone)]
struct ExpectedFormatRecord {
    kind: &'static str,
    name: &'static str,
    line: usize,
    col: usize,
}

#[derive(Debug, Clone)]
struct FormatSearchTerm {
    term: &'static str,
    kind: &'static str,
}

#[derive(Debug, Clone)]
struct FormatConformanceCase {
    format_name: &'static str,
    fixture_name: &'static str,
    fixture_path: &'static str,
    extensions: &'static [&'static str],
    backing: &'static str,
    expected_kinds: &'static [&'static str],
    expected_records: &'static [ExpectedFormatRecord],
    expected_absent_symbols: &'static [&'static str],
    search_terms: &'static [FormatSearchTerm],
    unsupported_notes: &'static str,
}

fn format_cases() -> Vec<FormatConformanceCase> {
    vec![
        FormatConformanceCase {
            format_name: "CSS",
            fixture_name: "sample_repo",
            fixture_path: "frontend/styles/header.css",
            extensions: &["css"],
            backing: "project-owned extras",
            expected_kinds: &["css_class", "css_id", "css_variable", "keyframes"],
            expected_records: &[
                ExpectedFormatRecord {
                    kind: "css_variable",
                    name: "--paper-bg",
                    line: 2,
                    col: 3,
                },
                ExpectedFormatRecord {
                    kind: "css_class",
                    name: ".headerNavigation",
                    line: 5,
                    col: 1,
                },
                ExpectedFormatRecord {
                    kind: "keyframes",
                    name: "fadeIn",
                    line: 13,
                    col: 12,
                },
            ],
            expected_absent_symbols: &[],
            search_terms: &[
                FormatSearchTerm {
                    term: ".headerNavigation",
                    kind: "css_class",
                },
                FormatSearchTerm {
                    term: "fadeIn",
                    kind: "keyframes",
                },
            ],
            unsupported_notes: "CSS extras extract selectors and keyframes, not full CSS semantics or cascade analysis.",
        },
        FormatConformanceCase {
            format_name: "HTML",
            fixture_name: "html_repo",
            fixture_path: "templates/base.html",
            extensions: &["html"],
            backing: "project-owned extras",
            expected_kinds: &["data_attribute", "html_class", "html_id", "html_tag"],
            expected_records: &[
                ExpectedFormatRecord {
                    kind: "html_tag",
                    name: "header",
                    line: 4,
                    col: 6,
                },
                ExpectedFormatRecord {
                    kind: "html_id",
                    name: "#mainHeader",
                    line: 4,
                    col: 17,
                },
            ],
            expected_absent_symbols: &[],
            search_terms: &[
                FormatSearchTerm {
                    term: "#mainHeader",
                    kind: "html_id",
                },
                FormatSearchTerm {
                    term: "data-testid",
                    kind: "data_attribute",
                },
            ],
            unsupported_notes: "HTML extras extract tag and attribute landmarks, not DOM or browser semantics.",
        },
        FormatConformanceCase {
            format_name: "Markdown",
            fixture_name: "sample_repo",
            fixture_path: "docs/guide.md",
            extensions: &["md", "markdown"],
            backing: "project-owned extras",
            expected_kinds: &["checklist", "link", "section"],
            expected_records: &[
                ExpectedFormatRecord {
                    kind: "section",
                    name: "Guide",
                    line: 1,
                    col: 3,
                },
                ExpectedFormatRecord {
                    kind: "checklist",
                    name: "Add parser integration",
                    line: 3,
                    col: 7,
                },
            ],
            expected_absent_symbols: &[],
            search_terms: &[
                FormatSearchTerm {
                    term: "Guide",
                    kind: "section",
                },
                FormatSearchTerm {
                    term: "README",
                    kind: "link",
                },
            ],
            unsupported_notes: "Markdown extras extract headings, checklist items, and links, not a full Markdown AST.",
        },
        FormatConformanceCase {
            format_name: "JSON",
            fixture_name: "sample_repo",
            fixture_path: "config/app.json",
            extensions: &["json"],
            backing: "project-owned extras",
            expected_kinds: &["key"],
            expected_records: &[ExpectedFormatRecord {
                kind: "key",
                name: "parserConfigEnabled",
                line: 4,
                col: 6,
            }],
            expected_absent_symbols: &["JsonStringFake"],
            search_terms: &[FormatSearchTerm {
                term: "parserConfigEnabled",
                kind: "key",
            }],
            unsupported_notes: "JSON extras record useful object keys and skip scalar values.",
        },
        FormatConformanceCase {
            format_name: "TOML",
            fixture_name: "sample_repo",
            fixture_path: "config/thinindex.toml",
            extensions: &["toml"],
            backing: "project-owned extras",
            expected_kinds: &["key", "table"],
            expected_records: &[
                ExpectedFormatRecord {
                    kind: "table",
                    name: "tool.thinindex",
                    line: 1,
                    col: 2,
                },
                ExpectedFormatRecord {
                    kind: "key",
                    name: "parser_config_enabled",
                    line: 3,
                    col: 1,
                },
            ],
            expected_absent_symbols: &["TomlStringFake"],
            search_terms: &[
                FormatSearchTerm {
                    term: "tool.thinindex",
                    kind: "table",
                },
                FormatSearchTerm {
                    term: "parser_config_enabled",
                    kind: "key",
                },
            ],
            unsupported_notes: "TOML extras record keys and tables and skip scalar values.",
        },
        FormatConformanceCase {
            format_name: "YAML",
            fixture_name: "sample_repo",
            fixture_path: "config/pipeline.yaml",
            extensions: &["yaml", "yml"],
            backing: "project-owned extras",
            expected_kinds: &["key", "section"],
            expected_records: &[
                ExpectedFormatRecord {
                    kind: "section",
                    name: "pipeline",
                    line: 1,
                    col: 1,
                },
                ExpectedFormatRecord {
                    kind: "key",
                    name: "name",
                    line: 2,
                    col: 3,
                },
            ],
            expected_absent_symbols: &["YamlStringFake"],
            search_terms: &[
                FormatSearchTerm {
                    term: "pipeline",
                    kind: "section",
                },
                FormatSearchTerm {
                    term: "name",
                    kind: "key",
                },
            ],
            unsupported_notes: "YAML extras record mapping keys and sections and skip scalar values.",
        },
    ]
}

#[test]
fn extras_backed_format_fixtures_pass_shared_conformance() {
    for case in format_cases() {
        let repo = fixture_repo(case.fixture_name);
        let root = repo.path();

        run_build(root);

        let snapshot = load_index_snapshot_from_sqlite(root);
        run_named_index_integrity_checks(case.format_name, &snapshot, &[case.fixture_path]);
        assert_extension_is_declared(&case);
        assert_case_records(&case, &snapshot.records);
        assert_case_absent_symbols(&case, &snapshot.records);
        assert_case_search_terms(&case, root);
    }
}

#[test]
fn extras_support_matrix_and_notices_cover_supported_formats() {
    let readme = repo_file("README.md");
    let product_boundary = repo_file("docs/PRODUCT_BOUNDARY.md");
    let notices = repo_file("THIRD_PARTY_NOTICES");

    for case in format_cases() {
        assert!(
            readme.contains(case.format_name),
            "README support matrix should mention {}",
            case.format_name,
        );
        assert!(
            readme.contains(case.backing),
            "README support matrix should identify {} backing as {}",
            case.format_name,
            case.backing,
        );
        assert!(
            !case.unsupported_notes.is_empty(),
            "case should document unsupported notes for {}",
            case.format_name,
        );
    }

    assert!(
        readme.contains("Languages and formats not listed are unsupported")
            && product_boundary.contains("Formats and languages not listed"),
        "docs should state that unlisted languages/formats are unsupported"
    );
    assert!(
        notices.contains("Project-owned extras extractors")
            && notices.contains("Third-party parser dependency: none"),
        "THIRD_PARTY_NOTICES should document extras-backed formats without fake grammar notices"
    );
    assert!(
        !notices.contains("License expression: GPL")
            && !notices.contains("License expression: AGPL"),
        "parser notices must not include GPL/AGPL parser dependencies"
    );
}

fn repo_file(path: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn assert_extension_is_declared(case: &FormatConformanceCase) {
    let extension = Path::new(case.fixture_path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    assert!(
        case.extensions.contains(&extension),
        "{} fixture extension `{extension}` should be listed in {:?}",
        case.format_name,
        case.extensions,
    );
}

fn assert_case_records(case: &FormatConformanceCase, records: &[IndexRecord]) {
    let file_records: Vec<&IndexRecord> = records
        .iter()
        .filter(|record| record.path == case.fixture_path && record.source == EXTRAS_SOURCE)
        .collect();

    assert!(
        !file_records.is_empty(),
        "{} should emit extras records for {}",
        case.format_name,
        case.fixture_path,
    );

    let actual_kinds: BTreeSet<&str> = file_records
        .iter()
        .map(|record| record.kind.as_str())
        .collect();

    for kind in case.expected_kinds {
        assert!(
            actual_kinds.contains(kind),
            "{} expected kind `{kind}` in {}, got {:?}",
            case.format_name,
            case.fixture_path,
            actual_kinds,
        );
    }

    for expected in case.expected_records {
        assert!(
            file_records.iter().any(|record| {
                record.kind == expected.kind
                    && record.name == expected.name
                    && record.line == expected.line
                    && record.col == expected.col
            }),
            "{} missing expected record {:?} in {}.\nrecords:\n{file_records:#?}",
            case.format_name,
            expected,
            case.fixture_path,
        );
    }
}

fn assert_case_absent_symbols(case: &FormatConformanceCase, records: &[IndexRecord]) {
    for absent in case.expected_absent_symbols {
        assert!(
            !records
                .iter()
                .any(|record| record.path == case.fixture_path && record.name == *absent),
            "{} scalar/comment symbol `{absent}` should not be indexed from {}",
            case.format_name,
            case.fixture_path,
        );
    }
}

fn assert_case_search_terms(case: &FormatConformanceCase, root: &Path) {
    for search in case.search_terms {
        let output = run_wi(root, &[search.term, "-t", search.kind]);
        assert!(
            output.contains(case.fixture_path) && output.contains(search.term),
            "{} expected wi to find `{}` as `{}`, got:\n{output}",
            case.format_name,
            search.term,
            search.kind,
        );
    }
}
