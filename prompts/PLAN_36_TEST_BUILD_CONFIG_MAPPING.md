# PLAN_36_TEST_BUILD_CONFIG_MAPPING.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_35_CONTEXT_PACK_V2_DEPENDENCY_AWARE.md is complete and green.

Progress:
- [x] Phase 1: inspect Plan 35 pack/impact evidence flow and current test/config heuristics
- [x] Phase 2: add shared file-role classification for source, test, build, package manifest, config, docs, generated, vendor, and unknown paths
- [x] Phase 3: add source/test pairing and package/build/config mappings into pack and impact
- [x] Phase 4: add role, pairing, operational mapping, determinism, and generated/vendor tests
- [x] Phase 5: document file roles and run required verification
- [x] Phase 6: commit completed Plan 36 work

Goal:
Map tests, build files, package manifests, and config files to source areas so impact/pack can include operationally relevant files.

Product rule:
Agents need to know what to test and which config/build files may affect a change.

Required:
- Add file-role classification.
- Add test/source pairing heuristics.
- Add build/package/config file classification.
- Add local relationships from package/build/config files to source trees where practical.
- Keep false positives labeled as heuristic.
- Do not add package-manager execution.
- Do not add network access.

File roles:
- source
- test
- build
- package_manifest
- config
- docs
- generated
- vendor
- unknown

Package/build/config examples:
- Cargo.toml
- package.json
- pyproject.toml
- go.mod
- pom.xml
- build.gradle
- Makefile
- CMakeLists.txt
- composer.json
- Gemfile
- pubspec.yaml
- csproj/fsproj/sln
- Dockerfile
- GitHub Actions YAML

Tests:
- role classifier fixtures
- source/test pairing fixtures
- build/config mapping affects impact
- generated/vendor files handled correctly
- output deterministic

Docs:
Document:
- role classification
- confidence/heuristic behavior
- how this affects pack/impact

Acceptance:
- file roles exist
- test/build/config mappings improve pack/impact
- false positives are labeled
- existing behavior remains stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- role model
- mapping behavior
- sample impact/pack changes
- verification commands and results
- commit hash
