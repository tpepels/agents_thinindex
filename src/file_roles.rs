#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileRole {
    Source,
    Test,
    Build,
    PackageManifest,
    Config,
    Docs,
    Generated,
    Vendor,
    Unknown,
}

impl FileRole {
    pub fn as_str(self) -> &'static str {
        match self {
            FileRole::Source => "source",
            FileRole::Test => "test",
            FileRole::Build => "build",
            FileRole::PackageManifest => "package_manifest",
            FileRole::Config => "config",
            FileRole::Docs => "docs",
            FileRole::Generated => "generated",
            FileRole::Vendor => "vendor",
            FileRole::Unknown => "unknown",
        }
    }
}

pub fn classify_path(path: &str) -> FileRole {
    let normalized = normalize_path(path);
    let filename = normalized.rsplit('/').next().unwrap_or(&normalized);

    if is_vendor_path(&normalized) {
        return FileRole::Vendor;
    }

    if is_generated_path(&normalized) {
        return FileRole::Generated;
    }

    if is_doc_path_normalized(&normalized) {
        return FileRole::Docs;
    }

    if is_test_path_normalized(&normalized) {
        return FileRole::Test;
    }

    if is_package_manifest_filename(filename) {
        return FileRole::PackageManifest;
    }

    if is_build_file_path(&normalized, filename) {
        return FileRole::Build;
    }

    if is_config_path(&normalized, filename) {
        return FileRole::Config;
    }

    if is_source_file(filename) {
        return FileRole::Source;
    }

    FileRole::Unknown
}

pub fn is_source_role(role: FileRole) -> bool {
    matches!(role, FileRole::Source | FileRole::Test)
}

pub fn is_operational_role(role: FileRole) -> bool {
    matches!(
        role,
        FileRole::Build | FileRole::PackageManifest | FileRole::Config
    )
}

pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

fn is_vendor_path(normalized: &str) -> bool {
    normalized.contains("/vendor/")
        || normalized.starts_with("vendor/")
        || normalized.contains("/vendors/")
        || normalized.starts_with("vendors/")
        || normalized.contains("/node_modules/")
        || normalized.starts_with("node_modules/")
        || normalized.contains("/third_party/")
        || normalized.starts_with("third_party/")
        || normalized.contains("/3rdparty/")
        || normalized.starts_with("3rdparty/")
        || normalized.contains("/external/")
        || normalized.starts_with("external/")
}

fn is_generated_path(normalized: &str) -> bool {
    let filename = normalized.rsplit('/').next().unwrap_or(normalized);

    normalized.contains("/generated/")
        || normalized.starts_with("generated/")
        || normalized.contains("/gen/")
        || normalized.starts_with("gen/")
        || normalized.contains("/dist/")
        || normalized.starts_with("dist/")
        || normalized.contains("/build/")
        || normalized.starts_with("build/")
        || normalized.contains("/target/")
        || normalized.starts_with("target/")
        || filename.contains(".generated.")
        || filename.ends_with(".gen.rs")
        || filename.ends_with(".pb.go")
        || filename.ends_with(".min.js")
        || filename.ends_with(".min.css")
}

fn is_doc_path_normalized(normalized: &str) -> bool {
    normalized.contains("/docs/")
        || normalized.starts_with("docs/")
        || normalized.ends_with(".md")
        || normalized.ends_with(".mdx")
        || normalized.ends_with(".rst")
        || normalized.ends_with(".adoc")
}

fn is_test_path_normalized(normalized: &str) -> bool {
    let filename = normalized.rsplit('/').next().unwrap_or(normalized);

    normalized.contains("/tests/")
        || normalized.starts_with("tests/")
        || normalized.contains("/test/")
        || normalized.starts_with("test/")
        || normalized.contains("/__tests__/")
        || normalized.starts_with("__tests__/")
        || normalized.contains("/spec/")
        || normalized.starts_with("spec/")
        || filename.starts_with("test_")
        || filename.ends_with("_test.rs")
        || filename.ends_with("_test.go")
        || filename.contains("_test")
        || filename.contains(".test.")
        || filename.contains(".spec.")
}

fn is_package_manifest_filename(filename: &str) -> bool {
    matches!(
        filename,
        "cargo.toml"
            | "package.json"
            | "pyproject.toml"
            | "go.mod"
            | "pom.xml"
            | "build.gradle"
            | "build.gradle.kts"
            | "composer.json"
            | "gemfile"
            | "pubspec.yaml"
            | "pubspec.yml"
            | "mix.exs"
            | "deno.json"
            | "deno.jsonc"
    ) || filename.ends_with(".csproj")
        || filename.ends_with(".fsproj")
        || filename.ends_with(".vbproj")
        || filename.ends_with(".sln")
}

fn is_build_file_path(normalized: &str, filename: &str) -> bool {
    matches!(
        filename,
        "makefile" | "cmakelists.txt" | "dockerfile" | "justfile" | "rakefile"
    ) || filename.starts_with("dockerfile.")
        || normalized.contains("/.github/workflows/")
        || normalized.starts_with(".github/workflows/")
        || normalized.contains("/.gitlab-ci.")
        || normalized.starts_with(".gitlab-ci.")
        || normalized.contains("/ci/")
        || normalized.starts_with("ci/")
}

fn is_config_path(normalized: &str, filename: &str) -> bool {
    normalized.contains("/config/")
        || normalized.starts_with("config/")
        || normalized.contains("/configs/")
        || normalized.starts_with("configs/")
        || normalized.contains("/routes/")
        || normalized.starts_with("routes/")
        || normalized.contains("/schemas/")
        || normalized.starts_with("schemas/")
        || filename.contains("config")
        || filename.contains("settings")
        || filename.contains("route")
        || filename.contains("router")
        || filename.contains("schema")
        || normalized.ends_with(".json")
        || normalized.ends_with(".toml")
        || normalized.ends_with(".yaml")
        || normalized.ends_with(".yml")
}

fn is_source_file(filename: &str) -> bool {
    filename.ends_with(".rs")
        || filename.ends_with(".py")
        || filename.ends_with(".js")
        || filename.ends_with(".jsx")
        || filename.ends_with(".ts")
        || filename.ends_with(".tsx")
        || filename.ends_with(".java")
        || filename.ends_with(".go")
        || filename.ends_with(".c")
        || filename.ends_with(".h")
        || filename.ends_with(".cc")
        || filename.ends_with(".cpp")
        || filename.ends_with(".cxx")
        || filename.ends_with(".hh")
        || filename.ends_with(".hpp")
        || filename.ends_with(".hxx")
        || filename.ends_with(".cs")
        || filename.ends_with(".sh")
        || filename.ends_with(".bash")
        || filename.ends_with(".rb")
        || filename.ends_with(".php")
        || filename.ends_with(".scala")
        || filename.ends_with(".kt")
        || filename.ends_with(".kts")
        || filename.ends_with(".swift")
        || filename.ends_with(".dart")
        || filename.ends_with(".nix")
}

#[cfg(test)]
mod tests {
    use super::{FileRole, classify_path};

    #[test]
    fn classifies_source_and_test_roles() {
        assert_eq!(classify_path("src/lib.rs"), FileRole::Source);
        assert_eq!(
            classify_path("tests/test_prompt_service.py"),
            FileRole::Test
        );
        assert_eq!(classify_path("app/foo.test.ts"), FileRole::Test);
    }

    #[test]
    fn classifies_build_package_config_and_docs_roles() {
        for path in [
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "go.mod",
            "pom.xml",
            "build.gradle",
            "composer.json",
            "Gemfile",
            "pubspec.yaml",
            "app.csproj",
            "app.fsproj",
            "app.sln",
        ] {
            assert_eq!(classify_path(path), FileRole::PackageManifest, "{path}");
        }

        for path in [
            "Makefile",
            "CMakeLists.txt",
            "Dockerfile",
            ".github/workflows/ci.yml",
        ] {
            assert_eq!(classify_path(path), FileRole::Build, "{path}");
        }

        assert_eq!(classify_path("config/app.json"), FileRole::Config);
        assert_eq!(classify_path("docs/guide.md"), FileRole::Docs);
    }

    #[test]
    fn generated_and_vendor_roles_win_before_source_like_names() {
        assert_eq!(classify_path("vendor/lib/service.py"), FileRole::Vendor);
        assert_eq!(classify_path("generated/client.rs"), FileRole::Generated);
        assert_eq!(classify_path("dist/app.min.js"), FileRole::Generated);
    }
}
