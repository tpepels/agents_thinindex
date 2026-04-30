use std::path::Path;

use thinindex::support::{SupportBackend, SupportLevel, support_level_definitions, support_matrix};

#[test]
fn support_levels_are_exactly_the_documented_policy_set() {
    let levels = SupportLevel::ALL
        .iter()
        .map(|level| level.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        levels,
        vec!["supported", "experimental", "blocked", "extras-backed"]
    );

    let definitions = support_level_definitions()
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>();
    assert_eq!(definitions, levels);
}

#[test]
fn support_matrix_entries_have_required_policy_fields() {
    let matrix = support_matrix();
    assert!(!matrix.is_empty(), "support matrix must not be empty");

    for entry in matrix {
        assert!(!entry.name.is_empty(), "entry missing name: {entry:?}");
        assert!(
            !entry.extensions.is_empty(),
            "{} should declare extensions",
            entry.name
        );
        assert!(
            !entry.known_gaps.is_empty(),
            "{} should document known gaps or blocker reason",
            entry.name
        );
        assert!(
            !entry.license_status.is_empty(),
            "{} should document license status",
            entry.name
        );

        match entry.support_level {
            SupportLevel::Supported | SupportLevel::Experimental => {
                assert_eq!(
                    entry.backend,
                    SupportBackend::TreeSitter,
                    "{} should use Tree-sitter when code-language backed",
                    entry.name
                );
                assert!(
                    entry.language_id.is_some(),
                    "{} should declare language id",
                    entry.name
                );
                assert!(
                    entry.grammar_package.is_some(),
                    "{} should declare grammar package",
                    entry.name
                );
                assert!(
                    entry.conformance_fixture_repo.is_some()
                        && entry.conformance_fixture_path.is_some(),
                    "{} should declare conformance fixture",
                    entry.name
                );
                assert!(
                    !entry.record_kinds.is_empty(),
                    "{} should declare record kinds",
                    entry.name
                );
            }
            SupportLevel::ExtrasBacked => {
                assert_eq!(entry.backend, SupportBackend::Extras);
                assert!(
                    entry.grammar_package.is_none(),
                    "{} extras-backed entry must not claim grammar package",
                    entry.name
                );
                assert!(
                    entry.license_status.contains("project-owned extras"),
                    "{} extras-backed entry should document project-owned status",
                    entry.name
                );
            }
            SupportLevel::Blocked => {
                assert_eq!(entry.backend, SupportBackend::None);
                assert!(
                    entry.grammar_package.is_none()
                        && entry.conformance_fixture_repo.is_none()
                        && entry.conformance_fixture_path.is_none()
                        && entry.record_kinds.is_empty(),
                    "{} blocked entry must not claim parser support",
                    entry.name
                );
            }
        }
    }
}

#[test]
fn supported_languages_have_conformance_fixtures_and_license_notices() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let notices = repo_file("THIRD_PARTY_NOTICES");

    for entry in support_matrix()
        .iter()
        .filter(|entry| entry.support_level == SupportLevel::Supported)
    {
        let fixture = root
            .join("tests")
            .join("fixtures")
            .join(entry.conformance_fixture_repo.expect("fixture repo"))
            .join(entry.conformance_fixture_path.expect("fixture path"));
        assert!(
            fixture.exists(),
            "{} supported entry should have fixture {}",
            entry.name,
            fixture.display()
        );

        let grammar_package = entry.grammar_package.expect("grammar package");
        assert!(
            notices.contains(grammar_package) && notices.contains("License expression: MIT"),
            "{} supported entry should have MIT grammar notice for {}",
            entry.name,
            grammar_package
        );
    }
}

#[test]
fn extras_backed_formats_have_fixtures_and_project_owned_notice() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let notices = repo_file("THIRD_PARTY_NOTICES");

    assert!(
        notices.contains("Project-owned extras extractors")
            && notices.contains("Third-party parser dependency: none")
    );

    for entry in support_matrix()
        .iter()
        .filter(|entry| entry.support_level == SupportLevel::ExtrasBacked)
    {
        let fixture = root
            .join("tests")
            .join("fixtures")
            .join(entry.conformance_fixture_repo.expect("fixture repo"))
            .join(entry.conformance_fixture_path.expect("fixture path"));
        assert!(
            fixture.exists(),
            "{} extras-backed entry should have fixture {}",
            entry.name,
            fixture.display()
        );
    }
}

#[test]
fn docs_use_support_levels_without_overclaiming_experimental_or_blocked_entries() {
    let readme = repo_file("README.md");
    let parser_support = repo_file("docs/PARSER_SUPPORT.md");
    let product_boundary = repo_file("docs/PRODUCT_BOUNDARY.md");
    let quality = repo_file("docs/QUALITY.md");

    for (level, _definition) in support_level_definitions() {
        assert!(
            readme.contains(level)
                && parser_support.contains(level)
                && product_boundary.contains(level)
                && quality.contains(level),
            "docs should mention support level `{level}`"
        );
    }

    for entry in support_matrix() {
        assert_doc_row_matches_entry(&readme, entry);
        assert_doc_row_matches_entry(&parser_support, entry);

        if matches!(
            entry.support_level,
            SupportLevel::Experimental | SupportLevel::Blocked
        ) {
            for line in table_lines_for(&readme, entry.name)
                .into_iter()
                .chain(table_lines_for(&parser_support, entry.name))
            {
                assert!(
                    !line.contains("| supported |"),
                    "{} must not be claimed as supported in docs line `{line}`",
                    entry.name
                );
            }
        }

        if entry.support_level == SupportLevel::ExtrasBacked {
            for line in table_lines_for(&readme, entry.name)
                .into_iter()
                .chain(table_lines_for(&parser_support, entry.name))
            {
                assert!(
                    line.contains("| extras-backed |") && line.contains("| extras |"),
                    "{} must be marked extras-backed/extras in `{line}`",
                    entry.name
                );
                assert!(
                    !line.contains("| tree_sitter |"),
                    "{} must not be described as Tree-sitter-backed in `{line}`",
                    entry.name
                );
            }
        }
    }
}

#[test]
fn blocked_entries_are_visible_but_have_no_backend_claim() {
    let readme = repo_file("README.md");
    let parser_support = repo_file("docs/PARSER_SUPPORT.md");

    for entry in support_matrix()
        .iter()
        .filter(|entry| entry.support_level == SupportLevel::Blocked)
    {
        for line in table_lines_for(&readme, entry.name)
            .into_iter()
            .chain(table_lines_for(&parser_support, entry.name))
        {
            assert!(
                line.contains("| blocked |") && line.contains("| none |"),
                "{} blocked entry should stay visible with no backend claim: `{line}`",
                entry.name
            );
        }
    }
}

fn assert_doc_row_matches_entry(contents: &str, entry: &thinindex::support::SupportEntry) {
    let lines = table_lines_for(contents, entry.name);
    assert!(
        !lines.is_empty(),
        "docs should include support matrix row for {}",
        entry.name
    );

    let expected_level = format!("| {} |", entry.support_level.as_str());
    let expected_backend = format!("| {} |", entry.backend.as_str());
    assert!(
        lines
            .iter()
            .any(|line| line.contains(&expected_level) && line.contains(&expected_backend)),
        "{} docs row should include level {} and backend {}; rows: {lines:?}",
        entry.name,
        entry.support_level.as_str(),
        entry.backend.as_str()
    );
}

fn table_lines_for<'a>(contents: &'a str, name: &str) -> Vec<&'a str> {
    let prefix = format!("| {name} |");
    contents
        .lines()
        .filter(|line| line.starts_with(&prefix))
        .collect()
}

fn repo_file(path: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}
