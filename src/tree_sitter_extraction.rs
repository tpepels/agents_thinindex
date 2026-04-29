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
            typescript_adapter(),
            tsx_adapter(),
        ])
    }
}

#[derive(Debug, Clone)]
pub struct TreeSitterExtractionEngine {
    registry: LanguageRegistry,
}

impl TreeSitterExtractionEngine {
    pub fn new(registry: LanguageRegistry) -> Self {
        Self { registry }
    }

    pub fn parse_file(&self, rel_path: &str, text: &str) -> Result<Vec<IndexRecord>> {
        let Some(adapter) = self.registry.adapter_for_path(rel_path) else {
            return Ok(Vec::new());
        };

        let language = (adapter.language)();
        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .with_context(|| format!("failed to load {} grammar", adapter.display_name))?;

        let Some(tree) = parser.parse(text, None) else {
            return Ok(Vec::new());
        };

        let query = Query::new(&language, adapter.query_pack.source)
            .with_context(|| format!("failed to compile {} query pack", adapter.display_name))?;
        let mapper = CaptureMapper::new(adapter, &query);

        mapper.records_from_query(rel_path, text, tree.root_node())
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
        extensions: &["js", "jsx"],
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
"#;

const JAVASCRIPT_QUERY: &str = r#"
(function_declaration name: (identifier) @name) @definition.function
(generator_function_declaration name: (identifier) @name) @definition.function
(class_declaration name: (identifier) @name) @definition.class
(method_definition name: (property_identifier) @name) @definition.method
(lexical_declaration (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])) @definition.function
(variable_declaration (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])) @definition.function
(lexical_declaration (variable_declarator name: (identifier) @name)) @definition.variable
(export_statement declaration: (function_declaration name: (identifier) @name)) @definition.export
(export_statement declaration: (class_declaration name: (identifier) @name)) @definition.export
"#;

const TYPESCRIPT_QUERY: &str = r#"
(function_declaration name: (identifier) @name) @definition.function
(generator_function_declaration name: (identifier) @name) @definition.function
(class_declaration name: (_) @name) @definition.class
(method_definition name: (property_identifier) @name) @definition.method
(interface_declaration name: (type_identifier) @name) @definition.interface
(type_alias_declaration name: (type_identifier) @name) @definition.type
(lexical_declaration (variable_declarator name: (identifier) @name value: [(arrow_function) (function_expression)])) @definition.function
(lexical_declaration (variable_declarator name: (identifier) @name)) @definition.variable
(export_statement declaration: (function_declaration name: (identifier) @name)) @definition.export
(export_statement declaration: (class_declaration name: (_) @name)) @definition.export
(export_statement declaration: (interface_declaration name: (type_identifier) @name)) @definition.export
(export_statement declaration: (type_alias_declaration name: (type_identifier) @name)) @definition.export
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
}
