mod common;

use common::*;
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

    let impact = run_wi(root, &["impact", "PromptService"]);
    assert!(
        impact.contains("Direct definitions:") && !impact.contains("semantic"),
        "impact should work without semantic facts and not invent semantic evidence:\n{impact}"
    );
}
