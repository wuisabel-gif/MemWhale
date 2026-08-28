//! Rho `config.toml` MCP server stanza.

use toml_edit::{value, DocumentMut, InlineTable, Item, Table, TableLike};

use super::{parse_toml, set_string};

const MCP_COMMAND: &str = "mw-mcp";
const MCP_TRANSPORT: &str = "stdio";
/// Rho 2.2+ transport key. The server is one JSON-RPC object per POST, not SSE.
const MCP_HTTP_TRANSPORT: &str = "streamable_http";

pub(super) struct McpTarget {
    pub transport: &'static str,
    pub command: Option<&'static str>,
    pub url: Option<String>,
    pub allow_insecure_http: bool,
    pub authorization_from_env: bool,
}

impl McpTarget {
    pub(super) fn stdio() -> Self {
        Self {
            transport: MCP_TRANSPORT,
            command: Some(MCP_COMMAND),
            url: None,
            allow_insecure_http: false,
            authorization_from_env: false,
        }
    }

    pub(super) fn http(
        url: String,
        allow_insecure_http: bool,
        authorization_from_env: bool,
    ) -> Self {
        Self {
            transport: MCP_HTTP_TRANSPORT,
            command: None,
            url: Some(url),
            allow_insecure_http,
            authorization_from_env,
        }
    }
}

fn mcp_server_matches(doc: &DocumentMut, target: &McpTarget) -> bool {
    memorywhale_server(doc).is_some_and(|server| {
        if server.get("enabled").and_then(Item::as_bool) == Some(false) {
            return false;
        }
        if server.get("transport").and_then(Item::as_str) != Some(target.transport) {
            return false;
        }
        if server.get("command").and_then(Item::as_str) != target.command {
            return false;
        }
        if server.get("url").and_then(Item::as_str) != target.url.as_deref() {
            return false;
        }
        let remove = if target.command.is_some() {
            STDIO_REMOVE.as_slice()
        } else {
            HTTP_REMOVE.as_slice()
        };
        if remove.iter().any(|key| server.get(key).is_some()) {
            return false;
        }
        if target.allow_insecure_http {
            if server.get("allow_insecure_http").and_then(Item::as_bool) != Some(true) {
                return false;
            }
        } else if server.get("allow_insecure_http").is_some() {
            return false;
        }
        http_auth_matches(server, target.authorization_from_env)
    })
}

fn authorization_env_name(server: &dyn TableLike) -> Result<Option<&str>, ()> {
    let Some(headers) = server.get("headers_from_env") else {
        return Ok(None);
    };
    if let Some(inline) = headers.as_inline_table() {
        return match inline.get("Authorization") {
            None => Ok(None),
            Some(value) => Ok(Some(value.as_str().ok_or(())?)),
        };
    }
    let Some(table) = headers.as_table_like() else {
        return Ok(None);
    };
    match table.get("Authorization") {
        None => Ok(None),
        Some(item) => Ok(Some(item.as_str().ok_or(())?)),
    }
}

fn http_auth_matches(server: &dyn TableLike, with_auth: bool) -> bool {
    let Ok(authorization) = authorization_env_name(server) else {
        return false;
    };
    if with_auth {
        authorization == Some("MEMORYWHALE_AUTHORIZATION")
    } else {
        authorization.is_none()
    }
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

const STDIO_REMOVE: [&str; 5] = [
    "url",
    "headers",
    "headers_from_env",
    "allow_insecure_http",
    "oauth",
];
const HTTP_REMOVE: [&str; 5] = ["command", "args", "cwd", "env", "env_from_env"];

pub(super) fn merge_mcp(existing: &str, target: &McpTarget) -> Result<(String, bool), String> {
    let mut doc = parse_toml(existing, "config.toml")?;
    require_mcp_tables(&doc)?;
    if let Some(server) = memorywhale_server(&doc) {
        if server.get("headers_from_env").is_some() {
            headers_from_env_is_table(server)?;
        }
    }
    if mcp_server_matches(&doc, target) {
        return Ok((existing.to_string(), false));
    }
    let server = memorywhale_server_table(&mut doc)?;
    let mut changed = false;
    changed |= set_string(server, "transport", target.transport);
    if let Some(command) = target.command {
        changed |= set_string(server, "command", command);
        for key in STDIO_REMOVE {
            changed |= remove_key(server, key);
        }
    } else {
        if let Some(url) = target.url.as_deref() {
            changed |= set_string(server, "url", url);
        }
        for key in HTTP_REMOVE {
            changed |= remove_key(server, key);
        }
        if target.allow_insecure_http {
            if server.get("allow_insecure_http").and_then(Item::as_bool) != Some(true) {
                server.insert("allow_insecure_http", value(true));
                changed = true;
            }
        } else {
            changed |= remove_key(server, "allow_insecure_http");
        }
        if target.authorization_from_env {
            changed |= set_authorization_from_env(server)?;
        } else {
            changed |= remove_authorization_from_env(server);
        }
    }
    if server.get("enabled").and_then(Item::as_bool) == Some(false) {
        server.insert("enabled", value(true));
        changed = true;
    }
    if !changed {
        return Ok((existing.to_string(), false));
    }
    Ok((doc.to_string(), true))
}

fn remove_key(table: &mut dyn TableLike, key: &str) -> bool {
    table.remove(key).is_some()
}

fn headers_from_env_is_table(table: &dyn TableLike) -> Result<bool, String> {
    match table.get("headers_from_env") {
        None => Ok(false),
        Some(item) if item.as_inline_table().is_some() || item.as_table_like().is_some() => {
            Ok(true)
        }
        Some(_) => Err(
            "invalid Rho config.toml; headers_from_env must be a table and the file was not changed"
                .to_string(),
        ),
    }
}

fn set_authorization_from_env(table: &mut dyn TableLike) -> Result<bool, String> {
    if http_auth_matches(table, true) {
        return Ok(false);
    }
    if headers_from_env_is_table(table)? {
        let item = table
            .get_mut("headers_from_env")
            .expect("headers_from_env exists");
        if let Some(inline) = item.as_inline_table_mut() {
            inline.insert("Authorization", "MEMORYWHALE_AUTHORIZATION".into());
            return Ok(true);
        }
        if let Some(existing) = item.as_table_like_mut() {
            existing.insert("Authorization", value("MEMORYWHALE_AUTHORIZATION"));
            return Ok(true);
        }
        return Err(
            "invalid Rho config.toml; headers_from_env must be a table and the file was not changed"
                .to_string(),
        );
    }
    let mut headers = InlineTable::new();
    headers.insert("Authorization", "MEMORYWHALE_AUTHORIZATION".into());
    table.insert("headers_from_env", Item::Value(headers.into()));
    Ok(true)
}

fn remove_authorization_from_env(table: &mut dyn TableLike) -> bool {
    let drop_table = {
        let Some(item) = table.get_mut("headers_from_env") else {
            return false;
        };
        if let Some(inline) = item.as_inline_table_mut() {
            if inline.remove("Authorization").is_none() {
                return false;
            }
            inline.is_empty()
        } else if let Some(existing) = item.as_table_like_mut() {
            if existing.remove("Authorization").is_none() {
                return false;
            }
            existing.is_empty()
        } else {
            return false;
        }
    };
    if drop_table {
        table.remove("headers_from_env");
    }
    true
}

pub(super) fn unmerge_mcp(existing: &str) -> Result<(String, bool), String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn merge_mcp(existing: &str) -> Result<(String, bool), String> {
        super::merge_mcp(existing, &McpTarget::stdio())
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

    #[test]
    fn merge_mcp_http_loopback_has_no_token_header() {
        let mode = McpTarget::http("http://127.0.0.1:7071/mcp".into(), false, false);
        let (merged, changed) = super::merge_mcp("", &mode).unwrap();
        assert!(changed);
        let doc: DocumentMut = merged.parse().unwrap();
        let server = doc["mcp"]["servers"]["memorywhale"].as_table().unwrap();
        assert_eq!(server["transport"].as_str(), Some("streamable_http"));
        assert_eq!(server["url"].as_str(), Some("http://127.0.0.1:7071/mcp"));
        assert!(server.get("command").is_none());
        assert!(server.get("allow_insecure_http").is_none());
        assert!(server.get("headers_from_env").is_none());
        let (again, changed_again) = super::merge_mcp(&merged, &mode).unwrap();
        assert!(!changed_again);
        assert_eq!(again, merged);
    }

    #[test]
    fn merge_mcp_http_preserves_unrelated_headers_from_env() {
        let original = r#"[mcp.servers.memorywhale]
transport = "streamable_http"
url = "http://192.168.1.42:7071/mcp"
headers_from_env = { X-Tenant = "acme" }
"#;
        let lan = McpTarget::http("http://192.168.1.42:7071/mcp".into(), true, true);
        let (merged, changed) = super::merge_mcp(original, &lan).unwrap();
        assert!(changed);
        let headers = &merged.parse::<DocumentMut>().unwrap()["mcp"]["servers"]["memorywhale"]
            ["headers_from_env"];
        assert_eq!(headers["X-Tenant"].as_str(), Some("acme"));
        assert_eq!(
            headers["Authorization"].as_str(),
            Some("MEMORYWHALE_AUTHORIZATION")
        );

        let loopback = McpTarget::http("http://127.0.0.1:7071/mcp".into(), false, false);
        let (cleared, changed) = super::merge_mcp(&merged, &loopback).unwrap();
        assert!(changed);
        let headers = &cleared.parse::<DocumentMut>().unwrap()["mcp"]["servers"]["memorywhale"]
            ["headers_from_env"];
        assert_eq!(headers["X-Tenant"].as_str(), Some("acme"));
        assert!(headers.get("Authorization").is_none());
    }

    #[test]
    fn merge_mcp_rejects_non_table_headers_from_env() {
        let original = r#"[mcp.servers.memorywhale]
transport = "streamable_http"
url = "http://192.168.1.42:7071/mcp"
headers_from_env = "nope"
"#;
        let lan = McpTarget::http("http://192.168.1.42:7071/mcp".into(), true, true);
        let err = super::merge_mcp(original, &lan).unwrap_err();
        assert!(err.contains("headers_from_env"));
        assert!(err.contains("was not changed"));

        let loopback = McpTarget::http("http://127.0.0.1:7071/mcp".into(), false, false);
        let err = super::merge_mcp(original, &loopback).unwrap_err();
        assert!(err.contains("headers_from_env"));
    }

    #[test]
    fn merge_mcp_http_rewrites_loopback_when_insecure_is_false() {
        let original = r#"[mcp.servers.memorywhale]
transport = "streamable_http"
url = "http://127.0.0.1:7071/mcp"
allow_insecure_http = false
"#;
        let loopback = McpTarget::http("http://127.0.0.1:7071/mcp".into(), false, false);
        let (merged, changed) = super::merge_mcp(original, &loopback).unwrap();
        assert!(changed);
        let server = merged.parse::<DocumentMut>().unwrap()["mcp"]["servers"]["memorywhale"]
            .as_table()
            .unwrap()
            .clone();
        assert!(server.get("allow_insecure_http").is_none());
    }

    #[test]
    fn merge_mcp_http_rewrites_leftover_stdio_env() {
        let original = r#"[mcp.servers.memorywhale]
transport = "streamable_http"
url = "http://127.0.0.1:7071/mcp"
env = { MEMORYWHALE_DATA_DIR = "/custom" }
"#;
        let loopback = McpTarget::http("http://127.0.0.1:7071/mcp".into(), false, false);
        let (merged, changed) = super::merge_mcp(original, &loopback).unwrap();
        assert!(changed);
        let server = merged.parse::<DocumentMut>().unwrap()["mcp"]["servers"]["memorywhale"]
            .as_table()
            .unwrap()
            .clone();
        assert!(server.get("env").is_none());
        assert_eq!(
            server.get("url").and_then(Item::as_str),
            Some("http://127.0.0.1:7071/mcp")
        );
    }

    #[test]
    fn merge_mcp_http_rejects_non_string_authorization() {
        let original = r#"[mcp.servers.memorywhale]
transport = "streamable_http"
url = "http://192.168.1.42:7071/mcp"
allow_insecure_http = true
headers_from_env = { Authorization = 1 }
"#;
        let lan = McpTarget::http("http://192.168.1.42:7071/mcp".into(), true, true);
        let (merged, changed) = super::merge_mcp(original, &lan).unwrap();
        assert!(changed);
        assert_eq!(
            merged.parse::<DocumentMut>().unwrap()["mcp"]["servers"]["memorywhale"]
                ["headers_from_env"]["Authorization"]
                .as_str(),
            Some("MEMORYWHALE_AUTHORIZATION")
        );
    }

    #[test]
    fn merge_mcp_http_lan_sets_insecure_and_auth_header() {
        let mode = McpTarget::http("http://192.168.1.42:7071/mcp".into(), true, true);
        let (merged, changed) = super::merge_mcp(
            "[mcp.servers.memorywhale]\ntransport = \"stdio\"\ncommand = \"mw-mcp\"\n",
            &mode,
        )
        .unwrap();
        assert!(changed);
        assert!(!merged.contains("command = \"mw-mcp\""));
        let doc: DocumentMut = merged.parse().unwrap();
        let server = doc["mcp"]["servers"]["memorywhale"].as_table().unwrap();
        assert_eq!(server["allow_insecure_http"].as_bool(), Some(true));
        assert_eq!(
            server["headers_from_env"]["Authorization"].as_str(),
            Some("MEMORYWHALE_AUTHORIZATION")
        );
    }
}
