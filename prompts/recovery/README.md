# Recovery Cycle Status

The recovery cycle is the current execution source of truth for thinindex until
`RECOVERY_10_RECOVERY_FINAL_AUDIT_AND_NEXT_DECISION.md` explicitly chooses a
next direction.

Old roadmap plans remain available in `prompts/` as background context only.
Superseded plan files remain available in `prompts/superseded/`. Do not resume
or extend those plans while recovery is active.

Execution starts with `PLAN_ORDER.md` and proceeds one recovery plan at a time.
Each plan is complete only after its checklist is checked, verification is
green or explicitly inapplicable with evidence, and the required commit exists.
