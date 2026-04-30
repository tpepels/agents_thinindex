use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Result};
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

use crate::model::IndexRecord;

pub const TREE_SITTER_SOURCE: &str = "tree_sitter";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LicenseEntry {
    pub package: &'static str,
    pub upstream: &'static str,
    pub license: &'static str,
    pub accepted_reason: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct QueryPack {
    pub source: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct GrammarAdapter {
    pub id: &'static str,
    pub display_name: &'static str,
    pub extensions: &'static [&'static str],
    pub language: fn() -> Language,
    pub query_pack: QueryPack,
    pub license: LicenseEntry,
}

#[derive(Debug, Clone)]
pub struct LanguageRegistry {
    adapters: Vec<GrammarAdapter>,
    extension_map: BTreeMap<&'static str, usize>,
}

impl LanguageRegistry {
    pub fn new(adapters: Vec<GrammarAdapter>) -> Self {
        let mut extension_map = BTreeMap::new();

        for (index, adapter) in adapters.iter().enumerate() {
            for extension in adapter.extensions {
                extension_map.insert(*extension, index);
            }
        }

        Self {
            adapters,
            extension_map,
        }
    }

    pub fn adapter_for_path(&self, path: &str) -> Option<&GrammarAdapter> {
        let extension = Path::new(path)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        self.extension_map
            .get(extension.as_str())
            .and_then(|index| self.adapters.get(*index))
    }

    pub fn licenses(&self) -> Vec<LicenseEntry> {
        let mut entries = Vec::new();

        for adapter in &self.adapters {
            if !entries
                .iter()
                .any(|entry: &LicenseEntry| entry.package == adapter.license.package)
            {
                entries.push(adapter.license);
            }
        }

        entries
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new(vec![
            rust_adapter(),
            python_adapter(),
            javascript_adapter(),
            jsx_adapter(),
            typescript_adapter(),
            tsx_adapter(),
            java_adapter(),
            go_adapter(),
            c_adapter(),
            cpp_adapter(),
            shell_adapter(),
            ruby_adapter(),
            php_adapter(),
        ])
    }
}

#[derive(Debug, Clone)]
pub struct TreeSitterExtractionEngine {
    registry: LanguageRegistry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFile {
    pub records: Vec<IndexRecord>,
    pub had_error: bool,
}

impl TreeSitterExtractionEngine {
    pub fn new(registry: LanguageRegistry) -> Self {
        Self { registry }
    }

    pub fn parse_file(&self, rel_path: &str, text: &str) -> Result<Vec<IndexRecord>> {
        Ok(self.parse_file_with_diagnostics(rel_path, text)?.records)
    }

    pub fn parse_file_with_diagnostics(&self, rel_path: &str, text: &str) -> Result<ParsedFile> {
        let Some(adapter) = self.registry.adapter_for_path(rel_path) else {
            return Ok(ParsedFile {
                records: Vec::new(),
                had_error: false,
            });
        };

        let language = (adapter.language)();
        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .with_context(|| format!("failed to load {} grammar", adapter.display_name))?;

        let Some(tree) = parser.parse(text, None) else {
            return Ok(ParsedFile {
                records: Vec::new(),
                had_error: false,
            });
        };
        let had_error = tree.root_node().has_error();

        let query = Query::new(&language, adapter.query_pack.source)
            .with_context(|| format!("failed to compile {} query pack", adapter.display_name))?;
        let mapper = CaptureMapper::new(adapter, &query);

        Ok(ParsedFile {
            records: mapper.records_from_query(rel_path, text, tree.root_node())?,
            had_error,
        })
    }

    pub fn licenses(&self) -> Vec<LicenseEntry> {
        self.registry.licenses()
    }
}

impl Default for TreeSitterExtractionEngine {
    fn default() -> Self {
        Self::new(LanguageRegistry::default())
    }
}

#[derive(Debug)]
pub struct CaptureMapper<'a> {
    adapter: &'a GrammarAdapter,
    query: &'a Query,
}

impl<'a> CaptureMapper<'a> {
    pub fn new(adapter: &'a GrammarAdapter, query: &'a Query) -> Self {
        Self { adapter, query }
    }

    pub fn records_from_query(
        &self,
        rel_path: &str,
        text: &str,
        root: tree_sitter::Node<'_>,
    ) -> Result<Vec<IndexRecord>> {
        let mut cursor = QueryCursor::new();
        let bytes = text.as_bytes();
        let capture_names = self.query.capture_names();
        let mut records = Vec::new();
        let mut matches = cursor.matches(self.query, root, bytes);

        while let Some(query_match) = matches.next() {
            let definition_capture = query_match.captures.iter().find_map(|capture| {
                let capture_name = capture_names[capture.index as usize];
                capture_name
                    .strip_prefix("definition.")
                    .map(|kind| (kind, capture.node))
            });

            let Some((capture_kind, definition_node)) = definition_capture else {
                continue;
            };

            let Some(name_node) = query_match.captures.iter().find_map(|capture| {
                let capture_name = capture_names[capture.index as usize];

                if capture_name == "name" {
                    Some(capture.node)
                } else {
                    None
                }
            }) else {
                continue;
            };

            let Ok(name) = name_node.utf8_text(bytes) else {
                continue;
            };
            let name = name.trim();

            if name.is_empty() {
                continue;
            }

            let kind = normalize_capture_kind(capture_kind);
            let start = name_node.start_position();
            let line = start.row + 1;
            let col = start.column + 1;
            let text_line = source_line(text, line).unwrap_or_else(|| {
                definition_node
                    .utf8_text(bytes)
                    .unwrap_or(name)
                    .lines()
                    .next()
                    .unwrap_or(name)
            });

            records.push(IndexRecord::new(
                rel_path,
                line,
                col,
                self.adapter.id,
                kind,
                name,
                text_line.trim(),
                TREE_SITTER_SOURCE,
            ));
        }

        records.sort_by(|a, b| {
            (&a.path, a.line, a.col, &a.kind, &a.name)
                .cmp(&(&b.path, b.line, b.col, &b.kind, &b.name))
        });
        records.dedup_by(|a, b| {
            a.path == b.path
                && a.line == b.line
                && a.col == b.col
                && a.kind == b.kind
                && a.name == b.name
        });

        Ok(records)
    }
}

fn source_line(text: &str, line: usize) -> Option<&str> {
    text.lines().nth(line.saturating_sub(1))
}

fn normalize_capture_kind(kind: &str) -> &str {
    match kind {
        "field" => "variable",
        "macro" => "function",
        "namespace" => "module",
        "constructor" => "method",
        other => other,
    }
}

fn rust_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "rs",
        display_name: "Rust",
        extensions: &["rs"],
        language: || tree_sitter_rust::LANGUAGE.into(),
        query_pack: QueryPack { source: RUST_QUERY },
        license: LicenseEntry {
            package: "tree-sitter-rust",
            upstream: "https://github.com/tree-sitter/tree-sitter-rust",
            license: "MIT",
            accepted_reason: "Rust grammar for bundled Tree-sitter extraction",
        },
    }
}

fn python_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "py",
        display_name: "Python",
        extensions: &["py"],
        language: || tree_sitter_python::LANGUAGE.into(),
        query_pack: QueryPack {
            source: PYTHON_QUERY,
        },
        license: LicenseEntry {
            package: "tree-sitter-python",
            upstream: "https://github.com/tree-sitter/tree-sitter-python",
            license: "MIT",
            accepted_reason: "Python grammar for bundled Tree-sitter extraction",
        },
    }
}

fn javascript_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "js",
        display_name: "JavaScript",
        extensions: &["js"],
        language: || tree_sitter_javascript::LANGUAGE.into(),
        query_pack: QueryPack {
            source: JAVASCRIPT_QUERY,
        },
        license: LicenseEntry {
            package: "tree-sitter-javascript",
            upstream: "https://github.com/tree-sitter/tree-sitter-javascript",
            license: "MIT",
            accepted_reason: "JavaScript and JSX grammar for bundled Tree-sitter extraction",
        },
    }
}

fn jsx_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "jsx",
        display_name: "JSX",
        extensions: &["jsx"],
        language: || tree_sitter_javascript::LANGUAGE.into(),
        query_pack: QueryPack {
            source: JAVASCRIPT_QUERY,
        },
        license: LicenseEntry {
            package: "tree-sitter-javascript",
            upstream: "https://github.com/tree-sitter/tree-sitter-javascript",
            license: "MIT",
            accepted_reason: "JavaScript and JSX grammar for bundled Tree-sitter extraction",
        },
    }
}

fn typescript_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "ts",
        display_name: "TypeScript",
        extensions: &["ts"],
        language: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        query_pack: QueryPack {
            source: TYPESCRIPT_QUERY,
        },
        license: LicenseEntry {
            package: "tree-sitter-typescript",
            upstream: "https://github.com/tree-sitter/tree-sitter-typescript",
            license: "MIT",
            accepted_reason: "TypeScript grammar for bundled Tree-sitter extraction",
        },
    }
}

fn tsx_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "tsx",
        display_name: "TSX",
        extensions: &["tsx"],
        language: || tree_sitter_typescript::LANGUAGE_TSX.into(),
        query_pack: QueryPack {
            source: TYPESCRIPT_QUERY,
        },
        license: LicenseEntry {
            package: "tree-sitter-typescript",
            upstream: "https://github.com/tree-sitter/tree-sitter-typescript",
            license: "MIT",
            accepted_reason: "TSX grammar for bundled Tree-sitter extraction",
        },
    }
}

fn java_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "java",
        display_name: "Java",
        extensions: &["java"],
        language: || tree_sitter_java::LANGUAGE.into(),
        query_pack: QueryPack { source: JAVA_QUERY },
        license: LicenseEntry {
            package: "tree-sitter-java",
            upstream: "https://github.com/tree-sitter/tree-sitter-java",
            license: "MIT",
            accepted_reason: "Java grammar for bundled Tree-sitter extraction",
        },
    }
}

fn go_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "go",
        display_name: "Go",
        extensions: &["go"],
        language: || tree_sitter_go::LANGUAGE.into(),
        query_pack: QueryPack { source: GO_QUERY },
        license: LicenseEntry {
            package: "tree-sitter-go",
            upstream: "https://github.com/tree-sitter/tree-sitter-go",
            license: "MIT",
            accepted_reason: "Go grammar for bundled Tree-sitter extraction",
        },
    }
}

fn c_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "c",
        display_name: "C",
        extensions: &["c", "h"],
        language: || tree_sitter_c::LANGUAGE.into(),
        query_pack: QueryPack { source: C_QUERY },
        license: LicenseEntry {
            package: "tree-sitter-c",
            upstream: "https://github.com/tree-sitter/tree-sitter-c",
            license: "MIT",
            accepted_reason: "C grammar for bundled Tree-sitter extraction",
        },
    }
}

fn cpp_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "cpp",
        display_name: "C++",
        extensions: &["cc", "cpp", "cxx", "hh", "hpp", "hxx"],
        language: || tree_sitter_cpp::LANGUAGE.into(),
        query_pack: QueryPack { source: CPP_QUERY },
        license: LicenseEntry {
            package: "tree-sitter-cpp",
            upstream: "https://github.com/tree-sitter/tree-sitter-cpp",
            license: "MIT",
            accepted_reason: "C++ grammar for bundled Tree-sitter extraction",
        },
    }
}

fn shell_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "sh",
        display_name: "Shell",
        extensions: &["bash", "sh"],
        language: || tree_sitter_bash::LANGUAGE.into(),
        query_pack: QueryPack {
            source: SHELL_QUERY,
        },
        license: LicenseEntry {
            package: "tree-sitter-bash",
            upstream: "https://github.com/tree-sitter/tree-sitter-bash",
            license: "MIT",
            accepted_reason: "Bash/Shell grammar for bundled Tree-sitter extraction",
        },
    }
}

fn ruby_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "rb",
        display_name: "Ruby",
        extensions: &["rb"],
        language: || tree_sitter_ruby::LANGUAGE.into(),
        query_pack: QueryPack { source: RUBY_QUERY },
        license: LicenseEntry {
            package: "tree-sitter-ruby",
            upstream: "https://github.com/tree-sitter/tree-sitter-ruby",
            license: "MIT",
            accepted_reason: "Ruby grammar for bundled Tree-sitter extraction",
        },
    }
}

fn php_adapter() -> GrammarAdapter {
    GrammarAdapter {
        id: "php",
        display_name: "PHP",
        extensions: &["php"],
        language: || tree_sitter_php::LANGUAGE_PHP.into(),
        query_pack: QueryPack { source: PHP_QUERY },
        license: LicenseEntry {
            package: "tree-sitter-php",
            upstream: "https://github.com/tree-sitter/tree-sitter-php",
            license: "MIT",
            accepted_reason: "PHP grammar for bundled Tree-sitter extraction",
        },
    }
}

const RUST_QUERY: &str = r#"
(function_item name: (identifier) @name) @definition.function
(struct_item name: (type_identifier) @name) @definition.struct
(enum_item name: (type_identifier) @name) @definition.enum
(trait_item name: (type_identifier) @name) @definition.trait
(mod_item name: (identifier) @name) @definition.module
(type_item name: (type_identifier) @name) @definition.type
(const_item name: (identifier) @name) @definition.constant
(static_item name: (identifier) @name) @definition.variable
"#;

const PYTHON_QUERY: &str = r#"
(class_definition name: (identifier) @name) @definition.class
(function_definition name: (identifier) @name) @definition.function
(assignment left: (identifier) @name) @definition.variable
(import_statement name: (dotted_name) @name) @definition.import
(import_from_statement module_name: (dotted_name) @name) @definition.import
"#;

const JAVASCRIPT_QUERY: &str = r#"
(function_declaration name: (identifier) @name) @definition.function
(generator_function_declaration name: (identifier) @name) @definition.function
(class_declaration name: (identifier) @name) @definition.class
(method_definition name: (property_identifier) @name) @definition.method
(lexical_declaration (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])) @definition.function
(variable_declaration (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])) @definition.function
(lexical_declaration (variable_declarator name: (identifier) @name value: [(number) (string)])) @definition.variable
(variable_declaration (variable_declarator name: (identifier) @name value: [(number) (string)])) @definition.variable
(import_statement source: (string) @name) @definition.import
(export_statement (export_clause (export_specifier name: (identifier) @name))) @definition.export
"#;

const TYPESCRIPT_QUERY: &str = r#"
(function_declaration name: (identifier) @name) @definition.function
(generator_function_declaration name: (identifier) @name) @definition.function
(class_declaration name: (_) @name) @definition.class
(method_definition name: (property_identifier) @name) @definition.method
(interface_declaration name: (type_identifier) @name) @definition.interface
(type_alias_declaration name: (type_identifier) @name) @definition.type
(lexical_declaration (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])) @definition.function
(lexical_declaration (variable_declarator name: (identifier) @name value: [(number) (string)])) @definition.variable
(import_statement source: (string) @name) @definition.import
(export_statement (export_clause (export_specifier name: (identifier) @name))) @definition.export
"#;

const JAVA_QUERY: &str = r#"
(class_declaration name: (identifier) @name) @definition.class
(interface_declaration name: (identifier) @name) @definition.interface
(enum_declaration name: (identifier) @name) @definition.enum
(record_declaration name: (identifier) @name) @definition.type
(method_declaration name: (identifier) @name) @definition.method
(constructor_declaration name: (identifier) @name) @definition.constructor
(field_declaration declarator: (variable_declarator name: (identifier) @name)) @definition.variable
(import_declaration (identifier) @name) @definition.import
(import_declaration (scoped_identifier) @name) @definition.import
"#;

const GO_QUERY: &str = r#"
(package_clause (package_identifier) @name) @definition.module
(function_declaration name: (identifier) @name) @definition.function
(method_declaration name: (field_identifier) @name) @definition.method
(type_declaration (type_spec name: (type_identifier) @name type: (struct_type))) @definition.struct
(type_declaration (type_spec name: (type_identifier) @name type: (interface_type))) @definition.interface
(type_declaration (type_spec name: (type_identifier) @name)) @definition.type
(const_declaration (const_spec name: (identifier) @name)) @definition.constant
(var_declaration (var_spec name: (identifier) @name)) @definition.variable
(import_declaration (import_spec path: (interpreted_string_literal) @name)) @definition.import
"#;

const C_QUERY: &str = r#"
(preproc_include path: (_) @name) @definition.import
(function_definition declarator: (function_declarator declarator: (identifier) @name)) @definition.function
(function_declarator declarator: (identifier) @name) @definition.function
(struct_specifier name: (type_identifier) @name body: (_)) @definition.struct
(union_specifier name: (type_identifier) @name body: (_)) @definition.type
(enum_specifier name: (type_identifier) @name body: (_)) @definition.enum
(type_definition declarator: (type_identifier) @name) @definition.type
(declaration declarator: (init_declarator declarator: (identifier) @name)) @definition.variable
"#;

const CPP_QUERY: &str = r#"
(preproc_include path: (_) @name) @definition.import
(namespace_definition name: (namespace_identifier) @name) @definition.module
(function_definition declarator: (function_declarator declarator: (identifier) @name)) @definition.function
(function_definition declarator: (function_declarator declarator: (field_identifier) @name)) @definition.method
(function_declarator declarator: (identifier) @name) @definition.function
(function_declarator declarator: (field_identifier) @name) @definition.method
(function_declarator declarator: (qualified_identifier name: (identifier) @name)) @definition.method
(class_specifier name: (type_identifier) @name) @definition.class
(struct_specifier name: (type_identifier) @name body: (_)) @definition.struct
(union_specifier name: (type_identifier) @name body: (_)) @definition.type
(enum_specifier name: (type_identifier) @name body: (_)) @definition.enum
(type_definition declarator: (type_identifier) @name) @definition.type
(declaration declarator: (init_declarator declarator: (identifier) @name)) @definition.variable
"#;

const SHELL_QUERY: &str = r#"
(function_definition name: (word) @name) @definition.function
(variable_assignment name: (variable_name) @name) @definition.variable
"#;

const RUBY_QUERY: &str = r#"
(class name: (constant) @name) @definition.class
(class name: (scope_resolution name: (_) @name)) @definition.class
(module name: (constant) @name) @definition.module
(module name: (scope_resolution name: (_) @name)) @definition.module
(method name: (_) @name) @definition.method
(singleton_method name: (_) @name) @definition.method
(assignment left: (constant) @name) @definition.constant
"#;

const PHP_QUERY: &str = r#"
(namespace_definition name: (namespace_name) @name) @definition.module
(interface_declaration name: (name) @name) @definition.interface
(trait_declaration name: (name) @name) @definition.trait
(class_declaration name: (name) @name) @definition.class
(enum_declaration name: (name) @name) @definition.enum
(function_definition name: (name) @name) @definition.function
(method_declaration name: (name) @name) @definition.method
(property_declaration (property_element (variable_name (name) @name))) @definition.variable
(const_declaration (const_element (name) @name)) @definition.constant
(include_expression (_) @name) @definition.import
(include_once_expression (_) @name) @definition.import
(require_expression (_) @name) @definition.import
(require_once_expression (_) @name) @definition.import
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framework_extracts_through_registry_query_and_capture_mapper() {
        let engine = TreeSitterExtractionEngine::default();
        let records = engine
            .parse_file(
                "src/lib.rs",
                "pub fn build_index() {}\npub struct PromptService;\n",
            )
            .expect("parse rust");

        assert!(records.iter().any(|record| {
            record.name == "build_index"
                && record.kind == "function"
                && record.source == TREE_SITTER_SOURCE
        }));
        assert!(records.iter().any(|record| {
            record.name == "PromptService"
                && record.kind == "struct"
                && record.source == TREE_SITTER_SOURCE
        }));
    }

    #[test]
    fn unsupported_extensions_emit_no_records() {
        let engine = TreeSitterExtractionEngine::default();
        let records = engine
            .parse_file("README.txt", "fn not_a_record() {}\n")
            .expect("parse unsupported");

        assert!(records.is_empty());
    }

    #[test]
    fn representative_language_queries_compile_and_emit_records() {
        let engine = TreeSitterExtractionEngine::default();
        let cases = [
            ("src/sample.rs", "pub fn rust_sample() {}\n", "rust_sample"),
            (
                "src/sample.py",
                "class PythonSample:\n    pass\n",
                "PythonSample",
            ),
            (
                "src/sample.js",
                "export function javascriptSample() {}\n",
                "javascriptSample",
            ),
            (
                "src/sample.jsx",
                "export function JsxSample() { return <main />; }\n",
                "JsxSample",
            ),
            (
                "src/sample.ts",
                "export interface TypeScriptSample {}\n",
                "TypeScriptSample",
            ),
            (
                "src/sample.tsx",
                "export function TsxSample() { return <main />; }\n",
                "TsxSample",
            ),
            (
                "src/Sample.java",
                "class JavaSample { void render() {} }\n",
                "JavaSample",
            ),
            (
                "src/sample.go",
                "package sample\nfunc GoSample() {}\n",
                "GoSample",
            ),
            (
                "src/sample.c",
                "int c_sample(void) { return 1; }\n",
                "c_sample",
            ),
            (
                "src/sample.cpp",
                "class CppSample { public: void render() {} };\n",
                "CppSample",
            ),
            (
                "src/sample.sh",
                "shell_sample() { echo ok; }\n",
                "shell_sample",
            ),
            ("src/sample.rb", "class RubySample\nend\n", "RubySample"),
            ("src/sample.php", "<?php\nclass PhpSample {}\n", "PhpSample"),
        ];

        for (path, text, expected) in cases {
            let records = engine.parse_file(path, text).expect("parse fixture");
            assert!(
                records.iter().any(|record| record.name == expected),
                "expected {expected} in {path}, got {records:#?}",
            );
            assert!(
                records
                    .iter()
                    .all(|record| record.source == TREE_SITTER_SOURCE),
                "expected Tree-sitter source in {path}, got {records:#?}",
            );
        }
    }
}
