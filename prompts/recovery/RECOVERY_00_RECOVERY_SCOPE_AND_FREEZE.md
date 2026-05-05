# RECOVERY_00_RECOVERY_SCOPE_AND_FREEZE.md

Use superpowers:subagent-driven-development.

Goal:
Freeze old roadmap execution and define the recovery cycle as the current source of truth.

Scope:
Planning/alignment only. Do not modify product code unless needed to make the recovery plan references accurate.

Current failure it addresses:
The roadmap expanded while basic product touchpoints were broken. The team needs a clear recovery boundary before more work continues.

Phases:
- [x] Inventory old active plan files.
- [x] Confirm recovery plans supersede roadmap execution until RECOVERY_10 says otherwise.
- [x] Add or update recovery README/status note.
- [x] Ensure PLAN_ORDER.md references RECOVERY_00 through RECOVERY_10.
- [x] Verify old plans are not deleted or renumbered.
- [x] Run validation.
- [x] Commit.

Rules:
- Do not delete old roadmap plans.
- Do not renumber old roadmap plans.
- Do not implement product features.
- Do not add packaging, licensing, payment, telemetry, cloud, or MCP work.
- Do not reintroduce Universal Ctags as production parser.
- Do not reintroduce WI.md.
- Do not reintroduce JSONL as canonical storage.

Acceptance:
- recovery cycle is clearly declared as current execution source.
- old roadmap remains available as background.
- PLAN_ORDER.md starts with RECOVERY_00.
- next action is unambiguous.

Verification:
- ls prompts/recovery
- grep -R "recovery cycle supersedes" prompts/recovery
- git status --short

Commit:
Define recovery cycle scope
