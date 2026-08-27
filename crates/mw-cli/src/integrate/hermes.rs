//! Hermes Agent configuration integration.

use std::env;
use std::fs;
use std::path::PathBuf;

use super::files::atomic_write;

const SERVER: &str = "  memorywhale:\n    command: \"mw-mcp\"\n";

/// Register MemoryWhale's MCP server in the current Hermes home.
///
/// Existing settings and comments are preserved. Both the original and
/// resulting documents are validated before any write occurs.
pub fn install() -> Result<PathBuf, String> {
    let hermes_home = if let Some(path) = env::var_os("HERMES_HOME") {
        PathBuf::from(path)
    } else {
        dirs::home_dir()
            .ok_or_else(|| "could not resolve the home directory".to_string())?
            .join(".hermes")
    };
    fs::create_dir_all(&hermes_home)
        .map_err(|err| format!("failed to create {}: {err}", hermes_home.display()))?;
    let config_path = hermes_home.join("config.yaml");
    let existing = if config_path.exists() {
        fs::read_to_string(&config_path)
            .map_err(|err| format!("failed to read {}: {err}", config_path.display()))?
    } else {
        String::new()
    };
    validate(&existing)?;
    let updated = add_server(&existing);
    validate(&updated)?;
    atomic_write(&config_path, &updated)?;
    Ok(config_path)
}

fn validate(config: &str) -> Result<(), String> {
    if config.trim().is_empty() {
        return Ok(());
    }
    let value: serde_yaml::Value = serde_yaml::from_str(config)
        .map_err(|err| format!("invalid Hermes config; file was not changed: {err}"))?;
    let root = value
        .as_mapping()
        .ok_or_else(|| "invalid Hermes config; expected a top-level YAML mapping".to_string())?;
    if let Some(servers) = root.get(serde_yaml::Value::String("mcp_servers".to_string())) {
        if !servers.is_mapping() {
            return Err(
                "invalid Hermes config; mcp_servers must be a YAML mapping and file was not changed"
                    .to_string(),
            );
        }
    }
    Ok(())
}

fn add_server(config: &str) -> String {
    if config.is_empty() {
        return format!("mcp_servers:\n{SERVER}");
    }

    let lines: Vec<&str> = config.split_inclusive('\n').collect();
    if let Some(start) = lines
        .iter()
        .position(|line| line.trim_end_matches(['\r', '\n']) == "mcp_servers:")
    {
        let end = lines
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(_, line)| {
                let content = line.trim_end_matches(['\r', '\n']);
                !content.is_empty()
                    && !content.starts_with(char::is_whitespace)
                    && !content.starts_with('#')
            })
            .map(|(index, _)| index)
            .unwrap_or(lines.len());
        if lines[start + 1..end]
            .iter()
            .any(|line| line.trim_end_matches(['\r', '\n']) == "  memorywhale:")
        {
            return config.to_string();
        }
        let mut output = lines[..end].concat();
        if !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(SERVER);
        output.push_str(&lines[end..].concat());
        return output;
    }

    let mut output = config.to_string();
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output.push_str("mcp_servers:\n");
    output.push_str(SERVER);
    output
}
