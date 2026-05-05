# Thinindex Value Workflows

Audit date: 2026-05-05.

This document records the product workflows RECOVERY_04 uses as the minimum
proof that thinindex is useful to humans and coding agents. The goal is not to
claim exhaustive semantic analysis; it is to prove the CLI gives bounded,
actionable next files before broad reads.

## Core Workflows

1. Find symbol.
   - Command: `wi <symbol>`
   - Expected value: returns the owning file:line for a named function, class,
     method, selector, heading, or config landmark.
   - Fixture proof: `wi CheckoutService` self-heals a missing index and returns
     `src/checkout_service.py`.

2. Find broad concept.
   - Command: `wi <term>`
   - Expected value: returns a small candidate set across code, docs, config, or
     style landmarks without dumping file contents.
   - Fixture proof: `wi checkout` returns checkout source/docs candidates within
     the default bounded output.

3. Inspect refs.
   - Command: `wi refs <term>`
   - Expected value: shows primary definitions plus deterministic reference
     evidence with compact reasons.
   - Fixture proof: `wi refs CheckoutService` shows consumer and test
     references with evidence.

4. Build context.
   - Command: `wi pack <term>`
   - Expected value: returns a bounded read set with primary files, dependencies,
     dependents, tests, docs, config/build files, reasons, and confidence
     labels.
   - Fixture proof: `wi pack CheckoutService` includes checkout source,
     payment dependency, consumer, tests, docs, and config.

5. Inspect impact.
   - Command: `wi impact <term>`
   - Expected value: returns plausible affected files with explicit reasons and
     confidence labels.
   - Fixture proof: `wi impact CheckoutService` includes consumer and test files
     with reason/confidence rows.

6. Diagnose state.
   - Command: `wi doctor`
   - Expected value: explains missing, stale, and current states with next
     actions.
   - Fixture proof: after the workflow self-heals the index, `wi doctor`
     reports current freshness. Existing doctor tests cover missing/stale
     states.

7. Initialize agent instructions.
   - Command: `wi-init`
   - Expected value: creates useful `AGENTS.md`, normalizes existing
     `CLAUDE.md`, and does not create `WI.md`.
   - Fixture proof: workflow test creates a legacy `CLAUDE.md`, runs `wi-init`,
     and verifies current instruction blocks.

## Real Repo Evidence

Local ignored repo used: `test_repos/fd`.

Commands run:

- `wi main -n 5`
- `wi refs main -n 8`
- `wi pack main -n 8`
- `wi impact main -n 8`
- `wi ignore -n 8`
- `wi doctor`

Observed value:

- `wi main -n 5` returned concrete entry points including `src/main.rs:61
  function main`.
- `wi ignore -n 8` returned useful broad candidates such as
  `src/filetypes.rs:21 function should_ignore`, filter functions, and related
  tests.
- `wi pack main -n 8` returned primary definitions, dependencies such as
  `src/cli.rs`, `src/dir_entry.rs`, `src/error.rs`, and operational context
  such as `Cargo.toml`.
- `wi impact main -n 8` returned primary definitions and build/config context
  with reasons and confidence labels.
- `wi doctor` accurately reported that this local third-party checkout has a
  fresh index but is missing `AGENTS.md` and `.dev_index/` ignore setup.

Known value gap:

- `wi refs main -n 8` found primary definitions but no direct references for
  `main`. That is acceptable for this plan because `main` is an entry point
  with limited call-site evidence, and pack/impact still returned useful
  context. Future workflow/value work should prefer real-repo queries that have
  richer reference evidence when assessing refs quality.

## Regression Coverage

`tests/wi.rs::core_value_workflows_are_useful_bounded_and_self_healing` covers:

- missing-index self-healing in a normal search workflow;
- symbol search;
- broad concept search;
- refs evidence;
- pack read set contents;
- impact reason/confidence output;
- bounded output sizes;
- doctor current-state diagnosis;
- `wi-init` AGENTS.md creation, existing CLAUDE.md normalization, and no
  `WI.md` reintroduction.

Normal workflow paths use `wi`, `wi refs`, `wi pack`, `wi impact`, `wi doctor`,
and `wi-init`; they do not run quality/comparator/real-repo checks.
