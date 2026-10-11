//! Read-only Claude Code doctor inspection.

use std::path::Path;

use serde_json::Value;

use super::{
    claude_config_dir, hook_command, is_memorywhale_hook_command, mcp_server_entry_matches,
    parse_settings, user_scoped_mcp_config_path, HookGroup, MCP_SERVER_NAME,
};
use crate::integrate::files::{
    command_on_path, mw_remember_executable, read_existing, skill_is_installed,
};
use crate::integrate::report::{IntegrationReport, McpFact, PieceStatus};

/// Inspect Claude Code MCP, hook, and skill status without mutating files or
/// running the Claude CLI.
pub(crate) fn doctor_report(mcp_stdio_ok: bool) -> IntegrationReport {
    let env_set = std::env::var_os("CLAUDE_CONFIG_DIR").is_some_and(|value| !value.is_empty());
    let Ok(config_dir) = claude_config_dir() else {
        return IntegrationReport::not_detected("Claude Code", "claude");
    };
    let mcp_path = user_scoped_mcp_config_path().unwrap_or_else(|| config_dir.join(".claude.json"));
    let detected =
        env_set || config_dir.exists() || mcp_path.is_file() || command_on_path("claude");
    if !detected {
        return IntegrationReport::not_detected("Claude Code", "claude");
    }
    inspect_at(
        &config_dir,
        &mcp_path,
        mw_remember_executable().ok().as_deref(),
        mcp_stdio_ok,
    )
}

fn inspect_at(
    config_dir: &Path,
    mcp_config_path: &Path,
    remember_path: Option<&Path>,
    mcp_stdio_ok: bool,
) -> IntegrationReport {
    IntegrationReport::detected(
        "Claude Code",
        "claude",
        inspect_mcp(mcp_config_path).into_status(mcp_stdio_ok),
        inspect_hook(config_dir, remember_path),
        PieceStatus::skill(skill_is_installed(config_dir)),
    )
}

fn inspect_mcp(mcp_config_path: &Path) -> McpFact {
    let content = match read_existing(mcp_config_path) {
        Ok(Some(text)) => text,
        Ok(None) => return McpFact::Absent,
        Err(_) => return McpFact::Unreadable,
    };
    let Ok(value) = serde_json::from_str::<Value>(&content) else {
        return McpFact::Unreadable;
    };
    let Some(servers) = value.get("mcpServers") else {
        return McpFact::Absent;
    };
    let Some(servers) = servers.as_object() else {
        return McpFact::Unreadable;
    };
    match servers.get(MCP_SERVER_NAME) {
        None => McpFact::Absent,
        Some(entry) if mcp_server_entry_matches(entry) => McpFact::Stdio,
        Some(_) => McpFact::Stale,
    }
}

fn inspect_hook(config_dir: &Path, remember_path: Option<&Path>) -> PieceStatus {
    let content = match read_existing(&config_dir.join("settings.json")) {
        Ok(None) => return PieceStatus::NotInstalled,
        Err(_) => return PieceStatus::Unreadable,
        Ok(Some(text)) => text,
    };
    let settings = match parse_settings(&content) {
        Ok(settings) => settings,
        Err(_) => return PieceStatus::Unreadable,
    };
    let Some(hooks) = settings.hooks.as_ref() else {
        return PieceStatus::NotInstalled;
    };
    let post = memorywhale_bash_commands(&hooks.post_tool_use);
    let failure = memorywhale_bash_commands(&hooks.post_tool_use_failure);
    if post.is_empty() && failure.is_empty() {
        return PieceStatus::NotInstalled;
    }
    if hook_commands_current(&post, remember_path) && hook_commands_current(&failure, remember_path)
    {
        PieceStatus::Installed
    } else {
        PieceStatus::Stale
    }
}

fn memorywhale_bash_commands(groups: &[HookGroup]) -> Vec<String> {
    groups
        .iter()
        .filter(|group| group.matcher.as_deref() == Some("Bash"))
        .flat_map(|group| group.hooks.iter().flatten())
        .filter(|entry| {
            entry
                .command
                .as_deref()
                .is_some_and(is_memorywhale_hook_command)
        })
        .filter_map(|entry| entry.command.clone())
        .collect()
}

fn hook_commands_current(commands: &[String], remember_path: Option<&Path>) -> bool {
    !commands.is_empty()
        && commands
            .iter()
            .all(|command| hook_command_is_current(command, remember_path))
}

fn hook_command_is_current(command: &str, remember_path: Option<&Path>) -> bool {
    match remember_path {
        Some(path) => command.trim() == hook_command(path),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::super::merge_settings;
    use super::*;
    use crate::integrate::report::McpStatus;
    use std::path::PathBuf;

    fn sandbox(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "mw-claude-doctor-{name}-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn remember() -> PathBuf {
        PathBuf::from("/home/me/.local/bin/mw-remember")
    }

    #[test]
    fn inspect_fresh_config_dir_reports_missing_pieces() {
        let dir = sandbox("fresh");
        let report = inspect_at(&dir, &dir.join(".claude.json"), Some(&remember()), true);
        assert!(report.detected);
        assert_eq!(report.mcp, McpStatus::NotConfigured);
        assert_eq!(report.hook, PieceStatus::NotInstalled);
        assert_eq!(report.skill, Some(PieceStatus::NotInstalled));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_partial_skill_only() {
        let dir = sandbox("skill-only");
        let skill_dir = dir.join("skills/memorywhale");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), crate::integrate::SKILL).unwrap();
        let report = inspect_at(&dir, &dir.join(".claude.json"), Some(&remember()), true);
        assert_eq!(report.mcp, McpStatus::NotConfigured);
        assert_eq!(report.hook, PieceStatus::NotInstalled);
        assert_eq!(report.skill, Some(PieceStatus::Installed));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_stale_hook_and_mcp() {
        let dir = sandbox("stale");
        let remember = remember();
        let (settings, _) = merge_settings("", &remember).unwrap();
        let stale = settings.replace(
            remember.display().to_string().as_str(),
            "/old/home/.local/bin/mw-remember",
        );
        std::fs::write(dir.join("settings.json"), stale).unwrap();
        std::fs::write(
            dir.join(".claude.json"),
            r#"{"mcpServers":{"memorywhale":{"command":"/missing/mw-mcp"}}}"#,
        )
        .unwrap();
        let report = inspect_at(&dir, &dir.join(".claude.json"), Some(&remember), true);
        assert_eq!(report.mcp, McpStatus::Stale);
        assert_eq!(report.hook, PieceStatus::Stale);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_full_install_uses_generic_mcp_probe() {
        let dir = sandbox("full");
        let remember = remember();
        let (settings, _) = merge_settings("", &remember).unwrap();
        std::fs::write(dir.join("settings.json"), settings).unwrap();
        std::fs::create_dir_all(dir.join("skills/memorywhale")).unwrap();
        std::fs::write(dir.join("skills/memorywhale/SKILL.md"), "skill").unwrap();
        std::fs::write(
            dir.join(".claude.json"),
            r#"{"mcpServers":{"memorywhale":{"command":"mw-mcp","args":[]}}}"#,
        )
        .unwrap();

        let reachable = inspect_at(&dir, &dir.join(".claude.json"), Some(&remember), true);
        assert_eq!(reachable.mcp, McpStatus::Configured { reachable: true });
        assert_eq!(reachable.hook, PieceStatus::Installed);
        assert_eq!(reachable.skill, Some(PieceStatus::Installed));

        let configured = inspect_at(&dir, &dir.join(".claude.json"), Some(&remember), false);
        assert_eq!(configured.mcp, McpStatus::Configured { reachable: false });
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_unreadable_settings_and_mcp_are_independent() {
        let dir = sandbox("unreadable");
        std::fs::write(dir.join("settings.json"), "{not json").unwrap();
        std::fs::write(dir.join(".claude.json"), "not-json").unwrap();
        std::fs::create_dir_all(dir.join("skills/memorywhale")).unwrap();
        std::fs::write(dir.join("skills/memorywhale/SKILL.md"), "skill").unwrap();
        let report = inspect_at(&dir, &dir.join(".claude.json"), Some(&remember()), true);
        assert_eq!(report.mcp, McpStatus::Unreadable);
        assert_eq!(report.hook, PieceStatus::Unreadable);
        assert_eq!(report.skill, Some(PieceStatus::Installed));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_missing_helper_does_not_report_hook_installed() {
        let dir = sandbox("no-helper");
        let remember = remember();
        let (settings, _) = merge_settings("", &remember).unwrap();
        let stale = settings.replace(
            remember.display().to_string().as_str(),
            "/old/bin/mw-remember",
        );
        std::fs::write(dir.join("settings.json"), stale).unwrap();
        let report = inspect_at(&dir, &dir.join(".claude.json"), None, false);
        assert_eq!(report.hook, PieceStatus::Stale);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_mixed_current_and_stale_hook_is_stale() {
        let dir = sandbox("mixed-hooks");
        let remember = remember();
        let current = hook_command(&remember);
        let settings = serde_json::json!({
            "hooks": {
                "PostToolUse": [{
                    "matcher": "Bash",
                    "hooks": [
                        {"type": "command", "command": current},
                        {"type": "command", "command": "\"/old/bin/mw-remember\" --from-hook claude"}
                    ]
                }],
                "PostToolUseFailure": [{
                    "matcher": "Bash",
                    "hooks": [
                        {"type": "command", "command": current}
                    ]
                }]
            }
        });
        std::fs::write(dir.join("settings.json"), settings.to_string()).unwrap();
        let report = inspect_at(&dir, &dir.join(".claude.json"), Some(&remember), false);
        assert_eq!(report.hook, PieceStatus::Stale);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_invalid_mcp_servers_container_is_unreadable() {
        let dir = sandbox("mcp-array");
        std::fs::write(dir.join(".claude.json"), r#"{"mcpServers":[]}"#).unwrap();
        let report = inspect_at(&dir, &dir.join(".claude.json"), Some(&remember()), true);
        assert_eq!(report.mcp, McpStatus::Unreadable);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_unreadable_skill_is_not_absence() {
        let dir = sandbox("bad-skill");
        std::fs::create_dir_all(dir.join("skills/memorywhale")).unwrap();
        std::fs::write(dir.join("skills/memorywhale/SKILL.md"), [0xff, 0xfe]).unwrap();
        let report = inspect_at(&dir, &dir.join(".claude.json"), Some(&remember()), false);
        assert_eq!(report.skill, Some(PieceStatus::Unreadable));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_legacy_python_hook_is_stale() {
        let dir = sandbox("legacy");
        std::fs::write(
            dir.join("settings.json"),
            r#"{
  "hooks": {
    "PostToolUse": [{
      "matcher": "Bash",
      "hooks": [{"type": "command", "command": "python3 \"/tmp/.claude/hooks/mw-record.py\""}]
    }],
    "PostToolUseFailure": [{
      "matcher": "Bash",
      "hooks": [{"type": "command", "command": "python3 \"/tmp/.claude/hooks/mw-record.py\""}]
    }]
  }
}"#,
        )
        .unwrap();
        let report = inspect_at(&dir, &dir.join(".claude.json"), Some(&remember()), false);
        assert_eq!(report.hook, PieceStatus::Stale);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
