use std::sync::LazyLock;

use regex::Regex;

pub const SENSITIVE_PATH_WARNING_LIMIT: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitivePathWarning {
    pub path: String,
    pub reason: &'static str,
}

pub fn sensitive_path_reason(path: &str) -> Option<&'static str> {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    let file_name = normalized.rsplit('/').next().unwrap_or(normalized.as_str());

    if matches!(
        file_name,
        ".env"
            | ".env.local"
            | ".env.production"
            | ".env.development"
            | ".envrc"
            | ".netrc"
            | ".npmrc"
            | ".pypirc"
            | "credentials"
            | "credentials.json"
            | "id_rsa"
            | "id_dsa"
            | "id_ecdsa"
            | "id_ed25519"
    ) {
        return Some("sensitive filename");
    }

    if file_name.ends_with(".pem")
        || file_name.ends_with(".key")
        || file_name.ends_with(".p12")
        || file_name.ends_with(".pfx")
    {
        return Some("private key or certificate-like extension");
    }

    if normalized
        .split('/')
        .any(|part| matches!(part, "secrets" | "secret" | "credentials" | "private_keys"))
    {
        return Some("sensitive directory name");
    }

    None
}

pub fn redact_sensitive_text(value: &str) -> String {
    let mut redacted = value.to_string();

    for regex in [
        &*KEY_VALUE_SECRET_RE,
        &*AWS_ACCESS_KEY_RE,
        &*GITHUB_TOKEN_RE,
        &*JWT_RE,
        &*PRIVATE_KEY_HEADER_RE,
    ] {
        redacted = regex
            .replace_all(&redacted, |captures: &regex::Captures<'_>| {
                if captures.len() > 2 {
                    format!("{}[REDACTED]", &captures[1])
                } else {
                    "[REDACTED]".to_string()
                }
            })
            .into_owned();
    }

    redacted
}

static KEY_VALUE_SECRET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)\b((?:api[_-]?key|access[_-]?token|auth[_-]?token|client[_-]?secret|password|passwd|private[_-]?key|secret|token)\s*[:=]\s*)("[^"\s]+"|'[^'\s]+'|[^\s"',;]+)"#,
    )
    .expect("valid key/value secret redaction regex")
});

static AWS_ACCESS_KEY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b").expect("valid AWS key regex"));

static GITHUB_TOKEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bgh[pousr]_[A-Za-z0-9_]{20,}\b").expect("valid token regex"));

static JWT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b")
        .expect("valid JWT regex")
});

static PRIVATE_KEY_HEADER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----").expect("valid private key regex")
});

#[cfg(test)]
mod tests {
    use super::{redact_sensitive_text, sensitive_path_reason};

    #[test]
    fn redacts_common_secret_values() {
        let text = "password=hunter2 api_key=\"abc123\" token='secret-token'";
        let redacted = redact_sensitive_text(text);

        assert!(redacted.contains("password=[REDACTED]"));
        assert!(redacted.contains("api_key=[REDACTED]"));
        assert!(redacted.contains("token=[REDACTED]"));
        assert!(!redacted.contains("hunter2"));
        assert!(!redacted.contains("abc123"));
        assert!(!redacted.contains("secret-token"));
    }

    #[test]
    fn flags_sensitive_looking_paths() {
        assert_eq!(
            sensitive_path_reason("config/.env.production"),
            Some("sensitive filename")
        );
        assert_eq!(
            sensitive_path_reason("secrets/prod.key"),
            Some("private key or certificate-like extension")
        );
        assert_eq!(sensitive_path_reason("src/lib.rs"), None);
    }
}
