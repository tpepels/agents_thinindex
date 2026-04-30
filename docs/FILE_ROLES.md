# File Roles

thinindex classifies indexed paths into coarse roles so `wi pack` and `wi impact` can include operationally relevant files with explicit confidence.

## Roles

- `source`: normal source files for supported code languages.
- `test`: files under common test paths or with test/spec filename conventions.
- `build`: build and CI files such as `Makefile`, `CMakeLists.txt`, `Dockerfile`, and GitHub Actions YAML.
- `package_manifest`: package or project manifests such as `Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`, `pom.xml`, `build.gradle`, `composer.json`, `Gemfile`, `pubspec.yaml`, `.csproj`, `.fsproj`, and `.sln`.
- `config`: config, route, schema, JSON, TOML, YAML, and similar settings files.
- `docs`: Markdown and documentation paths.
- `generated`: generated, minified, build-output, or target/dist paths.
- `vendor`: vendored or third-party dependency paths.
- `unknown`: paths without a stronger role.

Generated and vendor roles take precedence over source-like extensions so these paths do not look like normal application source.

## Mapping Behavior

For a source or test primary result, `wi pack` and `wi impact` can add heuristic rows for package, build, and config files that apply to the same source area. Root manifests and root build files apply broadly; config directories and same-top-level operational files apply to matching source trees.

Test mapping combines explicit references, dependency edges from test files, and same-name source/test conventions. These rows use `confidence: test-related`.

Operational build/config/package rows use `confidence: heuristic` because they are path-role relationships, not compiler or package-manager resolution. thinindex does not execute package managers, build tools, test runners, or network calls to prove these relationships.
