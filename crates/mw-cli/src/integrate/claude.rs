//! Claude Code integration: capture hook, skill, and MCP registration.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::files::{
    atomic_write, install_skill, mw_remember_executable, parse_revert, read_or_empty,
    remove_legacy_python_hook, remove_skill, write_or_remove, BundledLayout,
};
use crate::agent_hook::Agent;

const MCP_ADD: &str = "claude mcp add --scope user --transport stdio memorywhale -- mw-mcp";
const MCP_REMOVE: &str = "claude mcp remove --scope user memorywhale";

/// `mw integrate claude [--revert]`
pub fn cli(args: &[String]) -> Result<(), String> {
    if parse_revert(args, "usage: mw integrate claude [--revert]")? {
        report_revert(uninstall()?);
    } else {
        report_install(install()?);
    }
    Ok(())
}

struct InstallResult {
    config_dir: PathBuf,
    remember_path: PathBuf,
    settings_path: PathBuf,
    skill_path: PathBuf,
    mcp: McpOutcome,
}

struct RevertResult {
    config_dir: PathBuf,
    hook_removed: bool,
    skill_removed: bool,
    settings_updated: bool,
    mcp: McpOutcome,
}

enum McpOutcome {
    /// `claude mcp add`/`remove` succeeded.
    Changed,
    /// Already in the desired state.
    Unchanged,
    /// `claude` is not on PATH.
    CliMissing,
    /// `claude` ran but the add/remove failed.
    Failed,
}

struct ClaudePaths {
    bundled: BundledLayout,
    settings_path: PathBuf,
}

impl ClaudePaths {
    fn resolve() -> Result<Self, String> {
        let bundled = BundledLayout::from_config_dir(claude_config_dir()?);
        Ok(Self {
            settings_path: bundled.config_dir.join("settings.json"),
            bundled,
        })
    }
}

#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
struct ClaudeSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hooks: Option<Hooks>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
struct Hooks {
    #[serde(rename = "PostToolUse", default, skip_serializing_if = "Vec::is_empty")]
    post_tool_use: Vec<HookGroup>,
    #[serde(
        rename = "PostToolUseFailure",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    post_tool_use_failure: Vec<HookGroup>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Hooks {
    fn is_empty(&self) -> bool {
        self.post_tool_use.is_empty()
            && self.post_tool_use_failure.is_empty()
            && self.extra.is_empty()
    }
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
struct HookGroup {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    matcher: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hooks: Option<Vec<HookEntry>>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
struct HookEntry {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    hook_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    command: Option<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl HookEntry {
    fn command(remember_path: &Path) -> Self {
        Self {
            hook_type: Some("command".to_string()),
            command: Some(hook_command(remember_path)),
            extra: Map::new(),
        }
    }

    fn is_memorywhale(&self) -> bool {
        self.command
            .as_deref()
            .is_some_and(is_memorywhale_hook_command)
    }
}

fn install() -> Result<InstallResult, String> {
    let paths = ClaudePaths::resolve()?;
    let existing = read_or_empty(&paths.settings_path)?;
    let remember_path = mw_remember_executable()?;
    let (updated, settings_changed) = merge_settings(&existing, &remember_path)?;

    install_skill(&paths.bundled, super::SKILL)?;
    remove_legacy_python_hook(&paths.bundled.config_dir)?;

    if settings_changed {
        atomic_write(&paths.settings_path, &updated)?;
    }

    Ok(InstallResult {
        mcp: register_mcp(),
        config_dir: paths.bundled.config_dir,
        remember_path,
        settings_path: paths.settings_path,
        skill_path: paths.bundled.skill_path,
    })
}

fn uninstall() -> Result<RevertResult, String> {
    let paths = ClaudePaths::resolve()?;
    let existing = read_or_empty(&paths.settings_path)?;
    let (updated, settings_changed) = if paths.settings_path.exists() {
        unmerge_settings(&existing)?
    } else {
        (String::new(), false)
    };

    if settings_changed {
        write_or_remove(&paths.settings_path, &updated)?;
    }

    let hook_removed = remove_legacy_python_hook(&paths.bundled.config_dir)?;
    let skill_removed = remove_skill(&paths.bundled)?;

    Ok(RevertResult {
        mcp: unregister_mcp(),
        config_dir: paths.bundled.config_dir,
        hook_removed,
        skill_removed,
        settings_updated: settings_changed,
    })
}

fn claude_config_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("CLAUDE_CONFIG_DIR") {
        return Ok(PathBuf::from(path));
    }
    dirs::home_dir()
        .ok_or_else(|| "could not resolve the home directory".to_string())
        .map(|home| home.join(".claude"))
}

fn hook_command(remember_path: &Path) -> String {
    format!(
        "\"{}\" --from-hook {}",
        remember_path.display(),
        Agent::Claude.as_str()
    )
}

fn is_memorywhale_hook_command(command: &str) -> bool {
    let trimmed = command.trim();
    if trimmed.starts_with("python3 \"") && trimmed.ends_with("hooks/mw-record.py\"") {
        return true;
    }
    let Some((binary, rest)) = trimmed.split_once(" --from-hook") else {
        return false;
    };
    let rest = rest.trim();
    if !(rest.is_empty() || rest == "claude" || rest == "claude-code") {
        return false;
    }
    Path::new(binary.trim().trim_matches('"'))
        .file_stem()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "mw-remember")
}

fn parse_settings(existing: &str) -> Result<ClaudeSettings, String> {
    if existing.trim().is_empty() {
        return Ok(ClaudeSettings::default());
    }
    let root: Value = serde_json::from_str(existing)
        .map_err(|err| format!("invalid Claude settings.json; file was not changed: {err}"))?;
    if !root.is_object() {
        return Err("invalid Claude settings.json; expected a top-level object".to_string());
    }
    serde_json::from_value(root)
        .map_err(|err| format!("invalid Claude settings.json; file was not changed: {err}"))
}

fn serialize_settings(root: &ClaudeSettings) -> Result<String, String> {
    if root.hooks.is_none() && root.extra.is_empty() {
        return Ok(String::new());
    }
    serde_json::to_string_pretty(root)
        .map(|s| format!("{s}\n"))
        .map_err(|err| format!("failed to serialize settings.json: {err}"))
}

fn upsert_bash_hook(groups: &mut Vec<HookGroup>, entry: HookEntry) {
    remove_memorywhale_bash_hooks(groups);

    if let Some(group) = groups
        .iter_mut()
        .find(|group| group.matcher.as_deref() == Some("Bash"))
    {
        group.hooks.get_or_insert_with(Vec::new).push(entry);
    } else {
        groups.push(HookGroup {
            matcher: Some("Bash".to_string()),
            hooks: Some(vec![entry]),
            extra: Map::new(),
        });
    }
}

fn remove_memorywhale_bash_hooks(groups: &mut Vec<HookGroup>) {
    groups.retain_mut(|group| {
        if group.matcher.as_deref() != Some("Bash") {
            return true;
        }
        let Some(list) = group.hooks.as_mut() else {
            return true;
        };
        list.retain(|hook| !hook.is_memorywhale());
        !list.is_empty()
    });
}

fn merge_settings(existing: &str, remember_path: &Path) -> Result<(String, bool), String> {
    let before = parse_settings(existing)?;
    let mut root = before.clone();
    let entry = HookEntry::command(remember_path);

    let hooks = root.hooks.get_or_insert_with(Hooks::default);
    upsert_bash_hook(&mut hooks.post_tool_use, entry.clone());
    upsert_bash_hook(&mut hooks.post_tool_use_failure, entry);

    if root == before {
        return Ok((existing.to_string(), false));
    }
    Ok((serialize_settings(&root)?, true))
}

fn unmerge_settings(existing: &str) -> Result<(String, bool), String> {
    if existing.trim().is_empty() {
        return Ok((String::new(), false));
    }

    let before = parse_settings(existing)?;
    let mut root = before.clone();
    let Some(hooks) = root.hooks.as_mut() else {
        return Ok((existing.to_string(), false));
    };

    remove_memorywhale_bash_hooks(&mut hooks.post_tool_use);
    remove_memorywhale_bash_hooks(&mut hooks.post_tool_use_failure);
    if hooks.is_empty() {
        root.hooks = None;
    }

    if root == before {
        return Ok((existing.to_string(), false));
    }
    Ok((serialize_settings(&root)?, true))
}

const MCP_SERVER_NAME: &str = "memorywhale";

fn user_scoped_mcp_config_path_from(
    config_dir: Option<&Path>,
    home: Option<&Path>,
) -> Option<PathBuf> {
    if let Some(dir) = config_dir {
        return Some(dir.join(".claude.json"));
    }
    home.map(|path| path.join(".claude.json"))
}

fn user_scoped_mcp_config_path() -> Option<PathBuf> {
    let config_dir = std::env::var_os("CLAUDE_CONFIG_DIR").map(PathBuf::from);
    user_scoped_mcp_config_path_from(config_dir.as_deref(), dirs::home_dir().as_deref())
}

/// User-scoped MCP servers live in the top-level `mcpServers` map in `.claude.json`.
/// With the default config dir that file is `~/.claude.json`; when `CLAUDE_CONFIG_DIR`
/// is set, Claude Code stores it at `$CLAUDE_CONFIG_DIR/.claude.json` instead.
fn user_scoped_mcp_registered(server_name: &str) -> bool {
    user_scoped_mcp_entry(server_name).is_some_and(|entry| mcp_server_entry_matches(&entry))
}

fn user_scoped_mcp_entry(server_name: &str) -> Option<Value> {
    let path = user_scoped_mcp_config_path()?;
    let content = std::fs::read_to_string(&path).ok()?;
    user_scoped_mcp_entry_in_config(&content, server_name)
}

fn user_scoped_mcp_entry_in_config(content: &str, server_name: &str) -> Option<Value> {
    let value = serde_json::from_str::<Value>(content).ok()?;
    value
        .get("mcpServers")
        .and_then(|servers| servers.get(server_name))
        .cloned()
}

fn mcp_server_entry_matches(entry: &Value) -> bool {
    let Some(server) = entry.as_object() else {
        return false;
    };
    if server.contains_key("url") {
        return false;
    }
    match server.get("type") {
        None => {}
        Some(Value::String(kind)) if kind == "stdio" => {}
        Some(_) => return false,
    }
    if server.get("command").and_then(Value::as_str) != Some("mw-mcp") {
        return false;
    }
    match server.get("args") {
        None => true,
        Some(Value::Array(args)) => args.is_empty(),
        Some(_) => false,
    }
}

fn register_mcp() -> McpOutcome {
    if user_scoped_mcp_registered(MCP_SERVER_NAME) {
        return McpOutcome::Unchanged;
    }
    if user_scoped_mcp_entry(MCP_SERVER_NAME).is_some() {
        match Command::new("claude")
            .args(["mcp", "remove", "--scope", "user", MCP_SERVER_NAME])
            .status()
        {
            Ok(status) if status.success() => {}
            Err(err) if err.kind() == ErrorKind::NotFound => return McpOutcome::CliMissing,
            _ => return McpOutcome::Failed,
        }
    }
    match Command::new("claude")
        .args([
            "mcp",
            "add",
            "--scope",
            "user",
            "--transport",
            "stdio",
            MCP_SERVER_NAME,
            "--",
            "mw-mcp",
        ])
        .status()
    {
        Ok(status) if status.success() => McpOutcome::Changed,
        Err(err) if err.kind() == ErrorKind::NotFound => McpOutcome::CliMissing,
        _ => McpOutcome::Failed,
    }
}

fn unregister_mcp() -> McpOutcome {
    if user_scoped_mcp_entry(MCP_SERVER_NAME).is_none() {
        return McpOutcome::Unchanged;
    }
    match Command::new("claude")
        .args(["mcp", "remove", "--scope", "user", MCP_SERVER_NAME])
        .status()
    {
        Ok(status) if status.success() => McpOutcome::Changed,
        Err(err) if err.kind() == ErrorKind::NotFound => McpOutcome::CliMissing,
        _ => McpOutcome::Failed,
    }
}

fn report_install(result: InstallResult) {
    println!("MemoryWhale installed for Claude Code.");
    println!("  config:   {}", result.config_dir.display());
    println!(
        "  hook:     {} --from-hook {}",
        result.remember_path.display(),
        Agent::Claude.as_str()
    );
    println!("  settings: {}", result.settings_path.display());
    println!("  skill:    {}", result.skill_path.display());
    match result.mcp {
        McpOutcome::Changed | McpOutcome::Unchanged => {
            println!("  mcp:      memorywhale registered (user scope)");
        }
        McpOutcome::CliMissing => {
            println!(
                "  mcp:      not registered — install the Claude Code CLI and run:\n\
                          {MCP_ADD}"
            );
        }
        McpOutcome::Failed => {
            println!(
                "  mcp:      not registered — `claude mcp add` failed. Run:\n          {MCP_ADD}"
            );
        }
    }
    println!("Restart Claude Code to pick up hook and skill changes.");
}

fn report_revert(result: RevertResult) {
    println!("MemoryWhale removed from Claude Code.");
    println!("  config:   {}", result.config_dir.display());
    if result.hook_removed {
        println!("  hook:     removed");
    }
    if result.skill_removed {
        println!("  skill:    removed");
    }
    if result.settings_updated {
        println!("  settings: MemoryWhale hook entry removed");
    }
    match result.mcp {
        McpOutcome::Changed => {
            println!("  mcp:      memorywhale unregistered (user scope)");
        }
        McpOutcome::Unchanged => {}
        McpOutcome::CliMissing | McpOutcome::Failed => {
            println!(
                "  mcp:      not unregistered — run manually if needed:\n            {MCP_REMOVE}"
            );
        }
    }
    println!("Restart Claude Code to pick up the change.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_settings_adds_bash_hook_to_empty_config() {
        let hook = PathBuf::from("/home/me/.local/bin/mw-remember");
        let (merged, changed) = merge_settings("", &hook).unwrap();
        assert!(changed);
        let value: Value = serde_json::from_str(&merged).unwrap();
        let command = value["hooks"]["PostToolUse"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap();
        assert_eq!(command, hook_command(&hook));
        assert_eq!(value["hooks"]["PostToolUse"][0]["matcher"], "Bash");
        let failure_command = value["hooks"]["PostToolUseFailure"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap();
        assert_eq!(failure_command, hook_command(&hook));
    }

    #[test]
    fn merge_settings_preserves_other_settings_and_is_idempotent() {
        let hook = PathBuf::from("/tmp/bin/mw-remember");
        let original = r#"{
  "theme": "dark",
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Read",
        "hooks": [{"type": "command", "command": "echo read"}]
      }
    ]
  }
}"#;
        let (once, changed_once) = merge_settings(original, &hook).unwrap();
        assert!(changed_once);
        let parsed: Value = serde_json::from_str(&once).unwrap();
        assert_eq!(parsed["theme"], "dark");
        assert_eq!(parsed["hooks"]["PostToolUse"].as_array().unwrap().len(), 2);

        let (twice, changed_twice) = merge_settings(&once, &hook).unwrap();
        assert!(!changed_twice);
        assert_eq!(twice, once);
    }

    #[test]
    fn merge_settings_updates_stale_hook_path() {
        let hook = PathBuf::from("/new/home/.local/bin/mw-remember");
        let existing = r#"{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{"type": "command", "command": "python3 \"/old/home/.claude/hooks/mw-record.py\""}]
      }
    ]
  }
}"#;
        let (merged, changed) = merge_settings(existing, &hook).unwrap();
        assert!(changed);
        let command = serde_json::from_str::<Value>(&merged).unwrap()["hooks"]["PostToolUse"][0]
            ["hooks"][0]["command"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(command, hook_command(&hook));
    }

    #[test]
    fn merge_settings_rejects_invalid_json() {
        let err = merge_settings("{not json", &PathBuf::from("/tmp/mw-remember")).unwrap_err();
        assert!(err.contains("invalid Claude settings.json"));
    }

    #[test]
    fn unmerge_settings_removes_only_memorywhale_bash_hook() {
        let hook = PathBuf::from("/tmp/bin/mw-remember");
        let (installed, _) = merge_settings(
            r#"{
  "theme": "dark",
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Read",
        "hooks": [{"type": "command", "command": "echo read"}]
      },
      {
        "matcher": "Bash",
        "hooks": [
          {"type": "command", "command": "echo other"},
          {"type": "command", "command": "python3 \"/tmp/.claude/hooks/mw-record.py\""}
        ]
      }
    ]
  }
}"#,
            &hook,
        )
        .unwrap();
        let (reverted, changed) = unmerge_settings(&installed).unwrap();
        assert!(changed);
        let parsed: Value = serde_json::from_str(&reverted).unwrap();
        assert_eq!(parsed["theme"], "dark");
        let bash_hooks = parsed["hooks"]["PostToolUse"]
            .as_array()
            .unwrap()
            .iter()
            .find(|group| group.get("matcher") == Some(&json!("Bash")))
            .unwrap()["hooks"]
            .as_array()
            .unwrap();
        assert_eq!(bash_hooks.len(), 1);
        assert_eq!(bash_hooks[0]["command"], "echo other");
    }

    #[test]
    fn unmerge_settings_drops_empty_hook_groups() {
        let hook = PathBuf::from("/tmp/bin/mw-remember");
        let (installed, _) = merge_settings("", &hook).unwrap();
        let (reverted, changed) = unmerge_settings(&installed).unwrap();
        assert!(changed);
        assert!(reverted.trim().is_empty());
    }

    #[test]
    fn unmerge_settings_is_unchanged_without_memorywhale_hook() {
        let original = r#"{"theme":"dark"}"#;
        let (updated, changed) = unmerge_settings(original).unwrap();
        assert!(!changed);
        assert_eq!(updated, original);
    }

    #[test]
    fn merge_settings_deduplicates_memorywhale_hooks_across_bash_groups() {
        let hook = PathBuf::from("/tmp/bin/mw-remember");
        let existing = r#"{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{"type": "command", "command": "echo first"}]
      },
      {
        "matcher": "Bash",
        "hooks": [{"type": "command", "command": "python3 \"/old/.claude/hooks/mw-record.py\""}]
      }
    ]
  }
}"#;
        let (merged, changed) = merge_settings(existing, &hook).unwrap();
        assert!(changed);
        let parsed = serde_json::from_str::<Value>(&merged).unwrap();
        let groups = parsed["hooks"]["PostToolUse"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|group| group.get("matcher") == Some(&json!("Bash")))
            .collect::<Vec<_>>();
        assert_eq!(groups.len(), 1);
        let commands = groups[0]["hooks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hook| hook["command"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(commands, vec!["echo first", hook_command(&hook).as_str()]);
    }

    #[test]
    fn user_scoped_mcp_config_path_honors_claude_config_dir() {
        let custom = PathBuf::from("/tmp/custom-claude");
        assert_eq!(
            user_scoped_mcp_config_path_from(Some(custom.as_path()), None),
            Some(custom.join(".claude.json"))
        );
        let home = PathBuf::from("/home/me");
        assert_eq!(
            user_scoped_mcp_config_path_from(None, Some(home.as_path())),
            Some(home.join(".claude.json"))
        );
    }

    #[test]
    fn user_scoped_mcp_registered_in_config_reads_top_level_servers_only() {
        let config = r#"{
  "projects": {
    "/tmp/repo": {
      "mcpServers": {
        "memorywhale": {"command": "mw-mcp"}
      }
    }
  }
}"#;
        assert!(user_scoped_mcp_entry_in_config(config, "memorywhale").is_none());

        let config = r#"{
  "mcpServers": {
    "memorywhale": {"command": "mw-mcp", "args": []}
  }
}"#;
        let entry = user_scoped_mcp_entry_in_config(config, "memorywhale").unwrap();
        assert!(mcp_server_entry_matches(&entry));
        assert!(user_scoped_mcp_registered_in_config_matches(
            config,
            "memorywhale"
        ));
    }

    #[test]
    fn user_scoped_mcp_registered_rejects_stale_command() {
        let config = r#"{
  "mcpServers": {
    "memorywhale": {"command": "/missing/mw-mcp"}
  }
}"#;
        let entry = user_scoped_mcp_entry_in_config(config, "memorywhale").unwrap();
        assert!(!mcp_server_entry_matches(&entry));
        assert!(!user_scoped_mcp_registered_in_config_matches(
            config,
            "memorywhale"
        ));
    }

    #[test]
    fn user_scoped_mcp_registered_rejects_non_stdio_transport() {
        let config = r#"{
  "mcpServers": {
    "memorywhale": {
      "type": "http",
      "url": "https://example.com/mcp",
      "command": "mw-mcp",
      "args": []
    }
  }
}"#;
        let entry = user_scoped_mcp_entry_in_config(config, "memorywhale").unwrap();
        assert!(!mcp_server_entry_matches(&entry));
        assert!(!user_scoped_mcp_registered_in_config_matches(
            config,
            "memorywhale"
        ));
    }

    #[test]
    fn user_scoped_mcp_registered_rejects_null_transport() {
        let config = r#"{
  "mcpServers": {
    "memorywhale": {"type": null, "command": "mw-mcp", "args": []}
  }
}"#;
        let entry = user_scoped_mcp_entry_in_config(config, "memorywhale").unwrap();
        assert!(!mcp_server_entry_matches(&entry));
        assert!(!user_scoped_mcp_registered_in_config_matches(
            config,
            "memorywhale"
        ));
    }

    fn user_scoped_mcp_registered_in_config_matches(content: &str, server_name: &str) -> bool {
        user_scoped_mcp_entry_in_config(content, server_name)
            .is_some_and(|entry| mcp_server_entry_matches(&entry))
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_preserves_existing_file_permissions() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!(
            "mw-claude-perms-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let settings = dir.join("settings.json");
        fs::write(&settings, r#"{"theme":"dark"}"#).unwrap();
        fs::set_permissions(&settings, fs::Permissions::from_mode(0o600)).unwrap();

        atomic_write(&settings, r#"{"theme":"light"}"#).unwrap();

        assert_eq!(
            fs::metadata(&settings).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn is_memorywhale_hook_command_matches_python_and_remember() {
        assert!(is_memorywhale_hook_command(
            "python3 \"/home/me/.claude/hooks/mw-record.py\""
        ));
        assert!(is_memorywhale_hook_command(
            "\"/usr/local/bin/mw-remember\" --from-hook"
        ));
        assert!(is_memorywhale_hook_command(
            "\"/usr/local/bin/mw-remember\" --from-hook claude"
        ));
        assert!(!is_memorywhale_hook_command(
            "echo mentions mw-record.py in text"
        ));
        assert!(!is_memorywhale_hook_command("echo --from-hook"));
        assert!(!is_memorywhale_hook_command(
            "\"/usr/local/bin/mw-remember\" --from-hook rho"
        ));
    }

    #[test]
    fn unmerge_settings_does_not_remove_unrelated_bash_hooks() {
        let original = r#"{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{"type": "command", "command": "echo mentions mw-record.py in text"}]
      }
    ]
  }
}"#;
        let (updated, changed) = unmerge_settings(original).unwrap();
        assert!(!changed);
        assert_eq!(updated, original);
    }
}
