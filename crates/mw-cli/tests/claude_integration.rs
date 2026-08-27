use std::process::Command;

/// Run `mw` without inheriting PATH so MCP registration cannot touch a real Claude install.
fn mw_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mw"));
    cmd.env("PATH", "");
    cmd
}

fn sandbox(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "mw-claude-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn custom_claude_config_dir_mcp_state_is_read_from_matching_claude_json() {
    let claude_dir = sandbox("mcp-config-dir");
    std::fs::write(
        claude_dir.join(".claude.json"),
        r#"{"mcpServers":{"memorywhale":{"command":"mw-mcp","args":[]}}}"#,
    )
    .unwrap();

    let output = mw_cmd()
        .args(["integrate", "claude"])
        .env("CLAUDE_CONFIG_DIR", &claude_dir)
        .output()
        .expect("run Claude integration command");

    assert!(output.status.success(), "command failed: {output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("memorywhale registered (user scope)"),
        "missing MCP success message: {output:?}"
    );
}

#[test]
fn user_can_install_memorywhale_into_a_fresh_claude_config() {
    let claude_dir = sandbox("fresh");

    let output = mw_cmd()
        .args(["integrate", "claude"])
        .env("CLAUDE_CONFIG_DIR", &claude_dir)
        .output()
        .expect("run Claude integration command");

    assert!(output.status.success(), "command failed: {output:?}");
    assert!(!claude_dir.join("hooks/mw-record.py").exists());
    assert!(claude_dir.join("skills/memorywhale/SKILL.md").is_file());

    let settings: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(claude_dir.join("settings.json")).unwrap())
            .unwrap();
    let bash_group = settings["hooks"]["PostToolUse"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["matcher"] == "Bash")
        .expect("missing Bash hook group");
    let command = bash_group["hooks"][0]["command"].as_str().unwrap();
    assert!(command.contains("mw-remember"), "command: {command}");
    assert!(command.contains("--from-hook claude"), "command: {command}");
    assert!(!command.contains("python3"), "command: {command}");
    let failure_group = settings["hooks"]["PostToolUseFailure"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["matcher"] == "Bash")
        .expect("missing Bash PostToolUseFailure hook group");
    let failure_command = failure_group["hooks"][0]["command"].as_str().unwrap();
    assert!(
        failure_command.contains("mw-remember") && failure_command.contains("--from-hook claude"),
        "command: {failure_command}"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Claude Code"),
        "missing success message: {output:?}"
    );
}

#[test]
fn installing_memorywhale_preserves_existing_claude_settings() {
    let claude_dir = sandbox("existing");
    let settings_path = claude_dir.join("settings.json");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(
        &settings_path,
        r#"{
  "theme": "dark",
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Read",
        "hooks": [{"type": "command", "command": "echo read"}]
      }
    ]
  }
}
"#,
    )
    .unwrap();

    let output = mw_cmd()
        .args(["integrate", "claude-code"])
        .env("CLAUDE_CONFIG_DIR", &claude_dir)
        .output()
        .expect("run Claude integration command");

    assert!(output.status.success(), "command failed: {output:?}");
    let updated: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path).unwrap()).unwrap();
    assert_eq!(updated["theme"], "dark");
    assert_eq!(updated["hooks"]["PostToolUse"].as_array().unwrap().len(), 2);
}

#[test]
fn installing_memorywhale_twice_does_not_duplicate_the_hook() {
    let claude_dir = sandbox("idempotent");

    for _ in 0..2 {
        let output = mw_cmd()
            .args(["integrate", "claude"])
            .env("CLAUDE_CONFIG_DIR", &claude_dir)
            .output()
            .expect("run Claude integration command");
        assert!(output.status.success(), "command failed: {output:?}");
    }

    let updated: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(claude_dir.join("settings.json")).unwrap())
            .unwrap();
    let bash_hooks = updated["hooks"]["PostToolUse"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["matcher"] == "Bash")
        .unwrap()["hooks"]
        .as_array()
        .unwrap();
    assert_eq!(bash_hooks.len(), 1);
}

#[test]
fn invalid_claude_settings_are_left_untouched() {
    let claude_dir = sandbox("invalid");
    let settings_path = claude_dir.join("settings.json");
    std::fs::create_dir_all(&claude_dir).unwrap();
    let original = "{not json";
    std::fs::write(&settings_path, original).unwrap();

    let output = mw_cmd()
        .args(["integrate", "claude"])
        .env("CLAUDE_CONFIG_DIR", &claude_dir)
        .output()
        .expect("run Claude integration command");

    assert!(
        !output.status.success(),
        "invalid JSON was accepted: {output:?}"
    );
    assert_eq!(std::fs::read_to_string(settings_path).unwrap(), original);
    assert!(
        !claude_dir.join("hooks/mw-record.py").exists(),
        "legacy hook was written despite invalid settings: {output:?}"
    );
    assert!(
        !claude_dir.join("skills/memorywhale/SKILL.md").exists(),
        "skill was written despite invalid settings: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid Claude settings.json"),
        "unexpected error: {output:?}"
    );
}

#[test]
fn revert_removes_installed_claude_integration() {
    let claude_dir = sandbox("revert");

    let install = mw_cmd()
        .args(["integrate", "claude"])
        .env("CLAUDE_CONFIG_DIR", &claude_dir)
        .output()
        .expect("run Claude install");
    assert!(install.status.success(), "install failed: {install:?}");

    let revert = mw_cmd()
        .args(["integrate", "claude", "--revert"])
        .env("CLAUDE_CONFIG_DIR", &claude_dir)
        .output()
        .expect("run Claude revert");
    assert!(revert.status.success(), "revert failed: {revert:?}");

    assert!(!claude_dir.join("hooks/mw-record.py").exists());
    assert!(!claude_dir.join("skills/memorywhale/SKILL.md").exists());
    assert!(!claude_dir.join("settings.json").exists());
    assert!(
        String::from_utf8_lossy(&revert.stdout).contains("removed from Claude Code"),
        "missing revert message: {revert:?}"
    );
}

#[test]
fn revert_preserves_unrelated_claude_settings() {
    let claude_dir = sandbox("revert-preserve");
    let settings_path = claude_dir.join("settings.json");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(
        &settings_path,
        r#"{
  "theme": "dark",
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Read",
        "hooks": [{"type": "command", "command": "echo read"}]
      }
    ]
  }
}
"#,
    )
    .unwrap();

    let output = mw_cmd()
        .args(["integrate", "claude"])
        .env("CLAUDE_CONFIG_DIR", &claude_dir)
        .output()
        .expect("run Claude install");
    assert!(output.status.success(), "install failed: {output:?}");

    let revert = mw_cmd()
        .args(["integrate", "claude-code", "--revert"])
        .env("CLAUDE_CONFIG_DIR", &claude_dir)
        .output()
        .expect("run Claude revert");
    assert!(revert.status.success(), "revert failed: {revert:?}");

    let updated: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path).unwrap()).unwrap();
    assert_eq!(updated["theme"], "dark");
    assert_eq!(updated["hooks"]["PostToolUse"].as_array().unwrap().len(), 1);
    assert_eq!(updated["hooks"]["PostToolUse"][0]["matcher"], "Read");
}

#[test]
fn revert_leaves_invalid_claude_settings_untouched() {
    let claude_dir = sandbox("revert-invalid");
    let settings_path = claude_dir.join("settings.json");
    std::fs::create_dir_all(claude_dir.join("hooks")).unwrap();
    std::fs::write(claude_dir.join("hooks/mw-record.py"), "hook").unwrap();
    let original = "{not json";
    std::fs::write(&settings_path, original).unwrap();

    let output = mw_cmd()
        .args(["integrate", "claude", "--revert"])
        .env("CLAUDE_CONFIG_DIR", &claude_dir)
        .output()
        .expect("run Claude revert");

    assert!(
        !output.status.success(),
        "invalid JSON revert was accepted: {output:?}"
    );
    assert_eq!(std::fs::read_to_string(settings_path).unwrap(), original);
    assert!(claude_dir.join("hooks/mw-record.py").exists());
}

#[test]
fn revert_without_memorywhale_installed_is_a_noop_for_settings() {
    let claude_dir = sandbox("revert-noop");
    let settings_path = claude_dir.join("settings.json");
    std::fs::create_dir_all(&claude_dir).unwrap();
    let original = r#"{"theme":"dark"}"#;
    std::fs::write(&settings_path, original).unwrap();

    let output = mw_cmd()
        .args(["integrate", "claude", "--revert"])
        .env("CLAUDE_CONFIG_DIR", &claude_dir)
        .output()
        .expect("run Claude revert");

    assert!(output.status.success(), "revert failed: {output:?}");
    assert_eq!(std::fs::read_to_string(settings_path).unwrap(), original);
    assert!(
        !String::from_utf8_lossy(&output.stdout)
            .contains("settings: MemoryWhale hook entry removed"),
        "unexpected settings message: {output:?}"
    );
}
