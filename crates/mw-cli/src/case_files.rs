//! Explicit, human-authored debugging case files.
use chrono::Utc;
use rusqlite::{params, Connection};
use serde_json::json;

pub fn create(conn: &Connection, args: &[String]) -> Result<(), String> {
    let title = value(args, "--title")?;
    let ids = value(args, "--command-ids")?;
    let observations = value(args, "--observations").unwrap_or_default();
    let conclusion = value(args, "--conclusion").unwrap_or_default();
    let unresolved = value(args, "--unresolved").unwrap_or_default();
    let status = value(args, "--status").unwrap_or_else(|_| "open".into());
    if !["open", "resolved", "closed"].contains(&status.as_str()) {
        return Err("status must be open, resolved, or closed".into());
    }
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO case_files (title, observations, conclusion, unresolved_questions, status, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?6)", params![title, observations, conclusion, unresolved, status, Utc::now().to_rfc3339()]).map_err(|e| e.to_string())?;
    let case_id = tx.last_insert_rowid();
    for id in ids.split(',') {
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
        tx.execute("INSERT INTO case_file_commands (case_file_id, command_run_id, position) VALUES (?1,?2,?3)", params![case_id,id, id]).map_err(|e| e.to_string())?;
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
pub fn export(conn: &Connection, id: i64) -> Result<(), String> {
    let mut stmt=conn.prepare("SELECT title,observations,conclusion,unresolved_questions,status,created_at,updated_at FROM case_files WHERE id=?1").map_err(|e|e.to_string())?;
    let row=stmt.query_row([id],|r|Ok(json!({"id":id,"title":r.get::<_,String>(0)?,"observations":r.get::<_,String>(1)?,"conclusion":r.get::<_,String>(2)?,"unresolved_questions":r.get::<_,String>(3)?,"status":r.get::<_,String>(4)?,"created_at":r.get::<_,String>(5)?,"updated_at":r.get::<_,String>(6)?}))).map_err(|e|e.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&row).map_err(|e| e.to_string())?
    );
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
                "Failure".into(),
                "--command-ids".into(),
                "1".into(),
                "--observations".into(),
                "observed".into(),
            ],
        )
        .unwrap();
        assert_eq!(
            c.query_row("SELECT title FROM case_files", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "Failure"
        );
    }
}
