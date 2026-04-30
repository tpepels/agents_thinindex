use std::{fs, path::Path};

use tempfile::TempDir;
use thinindex::licensing::{
    Edition, LICENSE_SCHEMA_VERSION, LOCAL_TEST_LICENSE_PREFIX, LOCAL_TEST_SIGNATURE,
    LOCAL_TEST_VALIDATION, LicenseState, read_license_status,
};

fn repo_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn missing_local_license_file_keeps_free_core_available() {
    let temp = TempDir::new().expect("create temp dir");
    let status = read_license_status(temp.path().join("missing-license.json"));

    assert_eq!(status.edition, Edition::Free);
    assert_eq!(status.state, LicenseState::NoLicenseFile);
    assert!(
        status
            .reason
            .contains("free local edition remains available"),
        "unexpected reason: {}",
        status.reason
    );
}

#[test]
fn explicit_local_test_fixture_reports_pro_without_enforcement() {
    let temp = TempDir::new().expect("create temp dir");
    let path = temp.path().join("license.json");

    fs::write(
        &path,
        format!(
            r#"{{
                "schema_version": {LICENSE_SCHEMA_VERSION},
                "edition": "pro",
                "license_id": "{LOCAL_TEST_LICENSE_PREFIX}integration",
                "validation": "{LOCAL_TEST_VALIDATION}",
                "signature": "{LOCAL_TEST_SIGNATURE}"
            }}"#
        ),
    )
    .expect("write license fixture");

    let status = read_license_status(&path);
    assert_eq!(status.edition, Edition::Pro);
    assert_eq!(status.state, LicenseState::ValidLocalTestFixture);
    assert_eq!(status.path.as_deref(), Some(path.as_path()));
}

#[test]
fn production_like_pro_license_is_not_accepted_by_stub() {
    let temp = TempDir::new().expect("create temp dir");
    let path = temp.path().join("license.json");

    fs::write(
        &path,
        format!(
            r#"{{
                "schema_version": {LICENSE_SCHEMA_VERSION},
                "edition": "pro",
                "license_id": "future-production-license",
                "validation": "server",
                "signature": "not-implemented"
            }}"#
        ),
    )
    .expect("write license fixture");

    let status = read_license_status(&path);
    assert_eq!(status.edition, Edition::UnknownUnlicensed);
    assert_eq!(status.state, LicenseState::Invalid);
    assert!(
        status.reason.contains("local test fixture"),
        "unexpected reason: {}",
        status.reason
    );
}

#[test]
fn licensing_docs_are_honest_about_no_enforcement_or_network_activation() {
    let docs = repo_file("docs/LICENSING.md");
    let product_boundary = repo_file("docs/PRODUCT_BOUNDARY.md");
    let readme = repo_file("README.md");

    for required in [
        "No license enforcement is active",
        "No payment integration exists",
        "No network activation exists",
        "No telemetry",
        "free local core remains available",
        "THININDEX_LICENSE_FILE",
        "local-test-fixture",
    ] {
        assert!(
            docs.contains(required),
            "LICENSING.md should mention {required}"
        );
    }

    assert!(
        product_boundary.contains("license state model")
            && product_boundary.contains("no paid gates")
            && product_boundary.contains("free local core remains available"),
        "product boundary should document the inert licensing foundation"
    );
    assert!(
        readme.contains("docs/LICENSING.md")
            && readme.contains("No current command is blocked by license status"),
        "README should link licensing docs without claiming current gates"
    );

    for forbidden in [
        "paid features are currently gated",
        "license server is required",
        "network activation is required",
        "telemetry is required",
    ] {
        assert!(
            !docs.contains(forbidden)
                && !product_boundary.contains(forbidden)
                && !readme.contains(forbidden),
            "docs must not claim {forbidden}"
        );
    }
}
