//! Shared capture privacy policy.
//!
//! Every adapter that writes user-provided capture text should call
//! [`sanitize_capture`] before inserting it into SQLite. This module lives in
//! the core crate so the CLI, desktop shell, recovery paths, and future
//! adapters cannot silently drift into separate redaction implementations.

use regex::Regex;
use std::sync::OnceLock;

pub const REDACTED: &str = "[REDACTED]";
pub const DEFAULT_MAX_CAPTURE_BYTES: usize = 1_048_576;

/// Maximum bytes retained for one captured text field.
pub fn max_capture_bytes() -> usize {
    std::env::var("MEMORYWHALE_MAX_CAPTURE_BYTES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_MAX_CAPTURE_BYTES)
}

/// Redact known secret shapes and bound the stored value.
pub fn sanitize_capture(text: &str) -> String {
    truncate_capture(&redact(text), max_capture_bytes())
}

pub fn truncate_capture(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_string();
    }
    let mut end = limit.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!(
        "{}\n[TRUNCATED: stored {} of {} bytes]",
        &text[..end],
        end,
        text.len()
    )
}

/// Scrub common secret shapes before capture text lands in SQLite.
///
/// This is intentionally conservative rather than a guarantee. Set
/// `MEMORYWHALE_NO_REDACT=1` for the explicit raw-capture opt-out.
pub fn redact(text: &str) -> String {
    if std::env::var_os("MEMORYWHALE_NO_REDACT").is_some() {
        return text.to_string();
    }
    let mut out = text.to_string();
    for re in secret_patterns() {
        out = re
            .replace_all(&out, |caps: &regex::Captures| match caps.name("label") {
                Some(label) => format!("{}{}", label.as_str(), REDACTED),
                None => REDACTED.to_string(),
            })
            .into_owned();
    }
    out
}

fn secret_patterns() -> &'static [Regex] {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        [
            r#"(?i)(?P<label>\b(?:api[_-]?key|secret|token|password|passwd|pwd|access[_-]?key|client[_-]?secret)\b\s*[:=]\s*)['"]?[A-Za-z0-9/_+\-\.]{6,}['"]?"#,
            r#"(?i)(?P<label>--(?:api[_-]?key|secret|token|password|passwd|pwd|access[_-]?key|client[_-]?secret)\s+)[^\s]+"#,
            r#"(?i)(?P<label>bearer\s+)[A-Za-z0-9._\-]{8,}"#,
            r#"AKIA[0-9A-Z]{16}"#,
            r#"gh[pousr]_[A-Za-z0-9]{20,}"#,
            r#"xox[baprs]-[A-Za-z0-9\-]{10,}"#,
            r#"eyJ[A-Za-z0-9_\-]+\.eyJ[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+"#,
            r#"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----"#,
        ]
        .iter()
        .map(|pattern| Regex::new(pattern).expect("valid secret regex"))
        .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_and_truncates_before_storage() {
        let previous = std::env::var_os("MEMORYWHALE_MAX_CAPTURE_BYTES");
        std::env::set_var("MEMORYWHALE_MAX_CAPTURE_BYTES", "24");
        std::env::remove_var("MEMORYWHALE_NO_REDACT");
        let value = sanitize_capture("token=abcdef1234567890 and a long tail");
        assert!(value.contains("token=[REDACTED]"));
        assert!(value.contains("[TRUNCATED:"));
        if let Some(value) = previous {
            std::env::set_var("MEMORYWHALE_MAX_CAPTURE_BYTES", value);
        } else {
            std::env::remove_var("MEMORYWHALE_MAX_CAPTURE_BYTES");
        }
    }
}
