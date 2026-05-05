# Recovery Plan Order

This recovery cycle supersedes old roadmap execution until RECOVERY_10 says otherwise.

Execute one plan at a time. Do not skip ahead. Do not continue automatically after a plan unless explicitly asked.

## Order

0. prompts/recovery/RECOVERY_00_RECOVERY_SCOPE_AND_FREEZE.md
1. prompts/recovery/RECOVERY_01_CURRENT_STATE_AUDIT.md
2. prompts/recovery/RECOVERY_02_CORE_TOUCHPOINT_REPAIR.md
3. prompts/recovery/RECOVERY_03_PERFORMANCE_PROFILING_AND_BUDGETS.md
4. prompts/recovery/RECOVERY_04_VALUE_WORKFLOWS.md
5. prompts/recovery/RECOVERY_05_AGENT_INTEGRATION_MINIMUM_USEFUL.md
6. prompts/recovery/RECOVERY_06_RELEASE_DECISION_AUDIT.md
7. prompts/recovery/RECOVERY_07_PRODUCT_VALUE_SCORECARD.md
8. prompts/recovery/RECOVERY_08_ERROR_MESSAGE_AND_HELP_AUDIT.md
9. prompts/recovery/RECOVERY_09_MINIMUM_AGENT_ACCEPTANCE_TEST.md
10. prompts/recovery/RECOVERY_10_RECOVERY_FINAL_AUDIT_AND_NEXT_DECISION.md

## Standard execution prompt

Use this when asking an agent to continue:

Use superpowers:subagent-driven-development.

Read prompts/recovery/PLAN_ORDER.md.

Find the first recovery plan in order that is not complete.

Implement only that plan.

Rules:
- Treat the selected plan as authoritative.
- Do not implement later recovery plans.
- Do not resume the old roadmap.
- Use checkbox-based phase tracking in the plan file.
- Run the verification listed in the selected plan.
- Commit with the commit message specified in the selected plan after verification is green.

Final response:
- selected plan
- changed files
- verification commands and results
- commit hash
- next plan in order

## Completion rule

A plan is complete only if:
- its checklist is complete
- required tests/verifications passed or are explicitly inapplicable
- its commit exists
- its final report identifies the next plan

## Stop rule

If a core touchpoint is broken, stop roadmap work and fix recovery plans first.
