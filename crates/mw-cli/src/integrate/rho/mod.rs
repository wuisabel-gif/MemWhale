//! Rho integration: capture hook, skill, and MCP registration.

use std::path::{Path, PathBuf};

use toml_edit::{value, Array, ArrayOfTables, DocumentMut, Item, Table, TableLike};

use super::files::{
    atomic_write, install_skill, mw_remember_executable, read_or_empty, remove_legacy_python_hook,
    remove_skill, write_or_remove, BundledLayout,
};
use crate::agent_hook::Agent;

mod mcp;
use mcp::{merge_mcp, unmerge_mcp, McpTarget};

const HOOK_ID: &str = "memorywhale-record";
const HOOK_EVENT: &str = "after_tool_use";
const HOOK_TIMEOUT: &str = "15s";
const HOOK_TOOLS: [&str; 2] = ["bash", "powershell"];
const DEFAULT_MCP_HTTP_URL: &str = "http://127.0.0.1:7071/mcp";
const USAGE: &str = "usage: mw integrate rho [--revert] [--http [url]] [--token secret]";

#[derive(Debug)]
struct CliArgs {
    revert: bool,
    http: bool,
    url: Option<String>,
    token: Option<String>,
}

/// `mw integrate rho [--revert] [--http [url]] [--token secret]`
pub fn cli(args: &[String]) -> Result<(), String> {
    let parsed = parse_cli(args)?;
    if parsed.revert {
        report_revert(uninstall()?);
        return Ok(());
    }
    let mut auth_snapshot = None;
    let mode = if parsed.http {
        let url = parsed
            .url
            .clone()
            .unwrap_or_else(|| DEFAULT_MCP_HTTP_URL.to_string());
        let kind = classify_http_url(&url)?;
        if !kind.loopback && parsed.token.is_none() {
            return Err(
                "LAN HTTP requires --token (on the server: mw-serve --lan --print-token)"
                    .to_string(),
            );
        }
        if let Some(token) = parsed.token.as_deref() {
            auth_snapshot = Some(crate::serve_auth::snapshot_mcp_authorization()?);
            crate::serve_auth::write_mcp_authorization(token)?;
        }
        let with_auth = parsed.token.is_some() || !kind.loopback;
        McpTarget::http(url, kind.http && !kind.loopback, with_auth)
    } else {
        McpTarget::stdio()
    };
    match install(&mode) {
        Ok(installed) => {
            if !parsed.http {
                crate::serve_auth::remove_mcp_authorization()?;
            }
            report_install(installed);
            Ok(())
        }
        Err(err) => {
            if let Some(previous) = auth_snapshot {
                if let Err(restore_err) =
                    crate::serve_auth::restore_mcp_authorization(previous.as_deref())
                {
                    return Err(format!(
                        "{err}; also failed to restore mcp-authorization: {restore_err}"
                    ));
                }
            }
            Err(err)
        }
    }
}

fn parse_cli(args: &[String]) -> Result<CliArgs, String> {
    let mut revert = false;
    let mut http = false;
    let mut url = None;
    let mut token = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--revert" => revert = true,
            "--http" => {
                http = true;
                if let Some(next) = args.get(i + 1) {
                    if !next.starts_with('-') {
                        url = Some(next.clone());
                        i += 1;
                    }
                }
            }
            "--token" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--token needs a value".to_string())?;
                token = Some(value.clone());
                i += 1;
            }
            _ => return Err(USAGE.to_string()),
        }
        i += 1;
    }
    if revert && (http || token.is_some()) {
        return Err(USAGE.to_string());
    }
    if token.is_some() && !http {
        return Err("--token requires --http".to_string());
    }
    Ok(CliArgs {
        revert,
        http,
        url,
        token,
    })
}

struct HttpUrlKind {
    http: bool,
    loopback: bool,
}

fn classify_http_url(url: &str) -> Result<HttpUrlKind, String> {
    let (scheme, rest) = url
        .split_once("://")
        .ok_or_else(|| format!("invalid MCP URL {url:?}: missing scheme"))?;
    let http = match scheme {
        "http" => true,
        "https" => false,
        _ => {
            return Err(format!(
                "invalid MCP URL {url:?}: scheme must be http or https"
            ))
        }
    };
    let hostport = rest.split('/').next().unwrap_or(rest);
    let host = if let Some(rest) = hostport.strip_prefix('[') {
        rest.split(']').next().unwrap_or(rest)
    } else {
        match hostport.rsplit_once(':') {
            Some((name, port)) if port.chars().all(|c| c.is_ascii_digit()) => name,
            _ => hostport,
        }
    };
    let loopback = host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback());
    Ok(HttpUrlKind { http, loopback })
}

struct InstallResult {
    config_dir: PathBuf,
    remember_path: PathBuf,
    hooks_path: PathBuf,
    config_path: PathBuf,
    skill_path: PathBuf,
    mcp_summary: String,
    mcp_auth_export: Option<PathBuf>,
}

struct RevertResult {
    config_dir: PathBuf,
    hook_removed: bool,
    skill_removed: bool,
    hooks_updated: bool,
    mcp_updated: bool,
    auth_removed: bool,
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

fn install(mode: &McpTarget) -> Result<InstallResult, String> {
    let paths = RhoPaths::resolve()?;
    let existing_hooks = read_or_empty(&paths.hooks_path)?;
    let existing_config = read_or_empty(&paths.config_path)?;
    let remember_path = mw_remember_executable()?;
    let (hooks_updated, hooks_changed) = merge_hooks(&existing_hooks, &remember_path)?;
    let (config_updated, config_changed) = merge_mcp(&existing_config, mode)?;

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
        mcp_summary: if mode.command.is_some() {
            "memorywhale stdio (mw-mcp) registered in config.toml".to_string()
        } else {
            format!(
                "memorywhale HTTP JSON-RPC at {}",
                mode.url.as_deref().unwrap_or_default()
            )
        },
        mcp_auth_export: if mode.authorization_from_env {
            crate::serve_auth::mcp_authorization_path().ok()
        } else {
            None
        },
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
    let auth_removed = crate::serve_auth::remove_mcp_authorization()?;

    Ok(RevertResult {
        config_dir: paths.bundled.config_dir,
        hook_removed,
        skill_removed,
        hooks_updated: hooks_changed,
        mcp_updated: config_changed,
        auth_removed,
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

pub(super) fn parse_toml(existing: &str, what: &str) -> Result<DocumentMut, String> {
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

pub(super) fn set_string(table: &mut dyn TableLike, key: &str, expected: &str) -> bool {
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
    println!("  mcp:      {}", result.mcp_summary);
    if let Some(path) = result.mcp_auth_export {
        println!(
            "  auth:     export MEMORYWHALE_AUTHORIZATION=\"$(tr -d '\\n' < {})\"",
            path.display()
        );
    }
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
    if result.auth_removed {
        println!("  auth:     client bearer copy removed");
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
    fn classify_loopback_and_lan_urls() {
        let loopback = classify_http_url("http://127.0.0.1:7071/mcp").unwrap();
        assert!(loopback.http && loopback.loopback);
        let lan = classify_http_url("http://192.168.1.42:7071/mcp").unwrap();
        assert!(lan.http && !lan.loopback);
        let tls = classify_http_url("https://jetson.local/mcp").unwrap();
        assert!(!tls.http && !tls.loopback);
    }

    #[test]
    fn parse_cli_http_defaults_and_rejects_lan_without_token() {
        let args = parse_cli(&["--http".to_string()]).unwrap();
        assert!(args.http && args.url.is_none() && args.token.is_none());
        assert_eq!(
            parse_cli(&["--token".to_string(), "x".into()]).unwrap_err(),
            "--token requires --http"
        );
    }
}
