mod common;

use std::{collections::BTreeSet, path::Path};

use common::{
    fixture_repo, load_index_snapshot_from_sqlite, run_build, run_named_index_integrity_checks,
    run_wi,
};
use thinindex::{
    model::IndexRecord,
    support::{SupportBackend, support_matrix},
    tree_sitter_extraction::{
        ALLOWED_DEFINITION_CAPTURE_KINDS, ALLOWED_NORMALIZED_CAPTURE_KINDS, LanguageRegistry,
        QUERY_DEFINITION_CAPTURE_PREFIX, QUERY_INTERNAL_CAPTURE_PREFIX, QUERY_NAME_CAPTURE,
        TREE_SITTER_SOURCE, validate_query_specs,
    },
};

#[derive(Debug, Clone)]
struct ExpectedSymbol {
    kind: &'static str,
    name: &'static str,
    line: usize,
    col: usize,
}

#[derive(Debug, Clone)]
struct LanguageConformanceCase {
    language_name: &'static str,
    language_id: &'static str,
    fixture_path: &'static str,
    extensions: &'static [&'static str],
    grammar_package: &'static str,
    expected_kinds: &'static [&'static str],
    expected_symbols: &'static [ExpectedSymbol],
    expected_absent_symbols: &'static [&'static str],
    search_terms: &'static [&'static str],
    unsupported_notes: &'static str,
}

fn language_cases() -> Vec<LanguageConformanceCase> {
    vec![
        LanguageConformanceCase {
            language_name: "Rust",
            language_id: "rs",
            fixture_path: "src/rust/widget.rs",
            extensions: &["rs"],
            grammar_package: "tree-sitter-rust",
            expected_kinds: &["constant", "enum", "function", "struct", "trait"],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "struct",
                    name: "RustWidget",
                    line: 5,
                    col: 12,
                },
                ExpectedSymbol {
                    kind: "function",
                    name: "rust_entrypoint",
                    line: 25,
                    col: 8,
                },
            ],
            expected_absent_symbols: &["RustStringFake"],
            search_terms: &["RustWidget", "rust_entrypoint"],
            unsupported_notes: "Rust import/use records are deferred to the deterministic refs extractor.",
        },
        LanguageConformanceCase {
            language_name: "Python",
            language_id: "py",
            fixture_path: "src/python/widget.py",
            extensions: &["py"],
            grammar_package: "tree-sitter-python",
            expected_kinds: &["class", "function", "import", "variable"],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "class",
                    name: "PythonWidget",
                    line: 6,
                    col: 7,
                },
                ExpectedSymbol {
                    kind: "function",
                    name: "build_python_widget",
                    line: 13,
                    col: 5,
                },
            ],
            expected_absent_symbols: &["PythonCommentFake", "PythonStringFake"],
            search_terms: &["PythonWidget", "build_python_widget"],
            unsupported_notes: "Python decorators and full assignment target forms are not semantic analysis.",
        },
        LanguageConformanceCase {
            language_name: "JavaScript",
            language_id: "js",
            fixture_path: "src/javascript/widget.js",
            extensions: &["js"],
            grammar_package: "tree-sitter-javascript",
            expected_kinds: &[
                "class", "export", "function", "import", "method", "variable",
            ],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "class",
                    name: "JavaScriptWidget",
                    line: 5,
                    col: 7,
                },
                ExpectedSymbol {
                    kind: "function",
                    name: "buildJavaScriptWidget",
                    line: 11,
                    col: 10,
                },
            ],
            expected_absent_symbols: &["JavaScriptStringFake"],
            search_terms: &["JavaScriptWidget", "buildJavaScriptWidget"],
            unsupported_notes: "JavaScript extraction does not attempt runtime module or prototype semantics.",
        },
        LanguageConformanceCase {
            language_name: "JSX",
            language_id: "jsx",
            fixture_path: "src/javascript/widget.jsx",
            extensions: &["jsx"],
            grammar_package: "tree-sitter-javascript",
            expected_kinds: &["export", "function", "import"],
            expected_symbols: &[ExpectedSymbol {
                kind: "function",
                name: "JsxPanel",
                line: 3,
                col: 10,
            }],
            expected_absent_symbols: &[],
            search_terms: &["JsxPanel", "useJsxPanel"],
            unsupported_notes: "JSX element usage remains extras-backed; definitions are Tree-sitter-backed.",
        },
        LanguageConformanceCase {
            language_name: "TypeScript",
            language_id: "ts",
            fixture_path: "src/typescript/widget.ts",
            extensions: &["ts"],
            grammar_package: "tree-sitter-typescript",
            expected_kinds: &[
                "class",
                "export",
                "function",
                "import",
                "interface",
                "method",
                "type",
                "variable",
            ],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "interface",
                    name: "TypeScriptRenderable",
                    line: 3,
                    col: 18,
                },
                ExpectedSymbol {
                    kind: "variable",
                    name: "TS_LIMIT",
                    line: 9,
                    col: 7,
                },
            ],
            expected_absent_symbols: &["TypeScriptStringFake"],
            search_terms: &["TypeScriptRenderable", "TypeScriptFactory"],
            unsupported_notes: "TypeScript extraction is syntactic and does not resolve type aliases.",
        },
        LanguageConformanceCase {
            language_name: "TSX",
            language_id: "tsx",
            fixture_path: "src/typescript/widget.tsx",
            extensions: &["tsx"],
            grammar_package: "tree-sitter-typescript",
            expected_kinds: &["export", "function", "import"],
            expected_symbols: &[ExpectedSymbol {
                kind: "function",
                name: "TsxPanel",
                line: 3,
                col: 10,
            }],
            expected_absent_symbols: &[],
            search_terms: &["TsxPanel", "useTsxPanel"],
            unsupported_notes: "TSX element usage remains extras-backed; definitions are Tree-sitter-backed.",
        },
        LanguageConformanceCase {
            language_name: "Java",
            language_id: "java",
            fixture_path: "src/java/JavaWidget.java",
            extensions: &["java"],
            grammar_package: "tree-sitter-java",
            expected_kinds: &[
                "class",
                "enum",
                "import",
                "interface",
                "method",
                "type",
                "variable",
            ],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "class",
                    name: "JavaWidget",
                    line: 5,
                    col: 7,
                },
                ExpectedSymbol {
                    kind: "method",
                    name: "render",
                    line: 11,
                    col: 10,
                },
            ],
            expected_absent_symbols: &["JavaStringFake"],
            search_terms: &["JavaWidget", "JavaRecord"],
            unsupported_notes: "Java extraction does not resolve package visibility or inherited members.",
        },
        LanguageConformanceCase {
            language_name: "Go",
            language_id: "go",
            fixture_path: "src/go/widget.go",
            extensions: &["go"],
            grammar_package: "tree-sitter-go",
            expected_kinds: &[
                "constant",
                "function",
                "import",
                "interface",
                "method",
                "module",
                "struct",
                "type",
                "variable",
            ],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "function",
                    name: "NewGoWidget",
                    line: 19,
                    col: 6,
                },
                ExpectedSymbol {
                    kind: "type",
                    name: "GoID",
                    line: 17,
                    col: 6,
                },
            ],
            expected_absent_symbols: &["GoStringFake"],
            search_terms: &["NewGoWidget", "GoWidget"],
            unsupported_notes: "Go extraction does not resolve exported package API sets semantically.",
        },
        LanguageConformanceCase {
            language_name: "C",
            language_id: "c",
            fixture_path: "src/c/widget.c",
            extensions: &["c", "h"],
            grammar_package: "tree-sitter-c",
            expected_kinds: &["enum", "function", "import", "struct", "type", "variable"],
            expected_symbols: &[ExpectedSymbol {
                kind: "function",
                name: "build_c_widget",
                line: 13,
                col: 5,
            }],
            expected_absent_symbols: &["CStringFake"],
            search_terms: &["build_c_widget", "CWidget"],
            unsupported_notes: "C extraction is syntactic and does not expand macros.",
        },
        LanguageConformanceCase {
            language_name: "C++",
            language_id: "cpp",
            fixture_path: "src/cpp/widget.cpp",
            extensions: &["cc", "cpp", "cxx", "hh", "hpp", "hxx"],
            grammar_package: "tree-sitter-cpp",
            expected_kinds: &[
                "class", "enum", "function", "import", "method", "module", "struct", "variable",
            ],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "class",
                    name: "CppWidget",
                    line: 7,
                    col: 7,
                },
                ExpectedSymbol {
                    kind: "function",
                    name: "build_cpp_widget",
                    line: 22,
                    col: 5,
                },
            ],
            expected_absent_symbols: &["CppStringFake"],
            search_terms: &["CppWidget", "build_cpp_widget"],
            unsupported_notes: "C++ extraction is syntactic and does not instantiate templates.",
        },
        LanguageConformanceCase {
            language_name: "C#",
            language_id: "cs",
            fixture_path: "src/csharp/Widget.cs",
            extensions: &["cs"],
            grammar_package: "tree-sitter-c-sharp",
            expected_kinds: &[
                "class",
                "enum",
                "import",
                "interface",
                "method",
                "module",
                "struct",
                "type",
                "variable",
            ],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "class",
                    name: "CSharpWidget",
                    line: 22,
                    col: 18,
                },
                ExpectedSymbol {
                    kind: "method",
                    name: "Render",
                    line: 33,
                    col: 21,
                },
            ],
            expected_absent_symbols: &["CSharpStringFake"],
            search_terms: &["CSharpWidget", "Render"],
            unsupported_notes: "C# extraction is syntactic and does not resolve partial types or assemblies.",
        },
        LanguageConformanceCase {
            language_name: "Shell",
            language_id: "sh",
            fixture_path: "src/shell/widget.sh",
            extensions: &["bash", "sh"],
            grammar_package: "tree-sitter-bash",
            expected_kinds: &["function", "variable"],
            expected_symbols: &[ExpectedSymbol {
                kind: "function",
                name: "build_shell_widget",
                line: 5,
                col: 1,
            }],
            expected_absent_symbols: &["build_shell_string_fake"],
            search_terms: &["build_shell_widget", "render_shell_widget"],
            unsupported_notes: "Shell source/include semantics are deferred; functions and assignments are covered.",
        },
        LanguageConformanceCase {
            language_name: "Ruby",
            language_id: "rb",
            fixture_path: "src/ruby/widget.rb",
            extensions: &["rb"],
            grammar_package: "tree-sitter-ruby",
            expected_kinds: &["class", "constant", "method", "module"],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "class",
                    name: "RubyWidget",
                    line: 6,
                    col: 9,
                },
                ExpectedSymbol {
                    kind: "method",
                    name: "build_ruby_widget",
                    line: 12,
                    col: 14,
                },
            ],
            expected_absent_symbols: &["RubyStringFake"],
            search_terms: &["RubyWidget", "build_ruby_widget"],
            unsupported_notes: "Ruby require/load target extraction is deferred to reference extraction.",
        },
        LanguageConformanceCase {
            language_name: "Scala",
            language_id: "scala",
            fixture_path: "src/scala/Widget.scala",
            extensions: &["scala"],
            grammar_package: "tree-sitter-scala",
            expected_kinds: &[
                "class", "constant", "enum", "function", "import", "module", "trait", "type",
                "variable",
            ],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "class",
                    name: "ScalaWidget",
                    line: 13,
                    col: 7,
                },
                ExpectedSymbol {
                    kind: "function",
                    name: "buildScalaWidget",
                    line: 28,
                    col: 7,
                },
            ],
            expected_absent_symbols: &["ScalaStringFake"],
            search_terms: &["ScalaWidget", "buildScalaWidget"],
            unsupported_notes: "Scala extraction is syntactic and does not model implicits, givens, or extension resolution.",
        },
        LanguageConformanceCase {
            language_name: "PHP",
            language_id: "php",
            fixture_path: "src/php/widget.php",
            extensions: &["php"],
            grammar_package: "tree-sitter-php",
            expected_kinds: &[
                "class",
                "constant",
                "import",
                "interface",
                "method",
                "module",
                "trait",
                "variable",
            ],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "class",
                    name: "PhpWidget",
                    line: 22,
                    col: 7,
                },
                ExpectedSymbol {
                    kind: "method",
                    name: "buildPhpWidget",
                    line: 16,
                    col: 21,
                },
            ],
            expected_absent_symbols: &["PhpStringFake"],
            search_terms: &["PhpWidget", "buildPhpWidget"],
            unsupported_notes: "PHP extraction does not evaluate dynamic includes or autoloading.",
        },
        LanguageConformanceCase {
            language_name: "Kotlin",
            language_id: "kt",
            fixture_path: "src/kotlin/Widget.kt",
            extensions: &["kt", "kts"],
            grammar_package: "tree-sitter-kotlin-ng",
            expected_kinds: &[
                "class", "enum", "function", "import", "module", "type", "variable",
            ],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "class",
                    name: "KotlinWidget",
                    line: 11,
                    col: 7,
                },
                ExpectedSymbol {
                    kind: "function",
                    name: "buildKotlinWidget",
                    line: 25,
                    col: 9,
                },
            ],
            expected_absent_symbols: &["KotlinStringFake"],
            search_terms: &["KotlinWidget", "buildKotlinWidget"],
            unsupported_notes: "Kotlin interface/enum-class distinctions and extension resolution are not semantic analysis.",
        },
        LanguageConformanceCase {
            language_name: "Swift",
            language_id: "swift",
            fixture_path: "src/swift/Widget.swift",
            extensions: &["swift"],
            grammar_package: "tree-sitter-swift",
            expected_kinds: &[
                "class",
                "enum",
                "function",
                "import",
                "interface",
                "method",
                "struct",
                "type",
                "variable",
            ],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "class",
                    name: "SwiftWidget",
                    line: 19,
                    col: 7,
                },
                ExpectedSymbol {
                    kind: "function",
                    name: "buildSwiftWidget",
                    line: 37,
                    col: 6,
                },
            ],
            expected_absent_symbols: &["SwiftStringFake"],
            search_terms: &["SwiftWidget", "buildSwiftWidget"],
            unsupported_notes: "Swift extraction is syntactic and does not model extensions, overload sets, or module resolution.",
        },
        LanguageConformanceCase {
            language_name: "Dart",
            language_id: "dart",
            fixture_path: "src/dart/widget.dart",
            extensions: &["dart"],
            grammar_package: "tree-sitter-dart",
            expected_kinds: &[
                "class", "constant", "enum", "export", "function", "import", "method", "type",
                "variable",
            ],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "class",
                    name: "DartWidget",
                    line: 17,
                    col: 7,
                },
                ExpectedSymbol {
                    kind: "function",
                    name: "buildDartWidget",
                    line: 33,
                    col: 12,
                },
            ],
            expected_absent_symbols: &["DartStringFake"],
            search_terms: &["DartWidget", "buildDartWidget"],
            unsupported_notes: "Dart extraction is syntactic and does not resolve package imports, extensions, or type aliases semantically.",
        },
        LanguageConformanceCase {
            language_name: "Nix",
            language_id: "nix",
            fixture_path: "src/nix/default.nix",
            extensions: &["nix"],
            grammar_package: "tree-sitter-nix",
            expected_kinds: &["function", "import", "module"],
            expected_symbols: &[
                ExpectedSymbol {
                    kind: "import",
                    name: "importedModule",
                    line: 4,
                    col: 3,
                },
                ExpectedSymbol {
                    kind: "function",
                    name: "mkNixWidget",
                    line: 6,
                    col: 3,
                },
                ExpectedSymbol {
                    kind: "module",
                    name: "nixPackage",
                    line: 13,
                    col: 3,
                },
            ],
            expected_absent_symbols: &["NixStringFake"],
            search_terms: &["mkNixWidget", "nixPackage"],
            unsupported_notes: "Nix extraction intentionally records useful function, import-binding, and attrset symbols without indexing every scalar attribute.",
        },
    ]
}

#[test]
fn supported_language_fixtures_pass_shared_conformance() {
    let cases = language_cases();
    let repo = fixture_repo("language_pack");
    let root = repo.path();

    run_build(root);

    let snapshot = load_index_snapshot_from_sqlite(root);
    let expected_paths: Vec<&str> = cases.iter().map(|case| case.fixture_path).collect();
    run_named_index_integrity_checks(
        "parser conformance language pack fixture",
        &snapshot,
        &expected_paths,
    );

    for case in &cases {
        assert_extension_is_declared(case);
        assert_case_records(case, &snapshot.records);
        assert_case_absent_symbols(case, &snapshot.records);
        assert_case_search_terms(case, root);
    }
}

#[test]
fn parser_support_matrix_and_notices_cover_supported_languages() {
    let readme = repo_file("README.md");
    let notices = repo_file("THIRD_PARTY_NOTICES");

    for case in language_cases() {
        assert!(
            readme.contains(case.language_name),
            "README support matrix should mention {}",
            case.language_name,
        );
        assert!(
            readme.contains(case.grammar_package),
            "README support matrix should mention {}",
            case.grammar_package,
        );
        assert!(
            notices.contains(case.grammar_package) && notices.contains("License expression: MIT"),
            "THIRD_PARTY_NOTICES should list permissive notice for {}",
            case.grammar_package,
        );
        assert!(
            !case.unsupported_notes.is_empty(),
            "case should document unsupported notes for {}",
            case.language_name,
        );
    }

    assert!(
        readme.contains("not semantic or LSP-level analysis")
            && readme.contains("Languages and formats not listed are unsupported")
            && readme.contains("They are not silently parsed through line scanning"),
        "README must document parser limits honestly",
    );
}

#[test]
fn query_backed_registry_adapters_have_support_fixture_and_license_metadata() {
    let registry = LanguageRegistry::default();
    validate_query_specs(&registry).expect("registered query packs should validate");

    for adapter in registry.adapters() {
        assert!(
            !adapter.license.package.is_empty()
                && !adapter.license.upstream.is_empty()
                && !adapter.license.license.is_empty()
                && !adapter.license.accepted_reason.is_empty(),
            "{} should have complete grammar license metadata",
            adapter.display_name,
        );

        let entry = support_matrix()
            .iter()
            .find(|entry| {
                entry.backend == SupportBackend::TreeSitter && entry.language_id == Some(adapter.id)
            })
            .unwrap_or_else(|| {
                panic!(
                    "{} adapter `{}` should have a Tree-sitter support matrix entry",
                    adapter.display_name, adapter.id,
                )
            });

        assert_eq!(
            entry.grammar_package,
            Some(adapter.license.package),
            "{} support matrix grammar package should match adapter license metadata",
            adapter.display_name,
        );
        assert!(
            entry.conformance_fixture_repo.is_some() && entry.conformance_fixture_path.is_some(),
            "{} should declare a conformance fixture",
            adapter.display_name,
        );

        let support_extensions: BTreeSet<String> = entry
            .extensions
            .iter()
            .map(|extension| extension.trim_start_matches('.').to_string())
            .collect();
        for extension in adapter.extensions {
            assert!(
                support_extensions.contains(*extension),
                "{} adapter extension `{extension}` should be in support matrix {:?}",
                adapter.display_name,
                entry.extensions,
            );
        }
    }

    for entry in support_matrix()
        .iter()
        .filter(|entry| entry.backend == SupportBackend::TreeSitter)
    {
        let language_id = entry.language_id.expect("Tree-sitter language id");
        assert!(
            registry
                .adapters()
                .iter()
                .any(|adapter| adapter.id == language_id),
            "{} support entry should have a registered adapter",
            entry.name,
        );
    }
}

#[test]
fn parser_maintenance_guide_documents_query_guardrails() {
    let guide = repo_file("docs/PARSER_MAINTENANCE.md");

    for section in [
        "## Parser Architecture Overview",
        "## How LanguageRegistry Works",
        "## How Query Specs Work",
        "## Normalized Capture Names",
        "## Capture-To-Record Mapping Rules",
        "## How To Add A Language",
        "## How To Update A Language Query",
        "## How To Add Conformance Fixtures",
        "## How To Add Real-Repo Expected Symbols",
        "## How To Run Quality Gates",
        "## How To Handle Unsupported Syntax",
        "## How To Audit Grammar Licenses",
        "## What Not To Do",
    ] {
        assert!(guide.contains(section), "missing guide section {section}");
    }

    assert!(guide.contains(&format!("`@{QUERY_NAME_CAPTURE}`")));
    assert!(guide.contains(&format!("`@{QUERY_INTERNAL_CAPTURE_PREFIX}<purpose>`")));

    for kind in ALLOWED_DEFINITION_CAPTURE_KINDS {
        assert!(
            guide.contains(&format!("`@{QUERY_DEFINITION_CAPTURE_PREFIX}{kind}`")),
            "guide should document allowed definition capture {kind}",
        );
    }

    for kind in ALLOWED_NORMALIZED_CAPTURE_KINDS {
        assert!(
            guide.contains(&format!("`{kind}`")),
            "guide should document normalized kind {kind}",
        );
    }

    for forbidden in [
        "no line scanners for code symbols",
        "no hand parsers",
        "no broad regex parser",
        "no unsupported language support claims",
        "no grammar dependency without license entry",
    ] {
        assert!(
            guide.contains(forbidden),
            "guide should document forbidden pattern `{forbidden}`",
        );
    }

    let external_tagger_rule = format!("no {} parser fallback", ["c", "tags"].concat());
    assert!(
        guide.contains(&external_tagger_rule),
        "guide should document forbidden parser fallback"
    );
}

fn repo_file(path: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn assert_extension_is_declared(case: &LanguageConformanceCase) {
    let extension = Path::new(case.fixture_path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    assert!(
        case.extensions.contains(&extension),
        "{} fixture extension `{extension}` should be listed in {:?}",
        case.language_name,
        case.extensions,
    );
}

fn assert_case_records(case: &LanguageConformanceCase, records: &[IndexRecord]) {
    let file_records: Vec<&IndexRecord> = records
        .iter()
        .filter(|record| {
            record.path == case.fixture_path
                && record.lang == case.language_id
                && record.source == TREE_SITTER_SOURCE
        })
        .collect();

    assert!(
        !file_records.is_empty(),
        "{} should emit Tree-sitter records for {}",
        case.language_name,
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
            case.language_name,
            case.fixture_path,
            actual_kinds,
        );
    }

    for symbol in case.expected_symbols {
        assert!(
            file_records.iter().any(|record| {
                record.kind == symbol.kind
                    && record.name == symbol.name
                    && record.line == symbol.line
                    && record.col == symbol.col
            }),
            "{} missing expected symbol {:?} in {}.\nrecords:\n{file_records:#?}",
            case.language_name,
            symbol,
            case.fixture_path,
        );
    }
}

fn assert_case_absent_symbols(case: &LanguageConformanceCase, records: &[IndexRecord]) {
    for absent in case.expected_absent_symbols {
        assert!(
            !records
                .iter()
                .any(|record| record.path == case.fixture_path && record.name == *absent),
            "{} comment/string symbol `{absent}` should not be indexed from {}",
            case.language_name,
            case.fixture_path,
        );
    }
}

fn assert_case_search_terms(case: &LanguageConformanceCase, root: &Path) {
    for term in case.search_terms {
        let output = run_wi(root, &[*term]);
        assert!(
            output.contains(term),
            "{} expected wi to find `{term}`, got:\n{output}",
            case.language_name,
        );
    }
}
