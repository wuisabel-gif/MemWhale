//! Read-only `mw doctor` status for clients with opt-in hook capture (Codex,
//! Cursor): their MCP entry and capture hook, checked against the paths
//! `mw integrate` would write today.

use std::path::Path;

use super::files::{command_on_path, stable_executable};
use super::hook_capture;
use super::mcp_client::{installed_command, usable_executable};
use super::report::{IntegrationReport, McpStatus, PieceStatus};

pub(crate) fn doctor_report(
    title: &'static str,
    client: &'static str,
    hook_fix: &'static str,
    config_dir: &str,
    mcp_stdio_ok: bool,
) -> IntegrationReport {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let detected =
        command_on_path(client) || home.is_some_and(|home| home.join(config_dir).exists());
    if !detected {
        return IntegrationReport::not_detected(title, client);
    }
    IntegrationReport::capture_client(
        title,
        client,
        hook_fix,
        mcp_status(installed_command(client), mcp_stdio_ok),
        hook_status(hook_capture::installed_commands(client)),
    )
}

fn mcp_status(command: Result<Option<String>, String>, stdio_ok: bool) -> McpStatus {
    match command {
        Err(_) => McpStatus::Unreadable,
        Ok(None) => McpStatus::NotConfigured,
        Ok(Some(command)) if current(&command, "mw-mcp") => McpStatus::Configured {
            reachable: stdio_ok,
        },
        Ok(Some(_)) => McpStatus::Stale,
    }
}

fn hook_status(commands: Result<Vec<String>, String>) -> PieceStatus {
    match commands {
        Err(_) => PieceStatus::Unreadable,
        Ok(commands) if commands.is_empty() => PieceStatus::NotInstalled,
        Ok(commands)
            if commands.iter().all(|command| {
                hook_capture::hook_executable(command)
                    .is_some_and(|exe| current(exe, "mw-remember"))
            }) =>
        {
            PieceStatus::Installed
        }
        Ok(_) => PieceStatus::Stale,
    }
}

/// The configured executable still exists and is the path `mw integrate`
/// would write now (so not a versioned path an upgrade removed).
fn current(configured: &str, helper: &str) -> bool {
    usable_executable(Path::new(configured))
        && stable_executable(helper).is_ok_and(|path| path == Path::new(configured))
}
