# Active Plan Order

This file is the execution source of truth for active post-recovery roadmap work.

Execute one plan at a time. Do not skip ahead. Do not continue automatically after a plan unless explicitly asked.

## Current order

1. `prompts/PLAN_55_REFS_CONFIDENCE_REASON_OUTPUT_ALIGNMENT.md`
2. `prompts/PLAN_56_BOUNDED_IMPORT_EXPORT_FILE_REFERENCE_HARDENING.md`
3. `prompts/PLAN_57_GRAPH_INCREMENTAL_RELATIONSHIP_REBUILDS.md`
4. `prompts/PLAN_58_GO_PHP_REAL_REPO_SUPPORT_EVIDENCE.md`
5. `prompts/PLAN_59_AGENT_INTEGRATION_HELPERS_AND_MCP_DECISION.md`
6. `prompts/PLAN_60_OPTIONAL_QUALITY_CLI_DECISION.md`
7. `prompts/PLAN_61_REAL_REPO_EVIDENCE_STABILIZATION.md`
8. `prompts/PLAN_62_TARGET_PLATFORM_RELEASE_SMOKE.md`
9. `prompts/PLAN_63_SEMANTIC_FACT_USER_VALUE_DECISION.md`
10. `prompts/PLAN_64_MCP_INTEGRATION_DECISION.md`
11. `prompts/PLAN_65_NATIVE_PACKAGE_SIGNING_PLAN.md`

## Completion rule

A plan is complete only if:
- its checklist is complete
- required tests/verifications passed or are explicitly inapplicable
- its commit exists
- its final response identified the next plan

## Stop rules

Stop and do not continue to the next plan if:
- verification fails
- the plan exposes a release blocker
- build_index performance regresses
- stale/missing index self-healing regresses
- source/PATH binary schema agreement regresses
- `wi refs`, `wi pack`, or `wi impact` become noisier or less useful
- a plan requires scope outside its own file

## Invariants

Do not reintroduce:
- Universal Ctags as production parser
- `WI.md`
- JSONL as canonical storage
- package-manager execution
- network access
- telemetry
- cloud behavior
- payment/licensing enforcement
- unbounded output
- broad parser rewrites outside the selected plan
