//! Insert a command run into the local store.
//!
//! Shared by `mw-remember` (explicit CLI) and agent-hook capture so both paths
//! apply the same gate, redaction, and schema.

use chrono::Utc;
use rusqlite::{params, Connection};
use std::env;
use std::fs;
use std::path::PathBuf;

pub struct CommandRecord {
    pub cwd: Option<String>,
    pub exit_code: Option<i64>,
    pub stdout: String,
    pub stderr: String,
    pub notes: String,
    pub command_parts: Vec<String>,
    pub capture_kind: String,
}

/// Persist one command run. Returns `Ok(None)` when capture is off for `cwd`.
pub fn remember_command(mut record: CommandRecord) -> Result<Option<i64>, String> {
    if record.command_parts.is_empty() {
        return Err("missing command; pass it after --".to_string());
    }

    let gate = crate::capture_rule_for(record.cwd.as_deref());
    if !gate.mode.stores_anything() {
        return Ok(None);
    }
    if !gate.mode.stores_output() {
        record.stdout.clear();
        record.stderr.clear();
    }

    record.notes = append_environment_tags(record.notes);
    let stored_args = crate::sanitize_arguments(&record.command_parts);
    let command = stored_args[0].clone();
    let argv_json = serde_json::to_string(&stored_args)
        .map_err(|err| format!("failed to encode argv: {err}"))?;
    let created_at = Utc::now().to_rfc3339();
    let db_path = crate::database_path()?;
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("failed to create data dir: {err}"))?;
    }

    let conn = open_ready(&db_path)?;
    crate::restrict_path_permissions(&db_path, false)?;
    conn.execute(
        "
        INSERT INTO command_runs (command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at, capture_kind)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ",
        params![
            command,
            argv_json,
            record.cwd,
            record.exit_code,
            crate::sanitize_capture(&record.stdout),
            crate::sanitize_capture(&record.stderr),
            crate::sanitize_capture(&record.notes),
            created_at,
            record.capture_kind
        ],
    )
    .map_err(|err| format!("failed to insert command run: {err}"))?;
    let run_id = conn.last_insert_rowid();

    for (position, value) in stored_args.iter().enumerate() {
        conn.execute(
            "
            INSERT INTO command_arguments (command_run_id, position, value)
            VALUES (?1, ?2, ?3)
            ",
            params![run_id, position as i64, value],
        )
        .map_err(|err| format!("failed to insert argument: {err}"))?;
    }

    Ok(Some(run_id))
}

/// Open the database and make sure the schema is usable.
///
/// Shell hooks fire one writer per command, so several can be creating or
/// upgrading a brand-new database at the same instant. Schema initialization
/// can briefly lose a race even with the shared busy timeout.
/// Retry briefly rather than dropping the row.
fn open_ready(db_path: &std::path::Path) -> Result<Connection, String> {
    let mut last = String::new();
    for attempt in 0..5 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(80 * attempt));
        }
        match crate::storage::open_path(db_path) {
            Ok(conn) => return Ok(conn),
            Err(err) => last = err,
        }
    }
    Err(last)
}

fn append_environment_tags(notes: String) -> String {
    let mut tags = Vec::new();
    tags.push(format!("os:{}", env::consts::OS));
    if PathBuf::from("/.dockerenv").exists() || env::var_os("container").is_some() {
        tags.push("runtime:container".to_string());
    } else {
        tags.push("runtime:host".to_string());
    }
    if env::var_os("SSH_CONNECTION").is_some() || env::var_os("SSH_CLIENT").is_some() {
        tags.push("session:ssh".to_string());
    }
    if PathBuf::from("/etc/nv_tegra_release").exists() {
        tags.push("host:jetson".to_string());
    }

    if notes.trim().is_empty() {
        tags.join(" ")
    } else {
        format!("{} {}", notes.trim(), tags.join(" "))
    }
}
