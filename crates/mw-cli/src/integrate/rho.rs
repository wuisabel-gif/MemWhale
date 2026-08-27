//! Rho integration: capture hook, skill, and MCP registration.

use std::path::{Path, PathBuf};

use toml_edit::{value, Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, TableLike};

use super::files::{
    atomic_write, install_skill, mw_remember_executable, parse_revert, read_or_empty,
    remove_legacy_python_hook, remove_skill, write_or_remove, BundledLayout,
};
use crate::agent_hook::Agent;

const HOOK_ID: &str = "memorywhale-record";
const HOOK_EVENT: &str = "after_tool_use";
const HOOK_TIMEOUT: &str = "15s";
const HOOK_TOOLS: [&str; 2] = ["bash", "powershell"];
const MCP_COMMAND: &str = "mw-mcp";
const MCP_TRANSPORT: &str = "stdio";

/// `mw integrate rho [--revert]`
pub fn cli(args: &[String]) -> Result<(), String> {
    if parse_revert(args, "usage: mw integrate rho [--revert]")? {
        report_revert(uninstall()?);
    } else {
        report_install(install()?);
    }
    Ok(())
}

struct InstallResult {
    config_dir: PathBuf,
    remember_path: PathBuf,
    hooks_path: PathBuf,
    config_path: PathBuf,
    skill_path: PathBuf,
}

struct RevertResult {
    config_dir: PathBuf,
    hook_removed: bool,
    skill_removed: bool,
    hooks_updated: bool,
    mcp_updated: bool,
}

struct RhoPaths {
    bundled: BundledLayout,
    hooks_path: PathBuf,
    config_path: PathBuf,
}

impl RhoPaths {
    fn resolve() -> Result<Self, String> {
        let bundled = BundledLayout::from_config_dir(rho_home()?);
        Ok(Self {
            hooks_path: bundled.config_dir.join("hooks.toml"),
            config_path: bundled.config_dir.join("config.toml"),
            bundled,
        })
    }
}

fn install() -> Result<InstallResult, String> {
    let paths = RhoPaths::resolve()?;
    let existing_hooks = read_or_empty(&paths.hooks_path)?;
    let existing_config = read_or_empty(&paths.config_path)?;
    let remember_path = mw_remember_executable()?;
    let (hooks_updated, hooks_changed) = merge_hooks(&existing_hooks, &remember_path)?;
    let (config_updated, config_changed) = merge_mcp(&existing_config)?;

    install_skill(&paths.bundled, super::SKILL)?;
    remove_legacy_python_hook(&paths.bundled.config_dir)?;

    if hooks_changed {
        atomic_write(&paths.hooks_path, &hooks_updated)?;
    }
    if config_changed {
        atomic_write(&paths.config_path, &config_updated)?;
    }

    Ok(InstallResult {
        config_dir: paths.bundled.config_dir,
        remember_path,
        hooks_path: paths.hooks_path,
        config_path: paths.config_path,
        skill_path: paths.bundled.skill_path,
    })
}

fn uninstall() -> Result<RevertResult, String> {
    let paths = RhoPaths::resolve()?;
    let existing_hooks = read_or_empty(&paths.hooks_path)?;
    let existing_config = read_or_empty(&paths.config_path)?;
    let (hooks_updated, hooks_changed) = if paths.hooks_path.exists() {
        unmerge_hooks(&existing_hooks)?
    } else {
        (String::new(), false)
    };
    let (config_updated, config_changed) = if paths.config_path.exists() {
        unmerge_mcp(&existing_config)?
    } else {
        (String::new(), false)
    };

    if hooks_changed {
        write_or_remove(&paths.hooks_path, &hooks_updated)?;
    }
    if config_changed {
        write_or_remove(&paths.config_path, &config_updated)?;
    }

    let hook_removed = remove_legacy_python_hook(&paths.bundled.config_dir)?;
    let skill_removed = remove_skill(&paths.bundled)?;

    Ok(RevertResult {
        config_dir: paths.bundled.config_dir,
        hook_removed,
        skill_removed,
        hooks_updated: hooks_changed,
        mcp_updated: config_changed,
    })
}

fn rho_home() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("RHO_HOME") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    dirs::home_dir()
        .ok_or_else(|| "could not resolve the home directory".to_string())
        .map(|home| home.join(".rho"))
}

fn parse_toml(existing: &str, what: &str) -> Result<DocumentMut, String> {
    if existing.trim().is_empty() {
        return Ok(DocumentMut::new());
    }
    existing
        .parse::<DocumentMut>()
        .map_err(|err| format!("invalid Rho {what}; file was not changed: {err}"))
}

fn hook_command(remember_path: &Path) -> [String; 3] {
    [
        remember_path.display().to_string(),
        "--from-hook".to_string(),
        Agent::Rho.as_str().to_string(),
    ]
}

fn string_array(table: &Table, key: &str) -> Option<Vec<String>> {
    table
        .get(key)
        .and_then(|item| item.as_array())
        .map(|array| {
            array
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect()
        })
}

fn set_string(table: &mut dyn TableLike, key: &str, expected: &str) -> bool {
    if table.get(key).and_then(|item| item.as_str()) == Some(expected) {
        return false;
    }
    table.insert(key, value(expected));
    true
}

fn str_slice_eq(current: &[String], expected: &[&str]) -> bool {
    current.len() == expected.len() && current.iter().zip(expected.iter()).all(|(a, b)| a == b)
}

fn set_string_array(table: &mut Table, key: &str, expected: &[&str]) -> bool {
    if string_array(table, key).is_some_and(|current| str_slice_eq(&current, expected)) {
        return false;
    }
    let mut array = Array::new();
    for item in expected {
        array.push(*item);
    }
    table[key] = Item::Value(array.into());
    true
}

fn is_memorywhale_hook(table: &Table) -> bool {
    table.get("id").and_then(|item| item.as_str()) == Some(HOOK_ID)
}

fn hook_matches(table: &Table, hook_path: &Path) -> bool {
    let command = hook_command(hook_path);
    is_memorywhale_hook(table)
        && table.get("on").and_then(|item| item.as_str()) == Some(HOOK_EVENT)
        && string_array(table, "tools").is_some_and(|tools| str_slice_eq(&tools, &HOOK_TOOLS))
        && string_array(table, "command").is_some_and(|current| {
            str_slice_eq(
                &current,
                &[
                    command[0].as_str(),
                    command[1].as_str(),
                    command[2].as_str(),
                ],
            )
        })
        && table.get("timeout").and_then(|item| item.as_str()) == Some(HOOK_TIMEOUT)
}

fn apply_hook_fields(table: &mut Table, hook_path: &Path) -> bool {
    let command = hook_command(hook_path);
    let command_refs = [
        command[0].as_str(),
        command[1].as_str(),
        command[2].as_str(),
    ];
    let mut changed = false;
    changed |= set_string(table, "id", HOOK_ID);
    changed |= set_string(table, "on", HOOK_EVENT);
    changed |= set_string_array(table, "tools", &HOOK_TOOLS);
    changed |= set_string_array(table, "command", &command_refs);
    changed |= set_string(table, "timeout", HOOK_TIMEOUT);
    changed
}

fn require_hooks_version(doc: &DocumentMut) -> Result<(), String> {
    match doc.get("version") {
        None => Ok(()),
        Some(item) if item.as_integer() == Some(1) => Ok(()),
        Some(_) => Err(
            "unsupported Rho hooks.toml version; this installer writes version 1 and the file was not changed"
                .to_string(),
        ),
    }
}

fn merge_hooks(existing: &str, hook_path: &Path) -> Result<(String, bool), String> {
    let mut doc = parse_toml(existing, "hooks.toml")?;
    require_hooks_version(&doc)?;

    if let Some(hook) = doc.get("hook") {
        if !hook.is_array_of_tables() {
            return Err(
                "invalid Rho hooks.toml; hook must be an array of tables and the file was not changed"
                    .to_string(),
            );
        }
    }

    let mut changed = false;
    if doc.get("version").and_then(Item::as_integer) != Some(1) {
        doc["version"] = value(1);
        changed = true;
    }

    if doc.get("hook").is_none() {
        doc["hook"] = Item::ArrayOfTables(ArrayOfTables::new());
        changed = true;
    }
    let hooks = doc
        .get_mut("hook")
        .and_then(Item::as_array_of_tables_mut)
        .ok_or_else(|| {
            "invalid Rho hooks.toml; hook must be an array of tables and the file was not changed"
                .to_string()
        })?;

    let existing_index = hooks.iter().position(is_memorywhale_hook);
    if let Some(index) = existing_index {
        let already_matches = hooks
            .get(index)
            .is_some_and(|table| hook_matches(table, hook_path));
        if !changed && already_matches {
            return Ok((existing.to_string(), false));
        }
        if let Some(table) = hooks.get_mut(index) {
            changed |= apply_hook_fields(table, hook_path);
        }
    } else {
        let mut table = Table::new();
        apply_hook_fields(&mut table, hook_path);
        hooks.push(table);
        changed = true;
    }

    if !changed {
        return Ok((existing.to_string(), false));
    }
    Ok((doc.to_string(), true))
}

fn unmerge_hooks(existing: &str) -> Result<(String, bool), String> {
    if existing.trim().is_empty() {
        return Ok((String::new(), false));
    }
    let mut doc = parse_toml(existing, "hooks.toml")?;
    require_hooks_version(&doc)?;

    let Some(hook) = doc.get_mut("hook") else {
        return Ok((existing.to_string(), false));
    };
    let Some(hooks) = hook.as_array_of_tables_mut() else {
        return Err(
            "invalid Rho hooks.toml; hook must be an array of tables and the file was not changed"
                .to_string(),
        );
    };
    let before = hooks.len();
    hooks.retain(|table| !is_memorywhale_hook(table));
    if hooks.len() == before {
        return Ok((existing.to_string(), false));
    }
    if hooks.is_empty() {
        doc.as_table_mut().remove("hook");
    }
    if doc.as_table().iter().all(|(key, _)| key == "version") {
        return Ok((String::new(), true));
    }
    Ok((doc.to_string(), true))
}

fn mcp_server_matches(doc: &DocumentMut) -> bool {
    memorywhale_server(doc).is_some_and(|server| {
        server.get("transport").and_then(Item::as_str) == Some(MCP_TRANSPORT)
            && server.get("command").and_then(Item::as_str) == Some(MCP_COMMAND)
            && server.get("enabled").and_then(Item::as_bool) != Some(false)
    })
}

fn memorywhale_server(doc: &DocumentMut) -> Option<&dyn TableLike> {
    doc.get("mcp")
        .and_then(|mcp| mcp.get("servers"))
        .and_then(|servers| servers.get("memorywhale"))
        .and_then(Item::as_table_like)
}

fn require_mcp_tables(doc: &DocumentMut) -> Result<(), String> {
    if let Some(mcp) = doc.get("mcp") {
        if !mcp.is_table_like() {
            return Err(
                "invalid Rho config.toml; mcp must be a table and the file was not changed"
                    .to_string(),
            );
        }
        if let Some(servers) = mcp.get("servers") {
            if !servers.is_table_like() {
                return Err(
                    "invalid Rho config.toml; mcp.servers must be a table and the file was not changed"
                        .to_string(),
                );
            }
            if let Some(server) = servers.get("memorywhale") {
                if !server.is_table_like() {
                    return Err(
                        "invalid Rho config.toml; mcp.servers.memorywhale must be a table and the file was not changed"
                            .to_string(),
                    );
                }
            }
        }
    }
    Ok(())
}

fn table_like_mut(item: &mut Item) -> Result<&mut dyn TableLike, String> {
    item.as_table_like_mut().ok_or_else(|| {
        "invalid Rho config.toml; expected a table and the file was not changed".to_string()
    })
}

fn empty_child_table(inline: bool) -> Item {
    if inline {
        Item::Value(InlineTable::new().into())
    } else {
        let mut table = Table::new();
        table.set_implicit(true);
        Item::Table(table)
    }
}

fn memorywhale_server_table(doc: &mut DocumentMut) -> Result<&mut dyn TableLike, String> {
    let mcp_inline = doc.get("mcp").map(Item::is_inline_table).unwrap_or(false);
    let servers_inline = doc
        .get("mcp")
        .and_then(|mcp| mcp.get("servers"))
        .map(Item::is_inline_table)
        .unwrap_or(mcp_inline);
    let memorywhale_invalid = doc
        .get("mcp")
        .and_then(|mcp| mcp.get("servers"))
        .and_then(|servers| servers.get("memorywhale"))
        .is_some_and(|item| !item.is_table_like());
    if memorywhale_invalid {
        return Err(
            "invalid Rho config.toml; mcp.servers.memorywhale must be a table and the file was not changed"
                .to_string(),
        );
    }

    let root = doc.as_table_mut();
    if !root.contains_key("mcp") {
        let mut mcp = Table::new();
        mcp.set_implicit(true);
        root.insert("mcp", Item::Table(mcp));
    }

    {
        let mcp = table_like_mut(root.get_mut("mcp").expect("mcp exists"))?;
        if !mcp.contains_key("servers") {
            mcp.insert("servers", empty_child_table(mcp_inline));
        }
    }

    let memorywhale_missing = root
        .get("mcp")
        .and_then(|mcp| mcp.get("servers"))
        .and_then(|servers| servers.get("memorywhale"))
        .is_none();
    if memorywhale_missing {
        let mcp = table_like_mut(root.get_mut("mcp").expect("mcp exists"))?;
        let servers = table_like_mut(
            mcp.get_mut("servers")
                .expect("servers was just inserted or already existed"),
        )?;
        servers.insert("memorywhale", empty_child_table(servers_inline));
    }

    table_like_mut(
        root.get_mut("mcp")
            .expect("mcp exists")
            .get_mut("servers")
            .expect("servers exists")
            .get_mut("memorywhale")
            .expect("memorywhale entry exists"),
    )
}

fn merge_mcp(existing: &str) -> Result<(String, bool), String> {
    let mut doc = parse_toml(existing, "config.toml")?;
    require_mcp_tables(&doc)?;
    if mcp_server_matches(&doc) {
        return Ok((existing.to_string(), false));
    }
    let server = memorywhale_server_table(&mut doc)?;
    let mut changed = false;
    changed |= set_string(server, "transport", MCP_TRANSPORT);
    changed |= set_string(server, "command", MCP_COMMAND);
    if server.get("enabled").and_then(Item::as_bool) == Some(false) {
        server.insert("enabled", value(true));
        changed = true;
    }
    if !changed {
        return Ok((existing.to_string(), false));
    }
    Ok((doc.to_string(), true))
}

fn unmerge_mcp(existing: &str) -> Result<(String, bool), String> {
    if existing.trim().is_empty() {
        return Ok((String::new(), false));
    }
    let mut doc = parse_toml(existing, "config.toml")?;
    require_mcp_tables(&doc)?;

    let Some(mcp) = doc.get_mut("mcp") else {
        return Ok((existing.to_string(), false));
    };
    let Some(mcp_table) = mcp.as_table_like_mut() else {
        return Err(
            "invalid Rho config.toml; mcp must be a table and the file was not changed".to_string(),
        );
    };
    let Some(servers) = mcp_table.get_mut("servers") else {
        return Ok((existing.to_string(), false));
    };
    let Some(servers_table) = servers.as_table_like_mut() else {
        return Err(
            "invalid Rho config.toml; mcp.servers must be a table and the file was not changed"
                .to_string(),
        );
    };
    if servers_table.remove("memorywhale").is_none() {
        return Ok((existing.to_string(), false));
    }
    if servers_table.is_empty() {
        mcp_table.remove("servers");
    }
    if mcp_table.is_empty() {
        doc.as_table_mut().remove("mcp");
    }
    if doc.as_table().is_empty() {
        return Ok((String::new(), true));
    }
    Ok((doc.to_string(), true))
}

fn report_install(result: InstallResult) {
    println!("MemoryWhale installed for Rho.");
    println!("  config:   {}", result.config_dir.display());
    println!(
        "  hook:     {} --from-hook {}",
        result.remember_path.display(),
        Agent::Rho.as_str()
    );
    println!("  hooks:    {}", result.hooks_path.display());
    println!("  settings: {}", result.config_path.display());
    println!("  skill:    {}", result.skill_path.display());
    println!("  mcp:      memorywhale registered in config.toml");
    println!("Restart Rho to pick up hook, skill, and MCP changes.");
}

fn report_revert(result: RevertResult) {
    println!("MemoryWhale removed from Rho.");
    println!("  config:   {}", result.config_dir.display());
    if result.hook_removed {
        println!("  hook:     removed");
    }
    if result.skill_removed {
        println!("  skill:    removed");
    }
    if result.hooks_updated {
        println!("  hooks:    MemoryWhale hook entry removed");
    }
    if result.mcp_updated {
        println!("  mcp:      memorywhale unregistered");
    }
    println!("Restart Rho to pick up the change.");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hook(path: &str) -> PathBuf {
        PathBuf::from(path)
    }

    #[test]
    fn merge_hooks_adds_entry_to_empty_config() {
        let path = hook("/home/me/.local/bin/mw-remember");
        let (merged, changed) = merge_hooks("", &path).unwrap();
        assert!(changed);
        let doc: DocumentMut = merged.parse().unwrap();
        assert_eq!(doc["version"].as_integer(), Some(1));
        let table = doc["hook"].as_array_of_tables().unwrap().get(0).unwrap();
        assert!(hook_matches(table, &path));
    }

    #[test]
    fn merge_hooks_preserves_other_hooks_and_is_idempotent() {
        let path = hook("/tmp/bin/mw-remember");
        let original = r#"version = 1

[[hook]]
id = "fmt-rust"
on = "after_tool_use"
tools = ["edit", "write"]
command = ["./.rho/hooks/fmt-rust"]
timeout = "5s"
"#;
        let (once, changed_once) = merge_hooks(original, &path).unwrap();
        assert!(changed_once);
        let doc: DocumentMut = once.parse().unwrap();
        let hooks = doc["hook"].as_array_of_tables().unwrap();
        assert_eq!(hooks.len(), 2);
        assert_eq!(hooks.get(0).unwrap()["id"].as_str(), Some("fmt-rust"));

        let (twice, changed_twice) = merge_hooks(&once, &path).unwrap();
        assert!(!changed_twice);
        assert_eq!(twice, once);
    }

    #[test]
    fn merge_hooks_updates_stale_hook_path() {
        let path = hook("/new/home/.local/bin/mw-remember");
        let existing = r#"version = 1

[[hook]]
id = "memorywhale-record"
on = "after_tool_use"
tools = ["bash", "powershell"]
command = ["python3", "/old/home/.rho/hooks/mw-record.py"]
timeout = "15s"
"#;
        let (merged, changed) = merge_hooks(existing, &path).unwrap();
        assert!(changed);
        let table = merged.parse::<DocumentMut>().unwrap()["hook"]
            .as_array_of_tables()
            .unwrap()
            .get(0)
            .unwrap()
            .clone();
        assert_eq!(
            string_array(&table, "command").unwrap(),
            vec![
                path.display().to_string(),
                "--from-hook".to_string(),
                "rho".to_string()
            ]
        );
    }

    #[test]
    fn merge_hooks_rejects_invalid_toml() {
        let err = merge_hooks("version = [", &hook("/tmp/mw-remember")).unwrap_err();
        assert!(err.contains("invalid Rho hooks.toml"));
    }

    #[test]
    fn merge_hooks_rejects_unsupported_version() {
        let err = merge_hooks("version = 2\n", &hook("/tmp/mw-remember")).unwrap_err();
        assert!(err.contains("unsupported Rho hooks.toml version"));
    }

    #[test]
    fn unmerge_hooks_removes_only_memorywhale_hook() {
        let path = hook("/tmp/bin/mw-remember");
        let (installed, _) = merge_hooks(
            r#"version = 1

[[hook]]
id = "fmt-rust"
on = "after_tool_use"
command = ["./fmt"]
timeout = "5s"
"#,
            &path,
        )
        .unwrap();
        let (reverted, changed) = unmerge_hooks(&installed).unwrap();
        assert!(changed);
        let doc: DocumentMut = reverted.parse().unwrap();
        let hooks = doc["hook"].as_array_of_tables().unwrap();
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks.get(0).unwrap()["id"].as_str(), Some("fmt-rust"));
    }

    #[test]
    fn unmerge_hooks_drops_empty_file() {
        let path = hook("/tmp/bin/mw-remember");
        let (installed, _) = merge_hooks("", &path).unwrap();
        let (reverted, changed) = unmerge_hooks(&installed).unwrap();
        assert!(changed);
        assert!(reverted.trim().is_empty());
    }

    #[test]
    fn unmerge_hooks_is_unchanged_without_memorywhale_hook() {
        let original = "version = 1\n";
        let (updated, changed) = unmerge_hooks(original).unwrap();
        assert!(!changed);
        assert_eq!(updated, original);
    }

    #[test]
    fn merge_mcp_adds_server_to_empty_config() {
        let (merged, changed) = merge_mcp("").unwrap();
        assert!(changed);
        let doc: DocumentMut = merged.parse().unwrap();
        let server = doc["mcp"]["servers"]["memorywhale"].as_table().unwrap();
        assert_eq!(server["transport"].as_str(), Some("stdio"));
        assert_eq!(server["command"].as_str(), Some("mw-mcp"));
    }

    #[test]
    fn merge_mcp_preserves_other_settings_and_is_idempotent() {
        let original = r#"# keep me
[model]
provider = "openai"

[mcp.servers.filesystem]
transport = "stdio"
command = "npx"
"#;
        let (once, changed_once) = merge_mcp(original).unwrap();
        assert!(changed_once);
        assert!(once.contains("# keep me"));
        assert!(once.contains("provider = \"openai\""));
        assert!(once.contains("command = \"npx\""));
        assert!(once.contains("memorywhale"));

        let (twice, changed_twice) = merge_mcp(&once).unwrap();
        assert!(!changed_twice);
        assert_eq!(twice, once);
    }

    #[test]
    fn merge_mcp_preserves_existing_server_env() {
        let original = r#"[mcp.servers.memorywhale]
transport = "stdio"
command = "old-mcp"
env = { MEMORYWHALE_DATA_DIR = "/custom" }
"#;
        let (merged, changed) = merge_mcp(original).unwrap();
        assert!(changed);
        assert!(merged.contains("MEMORYWHALE_DATA_DIR"));
        let server = merged.parse::<DocumentMut>().unwrap()["mcp"]["servers"]["memorywhale"]
            .as_table()
            .cloned()
            .unwrap();
        assert_eq!(server["command"].as_str(), Some("mw-mcp"));
        assert_eq!(server["transport"].as_str(), Some("stdio"));
    }

    #[test]
    fn merge_mcp_enables_disabled_server() {
        let original = r#"[mcp.servers.memorywhale]
transport = "stdio"
command = "mw-mcp"
enabled = false
"#;
        let (merged, changed) = merge_mcp(original).unwrap();
        assert!(changed);
        let doc = merged.parse::<DocumentMut>().unwrap();
        let server = doc["mcp"]["servers"]["memorywhale"]
            .as_table_like()
            .expect("memorywhale server table");
        assert_eq!(server.get("enabled").and_then(Item::as_bool), Some(true));
    }

    #[test]
    fn merge_mcp_accepts_inline_memorywhale_table() {
        let original = r#"[mcp.servers]
memorywhale = { transport = "stdio", command = "old-mcp" }
"#;
        let (merged, changed) = merge_mcp(original).unwrap();
        assert!(changed);
        let doc = merged.parse::<DocumentMut>().unwrap();
        let server = doc["mcp"]["servers"]["memorywhale"]
            .as_table_like()
            .expect("inline memorywhale table");
        assert_eq!(server.get("command").and_then(Item::as_str), Some("mw-mcp"));
    }

    #[test]
    fn unmerge_mcp_removes_inline_memorywhale_table() {
        let original = r#"[model]
provider = "openai"

[mcp.servers]
memorywhale = { transport = "stdio", command = "mw-mcp" }
filesystem = { transport = "stdio", command = "npx" }
"#;
        let (reverted, changed) = unmerge_mcp(original).unwrap();
        assert!(changed);
        assert!(reverted.contains("provider = \"openai\""));
        assert!(reverted.contains("filesystem"));
        assert!(!reverted.contains("memorywhale"));
    }

    #[test]
    fn merge_mcp_accepts_empty_inline_mcp_table() {
        let original = r#"model = "gpt"

mcp = {}
"#;
        let (merged, changed) = merge_mcp(original).unwrap();
        assert!(changed);
        assert!(merged.contains("model = \"gpt\""));
        let doc = merged.parse::<DocumentMut>().unwrap();
        let server = doc["mcp"]["servers"]["memorywhale"]
            .as_table_like()
            .expect("inline memorywhale table");
        assert_eq!(server.get("command").and_then(Item::as_str), Some("mw-mcp"));
    }

    #[test]
    fn unmerge_mcp_removes_empty_inline_mcp_table() {
        let original = r#"model = "gpt"

mcp = {}
"#;
        let (installed, _) = merge_mcp(original).unwrap();
        let (reverted, changed) = unmerge_mcp(&installed).unwrap();
        assert!(changed);
        assert!(reverted.contains("model = \"gpt\""));
        assert!(!reverted.contains("memorywhale"));
        assert!(!reverted.contains("servers"));
    }

    #[test]
    fn merge_mcp_accepts_empty_inline_servers_table() {
        let original = r#"model = "gpt"

mcp = { servers = {} }
"#;
        let (merged, changed) = merge_mcp(original).unwrap();
        assert!(changed);
        assert!(merged.contains("model = \"gpt\""));
        let doc = merged.parse::<DocumentMut>().unwrap();
        let server = doc["mcp"]["servers"]["memorywhale"]
            .as_table_like()
            .expect("inline memorywhale table");
        assert_eq!(server.get("command").and_then(Item::as_str), Some("mw-mcp"));
    }

    #[test]
    fn unmerge_mcp_removes_empty_inline_servers_table() {
        let original = r#"model = "gpt"

mcp = { servers = {} }
"#;
        let (installed, _) = merge_mcp(original).unwrap();
        let (reverted, changed) = unmerge_mcp(&installed).unwrap();
        assert!(changed);
        assert!(reverted.contains("model = \"gpt\""));
        assert!(!reverted.contains("memorywhale"));
        assert_eq!(reverted.trim(), "model = \"gpt\"");
    }

    #[test]
    fn merge_mcp_accepts_inline_parent_tables() {
        let original = r#"model = "gpt"

[mcp]
servers = { memorywhale = { transport = "stdio", command = "old-mcp", enabled = false } }
"#;
        let (merged, changed) = merge_mcp(original).unwrap();
        assert!(changed);
        assert!(merged.contains("model = \"gpt\""));
        let doc = merged.parse::<DocumentMut>().unwrap();
        let server = doc["mcp"]["servers"]["memorywhale"]
            .as_table_like()
            .expect("inline memorywhale table");
        assert_eq!(server.get("command").and_then(Item::as_str), Some("mw-mcp"));
        assert_eq!(server.get("enabled").and_then(Item::as_bool), Some(true));
    }

    #[test]
    fn unmerge_mcp_removes_inline_parent_tables() {
        let original = r#"model = "gpt"

[mcp]
servers = { memorywhale = { transport = "stdio", command = "mw-mcp" }, filesystem = { transport = "stdio", command = "npx" } }
"#;
        let (reverted, changed) = unmerge_mcp(original).unwrap();
        assert!(changed);
        assert!(reverted.contains("model = \"gpt\""));
        assert!(reverted.contains("filesystem"));
        assert!(!reverted.contains("memorywhale"));
    }

    #[test]
    fn merge_mcp_rejects_invalid_toml() {
        let err = merge_mcp("model = [").unwrap_err();
        assert!(err.contains("invalid Rho config.toml"));
    }

    #[test]
    fn unmerge_mcp_removes_only_memorywhale_server() {
        let (installed, _) = merge_mcp(
            r#"[model]
provider = "openai"

[mcp.servers.filesystem]
transport = "stdio"
command = "npx"
"#,
        )
        .unwrap();
        let (reverted, changed) = unmerge_mcp(&installed).unwrap();
        assert!(changed);
        assert!(reverted.contains("provider = \"openai\""));
        assert!(reverted.contains("filesystem"));
        assert!(!reverted.contains("memorywhale"));
    }

    #[test]
    fn unmerge_mcp_drops_empty_file() {
        let (installed, _) = merge_mcp("").unwrap();
        let (reverted, changed) = unmerge_mcp(&installed).unwrap();
        assert!(changed);
        assert!(reverted.trim().is_empty());
    }

    #[test]
    fn unmerge_mcp_is_unchanged_without_memorywhale() {
        let original = "provider = \"openai\"\n";
        let (updated, changed) = unmerge_mcp(original).unwrap();
        assert!(!changed);
        assert_eq!(updated, original);
    }
}
