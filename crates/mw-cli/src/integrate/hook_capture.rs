//! Capture-only hook configuration adapter for Cursor and Codex (Interfaces).
//! Never executes a client. Reuses the MCP installer's generic guarded file
//! transaction, not MCP setup.
use super::mcp_client::{
    absolute, atomic, atomic_tracked, guard_sources, read, safe, sidecar, usable_executable,
    with_lock, UniqueJson, LIMIT,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

/// What differs between the clients whose hooks file this adapter edits.
struct Client {
    name: &'static str,
    /// The `mw integrate` / `--from-hook` name.
    flag: &'static str,
    /// Hooks file under `$HOME`.
    default_file: &'static str,
    events: &'static [&'static str],
    /// The `version` the file must declare, if the client uses one.
    version: Option<u64>,
    empty: &'static [u8],
    entry: fn(String) -> Value,
}

// https://cursor.com/docs/agent/hooks
const CURSOR: Client = Client {
    name: "Cursor",
    flag: "cursor",
    default_file: ".cursor/hooks.json",
    events: &["postToolUse", "postToolUseFailure"],
    version: Some(1),
    empty: b"{\"version\":1,\"hooks\":{}}",
    entry: |command| json!({"type":"command", "command":command, "matcher":"Shell", "timeout":5}),
};

// https://developers.openai.com/codex/hooks: entries are matcher groups.
const CODEX: Client = Client {
    name: "Codex",
    flag: "codex",
    default_file: ".codex/hooks.json",
    events: &["PostToolUse"],
    version: None,
    empty: b"{\"hooks\":{}}",
    entry: |command| json!({"matcher":"Bash", "hooks":[{"type":"command", "command":command, "timeout":5}]}),
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Owner {
    version: u8,
    state: String,
    entries: Value,
}

fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\"'\"'"))
}

fn desired(client: &Client) -> Result<Value, String> {
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("mw-remember"));
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(
            std::env::split_paths(&path)
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.join("mw-remember")),
        );
    }
    let exe = candidates
        .into_iter()
        .filter_map(|p| fs::canonicalize(p).ok())
        .find(|p| usable_executable(p))
        .ok_or("Cannot locate executable mw-remember beside mw or on PATH")?;
    let mut command = String::new();
    if let Some(dir) = std::env::var_os("MEMORYWHALE_DATA_DIR") {
        let dir = PathBuf::from(dir);
        let dir = if dir.is_absolute() {
            dir
        } else {
            std::env::current_dir()
                .map_err(|_| invalid(client))?
                .join(dir)
        };
        command.push_str("MEMORYWHALE_DATA_DIR=");
        command.push_str(&quote(dir.to_str().ok_or("Data directory is not UTF-8")?));
        command.push(' ');
    }
    command.push_str(&quote(exe.to_str().ok_or("mw-remember path is not UTF-8")?));
    command.push_str(" --from-hook ");
    command.push_str(client.flag);
    let entry = (client.entry)(command);
    Ok(Value::Object(
        client
            .events
            .iter()
            .map(|event| (event.to_string(), entry.clone()))
            .collect(),
    ))
}

fn invalid(client: &Client) -> String {
    format!(
        "Invalid {} hooks configuration (contents withheld)",
        client.name
    )
}

fn recovery(client: &Client) -> String {
    format!("{} capture ownership is interrupted or modified; manual recovery required (contents withheld)", client.name)
}

fn stale(client: &Client) -> String {
    format!("{} capture executable or data-directory settings changed; explicitly --revert with the same --hooks-file selection, then reinstall", client.name)
}

fn parse(client: &Client, bytes: Option<&[u8]>) -> Result<Value, String> {
    let mut doc = serde_json::from_slice::<UniqueJson>(bytes.unwrap_or(client.empty))
        .map_err(|_| invalid(client))?
        .0;
    if !doc.is_object()
        || client
            .version
            .is_some_and(|v| doc.get("version").and_then(Value::as_u64) != Some(v))
    {
        return Err(match client.version {
            Some(v) => format!(
                "{} hooks requires version {v} and an unambiguous JSON object",
                client.name
            ),
            None => format!("{} hooks requires an unambiguous JSON object", client.name),
        });
    }
    if doc.get("hooks").is_none() {
        doc["hooks"] = json!({});
    }
    let hooks = doc["hooks"].as_object().ok_or_else(|| invalid(client))?;
    for entries in hooks.values() {
        let entries = entries.as_array().ok_or_else(|| invalid(client))?;
        let bad = |e: &Value| !e.is_object() || e.get("command").is_some_and(|c| !c.is_string());
        if entries.iter().any(|e| {
            bad(e)
                || e.get("hooks").is_some_and(|nested| {
                    nested
                        .as_array()
                        .is_none_or(|nested| nested.iter().any(bad))
                })
        }) {
            return Err(invalid(client));
        }
    }
    Ok(doc)
}

/// A hook entry (or a Codex matcher group holding one) that runs MemoryWhale.
fn ours(entry: &Value) -> bool {
    let runs_mw = |e: &Value| {
        e.get("command").and_then(Value::as_str).is_some_and(|s| {
            let s = s.to_ascii_lowercase();
            s.contains("mw-remember") || s.contains("--from-hook")
        })
    };
    runs_mw(entry)
        || entry
            .get("hooks")
            .and_then(Value::as_array)
            .is_some_and(|nested| nested.iter().any(runs_mw))
}

pub fn cli(args: &[String]) -> Result<(), String> {
    run(&CURSOR, args).map_err(|e| e.replace("MCP", "Cursor capture"))
}

pub fn codex_cli(args: &[String]) -> Result<(), String> {
    run(&CODEX, args).map_err(|e| e.replace("MCP", "Codex capture"))
}

fn run(client: &Client, args: &[String]) -> Result<(), String> {
    let name = client.name;
    if !cfg!(any(target_os = "linux", target_os = "macos")) {
        return Err(format!(
            "{name} capture installation supports local Linux, macOS and WSL only"
        ));
    }
    let mut config = None;
    let mut mode = "install";
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--hooks-file" if config.is_none() => {
                i += 1;
                config = Some(PathBuf::from(args.get(i).filter(|s| !s.starts_with("--"))
                    .ok_or("--hooks-file requires a path")?));
            }
            "--dry-run" | "--check" | "--revert" if mode == "install" => mode = &args[i],
            _ => return Err(format!("Usage: mw integrate {} --capture [--hooks-file PATH] [--dry-run | --check | --revert]", client.flag)),
        }
        i += 1;
    }
    let custom = config.is_some();
    let path = absolute(match config {
        Some(p) => p,
        None => std::env::var_os("HOME")
            .map(PathBuf::from)
            .or_else(dirs::home_dir)
            .ok_or("Cannot determine home directory")?
            .join(client.default_file),
    })?;
    let journal = sidecar(
        &path,
        &format!(".memorywhale-{}-capture-owner.json", client.flag),
    );
    let lock = sidecar(&path, &format!(".memorywhale-{}-capture.lock", client.flag));
    safe(&lock)?;
    if fs::symlink_metadata(&lock).is_ok() {
        return Err(recovery(client));
    }
    let original = read(&path)?;
    let owned_bytes = read(&journal)?;
    let mut doc = parse(client, original.as_deref())?;
    let owner: Option<Owner> = owned_bytes
        .as_ref()
        .map(|b| {
            let value = serde_json::from_slice::<UniqueJson>(b)
                .map_err(|_| recovery(client))?
                .0;
            serde_json::from_value(value).map_err(|_| recovery(client))
        })
        .transpose()?;
    if let Some(o) = &owner {
        if o.version != 1
            || o.state != "active"
            || o.entries.as_object().map(|v| v.len()) != Some(client.events.len())
            || client
                .events
                .iter()
                .any(|e| !o.entries[*e].is_object() || !ours(&o.entries[*e]))
        {
            return Err(recovery(client));
        }
        for event in client.events {
            let count = doc["hooks"][event]
                .as_array()
                .map(|a| a.iter().filter(|v| **v == o.entries[event]).count());
            if count != Some(1) {
                return Err(recovery(client));
            }
        }
    }
    for (event, entries) in doc["hooks"].as_object().ok_or_else(|| invalid(client))? {
        for entry in entries.as_array().ok_or_else(|| invalid(client))? {
            if ours(entry)
                && !owner.as_ref().is_some_and(|o| {
                    client.events.contains(&event.as_str()) && o.entries[event] == *entry
                })
            {
                return Err("Unowned MemoryWhale hook conflict; no configuration changed".into());
            }
        }
    }
    if mode == "--revert" && owner.is_none() {
        println!("No owned {name} capture hooks to remove.");
        return Ok(());
    }
    if mode == "--check" && owner.is_none() {
        return Err(format!("{name} capture hooks are not installed"));
    }
    let entries = if mode == "--revert" {
        owner
            .as_ref()
            .ok_or_else(|| recovery(client))?
            .entries
            .clone()
    } else {
        let entries =
            desired(client).map_err(|e| if owner.is_some() { stale(client) } else { e })?;
        if let Some(o) = &owner {
            if o.entries != entries {
                return Err(stale(client));
            }
            println!("{name} capture local configuration and executable paths match; live capture not tested. No files changed.");
            return Ok(());
        }
        entries
    };
    for event in client.events {
        if mode == "--revert" {
            doc["hooks"][event]
                .as_array_mut()
                .ok_or_else(|| recovery(client))?
                .retain(|e| *e != entries[*event]);
        } else {
            if doc["hooks"].get(event).is_none() {
                doc["hooks"][event] = json!([]);
            }
            doc["hooks"][event]
                .as_array_mut()
                .ok_or_else(|| invalid(client))?
                .push(entries[*event].clone());
        }
    }
    let output = serde_json::to_vec_pretty(&doc).map_err(|_| invalid(client))?;
    if output.len() as u64 > LIMIT {
        return Err(format!("{name} hooks exceeds supported size limit"));
    }
    if mode == "--dry-run" {
        println!("Would install {name} shell-command capture hooks ({}); no files written, live capture not tested.", client.events.join(", "));
    } else {
        let parent = path.parent().ok_or_else(|| invalid(client))?;
        safe(parent)?;
        fs::create_dir_all(parent).map_err(|_| invalid(client))?;
        with_lock(&lock, |started| {
            guard_sources(&path, &original, &journal, &owned_bytes)?;
            let mut next = Owner {
                version: 1,
                state: "pending".into(),
                entries,
            };
            let pending = serde_json::to_vec(&next).map_err(|_| invalid(client))?;
            atomic_tracked(&journal, &owned_bytes, &pending, started)?;
            atomic(&path, &original, &output)?;
            if mode == "--revert" {
                if read(&journal)? != Some(pending) {
                    return Err(recovery(client));
                }
                fs::remove_file(&journal).map_err(|_| recovery(client))?;
            } else {
                next.state = "active".into();
                atomic(
                    &journal,
                    &Some(pending),
                    &serde_json::to_vec(&next).map_err(|_| invalid(client))?,
                )?;
            }
            Ok(())
        })?;
        println!("{name} capture hooks {}. Live capture not tested; normal {name} hook trust still applies.", if mode == "--revert" { "removed" } else { "installed" });
    }
    if custom {
        println!("Custom hooks file only; not automatically registered with {name}.");
    }
    Ok(())
}
