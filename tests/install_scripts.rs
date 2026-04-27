use std::{fs, path::Path};

#[test]
fn install_script_includes_wi_stats() {
    let install_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("install.sh");
    let contents = fs::read_to_string(&install_path).expect("read install.sh");
    assert!(
        contents.contains("wi-stats"),
        "install.sh should reference wi-stats"
    );
}

#[test]
fn uninstall_script_includes_wi_stats() {
    let uninstall_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("uninstall.sh");
    let contents = fs::read_to_string(&uninstall_path).expect("read uninstall.sh");
    assert!(
        contents.contains("wi-stats"),
        "uninstall.sh should reference wi-stats"
    );
}
