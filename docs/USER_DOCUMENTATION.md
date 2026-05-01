# User Documentation

This index is for using thinindex in a repository. It avoids implementation
details unless they affect normal command behavior.

## Start Here

- [Project overview](../README.md): what thinindex is, the command list, and the support matrix.
- [Getting started](GETTING_STARTED.md): shortest path from install to useful `wi` results.
- [Troubleshooting](TROUBLESHOOTING.md): use `wi doctor` output to fix missing, stale, or misconfigured setup.

## Core Workflow

Use these commands inside the repository you want to navigate:

```bash
wi-init
build_index
wi doctor
wi --help
wi <term>
wi refs <term>
wi pack <term>
wi impact <term>
wi-stats
```

The canonical command syntax, filters, examples, and subcommands are in
`wi --help`. Documentation should route you there rather than duplicate every
CLI option.

Useful docs:

- [Getting started](GETTING_STARTED.md): first index and first searches.
- [Agent integration](AGENT_INTEGRATION.md): recommended agent workflow and read-budget guidance.
- [Context packs](CONTEXT_PACKS.md): how `wi pack <term>` chooses a compact read set.
- [Impact analysis](IMPACT_ANALYSIS.md): what `wi impact <term>` does and does not prove.
- [Reference graph](REFERENCE_GRAPH.md): deterministic references behind `wi refs <term>`.

## Local-first Model

thinindex is local-first:

- no daemon;
- no source upload;
- no hosted search service;
- no telemetry;
- no account login;
- no license enforcement for current commands.

The canonical index is `.dev_index/index.sqlite`. It is a repo-local disposable
cache and should not be committed. See:

- [Security and privacy](SECURITY_PRIVACY.md)
- [Performance and repository size guidance](PERFORMANCE.md)
- [Product boundary](PRODUCT_BOUNDARY.md)

## Language And Format Support

Support levels are explicit:

- `supported`
- `experimental`
- `blocked`
- `extras-backed`

Tree-sitter provides syntax-tree facts for supported code languages. It does
not provide compiler, LSP, macro-expansion, type-checking, or runtime binding
semantics.

CSS, HTML, Markdown, JSON, TOML, and YAML are extras-backed deterministic
landmarks, not Tree-sitter-backed code-symbol parsers.

Read:

- [Parser support levels](PARSER_SUPPORT.md)
- [Language support dashboard](LANGUAGE_SUPPORT.md)

## Release Archives And Install Helpers

Release archives are local, source-upload-free, and credential-free. They ship
the thinindex binaries, release docs, `SBOM.md`, checksum sidecars,
install/uninstall helper scripts, and `THIRD_PARTY_NOTICES`.

Native package formats, completed signing/notarization, GitHub Release
publishing, package-manager publishing, and managed update channels remain
future or scaffolded work.

Read:

- [Releasing](RELEASING.md)
- [Installers and signing](INSTALLERS.md)
- [Release checklist](RELEASE_CHECKLIST.md)
- [License audit](LICENSE_AUDIT.md)
- [Licensing foundation](LICENSING.md)

## What thinindex Does Not Do

thinindex does not claim to be:

- a hosted search service;
- an IDE or LSP replacement;
- full semantic code understanding;
- a compiler or package-manager resolver;
- a secret scanner;
- a telemetry, account, payment, or activation system.

The current free/local commands remain usable without a license file. See
[licensing](LICENSING.md) and [product boundary](PRODUCT_BOUNDARY.md).

## Roadmap And History

Roadmap and plan documents are useful context, but they are not the main user
path.

- [Roadmap](ROADMAP.md)
- [Caveats and unimplemented summary](PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md)
- [General docs index](README.md)
