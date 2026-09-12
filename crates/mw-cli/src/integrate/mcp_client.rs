//! Thin Codex/Cursor MCP configuration adapter. No client process is executed.
//! The adjacent ownership journal stores only our entry, never a config backup.
//! Lock + source guards cover ordinary local concurrency, not hostile directory races.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

pub(super) const LIMIT: u64 = 1024 * 1024;
const RECOVERY: &str =
    "MCP setup was interrupted; manual recovery of config and ownership journal is required";
fn err() -> String {
    "Cannot safely access MCP configuration (no configuration contents shown)".into()
}
pub(super) fn absolute(path: PathBuf) -> Result<PathBuf, String> {
    let path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir().map_err(|_| err())?.join(path)
    };
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(err());
    }
    Ok(path)
}
pub(super) fn safe(path: &Path) -> Result<(), String> {
    for part in path.ancestors() {
        match fs::symlink_metadata(part) {
            Ok(m) if m.file_type().is_symlink() || (part != path && !m.is_dir()) => {
                return Err(err())
            }
            Ok(m) if part == path && !m.is_file() && !m.is_dir() => return Err(err()),
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err(err()),
        }
    }
    Ok(())
}
pub(super) fn read(path: &Path) -> Result<Option<Vec<u8>>, String> {
    safe(path)?;
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Ok(m) if m.is_file() && m.len() <= LIMIT => (),
        _ => return Err(err()),
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|_| err())?
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| err())?;
    if bytes.len() as u64 > LIMIT {
        return Err(err());
    }
    Ok(Some(bytes))
}
pub(super) fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}
fn exclusive(path: &Path) -> Result<fs::File, String> {
    safe(path)?;
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    opts.open(path).map_err(|_| {
        "MCP setup cannot obtain a private file; if interrupted, manual recovery is required".into()
    })
}
pub(super) fn atomic(path: &Path, expected: &Option<Vec<u8>>, bytes: &[u8]) -> Result<(), String> {
    atomic_tracked(path, expected, bytes, &mut false)
}
pub(super) fn atomic_tracked(
    path: &Path,
    expected: &Option<Vec<u8>>,
    bytes: &[u8],
    journal_started: &mut bool,
) -> Result<(), String> {
    if bytes.len() as u64 > LIMIT {
        return Err("MCP configuration exceeds the supported size limit".into());
    }
    if read(path)? != *expected {
        return Err(
            "MCP configuration changed concurrently; retry after inspecting ownership state".into(),
        );
    }
    let mut random = [0u8; 16];
    getrandom::getrandom(&mut random).map_err(|_| err())?;
    let temp = sidecar(
        path,
        &format!(".mw-tmp-{:032x}", u128::from_ne_bytes(random)),
    );
    let mut file = exclusive(&temp)?;
    let result = (|| {
        file.write_all(bytes).map_err(|_| err())?;
        if expected.is_some() {
            file.set_permissions(fs::metadata(path).map_err(|_| err())?.permissions())
                .map_err(|_| err())?;
        }
        file.sync_all().map_err(|_| err())?;
        if read(path)? != *expected {
            return Err("MCP configuration changed concurrently".into());
        }
        // A rename or subsequent directory sync failure may leave durable changes.
        *journal_started = true;
        fs::rename(&temp, path).map_err(|_| err())?;
        #[cfg(unix)]
        fs::File::open(path.parent().ok_or_else(err)?)
            .and_then(|f| f.sync_all())
            .map_err(|_| err())?;
        Ok(())
    })();
    let _ = fs::remove_file(&temp);
    result
}
pub(super) fn with_lock(
    lock: &Path,
    operation: impl FnOnce(&mut bool) -> Result<(), String>,
) -> Result<(), String> {
    let lock_file = exclusive(lock)?;
    let mut journal_started = false;
    let result = operation(&mut journal_started);
    drop(lock_file);
    if result.is_ok() || !journal_started {
        fs::remove_file(lock).map_err(|_| RECOVERY.to_string())?;
    }
    if result.is_err() && journal_started {
        return Err(RECOVERY.into());
    }
    result
}
pub(super) fn guard_sources(
    path: &Path,
    original: &Option<Vec<u8>>,
    journal: &Path,
    owned_bytes: &Option<Vec<u8>>,
) -> Result<(), String> {
    if read(path)? != *original || read(journal)? != *owned_bytes {
        return Err("MCP configuration changed concurrently".into());
    }
    Ok(())
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Owner {
    version: u8,
    client: String,
    state: String,
    entry: Value,
}

enum Config {
    Codex(toml_edit::DocumentMut),
    Cursor(Value),
}

// Reject ambiguous duplicate keys rather than silently discard unrelated settings.
pub(super) struct UniqueJson(pub(super) Value);
impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = UniqueJson;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("unambiguous JSON")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some((key, value)) = map.next_entry::<String, UniqueJson>()? {
                    if values.insert(key, value.0).is_some() {
                        return Err(serde::de::Error::custom("duplicate JSON key"));
                    }
                }
                Ok(UniqueJson(Value::Object(values)))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element::<UniqueJson>()? {
                    values.push(value.0);
                }
                Ok(UniqueJson(Value::Array(values)))
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(UniqueJson(json!(value)))
            }
            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(UniqueJson(json!(value)))
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(UniqueJson(json!(value)))
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(UniqueJson(json!(value)))
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Self::Value, E> {
                Ok(UniqueJson(json!(value)))
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueJson(Value::Null))
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}
impl Config {
    fn parse(client: &str, bytes: Option<&[u8]>) -> Result<Self, String> {
        let malformed = || "Malformed MCP configuration; contents withheld".to_string();
        if client == "codex" {
            let text = std::str::from_utf8(bytes.unwrap_or(b"")).map_err(|_| malformed())?;
            let doc = text
                .parse::<toml_edit::DocumentMut>()
                .map_err(|_| malformed())?;
            if doc
                .get("mcp_servers")
                .is_some_and(|v| v.as_table_like().is_none())
            {
                return Err(malformed());
            }
            Ok(Self::Codex(doc))
        } else {
            let doc = serde_json::from_slice::<UniqueJson>(bytes.unwrap_or(b"{}"))
                .map_err(|_| malformed())?
                .0;
            if !doc.is_object() || doc.get("mcpServers").is_some_and(|v| !v.is_object()) {
                return Err(malformed());
            }
            Ok(Self::Cursor(doc))
        }
    }
    fn entry(&self) -> Result<Option<Value>, String> {
        match self {
            Self::Cursor(v) => Ok(v
                .get("mcpServers")
                .and_then(|v| v.get("memorywhale"))
                .cloned()),
            Self::Codex(d) => d
                .get("mcp_servers")
                .and_then(|v| v.as_table_like())
                .and_then(|v| v.get("memorywhale"))
                .map(|item| {
                    // Canonical TOML representation includes unknown fields and comments;
                    // any user modification makes automatic removal fail closed.
                    Ok(Value::String(item.to_string()))
                })
                .transpose(),
        }
    }
    fn update(&mut self, entry: Option<&Value>) -> Result<(), String> {
        match self {
            Self::Cursor(v) => {
                if let Some(entry) = entry {
                    if v.get("mcpServers").is_none() {
                        v["mcpServers"] = json!({});
                    }
                    v["mcpServers"]["memorywhale"] = entry.clone();
                } else if let Some(servers) = v.get_mut("mcpServers").and_then(Value::as_object_mut)
                {
                    servers.remove("memorywhale");
                }
            }
            Self::Codex(d) => {
                if let Some(entry) = entry {
                    if d.get("mcp_servers").is_none() {
                        d["mcp_servers"] = toml_edit::Item::Table(toml_edit::Table::new());
                    }
                    let mut table = toml_edit::Table::new();
                    table["command"] = toml_edit::value(entry["command"].as_str().ok_or_else(err)?);
                    table["args"] = toml_edit::value(toml_edit::Array::new());
                    if let Some(env) = entry.get("env") {
                        let mut env_table = toml_edit::InlineTable::new();
                        env_table.insert(
                            "MEMORYWHALE_DATA_DIR",
                            toml_edit::Value::from(
                                env["MEMORYWHALE_DATA_DIR"].as_str().ok_or_else(err)?,
                            ),
                        );
                        table["env"] = toml_edit::value(env_table);
                    }
                    d["mcp_servers"]
                        .as_table_like_mut()
                        .ok_or_else(err)?
                        .insert("memorywhale", toml_edit::Item::Table(table));
                } else if let Some(t) = d.get_mut("mcp_servers").and_then(|i| i.as_table_like_mut())
                {
                    t.remove("memorywhale");
                }
            }
        }
        Ok(())
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        match self {
            Self::Codex(d) => Ok(d.to_string().into_bytes()),
            Self::Cursor(v) => serde_json::to_vec_pretty(v).map_err(|_| err()),
        }
    }

    fn command(&self) -> Option<&str> {
        match self {
            Self::Cursor(doc) => doc
                .get("mcpServers")?
                .get("memorywhale")?
                .get("command")?
                .as_str(),
            Self::Codex(doc) => doc
                .get("mcp_servers")?
                .get("memorywhale")?
                .get("command")?
                .as_str(),
        }
    }
}

pub(super) fn usable_executable(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !path.is_absolute() || !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return false;
        }
    }
    true
}

const REINSTALL: &str = "Owned MCP setup is stale or differs from the current executable/data-directory settings. Run mw integrate <client> --revert with the same --config selection, then reinstall with the intended MEMORYWHALE_DATA_DIR and reload the client; no files changed";

fn executable() -> Result<PathBuf, String> {
    let name = if cfg!(windows) {
        "mw-mcp.exe"
    } else {
        "mw-mcp"
    };
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join(name));
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(
            std::env::split_paths(&path)
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.join(name)),
        );
    }
    for candidate in candidates {
        if let Ok(path) = fs::canonicalize(candidate) {
            if usable_executable(&path) {
                return Ok(path);
            }
        }
    }
    Err("Cannot locate mw-mcp beside mw or on PATH; build/install the MCP binary first".into())
}

fn desired_entry(client: &str) -> Result<Value, String> {
    let exe = executable()?;
    let mut entry =
        json!({"command":exe.to_str().ok_or("MCP binary path is not UTF-8")?, "args":[]});
    if client == "cursor" {
        entry["type"] = json!("stdio");
    }
    if let Some(dir) = std::env::var_os("MEMORYWHALE_DATA_DIR") {
        // Anchor relative paths without changing parent/symlink filesystem semantics.
        let dir = PathBuf::from(dir);
        let dir = if dir.is_absolute() {
            dir
        } else {
            std::env::current_dir().map_err(|_| err())?.join(dir)
        };
        entry["env"] = json!({"MEMORYWHALE_DATA_DIR":dir.to_str().ok_or("Memory data directory is not UTF-8")?});
    }
    Ok(entry)
}
pub fn cli(args: &[String]) -> Result<(), String> {
    let client = args
        .first()
        .map(String::as_str)
        .filter(|s| matches!(*s, "codex" | "cursor"))
        .ok_or("Expected codex or cursor")?;
    let mut config = None;
    let mut mode = "install";
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config" if config.is_none() => {
                i += 1;
                config = Some(PathBuf::from(
                    args.get(i).ok_or("--config requires a file")?,
                ));
            }
            "--dry-run" | "--check" | "--revert" if mode == "install" => mode = args[i].as_str(),
            _ => return Err(
                "Usage: mw integrate codex|cursor [--config FILE] [--dry-run | --check | --revert]"
                    .into(),
            ),
        }
        i += 1;
    }
    let path = absolute(match config {
        Some(p) => p,
        None => {
            let home = || {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .or_else(dirs::home_dir)
                    .ok_or("Cannot determine home directory".to_string())
            };
            if client == "codex" {
                std::env::var_os("CODEX_HOME")
                    .map(PathBuf::from)
                    .map(Ok)
                    .unwrap_or_else(|| home().map(|p| p.join(".codex")))?
                    .join("config.toml")
            } else {
                home()?.join(".cursor/mcp.json")
            }
        }
    })?;
    let journal = sidecar(&path, ".memorywhale-mcp-owner.json");
    let lock = sidecar(&path, ".memorywhale-mcp.lock");
    safe(&lock)?;
    if fs::symlink_metadata(&lock).is_ok() {
        return Err(RECOVERY.into());
    }
    let original = read(&path)?;
    let owned_bytes = read(&journal)?;
    let mut doc = Config::parse(client, original.as_deref())?;
    let current = doc.entry()?;
    let owner: Option<Owner> = owned_bytes
        .as_ref()
        .map(|b| serde_json::from_slice::<Owner>(b).map_err(|_| RECOVERY.to_string()))
        .transpose()?;
    if let Some(o) = &owner {
        if o.version != 1 || o.client != client || o.state != "active" {
            return Err(RECOVERY.into());
        }
        if current.as_ref() != Some(&o.entry) {
            return Err(
                "Owned MemoryWhale entry was modified or removed; manual recovery required".into(),
            );
        }
    } else if current.is_some() {
        return Err(
            "An unowned MemoryWhale entry already exists; leaving configuration untouched".into(),
        );
    }
    if mode == "--check" {
        if owner.is_none() {
            return Err("MemoryWhale MCP configuration is not installed (configuration only; connectivity not tested)".into());
        }
        if !doc
            .command()
            .is_some_and(|command| usable_executable(Path::new(command)))
        {
            return Err(REINSTALL.into());
        }
        println!("MemoryWhale MCP configuration is installed (configuration only; connectivity not tested). No automatic capture or skills configured.");
        return Ok(());
    }
    if mode == "--revert" && owner.is_none() {
        println!("No owned MemoryWhale MCP configuration to remove.");
        return Ok(());
    }
    if mode == "--revert" {
        doc.update(None)?;
    } else {
        let entry = desired_entry(client).map_err(|error| {
            if owner.is_some() {
                REINSTALL.to_string()
            } else {
                error
            }
        })?;
        doc.update(Some(&entry))?;
    }
    let output = doc.bytes()?;
    if output.len() as u64 > LIMIT {
        return Err("MCP configuration exceeds the supported size limit".into());
    }
    // Snapshot the reparsed serialized form: TOML table decorations normalize on output.
    let entry = if mode == "--revert" {
        owner.as_ref().ok_or_else(err)?.entry.clone()
    } else {
        Config::parse(client, Some(&output))?
            .entry()?
            .ok_or_else(err)?
    };
    if mode != "--revert" && owner.is_some() {
        if current.as_ref() != Some(&entry) {
            return Err(REINSTALL.into());
        }
        println!("Owned MemoryWhale MCP configuration matches the current executable and data-directory settings; no changes. Connectivity not tested.");
        return Ok(());
    }
    if mode == "--dry-run" {
        println!("Would install MemoryWhale MCP configuration only; no files written, no connectivity test, no automatic capture or skills.");
        return Ok(());
    }
    let parent = path.parent().ok_or_else(err)?;
    safe(parent)?;
    fs::create_dir_all(parent).map_err(|_| err())?;
    with_lock(&lock, |journal_started| {
        guard_sources(&path, &original, &journal, &owned_bytes)?;
        let mut next = Owner {
            version: 1,
            client: client.into(),
            state: "pending".into(),
            entry,
        };
        let pending = serde_json::to_vec(&next).map_err(|_| err())?;
        atomic_tracked(&journal, &owned_bytes, &pending, journal_started)?;
        // After journaling any failure intentionally leaves pending ownership and lock.
        atomic(&path, &original, &output).map_err(|_| RECOVERY.to_string())?;
        if mode == "--revert" {
            fs::remove_file(&journal).map_err(|_| RECOVERY.to_string())?;
        } else {
            next.state = "active".into();
            atomic(
                &journal,
                &Some(pending),
                &serde_json::to_vec(&next).map_err(|_| err())?,
            )
            .map_err(|_| RECOVERY.to_string())?;
        }
        Ok(())
    })?;
    println!("MemoryWhale MCP configuration {}. Connectivity not tested; no automatic capture or skills configured.", if mode == "--revert" { "removed" } else { "installed" });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_changes_release_lock_but_postjournal_failures_retain_it() {
        let mut random = [0u8; 16];
        getrandom::getrandom(&mut random).unwrap();
        let root =
            std::env::temp_dir().join(format!("mw-lock-{:032x}", u128::from_ne_bytes(random)));
        fs::create_dir(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let path = root.join("config");
        let journal = root.join("owner");
        let lock = root.join("lock");
        for changed in [&path, &journal] {
            fs::write(&path, b"original").unwrap();
            fs::write(&journal, b"owner").unwrap();
            let original = read(&path).unwrap();
            let owned = read(&journal).unwrap();
            let result = with_lock(&lock, |_| {
                // Controlled competing edit after snapshots and lock acquisition.
                fs::write(changed, b"concurrent edit").unwrap();
                guard_sources(&path, &original, &journal, &owned)
            });
            assert!(result.unwrap_err().contains("changed concurrently"));
            assert!(!lock.exists());
            assert_eq!(fs::read(changed).unwrap(), b"concurrent edit");
            with_lock(&lock, |_| Ok(())).unwrap();
        }
        let result = with_lock(&lock, |started| {
            atomic_tracked(&journal, &None, b"pending", started)
        });
        assert!(result.is_err());
        assert!(!lock.exists(), "journal source guard is still pre-commit");
        let owned = read(&journal).unwrap();
        let result = with_lock(&lock, |started| {
            atomic_tracked(&journal, &owned, b"pending", started)?;
            Err("controlled failure after journal commit".into())
        });
        assert_eq!(result.unwrap_err(), RECOVERY);
        assert!(lock.exists());
        assert_eq!(fs::read(&journal).unwrap(), b"pending");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn toml_entry_snapshot_survives_other_servers_and_inline_parent() {
        for input in [
            "# comment\nsecret='private'\n[mcp_servers.other]\ncommand='other'\n",
            "# comment\nsecret='private'\nmcp_servers = {other = {command = 'other'}}\n",
        ] {
            let mut doc = Config::parse("codex", Some(input.as_bytes())).unwrap();
            doc.update(Some(&json!({"command":"/absolute/mw-mcp","args":[]})))
                .unwrap();
            let output = doc.bytes().unwrap();
            let reparsed = Config::parse("codex", Some(&output)).unwrap();
            let snapshot = reparsed.entry().unwrap().unwrap();
            assert!(!snapshot.to_string().contains("private"));
            assert!(!snapshot.to_string().contains("other"));
            let mut parsed = String::from_utf8(output)
                .unwrap()
                .parse::<toml_edit::DocumentMut>()
                .unwrap();
            let mut other = toml_edit::Table::new();
            other["command"] = toml_edit::value("later");
            parsed["mcp_servers"]
                .as_table_like_mut()
                .unwrap()
                .insert("later", toml_edit::Item::Table(other));
            let mut updated = Config::parse("codex", Some(parsed.to_string().as_bytes())).unwrap();
            assert_eq!(updated.entry().unwrap(), Some(snapshot));
            updated.update(None).unwrap();
            let remaining = String::from_utf8(updated.bytes().unwrap()).unwrap();
            assert!(remaining.contains("# comment"));
            assert!(remaining.contains("private"));
            assert!(remaining.contains("later"));
            assert!(!remaining.contains("mw-mcp"));
        }
    }
}
