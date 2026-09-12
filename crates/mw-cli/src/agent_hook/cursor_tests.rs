use super::*;

fn fixtures() -> Vec<Value> {
    serde_json::from_str(include_str!("../../tests/fixtures/cursor/hooks.json")).unwrap()
}
fn event() -> Value {
    fixtures()[0]["event"].clone()
}

#[test]
fn documented_outcome_pair_preserves_known_and_unknown_statuses() {
    let expected = [
        (Some(0), "exit_zero"),
        (Some(2), "nonzero_exit"),
        (None, "completed_exit_unknown"),
        (None, "timeout"),
        (None, "permission_denied"),
        (None, "cancelled"),
        (None, "completed_exit_unknown"),
    ];
    for (fixture, (exit, status)) in fixtures().iter().zip(expected) {
        let record = parse(&fixture["event"]).unwrap().unwrap();
        assert_eq!(record.exit_code, exit, "{}", fixture["case"]);
        assert_eq!(record.agent.as_deref(), Some("cursor"));
        assert_eq!(record.cwd.as_deref(), Some("/fixture/project"));
        assert_eq!(
            record.command_parts,
            vec![fixture["event"]["tool_input"]["command"].as_str().unwrap()]
        );
        assert!(record.notes.contains(&format!("cursor_status:{status}")));
    }
    let success = parse(&event()).unwrap().unwrap();
    assert_eq!(success.stdout, "cursor-fixture success\n");
    assert!(success.notes.contains("\"duration_ms\":12"));
    assert!(success.notes.contains("fixture-conversation"));
    assert!(success.notes.contains("fixture-generation"));
    assert!(success.notes.contains("fixture-success"));
    let denied = parse(&fixtures()[4]["event"]).unwrap().unwrap();
    assert!(denied.notes.contains("not_executed"));
}

#[test]
fn ignores_other_tools_and_overlapping_or_blocking_events() {
    for name in [
        "afterShellExecution",
        "preToolUse",
        "stop",
        "sessionStart",
        "PostToolUse",
    ] {
        let mut value = event();
        value["hook_event_name"] = json!(name);
        assert!(parse(&value).unwrap().is_none());
    }
    for tool in ["Read", "Bash", "MCP", "shell"] {
        let mut value = event();
        value["tool_name"] = json!(tool);
        assert!(parse(&value).unwrap().is_none());
    }
}

#[test]
fn refuses_missing_or_unusable_command_and_cwd_without_fallback() {
    for command in [
        Value::Null,
        json!(" "),
        json!("x".repeat(COMMAND_LIMIT + 1)),
        json!("a\u{0}b"),
    ] {
        let mut value = event();
        value["tool_input"]["command"] = command;
        assert!(parse(&value).is_err());
    }
    for cwd in [
        Value::Null,
        json!(""),
        json!("relative/path"),
        json!("/a\u{0}b"),
        json!(format!("/{}", "x".repeat(4096))),
    ] {
        let mut value = event();
        value["cwd"] = cwd;
        value["workspace_roots"] = json!(["/must-not-be-used"]);
        assert!(parse(&value).is_err());
    }
}

#[test]
fn never_infers_exit_codes_from_text_or_tool_success() {
    for result in [
        json!({}),
        json!({"exitCode":"0","stdout":"exit code 7"}),
        json!({"exitCode":-1}),
        json!({"exitCode":256}),
        json!({"exitCode":true}),
    ] {
        let mut value = event();
        value["tool_output"] = json!(result.to_string());
        assert_eq!(parse(&value).unwrap().unwrap().exit_code, None);
    }
    let mut value = fixtures()[3]["event"].clone();
    value["error_message"] = json!("exited with code 42");
    value["exitCode"] = json!(42); // Not part of the failure contract.
    assert_eq!(parse(&value).unwrap().unwrap().exit_code, None);
}

#[test]
fn bounds_unicode_outputs_and_ids_and_reports_truncation() {
    let mut value = event();
    value["tool_output"] =
        json!(json!({"stdout":"界".repeat(10000),"stderr":"é".repeat(15000)}).to_string());
    value["tool_use_id"] = json!("界".repeat(300));
    let record = parse(&value).unwrap().unwrap();
    assert!(record.stdout.len() <= TEXT_LIMIT && record.stderr.len() <= TEXT_LIMIT);
    assert!(record.stdout.ends_with("[cursor: output truncated]"));
    assert!(record.notes.contains("\"stdout_truncated\":true"));
    assert!(record.notes.contains("\"stderr_truncated\":true"));
    assert!(record.notes.contains("\"tool_use_id_truncated\":true"));
}

#[test]
fn does_not_import_unrelated_identity_or_policy_fields() {
    let mut value = event();
    value["agent"] = json!("claude");
    value["user_email"] = json!("private@example.test");
    value["notes"] = json!("untrusted note");
    value["capture_kind"] = json!("hook");
    value["tool_input"]["command"] = json!("  printf 'spacing retained'  ");
    let record = parse(&value).unwrap().unwrap();
    assert_eq!(record.agent.as_deref(), Some("cursor"));
    assert_eq!(record.capture_kind, "full");
    assert_eq!(record.command_parts[0], "  printf 'spacing retained'  ");
    assert!(!record.notes.contains("private@example.test"));
    assert!(!record.notes.contains("untrusted note"));
}
