# Universal Ctags Boundary

Universal Ctags is optional, external, not bundled, not required, and not used by production indexing.

The only permitted ctags surface is the isolated quality comparator under `src/quality/**`, its `tests/quality*` coverage, and quality documentation. The comparator may be used manually to compare thinindex output against an external tool, but its output must stay outside production SQLite `records` and `refs`.

Forbidden surfaces include production parser, indexer, store, search, refs, pack, impact, install, uninstall, release, and package paths. Normal installer docs and generated agent instructions must not mention ctags. Release artifacts must not include ctags binaries.

Run the structural gate directly with:

```bash
cargo test --test quality_ctags_allowlist
```
