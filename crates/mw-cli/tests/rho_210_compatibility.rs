//! Replay normalized live Rho 2.10.0 evidence without requiring Rho or a model.
use memorywhale_cli::agent_hook::{record_from_value, Agent};
use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

fn fixtures() -> Vec<Value> {
    serde_json::from_str(include_str!("fixtures/rho-2.10/hooks.json")).unwrap()
}

#[test]
fn live_outcomes_preserve_commands_but_never_infer_numeric_exits() {
    for case in fixtures() {
        let event = &case["event"];
        let record = record_from_value(event, Agent::Rho);
        if case["case"].as_str().unwrap().starts_with("cancel-") {
            assert!(
                record.is_none(),
                "pre-tool/session failure is not execution evidence"
            );
            continue;
        }
        let record = record.unwrap();
        assert_eq!(record.agent.as_deref(), Some("rho"));
        assert_eq!(record.exit_code, None);
        assert!(
            record.stdout.is_empty(),
            "hook envelope does not carry stdout"
        );
        assert_eq!(record.cwd.as_deref(), Some("/fixture/project"));
        assert_eq!(
            record.command_parts,
            vec![event["payload"]["capability"]["shell_command"]
                .as_str()
                .unwrap()]
        );
        assert!(record
            .notes
            .contains(event["payload"]["status"].as_str().unwrap()));
        if case["case"] == "nonzero" {
            assert!(record.stderr.contains("exit code: 7"));
            assert_eq!(record.exit_code, None, "text is not a numeric exit field");
        }
        if case["case"] == "denied" {
            assert!(record.stderr.contains("policy_denied"));
        }
        if case["case"] == "unsaved-success" {
            assert!(
                record.notes.contains("status:succeeded"),
                "explicit capture remains independent of session persistence"
            );
        }
    }
}

#[test]
fn missing_and_truncated_capabilities_do_not_invent_a_command_or_cwd() {
    let mut event = fixtures()[1]["event"].clone();
    event["payload"]["capability"] = Value::Null;
    let missing = record_from_value(&event, Agent::Rho).unwrap();
    assert_eq!(missing.command_parts, vec!["[rho:after_tool_use]"]);
    assert!(missing.notes.contains("command:unknown"));
    let mut event = fixtures()[1]["event"].clone();
    event["bounds"] = json!({"truncated":true,"fields":["payload.capability.shell_command","payload.capability.working_directory"]});
    let truncated = record_from_value(&event, Agent::Rho).unwrap();
    assert_eq!(truncated.command_parts, vec!["[rho:after_tool_use]"]);
    assert_eq!(truncated.cwd, None);
    assert!(truncated.stderr.contains("upstream truncation"));
    event["payload"]["status"] = json!("succeeded");
    assert!(record_from_value(&event, Agent::Rho).is_none());
}

struct Sandbox(PathBuf);
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn normalized_live_events_replay_into_an_isolated_store_and_remain_searchable() {
    let mut random = [0u8; 16];
    getrandom::getrandom(&mut random).unwrap();
    let path = std::env::temp_dir().join(format!("mw-rho210-{:032x}", u128::from_ne_bytes(random)));
    fs::create_dir(&path).unwrap();
    let sandbox = Sandbox(fs::canonicalize(path).unwrap());
    let project = sandbox.0.join("project");
    fs::create_dir(&project).unwrap();
    let data = sandbox.0.join("data");
    for mut fixture in fixtures() {
        let event = &mut fixture["event"];
        event["workspace"]["root"] = json!(project);
        if event["payload"].get("capability").is_some() {
            event["payload"]["capability"]["working_directory"] = json!(project);
        }
        let mut child = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
            .args(["--from-hook", "rho"])
            .env_clear()
            .env("HOME", &sandbox.0)
            .env("USERPROFILE", &sandbox.0)
            .env("MEMORYWHALE_DATA_DIR", &data)
            .env("PATH", "")
            .current_dir(&project)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(event.to_string().as_bytes())
            .unwrap();
        let result = child.wait_with_output().unwrap();
        assert!(result.status.success());
        assert!(result.stdout.is_empty());
    }
    let conn = rusqlite::Connection::open(data.join("memorywhale.sqlite3")).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM command_runs WHERE agent='rho' AND exit_code IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 4);
    let cancelled: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM command_runs WHERE command LIKE '%MW_RHO210_CANCEL%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        cancelled, 0,
        "no cancelled execution result was delivered in the live trace"
    );
    let output = Command::new(env!("CARGO_BIN_EXE_mw"))
        .args(["search", "MW_RHO210", "agent:rho"])
        .env_clear()
        .env("HOME", &sandbox.0)
        .env("USERPROFILE", &sandbox.0)
        .env("MEMORYWHALE_DATA_DIR", &data)
        .env("PATH", "")
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("MW_RHO210_UNSAVED"));
    assert!(text.contains("MW_RHO210_DENIED"));
    assert!(
        !project.join("cancel-started").exists(),
        "replay must not execute recorded commands"
    );
}
