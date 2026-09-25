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

/// Sanitize command arguments with awareness of flags whose value is the next
/// argument, such as `--token SECRET`. Per-argument sanitization cannot see
/// that relationship and would otherwise preserve the secret value unchanged.
pub fn sanitize_arguments(arguments: &[String]) -> Vec<String> {
    let mut sanitized = Vec::with_capacity(arguments.len());
    let mut redact_next = false;
    for argument in arguments {
        if redact_next {
            sanitized.push(if raw_capture_opt_out() {
                sanitize_capture(argument)
            } else {
                REDACTED.to_string()
            });
            redact_next = false;
            continue;
        }
        if let Some((flag, _)) = argument.split_once('=') {
            if is_secret_flag(flag) {
                sanitized.push(if raw_capture_opt_out() {
                    sanitize_capture(argument)
                } else {
                    sanitize_capture(&format!("{flag}={REDACTED}"))
                });
                continue;
            }
        }
        sanitized.push(sanitize_capture(argument));
        redact_next = is_secret_flag(argument) && !argument.contains('=');
    }
    sanitized
}

fn is_secret_flag(argument: &str) -> bool {
    let Some(flag) = argument.strip_prefix("--") else {
        return false;
    };
    let flag = strip_terminal_controls(flag);
    let flag = flag.split_once('=').map_or(flag.as_str(), |(name, _)| name);
    let normalized: String = flag
        .chars()
        .filter(|character| *character != '-' && *character != '_')
        .flat_map(char::to_lowercase)
        .collect();
    matches!(
        normalized.as_str(),
        "apikey"
            | "secret"
            | "token"
            | "password"
            | "passwd"
            | "pwd"
            | "accesskey"
            | "clientsecret"
    )
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
    if raw_capture_opt_out() {
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
    redact_control_split(out)
}

/// Remove CSI/OSC sequences and bare control characters, keeping `\n` and `\t`.
pub fn strip_terminal_controls(text: &str) -> String {
    terminal_controls().replace_all(text, "").into_owned()
}

fn terminal_controls() -> &'static Regex {
    static CONTROLS: OnceLock<Regex> = OnceLock::new();
    CONTROLS.get_or_init(|| {
        Regex::new(
            r"(?:\x1b\[|\x{009b})[0-?]*[ -/]*[@-~]|(?:\x1b\]|\x{009d})[^\x07\x1b\x{009c}]*(?:\x07|\x1b\\|\x{009c})|[\p{Cc}&&[^\n\t]]",
        )
        .expect("valid terminal control pattern")
    })
}

/// Catch secrets whose label or value is split by terminal controls, such as
/// `tok\x1ben=...` or `pass\x1b[0mword=...`. Matching runs on a
/// control-stripped view, but only the secret value's span is replaced in the
/// original, so colors and other controls elsewhere in stored output survive.
fn redact_control_split(mut out: String) -> String {
    if !terminal_controls().is_match(&out) {
        return out;
    }
    for re in secret_patterns() {
        // `visible` drops controls; `origin[i]` is the byte offset in `out` of
        // visible byte `i`. A visible char's bytes stay contiguous in `out`.
        let mut visible = String::with_capacity(out.len());
        let mut origin = Vec::with_capacity(out.len());
        let mut kept = 0;
        for control in terminal_controls()
            .find_iter(&out)
            .map(|m| m.range())
            .chain(std::iter::once(out.len()..out.len()))
        {
            visible.push_str(&out[kept..control.start]);
            origin.extend(kept..control.start);
            kept = control.end;
        }
        let mut redacted = String::with_capacity(out.len());
        let mut cursor = 0;
        for caps in re.captures_iter(&visible) {
            let whole = caps.get(0).expect("whole match");
            let value_start = caps
                .name("label")
                .map_or(whole.start(), |label| label.end());
            if value_start >= whole.end() {
                continue;
            }
            let start = origin[value_start];
            redacted.push_str(&out[cursor..start]);
            redacted.push_str(REDACTED);
            cursor = origin[whole.end() - 1] + 1;
        }
        if cursor > 0 {
            redacted.push_str(&out[cursor..]);
            out = redacted;
        }
    }
    out
}

fn raw_capture_opt_out() -> bool {
    std::env::var("MEMORYWHALE_NO_REDACT").ok().as_deref() == Some("1")
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
    use std::ffi::OsString;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        no_redact: Option<OsString>,
        max_bytes: Option<OsString>,
    }

    impl EnvGuard {
        fn new() -> Self {
            Self {
                no_redact: std::env::var_os("MEMORYWHALE_NO_REDACT"),
                max_bytes: std::env::var_os("MEMORYWHALE_MAX_CAPTURE_BYTES"),
            }
        }

        fn clear_no_redact(&self) {
            std::env::remove_var("MEMORYWHALE_NO_REDACT");
        }

        fn clear_max_bytes(&self) {
            std::env::remove_var("MEMORYWHALE_MAX_CAPTURE_BYTES");
        }

        fn set_no_redact(&self, value: &str) {
            std::env::set_var("MEMORYWHALE_NO_REDACT", value);
        }

        fn set_max_bytes(&self, value: &str) {
            std::env::set_var("MEMORYWHALE_MAX_CAPTURE_BYTES", value);
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = self.no_redact.take() {
                std::env::set_var("MEMORYWHALE_NO_REDACT", value);
            } else {
                std::env::remove_var("MEMORYWHALE_NO_REDACT");
            }
            if let Some(value) = self.max_bytes.take() {
                std::env::set_var("MEMORYWHALE_MAX_CAPTURE_BYTES", value);
            } else {
                std::env::remove_var("MEMORYWHALE_MAX_CAPTURE_BYTES");
            }
        }
    }

    #[test]
    fn redacts_and_truncates_before_storage() {
        let _guard = ENV_LOCK.lock().unwrap();
        let env = EnvGuard::new();
        env.set_max_bytes("24");
        env.clear_no_redact();
        assert!(redact("token=abcdef1234567890").contains("token=[REDACTED]"));
        let value = sanitize_capture("token=abcdef1234567890 and a long tail");
        assert!(!value.contains("abcdef1234567890"));
        assert!(value.contains("[TRUNCATED:") || value.starts_with("[TRUNC"));
        assert!(value.len() <= 24);
    }

    #[test]
    fn sanitizes_split_and_equals_secret_arguments() {
        let _guard = ENV_LOCK.lock().unwrap();
        let env = EnvGuard::new();
        env.clear_no_redact();
        env.clear_max_bytes();
        let arguments = vec![
            "curl".to_string(),
            "--token".to_string(),
            "hunter2secret99".to_string(),
            "--password=a!".to_string(),
            "--api_key".to_string(),
            "short".to_string(),
            "--accesskey".to_string(),
            "special value!".to_string(),
            "--clientsecret=short!".to_string(),
        ];
        assert_eq!(
            sanitize_arguments(&arguments),
            [
                "curl",
                "--token",
                "[REDACTED]",
                "--password=[REDACTED]",
                "--api_key",
                "[REDACTED]",
                "--accesskey",
                "[REDACTED]",
                "--clientsecret=[REDACTED]"
            ]
        );
    }

    #[test]
    fn split_arguments_preserve_raw_capture_opt_out() {
        let _guard = ENV_LOCK.lock().unwrap();
        let env = EnvGuard::new();
        env.set_no_redact("1");
        env.clear_max_bytes();
        let arguments = vec![
            "--token".to_string(),
            "short!".to_string(),
            "--password=a!".to_string(),
        ];
        assert_eq!(
            sanitize_arguments(&arguments),
            ["--token", "short!", "--password=a!"]
        );
    }

    #[test]
    fn equals_arguments_respect_capture_limit() {
        let _guard = ENV_LOCK.lock().unwrap();
        let env = EnvGuard::new();
        env.clear_no_redact();
        env.set_max_bytes("12");
        let arguments = vec!["--password=very-long-secret".to_string()];
        let sanitized = sanitize_arguments(&arguments);
        assert!(sanitized[0].len() <= 12);
    }

    #[test]
    fn redacts_secrets_split_by_terminal_controls() {
        let _guard = ENV_LOCK.lock().unwrap();
        let env = EnvGuard::new();
        env.clear_no_redact();
        env.clear_max_bytes();
        for input in [
            "tok\x1ben=abc123secret",
            "pass\x1b[31mword=abc123secret",
            "api\x1b]0;title\x07_key: abc123secret",
            "token=abc\x1b[0m123secret",
            "gh\x1b[1mp_0123456789abcdefghijABCDEF",
            "tok\ren=abc123secret",
        ] {
            let redacted = redact(input);
            assert!(redacted.contains(REDACTED), "{input:?} -> {redacted:?}");
            assert!(
                !strip_terminal_controls(&redacted).contains("123secret")
                    && !redacted.contains("0123456789abcdef"),
                "{input:?} -> {redacted:?}"
            );
        }
        // Controls outside the secret value are preserved.
        assert_eq!(
            redact("\x1b[31mtok\x1ben=abc123secret\x1b[0m done"),
            "\x1b[31mtok\x1ben=[REDACTED]\x1b[0m done"
        );
        assert_eq!(redact("\x1b[32mok\x1b[0m"), "\x1b[32mok\x1b[0m");
        assert_eq!(
            sanitize_arguments(&["--to\x1bken".to_string(), "short".to_string()]),
            ["--to\x1bken", "[REDACTED]"]
        );
    }

    #[test]
    fn raw_capture_opt_out_requires_value_one() {
        let _guard = ENV_LOCK.lock().unwrap();
        let env = EnvGuard::new();
        env.clear_max_bytes();
        env.set_no_redact("0");
        assert!(!redact("token=abcdef123456").contains("abcdef123456"));
        env.set_no_redact("1");
        assert!(redact("token=abcdef123456").contains("abcdef123456"));
    }
}
