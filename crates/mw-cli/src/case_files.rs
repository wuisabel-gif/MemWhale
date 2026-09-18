//! Explicit, human-authored debugging case files.
use chrono::Utc;
use rusqlite::{params, Connection};
use serde_json::json;

pub fn create(conn: &Connection, args: &[String]) -> Result<(), String> {
    let title = crate::sanitize_capture(&value(args, "--title")?);
    let ids = value(args, "--command-ids")?;
    let observations = crate::sanitize_capture(&value(args, "--observations").unwrap_or_default());
    let conclusion = crate::sanitize_capture(&value(args, "--conclusion").unwrap_or_default());
    let unresolved = crate::sanitize_capture(&value(args, "--unresolved").unwrap_or_default());
    let status = value(args, "--status").unwrap_or_else(|_| "open".into());
    if !["open", "resolved", "closed"].contains(&status.as_str()) {
        return Err("status must be open, resolved, or closed".into());
    }
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO case_files (title, observations, conclusion, unresolved_questions, status, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?6)", params![title, observations, conclusion, unresolved, status, Utc::now().to_rfc3339()]).map_err(|e| e.to_string())?;
    let case_id = tx.last_insert_rowid();
    for (position, id) in ids.split(',').enumerate() {
        let id: i64 = id
            .trim()
            .parse()
            .map_err(|_| format!("invalid command id: {id}"))?;
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM command_runs WHERE id=?1)",
                [id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if !exists {
            return Err(format!("command run {id} does not exist"));
        }
        tx.execute("INSERT INTO case_file_commands (case_file_id, command_run_id, position) VALUES (?1,?2,?3)", params![case_id,id, position as i64]).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    println!("case file #{case_id} created");
    Ok(())
}
fn value(args: &[String], flag: &str) -> Result<String, String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
        .ok_or_else(|| format!("missing {flag}"))
}
pub fn list(conn: &Connection) -> Result<(), String> {
    let mut s = conn
        .prepare("SELECT id,title,status,updated_at FROM case_files ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = s
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        let (id, t, st, u) = r.map_err(|e| e.to_string())?;
        println!("#{id} [{st}] {t} ({u})");
    }
    Ok(())
}
pub fn show(conn: &Connection, id: i64) -> Result<(), String> {
    let row=conn.query_row("SELECT title,observations,conclusion,unresolved_questions,status,created_at,updated_at FROM case_files WHERE id=?1", [id], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,String>(6)?))).map_err(|e|e.to_string())?;
    println!("#{} [{}] {}\nCreated: {}\nUpdated: {}\n\nEvidence / observations:\n{}\n\nConclusion (interpretation):\n{}\n\nUnresolved questions:\n{}",id,row.4,row.0,row.5,row.6,row.1,row.2,row.3);
    let mut s=conn.prepare("SELECT c.id,c.command,c.exit_code FROM case_file_commands x JOIN command_runs c ON c.id=x.command_run_id WHERE x.case_file_id=?1 ORDER BY x.position").map_err(|e|e.to_string())?;
    for r in s
        .query_map([id], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<i64>>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
    {
        let (id, c, e) = r.map_err(|e| e.to_string())?;
        println!("Attempt command #{id} (exit {:?}): {c}", e);
    }
    Ok(())
}
pub fn export(conn: &Connection, id: i64, args: &[String]) -> Result<(), String> {
    let format = args
        .windows(2)
        .find(|w| w[0] == "--format")
        .map(|w| w[1].as_str())
        .unwrap_or("json");
    if !["json", "markdown", "md"].contains(&format) {
        return Err("format must be json or markdown".into());
    }
    let mut stmt=conn.prepare("SELECT title,observations,conclusion,unresolved_questions,status,created_at,updated_at FROM case_files WHERE id=?1").map_err(|e|e.to_string())?;
    let row=stmt.query_row([id],|r|Ok(json!({"id":id,"title":r.get::<_,String>(0)?,"observations":r.get::<_,String>(1)?,"conclusion":r.get::<_,String>(2)?,"unresolved_questions":r.get::<_,String>(3)?,"status":r.get::<_,String>(4)?,"created_at":r.get::<_,String>(5)?,"updated_at":r.get::<_,String>(6)?}))).map_err(|e|e.to_string())?;
    let mut commands = Vec::new();
    let mut command_ids = Vec::new();
    let mut commands_stmt = conn.prepare("SELECT x.position,c.id,c.command,c.exit_code,c.stdout,c.stderr FROM case_file_commands x JOIN command_runs c ON c.id=x.command_run_id WHERE x.case_file_id=?1 ORDER BY x.position").map_err(|e| e.to_string())?;
    for item in commands_stmt
        .query_map([id], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?
    {
        let (position, command_id, command, exit_code, stdout, stderr) =
            item.map_err(|e| e.to_string())?;
        command_ids.push(command_id);
        commands.push(json!({"position": position, "command_id": command_id, "command": crate::sanitize_capture(&command), "exit_code": exit_code, "stdout": crate::sanitize_capture(&stdout), "stderr": crate::sanitize_capture(&stderr)}));
    }
    let mut exported = row;
    exported["command_ids"] = json!(command_ids);
    exported["evidence"] = json!(commands);
    if format == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&exported).map_err(|e| e.to_string())?
        );
    } else {
        println!("# {}\n\n- Status: {}\n- Created: {}\n- Updated: {}\n\n## Evidence / observations\n\n{}\n\n## Conclusion\n\n{}\n\n## Unresolved questions\n\n{}\n\n## Selected command links\n", exported["title"], exported["status"], exported["created_at"], exported["updated_at"], crate::sanitize_capture(exported["observations"].as_str().unwrap_or_default()), crate::sanitize_capture(exported["conclusion"].as_str().unwrap_or_default()), crate::sanitize_capture(exported["unresolved_questions"].as_str().unwrap_or_default()));
        for command in commands {
            println!(
                "- [Command #{}](mw://command/{}) — exit {:?}: `{}`",
                command["command_id"],
                command["command_id"],
                command["exit_code"],
                command["command"]
            );
            println!(
                "  - stdout: {}\n  - stderr: {}",
                command["stdout"], command["stderr"]
            );
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creates_and_shows_case() {
        let c = Connection::open_in_memory().unwrap();
        crate::storage::initialize(&c).unwrap();
        c.execute("INSERT INTO command_runs(command,argv_json,created_at) VALUES('cargo test','[]','now')",[]).unwrap();
        create(
            &c,
            &[
                "--title".into(),
                "Failure token=supersecret".into(),
                "--command-ids".into(),
                "1".into(),
                "--observations".into(),
                "observed token=anothersecret".into(),
                "--conclusion".into(),
                "fixed token=thirdsecret".into(),
                "--unresolved".into(),
                "question token=fourthsecret".into(),
            ],
        )
        .unwrap();
        assert_eq!(
            c.query_row("SELECT title FROM case_files", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "Failure token=[REDACTED]"
        );
        let fields: (String, String, String) = c
            .query_row(
                "SELECT observations, conclusion, unresolved_questions FROM case_files",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(fields.0, "observed token=[REDACTED]");
        assert_eq!(fields.1, "fixed token=[REDACTED]");
        assert_eq!(fields.2, "question token=[REDACTED]");
    }

    #[test]
    fn preserves_requested_command_order_in_evidence_links() {
        let c = Connection::open_in_memory().unwrap();
        crate::storage::initialize(&c).unwrap();
        for id in [20_i64, 7] {
            c.execute("INSERT INTO command_runs(id,command,argv_json,created_at) VALUES(?1, 'test', '[]', 'now')", [id]).unwrap();
        }
        create(
            &c,
            &[
                "--title".into(),
                "ordered".into(),
                "--command-ids".into(),
                "20,7".into(),
            ],
        )
        .unwrap();
        let positions: Vec<(i64, i64)> = c
            .prepare("SELECT position, command_run_id FROM case_file_commands ORDER BY position")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(positions, vec![(0, 20), (1, 7)]);
    }
}
