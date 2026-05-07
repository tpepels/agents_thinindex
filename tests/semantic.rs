mod common;

use common::*;
use std::fs;
use thinindex::{
    indexer::build_index_with_semantic_adapters,
    model::{SemanticFact, SemanticFactKind},
    semantic::{SemanticAdapterRegistry, StaticSemanticAdapter},
    store::{load_records, load_semantic_facts},
};

#[test]
fn fake_adapter_can_store_semantic_facts() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(
        root,
        "src/service.py",
        "class PromptService: pass\n\ndef call_service():\n    return PromptService()\n",
    );

    let fact = SemanticFact::new(
        "src/service.py",
        4,
        12,
        SemanticFactKind::CallTarget,
        "PromptService",
        Some("src/service.py"),
        Some(1),
        Some(7),
        Some("fake adapter resolved constructor call"),
        "",
        "",
    );
    let registry = SemanticAdapterRegistry::new(vec![Box::new(StaticSemanticAdapter::new(
        "fake-semantic",
        vec![fact],
    ))]);

    let stats =
        build_index_with_semantic_adapters(root, &registry).expect("build with semantic adapter");
    assert_eq!(stats.semantic_facts, 1);

    let facts = load_semantic_facts(root).expect("load semantic facts");
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].kind, SemanticFactKind::CallTarget);
    assert_eq!(facts[0].confidence, "semantic");
    assert_eq!(facts[0].adapter, "fake-semantic");

    let records = load_records(root).expect("load records");
    assert!(
        records.iter().all(|record| record.source != "semantic"),
        "semantic facts must stay out of parser records: {records:#?}"
    );
}

#[test]
fn unavailable_or_failing_adapters_are_skipped_cleanly() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "src/service.py", "class PromptService: pass\n");

    let registry = SemanticAdapterRegistry::new(vec![
        Box::new(StaticSemanticAdapter::unavailable("missing-lsp")),
        Box::new(StaticSemanticAdapter::failing("broken-lsp")),
    ]);

    let stats =
        build_index_with_semantic_adapters(root, &registry).expect("missing adapters are skipped");
    assert_eq!(stats.semantic_facts, 0);
    assert!(
        load_semantic_facts(root)
            .expect("load semantic facts")
            .is_empty()
    );
}

#[test]
fn semantic_facts_do_not_pollute_parser_records_or_normal_commands() {
    let repo = temp_repo();
    let root = repo.path();
    write_file(root, "src/service.py", "class PromptService: pass\n");

    run_build(root);

    assert!(
        load_semantic_facts(root)
            .expect("load semantic facts")
            .is_empty(),
        "normal build should not run semantic adapters"
    );

    let records = load_records(root).expect("load records");
    assert!(
        records.iter().all(|record| record.source != "semantic"),
        "semantic facts must not be written into parser records: {records:#?}"
    );

    let pack = run_wi(root, &["pack", "PromptService"]);
    assert!(
        pack.contains("Primary definitions:") && !pack.contains("semantic"),
        "pack should work without semantic facts and not invent semantic evidence:\n{pack}"
    );

    let refs = run_wi(root, &["refs", "PromptService"]);
    assert!(
        refs.contains("Primary:") && !refs.contains("semantic"),
        "refs should work without semantic facts and not invent semantic evidence:\n{refs}"
    );

    let impact = run_wi(root, &["impact", "PromptService"]);
    assert!(
        impact.contains("Direct definitions:") && !impact.contains("semantic"),
        "impact should work without semantic facts and not invent semantic evidence:\n{impact}"
    );
}

#[test]
fn semantic_fact_docs_mark_feature_internal_and_deferred() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let read = |relpath: &str| {
        fs::read_to_string(format!("{repo_root}/{relpath}"))
            .unwrap_or_else(|error| panic!("read {relpath}: {error}"))
    };

    let semantic_adapters = read("docs/SEMANTIC_ADAPTERS.md");
    assert!(semantic_adapters.contains("## Product Status"));
    assert!(semantic_adapters.contains("Semantic facts are internal and deferred"));
    assert!(semantic_adapters.contains("no normal `wi`, `wi refs`, `wi pack`, or `wi impact`"));
    assert!(semantic_adapters.contains("No semantic adapter is bundled or required by default"));
    assert!(semantic_adapters.contains("test-only static adapter"));

    let readme = read("README.md");
    assert!(readme.contains("internal/deferred semantic-facts table"));
    assert!(readme.contains("not consumed by normal `wi`, `wi refs`, `wi pack`, or `wi impact`"));

    let context_packs = read("docs/CONTEXT_PACKS.md");
    assert!(
        context_packs.contains(
            "Semantic facts are stored separately as an internal/deferred adapter boundary"
        )
    );
    assert!(context_packs.contains("not consumed by baseline pack output"));

    let impact = read("docs/IMPACT_ANALYSIS.md");
    assert!(impact.contains("current `wi impact` output does not consume `semantic_facts`"));
    assert!(impact.contains("Semantic facts are internal/deferred"));

    let reference_graph = read("docs/REFERENCE_GRAPH.md");
    assert!(reference_graph.contains("Semantic facts are internal/deferred"));
    assert!(reference_graph.contains("normal `wi refs` output does not consume them"));

    let roadmap = read("docs/ROADMAP.md");
    assert!(roadmap.contains("Semantic facts are an internal/deferred adapter boundary"));
    assert!(roadmap.contains("normal refs, context, and impact output does not consume them"));
}
