//! Codewhale's versioned post-admission execution receipt, not legacy hook env.
//! No pre-hook join: requested inputs may be rewritten before execution.
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;

use crate::remember::CommandRecord;

pub const MAX_HOOK_BYTES: u64 = 128 * 1024;

#[derive(Deserialize)]
struct Envelope {
    schema_version: u64,
    event: String,
    tool_name: Option<String>,
    session_id: Option<String>,
    tool_call_id: Option<String>,
    session_id_truncated: bool,
    tool_call_id_truncated: bool,
    tool_name_truncated: bool,
    execution_receipt: Option<Receipt>,
}

#[derive(Deserialize)]
struct Receipt {
    schema_version: u64,
    command: String,
    cwd: String,
    command_truncated: bool,
    cwd_truncated: bool,
    execution: String,
    completion: String,
    exit_code: Option<i64>,
    stdout: String,
    stderr: String,
    stdout_truncated: bool,
    stderr_truncated: bool,
    #[serde(default = "default_output_mode")]
    output_mode: String,
}

fn default_output_mode() -> String {
    "separate".to_string()
}

pub fn record_from_slice(bytes: &[u8]) -> Result<Option<CommandRecord>, &'static str> {
    if bytes.len() as u64 > MAX_HOOK_BYTES {
        return Err("Codewhale receipt exceeds 128 KiB; event skipped");
    }
    let envelope = serde_json::from_slice(bytes)
        .map_err(|_| "invalid Codewhale receipt JSON; event skipped")?;
    parse(envelope)
}

pub(super) fn record_from_value(value: &Value) -> Option<CommandRecord> {
    parse(serde_json::from_value(value.clone()).ok()?)
        .ok()
        .flatten()
}

fn identifier(value: Option<String>) -> Result<String, &'static str> {
    value
        .filter(|v| !v.trim().is_empty() && v.len() <= 256 && !v.chars().any(char::is_control))
        .ok_or("Codewhale correlation identifier unavailable; event skipped")
}

fn output(mut text: String, upstream_truncated: bool, stream: &str) -> String {
    let local_truncated = text.len() > 20_000;
    if local_truncated {
        let mut end = 20_000;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
    }
    if upstream_truncated || local_truncated {
        text.push_str(&format!("\n[codewhale: {stream} preview truncated]"));
    }
    text
}

fn parse(envelope: Envelope) -> Result<Option<CommandRecord>, &'static str> {
    if envelope.schema_version != 1
        || envelope.event != "tool_call_after"
        || !matches!(
            envelope.tool_name.as_deref(),
            Some("exec_shell" | "Bash" | "bash")
        )
    {
        return Ok(None);
    }
    let Some(receipt) = envelope.execution_receipt else {
        return Ok(None);
    };
    if envelope.session_id_truncated
        || envelope.tool_call_id_truncated
        || envelope.tool_name_truncated
    {
        return Err("Codewhale correlation identity truncated; event skipped");
    }
    if receipt.schema_version != 1
        || receipt.execution != "started"
        || !matches!(
            receipt.completion.as_str(),
            "completed" | "failed" | "killed" | "timed_out"
        )
    {
        return Ok(None);
    }
    if receipt.command_truncated
        || receipt.cwd_truncated
        || receipt.command.trim().is_empty()
        || receipt.command.len() > 4096
        || receipt.command.contains('\0')
        || receipt.cwd.len() > 4096
        || receipt.cwd.contains('\0')
        || !Path::new(&receipt.cwd).is_absolute()
    {
        return Err("Codewhale execution identity unavailable or truncated; event skipped");
    }
    let session_id = identifier(envelope.session_id)?;
    let tool_call_id = identifier(envelope.tool_call_id)?;
    if !matches!(
        receipt.output_mode.as_str(),
        "separate" | "combined" | "unavailable"
    ) {
        return Err("unsupported Codewhale output representation; event skipped");
    }
    let stdout_truncated = receipt.stdout_truncated || receipt.stdout.len() > 20_000;
    let stderr_truncated = receipt.stderr_truncated || receipt.stderr.len() > 20_000;
    let metadata = json!({"schema_version":1,"session_id":session_id,"tool_call_id":tool_call_id,
        "execution":receipt.execution,"completion":receipt.completion,
        "output_mode":receipt.output_mode,
        "stdout_truncated":stdout_truncated,"stderr_truncated":stderr_truncated});
    let (stdout, stderr) = match receipt.output_mode.as_str() {
        "separate" => (
            output(receipt.stdout, stdout_truncated, "stdout"),
            output(receipt.stderr, stderr_truncated, "stderr"),
        ),
        "combined" => {
            if !receipt.stderr.is_empty() {
                return Err("ambiguous Codewhale combined output; event skipped");
            }
            (
                format!(
                    "[codewhale: combined stdout/stderr preview]\n{}",
                    output(receipt.stdout, stdout_truncated, "combined output")
                ),
                String::new(),
            )
        }
        _ => (String::new(), String::new()),
    };
    Ok(Some(CommandRecord {
        cwd: Some(receipt.cwd),
        exit_code: receipt.exit_code,
        command_parts: vec![receipt.command],
        stdout,
        stderr,
        notes: format!(
            "agent:codewhale codewhale_completion:{} codewhale_metadata:{metadata}",
            receipt.completion
        ),
        capture_kind: "full".into(),
        agent: Some(memorywhale_core::provenance::AGENT_CODEWHALE.into()),
    }))
}
