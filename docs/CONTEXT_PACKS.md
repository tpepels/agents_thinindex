# Context Packs

`wi pack <term>` returns a bounded read set for implementation work. It is designed for agents that need the smallest useful set of files before editing, not a full source dump.

Pack output is built from SQLite `records`, `refs`, `dependencies`, and the same evidence model used by `wi impact`. Optional semantic facts are stored separately and are not required for baseline pack output. Every row includes a reason and confidence label.

File-role classification controls how test, build, package manifest, config, docs, generated, and vendor paths are grouped. See [FILE_ROLES.md](FILE_ROLES.md).

## Groups

- `Primary definitions`: direct symbol or structured-landmark matches for the query.
- `Direct references`: high-signal references to the primary names, such as imports, calls, exports, type references, or precise local matches.
- `Dependencies`: files imported by the primary definition files.
- `Dependents`: files that import or otherwise depend on the primary definition files.
- `Tests`: test-path references, test dependency edges, and same-name test conventions.
- `Configs/build files`: config, route, schema, JSON, TOML, YAML, and build/workflow references.
- `Docs/examples`: documentation references plus fixture/example files when they are relevant but lower confidence.
- `Unresolved hints`: unresolved dependency evidence and other unknown areas that may matter.

Rows are deduplicated by file across non-primary groups. If one file has multiple reasons, the earlier and stronger group wins.

## Ranking Model

Pack ranking favors:

- exact primary symbol matches;
- local dependency proximity around the primary files;
- stronger confidence labels before heuristic evidence;
- source-like files before test, docs, fixture, or example paths;
- files with more references when other ranking signals are equal;
- test/source pairing through conventional test paths and same-name files.

Quality manifest hints can improve this ranking when future plans add them. Without a manifest, pack output remains deterministic and evidence-backed.

## Caps

The default pack size is intentionally small. `-n` sets the total output budget, with primary rows always shown first and remaining slots distributed across bounded groups. Group caps keep broad text matches, tests, docs, configs, examples, and unresolved hints from crowding out high-signal source relationships.

## Agent Use

For implementation work, run:

```bash
wi pack <term>
```

Read the returned files first. If the pack is insufficient, use `wi impact <term>` for a broader affected-file view, then fall back to targeted repository search. Pack output does not include full file contents and does not claim complete semantic coverage.
