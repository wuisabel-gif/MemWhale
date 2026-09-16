//! Explicit, local-only debugging handoff export.
use rusqlite::{params, Connection};
use serde::Serialize;
use std::fs;
use std::path::Path;
const LIMIT: usize = 16_000;
#[derive(Serialize)]
struct Record {
    id: String,
    kind: String,
    evidence: String,
    provenance: serde_json::Value,
    source_ids: Vec<String>,
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
            let r=conn.query_row("SELECT command,stdout,stderr,cwd,created_at,agent FROM command_runs WHERE id=?1",params![id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,String>(4)?,r.get::<_,Option<String>>(5)?))).map_err(|_|format!("record not found: {key}"))?;
            evidence.push(Record {
                id: key.clone(),
                kind: kind.into(),
                evidence: format!(
                    "$ {}\n{}\n{}",
                    r.0,
                    bounded(r.1, &mut notices, "stdout"),
                    bounded(r.2, &mut notices, "stderr")
                ),
                provenance: serde_json::json!({"cwd":r.3,"created_at":r.4,"agent":r.5}),
                source_ids: vec![key.clone()],
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
            evidence.push(Record{id:key.clone(),kind:kind.into(),evidence:bounded(r.0,&mut notices,"transcript"),provenance:serde_json::json!({"cwd":r.1,"started_at":r.2,"ended_at":r.3,"status":r.4}),source_ids:vec![key.clone()]});
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
        let mut s: String = "# Debugging handoff\n\n## Notices\n".into();
        for n in &h.notices {
            s.push_str(&format!("- {n}\n"));
        }
        s.push_str("\n## Evidence\n");
        for r in &h.evidence {
            s.push_str(&format!(
                "### {} ({})\n\nSource IDs: `{}`\n\n```text\n{}\n```\n\nProvenance: `{}`\n\n",
                r.id,
                r.kind,
                r.source_ids.join("`, `"),
                r.evidence,
                serde_json::to_string(&r.provenance).unwrap()
            ));
        }
        s.push_str("## Unresolved questions\n- What remains to be verified?\n");
        s
    };
    if output.exists() {
        return Err(format!("refusing to overwrite {}", output.display()));
    }
    if let Some(p) = output.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    fs::write(output, content).map_err(|e| format!("write handoff: {e}"))
}
