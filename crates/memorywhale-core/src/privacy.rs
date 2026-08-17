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
    let mut content_limit = limit;
    for _ in 0..8 {
        let stored = utf8_prefix_len(text, content_limit);
        let marker = format!("\n[TRUNCATED: stored {stored} of {} bytes]", text.len());
        if marker.len() >= limit {
            return take_utf8_prefix("[TRUNCATED]", limit);
        }
        let next_limit = limit - marker.len();
        if next_limit == content_limit {
            return format!("{}{}", &text[..stored], marker);
        }
        content_limit = next_limit;
    }
    take_utf8_prefix("[TRUNCATED]", limit)
}

fn take_utf8_prefix(text: &str, limit: usize) -> String {
    text[..utf8_prefix_len(text, limit)].to_string()
}

fn utf8_prefix_len(text: &str, limit: usize) -> usize {
    let mut end = limit.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    end
}

/// Scrub common secret shapes before capture text lands in SQLite.
///
/// This is intentionally conservative rather than a guarantee. Set
/// `MEMORYWHALE_NO_REDACT=1` for the explicit raw-capture opt-out.
pub fn redact(text: &str) -> String {
    if std::env::var("MEMORYWHALE_NO_REDACT").ok().as_deref() == Some("1") {
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
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn redacts_and_truncates_before_storage() {
        let _guard = ENV_LOCK.lock().unwrap();
        let previous = std::env::var_os("MEMORYWHALE_MAX_CAPTURE_BYTES");
        std::env::set_var("MEMORYWHALE_MAX_CAPTURE_BYTES", "24");
        std::env::remove_var("MEMORYWHALE_NO_REDACT");
        assert!(redact("token=abcdef1234567890").contains("token=[REDACTED]"));
        let value = sanitize_capture("token=abcdef1234567890 and a long tail");
        assert!(!value.contains("abcdef1234567890"));
        assert!(value.contains("[TRUNCATED:") || value.starts_with("[TRUNC"));
        assert!(value.len() <= 24);
        if let Some(value) = previous {
            std::env::set_var("MEMORYWHALE_MAX_CAPTURE_BYTES", value);
        } else {
            std::env::remove_var("MEMORYWHALE_MAX_CAPTURE_BYTES");
        }
    }

    #[test]
    fn raw_capture_opt_out_requires_value_one() {
        let _guard = ENV_LOCK.lock().unwrap();
        let previous = std::env::var_os("MEMORYWHALE_NO_REDACT");
        std::env::set_var("MEMORYWHALE_NO_REDACT", "0");
        assert!(!redact("token=abcdef123456").contains("abcdef123456"));
        std::env::set_var("MEMORYWHALE_NO_REDACT", "1");
        assert!(redact("token=abcdef123456").contains("abcdef123456"));
        if let Some(value) = previous {
            std::env::set_var("MEMORYWHALE_NO_REDACT", value);
        } else {
            std::env::remove_var("MEMORYWHALE_NO_REDACT");
        }
    }
}
