//! Capture-only Cursor configuration adapter (Interfaces). Never executes a client.
//! Reuses the MCP installer's generic guarded file transaction, not MCP setup.
use super::mcp_client::{
    absolute, atomic, atomic_tracked, guard_sources, read, safe, sidecar, usable_executable,
    with_lock, UniqueJson, LIMIT,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

const EVENTS: [&str; 2] = ["postToolUse", "postToolUseFailure"];
const RECOVERY: &str = "Cursor capture ownership is interrupted or modified; manual recovery required (contents withheld)";
const STALE: &str = "Cursor capture executable or data-directory settings changed; explicitly --revert with the same --hooks-file selection, then reinstall";
const INVALID: &str = "Invalid Cursor hooks configuration (contents withheld)";

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

fn desired() -> Result<Value, String> {
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
            std::env::current_dir().map_err(|_| INVALID)?.join(dir)
        };
        command.push_str("MEMORYWHALE_DATA_DIR=");
        command.push_str(&quote(dir.to_str().ok_or("Data directory is not UTF-8")?));
        command.push(' ');
    }
    command.push_str(&quote(exe.to_str().ok_or("mw-remember path is not UTF-8")?));
    command.push_str(" --from-hook cursor");
    let entry = json!({"type":"command", "command":command, "matcher":"Shell", "timeout":5});
    Ok(json!({"postToolUse":entry, "postToolUseFailure":entry}))
}

fn parse(bytes: Option<&[u8]>) -> Result<Value, String> {
    let mut doc =
        serde_json::from_slice::<UniqueJson>(bytes.unwrap_or(b"{\"version\":1,\"hooks\":{}}"))
            .map_err(|_| INVALID)?
            .0;
    if !doc.is_object() || doc.get("version").and_then(Value::as_u64) != Some(1) {
        return Err("Cursor hooks requires version 1 and an unambiguous JSON object".into());
    }
    if doc.get("hooks").is_none() {
        doc["hooks"] = json!({});
    }
    let hooks = doc["hooks"].as_object().ok_or(INVALID)?;
    for entries in hooks.values() {
        let entries = entries.as_array().ok_or(INVALID)?;
        if entries
            .iter()
            .any(|e| !e.is_object() || e.get("command").is_some_and(|c| !c.is_string()))
        {
            return Err(INVALID.into());
        }
    }
    Ok(doc)
}

fn ours(entry: &Value) -> bool {
    entry
        .get("command")
        .and_then(Value::as_str)
        .is_some_and(|s| {
            let s = s.to_ascii_lowercase();
            s.contains("mw-remember") || s.contains("--from-hook cursor")
        })
}

pub fn cli(args: &[String]) -> Result<(), String> {
    run(args).map_err(|e| e.replace("MCP", "Cursor capture"))
}

fn run(args: &[String]) -> Result<(), String> {
    if !cfg!(any(target_os = "linux", target_os = "macos")) {
        return Err("Cursor capture installation supports local Linux, macOS and WSL only".into());
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
            _ => return Err("Usage: mw integrate cursor --capture [--hooks-file PATH] [--dry-run | --check | --revert]".into()),
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
            .join(".cursor/hooks.json"),
    })?;
    let journal = sidecar(&path, ".memorywhale-cursor-capture-owner.json");
    let lock = sidecar(&path, ".memorywhale-cursor-capture.lock");
    safe(&lock)?;
    if fs::symlink_metadata(&lock).is_ok() {
        return Err(RECOVERY.into());
    }
    let original = read(&path)?;
    let owned_bytes = read(&journal)?;
    let mut doc = parse(original.as_deref())?;
    let owner: Option<Owner> = owned_bytes
        .as_ref()
        .map(|b| {
            let value = serde_json::from_slice::<UniqueJson>(b)
                .map_err(|_| RECOVERY)?
                .0;
            serde_json::from_value(value).map_err(|_| RECOVERY)
        })
        .transpose()?;
    if let Some(o) = &owner {
        if o.version != 1
            || o.state != "active"
            || o.entries.as_object().map(|v| v.len()) != Some(2)
            || EVENTS
                .iter()
                .any(|e| !o.entries[*e].is_object() || !ours(&o.entries[*e]))
        {
            return Err(RECOVERY.into());
        }
        for event in EVENTS {
            let count = doc["hooks"][event]
                .as_array()
                .map(|a| a.iter().filter(|v| **v == o.entries[event]).count());
            if count != Some(1) {
                return Err(RECOVERY.into());
            }
        }
    }
    for (event, entries) in doc["hooks"].as_object().ok_or(INVALID)? {
        for entry in entries.as_array().ok_or(INVALID)? {
            if ours(entry)
                && !owner
                    .as_ref()
                    .is_some_and(|o| EVENTS.contains(&event.as_str()) && o.entries[event] == *entry)
            {
                return Err("Unowned MemoryWhale hook conflict; no configuration changed".into());
            }
        }
    }
    if mode == "--revert" && owner.is_none() {
        println!("No owned Cursor capture hooks to remove.");
        return Ok(());
    }
    if mode == "--check" && owner.is_none() {
        return Err("Cursor capture hooks are not installed".into());
    }
    let entries = if mode == "--revert" {
        owner.as_ref().ok_or(RECOVERY)?.entries.clone()
    } else {
        let entries = desired().map_err(|e| {
            if owner.is_some() {
                STALE.to_string()
            } else {
                e
            }
        })?;
        if let Some(o) = &owner {
            if o.entries != entries {
                return Err(STALE.into());
            }
            println!("Cursor capture local configuration and executable paths match; live capture not tested. No files changed.");
            return Ok(());
        }
        entries
    };
    for event in EVENTS {
        if mode == "--revert" {
            doc["hooks"][event]
                .as_array_mut()
                .ok_or(RECOVERY)?
                .retain(|e| *e != entries[event]);
        } else {
            if doc["hooks"].get(event).is_none() {
                doc["hooks"][event] = json!([]);
            }
            doc["hooks"][event]
                .as_array_mut()
                .ok_or(INVALID)?
                .push(entries[event].clone());
        }
    }
    let output = serde_json::to_vec_pretty(&doc).map_err(|_| INVALID)?;
    if output.len() as u64 > LIMIT {
        return Err("Cursor hooks exceeds supported size limit".into());
    }
    if mode == "--dry-run" {
        println!("Would install two Shell-only Cursor capture hooks; no files written, live capture not tested.");
    } else {
        let parent = path.parent().ok_or(INVALID)?;
        safe(parent)?;
        fs::create_dir_all(parent).map_err(|_| INVALID)?;
        with_lock(&lock, |started| {
            guard_sources(&path, &original, &journal, &owned_bytes)?;
            let mut next = Owner {
                version: 1,
                state: "pending".into(),
                entries,
            };
            let pending = serde_json::to_vec(&next).map_err(|_| INVALID)?;
            atomic_tracked(&journal, &owned_bytes, &pending, started)?;
            atomic(&path, &original, &output)?;
            if mode == "--revert" {
                if read(&journal)? != Some(pending) {
                    return Err(RECOVERY.into());
                }
                fs::remove_file(&journal).map_err(|_| RECOVERY)?;
            } else {
                next.state = "active".into();
                atomic(
                    &journal,
                    &Some(pending),
                    &serde_json::to_vec(&next).map_err(|_| INVALID)?,
                )?;
            }
            Ok(())
        })?;
        println!("Cursor capture hooks {}. Live capture not tested; normal Cursor hook trust still applies.", if mode == "--revert" { "removed" } else { "installed" });
    }
    if custom {
        println!("Custom hooks file only; not automatically registered with Cursor.");
    }
    Ok(())
}
