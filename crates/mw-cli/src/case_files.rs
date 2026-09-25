//! Explicit, human-authored debugging case files.
use chrono::Utc;
use rusqlite::{params, Connection};
use serde_json::json;

pub fn create(conn: &Connection, args: &[String]) -> Result<(), String> {
    let opts = parse_options(args)?;
    let get = |flag: &str| opts.get(flag).cloned();
    let title = clean(&get("--title").ok_or("missing --title")?);
    let ids = get("--command-ids").ok_or("missing --command-ids")?;
    let observations = clean(&get("--observations").unwrap_or_default());
    let conclusion = clean(&get("--conclusion").unwrap_or_default());
    let unresolved = clean(&get("--unresolved").unwrap_or_default());
    let status = get("--status").unwrap_or_else(|| "open".into());
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
fn json_string(value: &serde_json::Value) -> String {
    value.as_str().map(str::to_string).unwrap_or_default()
}

/// Stored text is untrusted terminal content: redact, then strip CSI/OSC and
/// bare control characters, then redact again (controls can split a label).
/// Redact, strip terminal controls, then redact again, so a control split
/// inside a secret label cannot hide it. Use for any stored or printed text.
pub fn clean(text: &str) -> String {
    crate::sanitize_capture(&crate::github::neutralize_terminal_controls(
        &crate::sanitize_capture(text),
    ))
}

fn clean_strings(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(text) => *text = clean(text),
        serde_json::Value::Array(items) => items.iter_mut().for_each(clean_strings),
        serde_json::Value::Object(map) => map.values_mut().for_each(clean_strings),
        _ => {}
    }
}

fn exit_label(code: Option<i64>) -> String {
    code.map_or_else(|| "none".to_string(), |c| c.to_string())
}

/// A Markdown fenced code block whose fence is longer than any backtick run in
/// `text`, so captured content cannot close it early or inject Markdown/HTML.
fn fenced(text: &str) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(longest.max(2) + 1);
    format!("{fence}text\n{text}\n{fence}")
}

const CREATE_FLAGS: [&str; 6] = [
    "--title",
    "--command-ids",
    "--observations",
    "--conclusion",
    "--unresolved",
    "--status",
];

/// Strict `--flag value` pairs: every token must be a known flag followed by a
/// value that is not itself a flag, and each flag may appear once. Anything
/// else is an error rather than a silently shifted or defaulted field.
fn parse_options(
    args: &[String],
) -> Result<std::collections::HashMap<&'static str, String>, String> {
    let mut opts = std::collections::HashMap::new();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let flag = CREATE_FLAGS
            .iter()
            .find(|f| **f == arg)
            .ok_or_else(|| format!("unexpected argument {arg:?}"))?;
        let value = it
            .next()
            .filter(|v| !v.starts_with("--"))
            .ok_or_else(|| format!("{flag} needs a value"))?;
        if opts.insert(*flag, value.clone()).is_some() {
            return Err(format!("{flag} given more than once"));
        }
    }
    Ok(opts)
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
        println!("#{id} [{}] {} ({})", clean(&st), clean(&t), clean(&u));
    }
    Ok(())
}
pub fn show(conn: &Connection, id: i64) -> Result<(), String> {
    print!("{}", render_show(conn, id)?);
    Ok(())
}

fn render_show(conn: &Connection, id: i64) -> Result<String, String> {
    let row=conn.query_row("SELECT title,observations,conclusion,unresolved_questions,status,created_at,updated_at FROM case_files WHERE id=?1", [id], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,String>(6)?))).map_err(|e|e.to_string())?;
    let mut out = format!("#{} [{}] {}\nCreated: {}\nUpdated: {}\n\nEvidence / observations:\n{}\n\nConclusion (interpretation):\n{}\n\nUnresolved questions:\n{}\n",id,clean(&row.4),clean(&row.0),clean(&row.5),clean(&row.6),clean(&row.1),clean(&row.2),clean(&row.3));
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
        out.push_str(&format!(
            "Attempt command #{id} (exit {}): {}\n",
            exit_label(e),
            clean(&c)
        ));
    }
    Ok(out)
}
pub fn export(conn: &Connection, id: i64, args: &[String]) -> Result<(), String> {
    print!("{}", render_export(conn, id, args)?);
    Ok(())
}

fn render_export(conn: &Connection, id: i64, args: &[String]) -> Result<String, String> {
    let format = match args {
        [] => "json",
        [flag, value] if flag == "--format" => value.as_str(),
        _ => return Err("usage: mw case export <id> [--format json|markdown]".into()),
    };
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
    // Every exported string gets the same redact → strip controls → redact
    // pass as `show`, so a control split inside a secret label cannot hide it.
    clean_strings(&mut exported);
    if format == "json" {
        return Ok(format!(
            "{}\n",
            serde_json::to_string_pretty(&exported).map_err(|e| e.to_string())?
        ));
    }
    let field = |key: &str| clean(&json_string(&exported[key]));
    // The title is a heading: keep it on one line.
    let title = field("title")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut out = format!(
        "# {title}\n\n- Status: {}\n- Created: {}\n- Updated: {}\n\n## Evidence / observations\n\n{}\n\n## Conclusion\n\n{}\n\n## Unresolved questions\n\n{}\n\n## Selected command links\n",
        field("status"),
        field("created_at"),
        field("updated_at"),
        field("observations"),
        field("conclusion"),
        field("unresolved_questions"),
    );
    for command in commands {
        let command_id = command["command_id"].as_i64().unwrap_or_default();
        let text = |key: &str| fenced(&clean(&json_string(&command[key])));
        out.push_str(&format!(
            "\n### [Command #{command_id}](mw://command/{command_id}) — exit {}\n\n{}\n\nstdout:\n\n{}\n\nstderr:\n\n{}\n",
            exit_label(command["exit_code"].as_i64()),
            text("command"),
            text("stdout"),
            text("stderr"),
        ));
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_export_redacts_secrets_split_by_terminal_controls() {
        let c = Connection::open_in_memory().unwrap();
        crate::storage::initialize(&c).unwrap();
        c.execute(
            "INSERT INTO command_runs(command,argv_json,created_at,stdout,stderr) VALUES('run tok\x1ben=abc123secret','[]','now','tok\x1ben=abc123secret','')",
            [],
        )
        .unwrap();
        let args: Vec<String> = ["--title", "tok\x1ben=abc123secret", "--command-ids", "1"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        create(&c, &args).unwrap();
        let stored: String = c
            .query_row("SELECT title FROM case_files", [], |r| r.get(0))
            .unwrap();
        assert!(
            !stored.contains("abc123secret"),
            "stored at rest: {stored:?}"
        );
        let out = render_export(&c, 1, &[]).unwrap();
        assert!(!out.contains("abc123secret"), "{out}");
        assert!(!out.contains("\\u001b"), "{out}");
    }
    #[test]
    fn export_rejects_malformed_arguments() {
        let c = Connection::open_in_memory().unwrap();
        crate::storage::initialize(&c).unwrap();
        for args in [
            vec!["--format"],
            vec!["--bogus", "x"],
            vec!["--format", "json", "extra"],
            vec!["--format", "yaml"],
        ] {
            let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            assert!(render_export(&c, 1, &args).is_err(), "{args:?}");
        }
    }
    #[test]
    fn malformed_create_arguments_are_rejected_without_writing() {
        let c = Connection::open_in_memory().unwrap();
        crate::storage::initialize(&c).unwrap();
        c.execute("INSERT INTO command_runs(command,argv_json,created_at) VALUES('cargo test','[]','now')",[]).unwrap();
        let bad: [&[&str]; 6] = [
            &["--title", "--command-ids", "1"],
            &[
                "--title",
                "t",
                "--command-ids",
                "1",
                "--status",
                "--conclusion",
                "x",
            ],
            &["--title", "t", "--command-ids", "1", "--observations"],
            &["--title", "t", "--command-ids", "1", "stray"],
            &["--title", "t", "--title", "u", "--command-ids", "1"],
            &["--title", "t", "--command-ids", "1", "--bogus", "x"],
        ];
        for args in bad {
            let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            assert!(create(&c, &args).is_err(), "{args:?} should be rejected");
        }
        let count: i64 = c
            .query_row("SELECT COUNT(*) FROM case_files", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
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

    #[test]
    fn markdown_fields_are_plain_text_not_json_literals() {
        assert_eq!(json_string(&json!("cargo test")), "cargo test");
        assert_eq!(json_string(&json!(null)), "");
        let command = json!({"command": "echo hi", "stdout": "hi\n"});
        assert_eq!(json_string(&command["command"]), "echo hi");
        assert_eq!(json_string(&command["stdout"]), "hi\n");
        assert!(!format!("`{}`", json_string(&command["command"])).contains('"'));
    }

    fn case_with(c: &Connection, runs: &[(i64, &str, &str)], ids: &str) {
        for (id, command, stdout) in runs {
            c.execute("INSERT INTO command_runs(id,command,argv_json,stdout,exit_code,created_at) VALUES(?1,?2,'[]',?3,1,'now')", params![id, command, stdout]).unwrap();
        }
        let args = ["--title", "case", "--command-ids", ids];
        create(c, &args.map(String::from)).unwrap();
    }

    #[test]
    fn export_includes_ordered_command_links_and_labels_by_id() {
        let c = Connection::open_in_memory().unwrap();
        crate::storage::initialize(&c).unwrap();
        case_with(&c, &[(20, "b", ""), (7, "a", "")], "20,7");
        let json: serde_json::Value =
            serde_json::from_str(&render_export(&c, 1, &[]).unwrap()).unwrap();
        assert_eq!(json["command_ids"], json!([20, 7]));
        assert_eq!(json["evidence"][0]["command_id"], 20);
        assert_eq!(json["evidence"][1]["position"], 1);
        let md = render_export(&c, 1, &["--format".into(), "md".into()]).unwrap();
        let first = md.find("[Command #20](mw://command/20)").unwrap();
        assert!(first < md.find("[Command #7](mw://command/7)").unwrap());
        assert!(!md.contains("Command #0"));
    }

    #[test]
    fn markdown_export_fences_captured_text_and_strips_controls() {
        let c = Connection::open_in_memory().unwrap();
        crate::storage::initialize(&c).unwrap();
        let command = "echo ```` `x` <img src=x onerror=alert(1)> \x1b]0;pwned\x07\x1b[2J";
        case_with(&c, &[(3, command, "ok\n# Injected heading\n```")], "3");
        let md = render_export(&c, 1, &["--format".into(), "markdown".into()]).unwrap();
        assert!(!md.contains('\x1b'));
        assert!(!md.contains("pwned"));
        // Fence is longer than the longest backtick run (4) in the command.
        assert!(md.contains("`````text\necho ```` `x` <img src=x onerror=alert(1)> \n`````"));
        assert!(md.contains("````text\nok\n# Injected heading\n```\n````"));
        assert!(md.contains("exit 1"));
        assert!(!md.contains("Some("));
    }

    #[test]
    fn show_redacts_and_strips_terminal_controls() {
        let c = Connection::open_in_memory().unwrap();
        crate::storage::initialize(&c).unwrap();
        case_with(
            &c,
            &[(5, "\x1b[2Jcurl token=supersecret\x1b]0;t\x07", "")],
            "5",
        );
        let out = render_show(&c, 1).unwrap();
        assert!(!out.contains('\x1b'));
        assert!(!out.contains("supersecret"));
        assert!(out.contains("Attempt command #5 (exit 1): curl token=[REDACTED]"));
    }
}
