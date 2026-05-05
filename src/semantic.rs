use std::path::Path;

use anyhow::{Result, bail};

use crate::model::{DependencyEdge, IndexRecord, SemanticFact};

pub const SEMANTIC_CONFIDENCE: &str = "semantic";

pub trait SemanticAdapter: Send + Sync {
    fn name(&self) -> &str;

    fn is_available(&self, _root: &Path) -> bool {
        true
    }

    fn collect_facts(
        &self,
        root: &Path,
        records: &[IndexRecord],
        dependencies: &[DependencyEdge],
    ) -> Result<Vec<SemanticFact>>;
}

#[derive(Default)]
pub struct SemanticAdapterRegistry {
    adapters: Vec<Box<dyn SemanticAdapter>>,
}

impl SemanticAdapterRegistry {
    pub fn new(adapters: Vec<Box<dyn SemanticAdapter>>) -> Self {
        Self { adapters }
    }

    pub fn disabled() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }

    pub fn collect_facts(
        &self,
        root: &Path,
        records: &[IndexRecord],
        dependencies: &[DependencyEdge],
    ) -> Vec<SemanticFact> {
        let mut facts = Vec::new();

        for adapter in &self.adapters {
            if !adapter.is_available(root) {
                continue;
            }

            let Ok(mut adapter_facts) = adapter.collect_facts(root, records, dependencies) else {
                continue;
            };

            for fact in &mut adapter_facts {
                if fact.adapter.is_empty() {
                    fact.adapter = adapter.name().to_string();
                }

                if fact.confidence.is_empty() {
                    fact.confidence = SEMANTIC_CONFIDENCE.to_string();
                }
            }

            facts.append(&mut adapter_facts);
        }

        finalize_semantic_facts(facts)
    }
}

#[derive(Debug, Clone)]
pub struct StaticSemanticAdapter {
    name: String,
    facts: Vec<SemanticFact>,
    available: bool,
    fail: bool,
}

impl StaticSemanticAdapter {
    pub fn new(name: impl Into<String>, facts: Vec<SemanticFact>) -> Self {
        Self {
            name: name.into(),
            facts,
            available: true,
            fail: false,
        }
    }

    pub fn unavailable(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            facts: Vec::new(),
            available: false,
            fail: false,
        }
    }

    pub fn failing(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            facts: Vec::new(),
            available: true,
            fail: true,
        }
    }
}

impl SemanticAdapter for StaticSemanticAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self, _root: &Path) -> bool {
        self.available
    }

    fn collect_facts(
        &self,
        _root: &Path,
        _records: &[IndexRecord],
        _dependencies: &[DependencyEdge],
    ) -> Result<Vec<SemanticFact>> {
        if self.fail {
            bail!("static semantic adapter failed");
        }

        Ok(self.facts.clone())
    }
}

pub fn finalize_semantic_facts(mut facts: Vec<SemanticFact>) -> Vec<SemanticFact> {
    facts.sort_by(semantic_fact_sort_key);
    facts.dedup_by(|a, b| {
        a.source_path == b.source_path
            && a.source_line == b.source_line
            && a.source_col == b.source_col
            && a.kind == b.kind
            && a.symbol == b.symbol
            && a.target_path == b.target_path
            && a.adapter == b.adapter
    });
    facts
}

fn semantic_fact_sort_key(a: &SemanticFact, b: &SemanticFact) -> std::cmp::Ordering {
    a.source_path
        .cmp(&b.source_path)
        .then(a.source_line.cmp(&b.source_line))
        .then(a.source_col.cmp(&b.source_col))
        .then(a.kind.cmp(&b.kind))
        .then(a.symbol.cmp(&b.symbol))
        .then(a.target_path.cmp(&b.target_path))
        .then(a.target_line.cmp(&b.target_line))
        .then(a.target_col.cmp(&b.target_col))
        .then(a.adapter.cmp(&b.adapter))
}

#[cfg(test)]
mod tests {
    use crate::model::{SemanticFact, SemanticFactKind};

    use super::{SEMANTIC_CONFIDENCE, SemanticAdapterRegistry, StaticSemanticAdapter};

    #[test]
    fn static_adapter_supplies_default_fact_metadata() {
        let fact = SemanticFact::new(
            "src/caller.py",
            1,
            1,
            SemanticFactKind::CallTarget,
            "build_prompt",
            Some("src/service.py"),
            Some(3),
            Some(5),
            Some("call resolved by fake adapter"),
            "",
            "",
        );

        let registry = SemanticAdapterRegistry::new(vec![Box::new(StaticSemanticAdapter::new(
            "fake",
            vec![fact],
        ))]);
        let facts = registry.collect_facts(std::path::Path::new("."), &[], &[]);

        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].adapter, "fake");
        assert_eq!(facts[0].confidence, SEMANTIC_CONFIDENCE);
    }

    #[test]
    fn unavailable_and_failing_adapters_are_skipped() {
        let registry = SemanticAdapterRegistry::new(vec![
            Box::new(StaticSemanticAdapter::unavailable("missing")),
            Box::new(StaticSemanticAdapter::failing("broken")),
        ]);

        let facts = registry.collect_facts(std::path::Path::new("."), &[], &[]);
        assert!(facts.is_empty());
    }
}
