//! Read-only Rho doctor inspection.

use std::path::Path;

use super::mcp::inspect_memorywhale;
use super::{hook_matches, is_memorywhale_hook, parse_toml, require_hooks_version, rho_home};
use crate::integrate::files::{
    command_on_path, mw_remember_executable, read_existing, skill_is_installed,
};
use crate::integrate::report::{IntegrationReport, McpFact, PieceStatus};

/// Inspect Rho MCP, hook, and skill status without mutating files or probing
/// HTTP endpoints.
pub(crate) fn doctor_report(mcp_stdio_ok: bool) -> IntegrationReport {
    let env_set = std::env::var_os("RHO_HOME").is_some_and(|value| !value.is_empty());
    let Ok(config_dir) = rho_home() else {
        return IntegrationReport::not_detected("Rho", "rho");
    };
    let detected = env_set || config_dir.exists() || command_on_path("rho");
    if !detected {
        return IntegrationReport::not_detected("Rho", "rho");
    }
    inspect_at(
        &config_dir,
        mw_remember_executable().ok().as_deref(),
        mcp_stdio_ok,
    )
}

fn inspect_at(
    config_dir: &Path,
    remember_path: Option<&Path>,
    mcp_stdio_ok: bool,
) -> IntegrationReport {
    IntegrationReport::detected(
        "Rho",
        "rho",
        inspect_mcp(config_dir).into_status(mcp_stdio_ok),
        inspect_hook(config_dir, remember_path),
        PieceStatus::skill(skill_is_installed(config_dir)),
    )
}

fn inspect_mcp(config_dir: &Path) -> McpFact {
    match read_existing(&config_dir.join("config.toml")) {
        Ok(Some(text)) => inspect_memorywhale(&text),
        Ok(None) => McpFact::Absent,
        Err(_) => McpFact::Unreadable,
    }
}

fn inspect_hook(config_dir: &Path, remember_path: Option<&Path>) -> PieceStatus {
    let existing = match read_existing(&config_dir.join("hooks.toml")) {
        Ok(None) => return PieceStatus::NotInstalled,
        Err(_) => return PieceStatus::Unreadable,
        Ok(Some(text)) if text.trim().is_empty() => return PieceStatus::NotInstalled,
        Ok(Some(text)) => text,
    };
    let doc = match parse_toml(&existing, "hooks.toml") {
        Ok(doc) => doc,
        Err(_) => return PieceStatus::Unreadable,
    };
    if require_hooks_version(&doc).is_err() {
        return PieceStatus::Unreadable;
    }
    let Some(hook) = doc.get("hook") else {
        return PieceStatus::NotInstalled;
    };
    let Some(hooks) = hook.as_array_of_tables() else {
        return PieceStatus::Unreadable;
    };
    let Some(table) = hooks.iter().find(|table| is_memorywhale_hook(table)) else {
        return PieceStatus::NotInstalled;
    };
    match remember_path {
        Some(path) if hook_matches(table, path) => PieceStatus::Installed,
        _ => PieceStatus::Stale,
    }
}

#[cfg(test)]
mod tests {
    use super::super::mcp::{merge_mcp, McpTarget};
    use super::super::merge_hooks;
    use super::*;
    use crate::integrate::report::McpStatus;
    use std::path::PathBuf;

    fn sandbox(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "mw-rho-doctor-{name}-{}-{}",
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
    fn inspect_fresh_rho_home_reports_missing_pieces() {
        let dir = sandbox("fresh");
        let report = inspect_at(&dir, Some(&remember()), true);
        assert!(report.detected);
        assert_eq!(report.mcp, McpStatus::NotConfigured);
        assert_eq!(report.hook, PieceStatus::NotInstalled);
        assert_eq!(report.skill, Some(PieceStatus::NotInstalled));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_partial_and_stale_rho_install() {
        let dir = sandbox("partial");
        std::fs::create_dir_all(dir.join("skills/memorywhale")).unwrap();
        std::fs::write(dir.join("skills/memorywhale/SKILL.md"), "skill").unwrap();
        let (hooks, _) = merge_hooks("", &remember()).unwrap();
        let stale_hooks = hooks.replace(
            "/home/me/.local/bin/mw-remember",
            "/old/home/.local/bin/mw-remember",
        );
        std::fs::write(dir.join("hooks.toml"), stale_hooks).unwrap();
        std::fs::write(
            dir.join("config.toml"),
            r#"[mcp.servers.memorywhale]
transport = "stdio"
command = "/missing/mw-mcp"
"#,
        )
        .unwrap();
        let report = inspect_at(&dir, Some(&remember()), true);
        assert_eq!(report.skill, Some(PieceStatus::Installed));
        assert_eq!(report.hook, PieceStatus::Stale);
        assert_eq!(report.mcp, McpStatus::Stale);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_full_stdio_and_http_without_leaking_secrets() {
        let dir = sandbox("full");
        let remember = remember();
        let (hooks, _) = merge_hooks("", &remember).unwrap();
        std::fs::write(dir.join("hooks.toml"), hooks).unwrap();
        std::fs::create_dir_all(dir.join("skills/memorywhale")).unwrap();
        std::fs::write(dir.join("skills/memorywhale/SKILL.md"), "skill").unwrap();
        let (stdio, _) = merge_mcp("", &McpTarget::stdio()).unwrap();
        std::fs::write(dir.join("config.toml"), &stdio).unwrap();

        let reachable = inspect_at(&dir, Some(&remember), true);
        assert_eq!(reachable.mcp, McpStatus::Configured { reachable: true });
        assert_eq!(reachable.hook, PieceStatus::Installed);
        assert_eq!(reachable.skill, Some(PieceStatus::Installed));

        std::fs::write(
            dir.join("config.toml"),
            r#"[mcp.servers.memorywhale]
transport = "streamable_http"
url = "http://127.0.0.1:7071/mcp"
headers = { Authorization = "Bearer supersecret-token" }
"#,
        )
        .unwrap();
        let http = inspect_at(&dir, Some(&remember), true);
        assert_eq!(http.mcp, McpStatus::Configured { reachable: false });
        let rendered = http.render();
        assert!(!rendered.contains("supersecret-token"));
        assert!(!rendered.contains("127.0.0.1"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_missing_helper_does_not_report_hook_installed() {
        let dir = sandbox("no-helper");
        let (hooks, _) = merge_hooks("", &remember()).unwrap();
        let stale = hooks.replace("/home/me/.local/bin/mw-remember", "/old/bin/mw-remember");
        std::fs::write(dir.join("hooks.toml"), stale).unwrap();
        let report = inspect_at(&dir, None, false);
        assert_eq!(report.hook, PieceStatus::Stale);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_unreadable_skill_is_not_absence() {
        let dir = sandbox("bad-skill");
        std::fs::create_dir_all(dir.join("skills/memorywhale")).unwrap();
        std::fs::write(dir.join("skills/memorywhale/SKILL.md"), [0xff, 0xfe]).unwrap();
        let report = inspect_at(&dir, Some(&remember()), false);
        assert_eq!(report.skill, Some(PieceStatus::Unreadable));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_unreadable_hooks_leave_mcp_readable() {
        let dir = sandbox("unreadable");
        std::fs::write(dir.join("hooks.toml"), "version = [").unwrap();
        let (stdio, _) = merge_mcp("", &McpTarget::stdio()).unwrap();
        std::fs::write(dir.join("config.toml"), stdio).unwrap();
        let report = inspect_at(&dir, Some(&remember()), false);
        assert_eq!(report.hook, PieceStatus::Unreadable);
        assert_eq!(report.mcp, McpStatus::Configured { reachable: false });
        let _ = std::fs::remove_dir_all(&dir);
    }
}
