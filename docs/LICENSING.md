# Licensing Foundation

thinindex has an inert local licensing foundation. It exists so future product
work can use one model for edition/status handling, but it does not enforce paid
features today.

## Current Behavior

- No license enforcement is active.
- No payment integration exists.
- No network activation exists.
- No license server exists.
- No cloud account behavior exists.
- No telemetry is collected.
- No current command is blocked by license status.
- The free local core remains available without a license file.

The current free local core includes `wi-init`, `build_index`, `wi <term>`,
`wi refs`, `wi pack`, `wi impact`, `wi bench`, `wi-stats`, local SQLite index
storage, repository instruction generation, local cache rebuilds, and
no-network operation.

## Edition Model

The internal edition model has three states:

- `free`: the local/free edition, including the current normal workflow.
- `pro`: a future paid edition. Today this can only be reached by explicit
  local test fixtures.
- `unknown/unlicensed`: an invalid, unknown, unreadable, unsupported, or
  non-fixture license status.

Invalid or missing license data must not reduce current functionality. Missing
license data reports the free local edition.

## Local License File Path

The local license path design is:

1. `THININDEX_LICENSE_FILE` when set.
2. On Windows, `%APPDATA%\thinindex\license.json`.
3. On Unix-like systems, `$XDG_CONFIG_HOME/thinindex/license.json` when set.
4. Otherwise, `$HOME/.config/thinindex/license.json`.

This path is a design for future local status reads. Users do not need to create
it for the current free/local workflow.

## Validation Stub

The current validator is intentionally narrow. It accepts only explicit local
test fixtures with:

- `schema_version`: `1`
- `edition`: `pro`
- `validation`: `local-test-fixture`
- `signature`: `thinindex-local-test-fixture`
- `license_id` starting with `thinindex-local-test-`

Production-like Pro licenses, future server validation markers, unknown
editions, unsupported schema versions, and malformed JSON are reported as
`unknown/unlicensed`.

This is not a cryptographic license validator. It is a test-only placeholder so
future activation, signing, offline validation, or team licensing work has a
small local model to build on.

## Deferred Future Work

Future paid licensing work must be a separate plan. It must document:

- activation flow
- offline behavior
- local cache shape
- privacy behavior
- refund/support workflow
- payment provider boundaries
- server/network requirements, if any
- which features, if any, are gated

Before any enforcement is added, the docs and tests must continue to prove that
basic local indexing and repository search remain usable without a paid license.
