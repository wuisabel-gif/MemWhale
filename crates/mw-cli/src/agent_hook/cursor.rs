//! Documentation-derived local Cursor Agent/Shell hook contract.
//! Only the generic outcome pair is consumed; afterShellExecution is deliberately
//! ignored so one execution is not captured through two overlapping surfaces.
use std::path::Path;

use serde_json::{json, Map, Value};

use crate::remember::CommandRecord;

const TEXT_LIMIT: usize = 20_000;
const COMMAND_LIMIT: usize = 8 * 1024;
const ID_LIMIT: usize = 256;

fn bounded(text: &str, limit: usize) -> (String, bool) {
    if text.len() <= limit {
        return (text.to_owned(), false);
    }
    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    (text[..end].to_owned(), true)
}

fn output(text: &str) -> (String, bool) {
    const MARKER: &str = "\n[cursor: output truncated]";
    if text.len() <= TEXT_LIMIT {
        return (text.to_owned(), false);
    }
    let (mut text, _) = bounded(text, TEXT_LIMIT - MARKER.len());
    text.push_str(MARKER);
    (text, true)
}

pub(super) fn parse(payload: &Value) -> Result<Option<CommandRecord>, &'static str> {
    let payload = payload
        .as_object()
        .ok_or("Cursor hook input must be an object")?;
    let event = payload
        .get("hook_event_name")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !matches!(event, "postToolUse" | "postToolUseFailure")
        || payload.get("tool_name").and_then(Value::as_str) != Some("Shell")
    {
        return Ok(None);
    }
    let command = payload
        .get("tool_input")
        .and_then(Value::as_object)
        .and_then(|input| input.get("command"))
        .and_then(Value::as_str)
        .ok_or("Cursor Shell event has no command; event skipped")?;
    if command.trim().is_empty() || command.len() > COMMAND_LIMIT || command.contains('\0') {
        return Err(
            "Cursor command is empty, invalid, or oversized; event skipped without truncating it",
        );
    }
    // Unknown cwd cannot safely honor per-directory capture exclusions. Never
    // substitute the hook process cwd or workspace_roots for an actual cwd.
    let cwd = payload
        .get("cwd")
        .and_then(Value::as_str)
        .filter(|cwd| !cwd.contains('\0') && cwd.len() <= 4096 && Path::new(cwd).is_absolute())
        .ok_or(
            "Cursor event has no valid absolute cwd; event skipped to preserve capture policy",
        )?;
    let mut metadata = Map::new();
    metadata.insert("event".into(), json!(event));
    for key in [
        "conversation_id",
        "generation_id",
        "tool_use_id",
        "cursor_version",
    ] {
        if let Some(value) = payload.get(key).and_then(Value::as_str) {
            let (value, truncated) = bounded(value, ID_LIMIT);
            metadata.insert(key.into(), json!(value));
            if truncated {
                metadata.insert(format!("{key}_truncated"), json!(true));
            }
        }
    }
    if let Some(duration) = payload.get("duration").and_then(Value::as_u64) {
        metadata.insert("duration_ms".into(), json!(duration));
    }
    if let Some(sandbox) = payload.get("sandbox").and_then(Value::as_bool) {
        metadata.insert("sandbox".into(), json!(sandbox));
    }
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut exit_code = None;
    let status;
    if event == "postToolUse" {
        // Cursor documents tool_output as a JSON-stringified result, not raw
        // terminal text. Unknown/malformed shapes do not become fabricated output.
        let decoded = payload
            .get("tool_output")
            .and_then(Value::as_str)
            .and_then(|text| serde_json::from_str::<Value>(text).ok());
        if let Some(result) = decoded.as_ref().and_then(Value::as_object) {
            for (field, destination) in [("stdout", &mut stdout), ("stderr", &mut stderr)] {
                if let Some(text) = result.get(field).and_then(Value::as_str) {
                    let (text, truncated) = output(text);
                    *destination = text;
                    if truncated {
                        metadata.insert(format!("{field}_truncated"), json!(true));
                    }
                }
            }
            exit_code = result
                .get("exitCode")
                .and_then(Value::as_i64)
                .filter(|code| (0..=255).contains(code));
            if stdout.is_empty()
                && stderr.is_empty()
                && !result.contains_key("stdout")
                && !result.contains_key("stderr")
            {
                metadata.insert("output_unavailable".into(), json!(true));
            }
        } else {
            metadata.insert("output_unavailable".into(), json!(true));
        }
        status = match exit_code {
            Some(0) => "exit_zero",
            Some(_) => "nonzero_exit",
            None => "completed_exit_unknown",
        };
        metadata.insert("execution".into(), json!("client_reported_result"));
    } else {
        let failure = payload
            .get("failure_type")
            .and_then(Value::as_str)
            .unwrap_or("");
        let interrupted = payload.get("is_interrupt").and_then(Value::as_bool);
        if let Some(interrupted) = interrupted {
            metadata.insert("is_interrupt".into(), json!(interrupted));
        }
        status = if failure == "permission_denied" {
            "permission_denied"
        } else if interrupted == Some(true) {
            "cancelled"
        } else if failure == "timeout" {
            "timeout"
        } else {
            "tool_failure"
        };
        if !failure.is_empty() {
            let (failure, truncated) = bounded(failure, ID_LIMIT);
            metadata.insert("failure_type".into(), json!(failure));
            if truncated {
                metadata.insert("failure_type_truncated".into(), json!(true));
            }
        }
        metadata.insert(
            "execution".into(),
            json!(if failure == "permission_denied" {
                "not_executed"
            } else {
                "not_confirmed"
            }),
        );
        if let Some(message) = payload.get("error_message").and_then(Value::as_str) {
            let (message, truncated) = output(message);
            stderr = message;
            if truncated {
                metadata.insert("stderr_truncated".into(), json!(true));
            }
        }
        // An error/timeout/denial is not an observed numeric process exit code.
    }
    metadata.insert("exit_code_known".into(), json!(exit_code.is_some()));
    Ok(Some(CommandRecord {
        cwd: Some(cwd.to_owned()),
        command_parts: vec![command.to_owned()],
        stdout,
        stderr,
        exit_code,
        notes: format!(
            "agent:cursor cursor_status:{status} cursor_metadata:{}",
            Value::Object(metadata)
        ),
        capture_kind: "full".into(),
        agent: Some("cursor".into()),
    }))
}

#[cfg(test)]
#[path = "cursor_tests.rs"]
mod tests;
