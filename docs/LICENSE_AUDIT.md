# License Audit

thinindex uses a committed `cargo-deny` configuration to audit Cargo dependency licenses before release packaging.

Run the audit with:

```bash
cargo deny check licenses
```

or:

```bash
make license-audit
```

The audit must pass against the committed `Cargo.lock` before proprietary Windows, macOS, or Linux release artifacts are cut. Passing this command is a dependency-policy check, not legal advice.

## Policy

Allowed licenses are listed explicitly in `deny.toml` and are limited to permissive licenses currently approved for the dependency graph:

- MIT
- Apache-2.0
- Apache-2.0 WITH LLVM-exception
- BSD-2-Clause
- BSD-3-Clause
- ISC
- Zlib
- Unicode-3.0
- CC0-1.0
- Unlicense

Dependencies with GPL, AGPL, LGPL-only, MPL-only, EPL, CDDL, unknown, no-license, custom, or non-commercial terms are blocked unless a future plan adds an explicit documented review exception. No such exception is configured today.

Dual-license expressions are acceptable only when `cargo-deny` can satisfy the expression through an allowed permissive branch. Current metadata includes `r-efi` as `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; the audit accepts that dependency through the MIT/Apache alternatives and keeps the expression visible in `Cargo.lock`.

## Parser and Grammar Audit

All supported code-language extraction goes through the Tree-sitter registry, query specs, normalized captures, conformance fixtures, documentation entries, and notice entries. Adding a language requires updating each of those surfaces.

Current bundled parser/grammar crates are MIT-licensed:

- `tree-sitter`
- `tree-sitter-language`
- `tree-sitter-bash`
- `tree-sitter-c`
- `tree-sitter-c-sharp`
- `tree-sitter-cpp`
- `tree-sitter-dart`
- `tree-sitter-go`
- `tree-sitter-java`
- `tree-sitter-javascript`
- `tree-sitter-kotlin-ng`
- `tree-sitter-nix`
- `tree-sitter-php`
- `tree-sitter-python`
- `tree-sitter-ruby`
- `tree-sitter-rust`
- `tree-sitter-scala`
- `tree-sitter-swift`
- `tree-sitter-typescript`

Generated parser sources are bundled through those grammar crates and are covered by the crate license metadata recorded in `THIRD_PARTY_NOTICES`.

Universal Ctags is not bundled, not used, not detected, and not required by active code, tests, install scripts, or release documentation.

## SQLite Status

`rusqlite` is built with the `bundled` feature. That enables `libsqlite3-sys` to compile and link vendored SQLite source, so release artifacts do not rely on a system SQLite library. `rusqlite` and `libsqlite3-sys` are MIT-licensed. Upstream documents bundled SQLite source as public domain.

## Release Notice Requirement

`THIRD_PARTY_NOTICES` is part of release artifacts. Before publishing, verify that it covers:

- direct runtime dependencies that should be noticed
- bundled Tree-sitter parser and grammar crates
- generated parser source status
- bundled SQLite and `libsqlite3-sys` status
- the explicit statement that Universal Ctags is not bundled and not used

If `cargo deny check licenses` fails, or if a dependency has unknown/copyleft/custom/non-commercial terms, proprietary packaging remains blocked until the dependency is removed or a documented review exception is approved in a future plan.
