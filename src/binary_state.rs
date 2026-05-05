use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::model::INDEX_SCHEMA_VERSION;

pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCheckoutInfo {
    pub version: Option<String>,
    pub schema_version: Option<u32>,
    pub cargo_toml: PathBuf,
    pub model_rs: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinarySourceMismatch {
    pub source: SourceCheckoutInfo,
    pub binary_name: &'static str,
}

pub fn version_with_schema() -> String {
    format!("{PACKAGE_VERSION} (index schema {INDEX_SCHEMA_VERSION})")
}

pub fn print_version_if_requested(binary_name: &str) -> bool {
    let mut args = env::args_os().skip(1);
    let Some(first) = args.next() else {
        return false;
    };

    if args.next().is_some() {
        return false;
    }

    if first == "--version" || first == "-V" {
        println!("{binary_name} {}", version_with_schema());
        true
    } else {
        false
    }
}

pub fn source_checkout_info(root: &Path) -> Option<SourceCheckoutInfo> {
    let cargo_toml = root.join("Cargo.toml");
    let model_rs = root.join("src/model.rs");
    let cargo = fs::read_to_string(&cargo_toml).ok()?;

    if !package_name_is_thinindex(&cargo) || !model_rs.exists() {
        return None;
    }

    let model = fs::read_to_string(&model_rs).ok();

    Some(SourceCheckoutInfo {
        version: parse_manifest_version(&cargo),
        schema_version: model.as_deref().and_then(parse_schema_version),
        cargo_toml,
        model_rs,
    })
}

pub fn source_binary_mismatch(
    root: &Path,
    binary_name: &'static str,
) -> Option<BinarySourceMismatch> {
    let source = source_checkout_info(root)?;
    let version_mismatch = source
        .version
        .as_deref()
        .is_some_and(|version| version != PACKAGE_VERSION);
    let schema_mismatch = source
        .schema_version
        .is_some_and(|schema_version| schema_version != INDEX_SCHEMA_VERSION);

    if version_mismatch || schema_mismatch {
        Some(BinarySourceMismatch {
            source,
            binary_name,
        })
    } else {
        None
    }
}

pub fn ensure_binary_matches_source(root: &Path, binary_name: &'static str) -> Result<()> {
    if let Some(mismatch) = source_binary_mismatch(root, binary_name) {
        anyhow::bail!("{}", mismatch.error_message());
    }

    Ok(())
}

impl BinarySourceMismatch {
    pub fn error_message(&self) -> String {
        format!(
            "installed `{}` does not match this thinindex source checkout\nbinary: version {} schema {}\nsource: version {} schema {}\nnext: run `./install.sh` from {} or use `cargo run --bin {}` from this checkout",
            self.binary_name,
            PACKAGE_VERSION,
            INDEX_SCHEMA_VERSION,
            self.source.version.as_deref().unwrap_or("unknown"),
            self.source
                .schema_version
                .map(|version| version.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            self.source
                .cargo_toml
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .display(),
            self.binary_name,
        )
    }

    pub fn doctor_message(&self) -> String {
        format!(
            "running {} {} schema {}, but source checkout declares version {} schema {}",
            self.binary_name,
            PACKAGE_VERSION,
            INDEX_SCHEMA_VERSION,
            self.source.version.as_deref().unwrap_or("unknown"),
            self.source
                .schema_version
                .map(|version| version.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        )
    }
}

fn package_name_is_thinindex(cargo_toml: &str) -> bool {
    cargo_toml
        .lines()
        .any(|line| manifest_value(line, "name").is_some_and(|value| value == "thinindex"))
}

fn parse_manifest_version(cargo_toml: &str) -> Option<String> {
    cargo_toml
        .lines()
        .find_map(|line| manifest_value(line, "version"))
        .map(ToOwned::to_owned)
}

fn manifest_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let trimmed = line.trim();
    let (line_key, value) = trimmed.split_once('=')?;
    if line_key.trim() != key {
        return None;
    }

    Some(value.trim().trim_matches('"'))
}

fn parse_schema_version(model_rs: &str) -> Option<u32> {
    model_rs.lines().find_map(|line| {
        let trimmed = line.trim();
        let value = trimmed
            .strip_prefix("pub const INDEX_SCHEMA_VERSION: u32 = ")?
            .strip_suffix(';')?;
        value.parse().ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_schema_text_tracks_schema_constant() {
        assert!(
            version_with_schema().contains(&format!("index schema {INDEX_SCHEMA_VERSION}")),
            "version output should include the compiled index schema"
        );
    }

    #[test]
    fn parses_source_checkout_schema_and_version() {
        let cargo = r#"
[package]
name = "thinindex"
version = "1.2.3"
"#;
        let model = "pub const INDEX_SCHEMA_VERSION: u32 = 99;\n";

        assert!(package_name_is_thinindex(cargo));
        assert_eq!(parse_manifest_version(cargo).as_deref(), Some("1.2.3"));
        assert_eq!(parse_schema_version(model), Some(99));
    }
}
