//! Explicit, local-only debugging handoff export.
use rusqlite::{params, Connection};
use serde::Serialize;
use std::{fs, io::Write, path::Path};
const LIMIT: usize = 16_000;
#[derive(Serialize)]
struct Record {
    id: String,
    kind: String,
    evidence: String,
    provenance: serde_json::Value,
    source_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    argv_json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invocation: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exit_code: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exit_outcome: Option<String>,
}
#[derive(Serialize)]
struct Handoff {
    format_version: u8,
    evidence: Vec<Record>,
    unresolved_questions: Vec<String>,
    notices: Vec<String>,
}
fn bounded(s: String, notices: &mut Vec<String>, label: &str) -> String {
    let s = crate::redact(&s);
    if s.len() > LIMIT {
        notices.push(format!("{label} truncated to {LIMIT} bytes"));
        crate::truncate_capture(&s, LIMIT)
    } else {
        s
    }
}
fn fence(text: &str) -> String {
    let mut run = 0;
    let mut max_run = 0;
    for c in text.chars() {
        if c == '`' {
            run += 1;
            max_run = max_run.max(run);
        } else {
            run = 0;
        }
    }
    "`".repeat(3.max(max_run + 1))
}
fn markdown_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('#', "\\#")
}
fn markdown_block(label: &str, text: &str) -> String {
    let f = fence(text);
    format!("{label}\n{f}text\n{text}\n{f}\n")
}
fn outcome(code: Option<i64>) -> String {
    match code {
        Some(0) => "success".into(),
        Some(c) => format!("exited with status {c}"),
        None => "unknown exit outcome".into(),
    }
}
pub fn export(
    conn: &Connection,
    ids: &[String],
    format: &str,
    output: &Path,
) -> Result<(), String> {
    if ids.is_empty() {
        return Err("handoff requires explicit record IDs (c:ID or s:ID); nothing is selected automatically".into());
    }
    if format != "json" && format != "markdown" {
        return Err("format must be markdown or json".into());
    }
    let mut notices = vec![
        "Local export only; no upload or network operation was performed.".into(),
        "Evidence is not a causal claim; provenance is reported as stored.".into(),
    ];
    let mut evidence = Vec::new();
    for key in ids {
        let (kind, id): (&str, i64) = if let Some(v) = key.strip_prefix("c:") {
            (
                "command",
                v.parse().map_err(|_| format!("invalid ID {key}"))?,
            )
        } else if let Some(v) = key.strip_prefix("s:") {
            (
                "session",
                v.parse().map_err(|_| format!("invalid ID {key}"))?,
            )
        } else {
            return Err(format!("invalid record ID {key}; use c:ID or s:ID"));
        };
        if kind == "command" {
            let r=conn.query_row("SELECT command,argv_json,cwd,exit_code,stdout,stderr,created_at,agent FROM command_runs WHERE id=?1",params![id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<String>>(2)?,r.get::<_,Option<i64>>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,String>(6)?,r.get::<_,Option<String>>(7)?))).map_err(|_|format!("record not found: {key}"))?;
            let invocation: Vec<String> = serde_json::from_str::<Vec<String>>(&r.1)
                .unwrap_or_else(|_| vec![r.0.clone()])
                .into_iter()
                .map(|v| crate::redact(&v))
                .collect();
            let argv_json = serde_json::to_string(&invocation).map_err(|e| e.to_string())?;
            let command = crate::redact(&r.0);
            let ev = format!(
                "$ {}\n{}\n{}",
                command,
                bounded(r.4, &mut notices, "stdout"),
                bounded(r.5, &mut notices, "stderr")
            );
            evidence.push(Record {
                id: key.clone(),
                kind: kind.into(),
                evidence: ev,
                provenance: serde_json::json!({"cwd":r.2,"created_at":r.6,"agent":r.7}),
                source_ids: vec![key.clone()],
                argv_json: Some(argv_json),
                invocation: Some(invocation),
                exit_code: r.3,
                exit_outcome: Some(outcome(r.3)),
            });
        } else {
            let r = conn
                .query_row(
                    "SELECT transcript,cwd,started_at,ended_at,status FROM sessions WHERE id=?1",
                    params![id],
                    |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, Option<String>>(1)?,
                            r.get::<_, String>(2)?,
                            r.get::<_, String>(3)?,
                            r.get::<_, String>(4)?,
                        ))
                    },
                )
                .map_err(|_| format!("record not found: {key}"))?;
            evidence.push(Record{id:key.clone(),kind:kind.into(),evidence:bounded(r.0,&mut notices,"transcript"),provenance:serde_json::json!({"cwd":r.1,"started_at":r.2,"ended_at":r.3,"status":r.4}),source_ids:vec![key.clone()],argv_json:None,invocation:None,exit_code:None,exit_outcome:None});
        }
    }
    let h = Handoff {
        format_version: 1,
        evidence,
        unresolved_questions: vec!["What remains to be verified?".into()],
        notices,
    };
    let content = if format == "json" {
        serde_json::to_string_pretty(&h).unwrap()
    } else {
        let mut s = "# Debugging handoff\n\n## Notices\n".to_string();
        for n in &h.notices {
            s.push_str(&format!("- {n}\n"));
        }
        s.push_str("\n## Evidence\n");
        for r in &h.evidence {
            s.push_str(&format!(
                "### {} ({})\n\n",
                markdown_text(&r.id),
                markdown_text(&r.kind)
            ));
            s.push_str(&markdown_block("Source IDs:", &r.source_ids.join("\n")));
            s.push_str(&markdown_block(
                "Invocation:",
                &r.invocation
                    .as_ref()
                    .map(|v| serde_json::to_string(v).unwrap())
                    .unwrap_or_default(),
            ));
            s.push_str(&markdown_block(
                "argv_json:",
                r.argv_json.as_deref().unwrap_or(""),
            ));
            s.push_str(&markdown_block(
                "Exit outcome:",
                r.exit_outcome.as_deref().unwrap_or("not applicable"),
            ));
            s.push_str(&markdown_block("Evidence:", &r.evidence));
            s.push_str(&markdown_block(
                "Provenance:",
                &serde_json::to_string_pretty(&r.provenance).unwrap(),
            ));
            s.push('\n');
        }
        s += "## Unresolved questions\n- What remains to be verified?\n";
        s
    };
    if let Some(p) = output.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| format!("refusing to overwrite {}: {e}", output.display()))?;
    file.write_all(content.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("write handoff: {e}"))
}
#[cfg(test)]
mod tests {
    use super::{fence, markdown_block, markdown_text};
    #[test]
    fn fence_handles_backticks() {
        assert!(fence("x ``` y").len() > 3);
    }

    #[test]
    fn markdown_rendering_is_literal_safe() {
        let hostile = "line 1\n```\n<script>* [active]_markup_</script>";
        let rendered = markdown_block("Evidence:", hostile);
        assert!(rendered.contains(hostile));
        assert!(rendered.starts_with("Evidence:\n````text\n"));
        assert_eq!(markdown_text("a`*[x]#<b>"), "a\\`\\*\\[x\\]\\#\\<b\\>");
    }
}
