use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

pub const LICENSE_SCHEMA_VERSION: u32 = 1;
pub const LICENSE_FILE_ENV: &str = "THININDEX_LICENSE_FILE";
pub const LOCAL_TEST_VALIDATION: &str = "local-test-fixture";
pub const LOCAL_TEST_SIGNATURE: &str = "thinindex-local-test-fixture";
pub const LOCAL_TEST_LICENSE_PREFIX: &str = "thinindex-local-test-";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edition {
    Free,
    Pro,
    UnknownUnlicensed,
}

impl Edition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Pro => "pro",
            Self::UnknownUnlicensed => "unknown/unlicensed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseState {
    NoLicenseFile,
    ExplicitFree,
    ValidLocalTestFixture,
    Invalid,
}

impl LicenseState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoLicenseFile => "no_license_file",
            Self::ExplicitFree => "explicit_free",
            Self::ValidLocalTestFixture => "valid_local_test_fixture",
            Self::Invalid => "invalid",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseStatus {
    pub edition: Edition,
    pub state: LicenseState,
    pub path: Option<PathBuf>,
    pub reason: &'static str,
}

impl LicenseStatus {
    pub fn free_without_file(path: Option<PathBuf>) -> Self {
        Self {
            edition: Edition::Free,
            state: LicenseState::NoLicenseFile,
            path,
            reason: "free local edition remains available; no license file was found",
        }
    }

    fn explicit_free(path: Option<PathBuf>) -> Self {
        Self {
            edition: Edition::Free,
            state: LicenseState::ExplicitFree,
            path,
            reason: "explicit free local license file",
        }
    }

    fn local_test_pro(path: Option<PathBuf>) -> Self {
        Self {
            edition: Edition::Pro,
            state: LicenseState::ValidLocalTestFixture,
            path,
            reason: "accepted local test fixture only",
        }
    }

    fn invalid(path: Option<PathBuf>, reason: &'static str) -> Self {
        Self {
            edition: Edition::UnknownUnlicensed,
            state: LicenseState::Invalid,
            path,
            reason,
        }
    }
}

pub fn configured_license_path() -> Option<PathBuf> {
    if let Some(path) = non_empty_env_path(LICENSE_FILE_ENV) {
        return Some(path);
    }

    default_license_path_from_env()
}

pub fn read_configured_license_status() -> LicenseStatus {
    let Some(path) = configured_license_path() else {
        return LicenseStatus::free_without_file(None);
    };

    read_license_status(&path)
}

pub fn read_license_status(path: impl AsRef<Path>) -> LicenseStatus {
    let path = path.as_ref();
    let status_path = Some(path.to_path_buf());

    if !path.exists() {
        return LicenseStatus::free_without_file(status_path);
    }

    match fs::read_to_string(path) {
        Ok(contents) => license_status_from_json(&contents, status_path),
        Err(_) => LicenseStatus::invalid(status_path, "license file could not be read"),
    }
}

pub fn license_status_from_json(contents: &str, path: Option<PathBuf>) -> LicenseStatus {
    let Ok(file) = serde_json::from_str::<LicenseFile>(contents) else {
        return LicenseStatus::invalid(path, "license file is not valid JSON");
    };

    if file.schema_version != LICENSE_SCHEMA_VERSION {
        return LicenseStatus::invalid(path, "unsupported license schema version");
    }

    match file.edition.as_str() {
        "free" => LicenseStatus::explicit_free(path),
        "pro" if file.is_valid_local_test_fixture() => LicenseStatus::local_test_pro(path),
        "pro" => LicenseStatus::invalid(path, "pro license is not an accepted local test fixture"),
        _ => LicenseStatus::invalid(path, "unknown license edition"),
    }
}

pub fn license_path_from_parts(
    explicit_file: Option<&Path>,
    xdg_config_home: Option<&Path>,
    home: Option<&Path>,
    appdata: Option<&Path>,
) -> Option<PathBuf> {
    if let Some(path) = explicit_file {
        return Some(path.to_path_buf());
    }

    if cfg!(windows) {
        return appdata.map(|path| path.join("thinindex").join("license.json"));
    }

    if let Some(path) = xdg_config_home {
        return Some(path.join("thinindex").join("license.json"));
    }

    home.map(|path| path.join(".config").join("thinindex").join("license.json"))
}

fn default_license_path_from_env() -> Option<PathBuf> {
    license_path_from_parts(
        None,
        non_empty_env_path("XDG_CONFIG_HOME").as_deref(),
        non_empty_env_path("HOME").as_deref(),
        non_empty_env_path("APPDATA").as_deref(),
    )
}

fn non_empty_env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[derive(Debug, Deserialize)]
struct LicenseFile {
    #[serde(default)]
    schema_version: u32,
    edition: String,
    #[serde(default)]
    license_id: Option<String>,
    #[serde(default)]
    validation: Option<String>,
    #[serde(default)]
    signature: Option<String>,
}

impl LicenseFile {
    fn is_valid_local_test_fixture(&self) -> bool {
        self.validation.as_deref() == Some(LOCAL_TEST_VALIDATION)
            && self.signature.as_deref() == Some(LOCAL_TEST_SIGNATURE)
            && self
                .license_id
                .as_deref()
                .is_some_and(|id| id.starts_with(LOCAL_TEST_LICENSE_PREFIX))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        Edition, LICENSE_SCHEMA_VERSION, LOCAL_TEST_LICENSE_PREFIX, LOCAL_TEST_SIGNATURE,
        LOCAL_TEST_VALIDATION, LicenseState, license_path_from_parts, license_status_from_json,
    };

    #[test]
    fn valid_local_test_fixture_can_report_pro() {
        let status = license_status_from_json(
            &format!(
                r#"{{
                    "schema_version": {LICENSE_SCHEMA_VERSION},
                    "edition": "pro",
                    "license_id": "{LOCAL_TEST_LICENSE_PREFIX}fixture",
                    "validation": "{LOCAL_TEST_VALIDATION}",
                    "signature": "{LOCAL_TEST_SIGNATURE}"
                }}"#
            ),
            None,
        );

        assert_eq!(status.edition, Edition::Pro);
        assert_eq!(status.state, LicenseState::ValidLocalTestFixture);
    }

    #[test]
    fn pro_without_fixture_markers_is_not_accepted() {
        let status = license_status_from_json(
            &format!(
                r#"{{
                    "schema_version": {LICENSE_SCHEMA_VERSION},
                    "edition": "pro",
                    "license_id": "prod-license",
                    "validation": "future-server",
                    "signature": "not-implemented"
                }}"#
            ),
            None,
        );

        assert_eq!(status.edition, Edition::UnknownUnlicensed);
        assert_eq!(status.state, LicenseState::Invalid);
    }

    #[test]
    fn explicit_free_license_reports_free() {
        let status = license_status_from_json(
            &format!(
                r#"{{
                    "schema_version": {LICENSE_SCHEMA_VERSION},
                    "edition": "free"
                }}"#
            ),
            None,
        );

        assert_eq!(status.edition, Edition::Free);
        assert_eq!(status.state, LicenseState::ExplicitFree);
    }

    #[test]
    fn license_path_design_prefers_explicit_then_config_dirs() {
        let explicit = Path::new("/tmp/thinindex-license.json");
        assert_eq!(
            license_path_from_parts(Some(explicit), None, None, None),
            Some(explicit.to_path_buf())
        );

        if cfg!(windows) {
            assert_eq!(
                license_path_from_parts(
                    None,
                    Some(Path::new("/xdg")),
                    Some(Path::new("/home/user")),
                    Some(Path::new("C:/Users/user/AppData/Roaming")),
                ),
                Some(
                    Path::new("C:/Users/user/AppData/Roaming/thinindex/license.json").to_path_buf()
                )
            );
        } else {
            assert_eq!(
                license_path_from_parts(
                    None,
                    Some(Path::new("/xdg")),
                    Some(Path::new("/home/user")),
                    None,
                ),
                Some(Path::new("/xdg/thinindex/license.json").to_path_buf())
            );
            assert_eq!(
                license_path_from_parts(None, None, Some(Path::new("/home/user")), None),
                Some(Path::new("/home/user/.config/thinindex/license.json").to_path_buf())
            );
        }
    }
}
