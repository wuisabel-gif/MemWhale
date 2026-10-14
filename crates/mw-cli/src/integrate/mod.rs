//! Agent integrations installed by `mw integrate`.

mod capture_doctor;
mod files;
mod report;
mod skill_files;

pub mod claude;
pub mod hermes;
pub mod hook_capture;
pub mod mcp_client;
pub mod portable;
pub mod rho;

pub(crate) const SKILL: &str = include_str!("../../integrate/SKILL.md");

/// Claude Code, Rho, Codex, Cursor, and Gemini CLI integration status for
/// `mw doctor`, with what the database shows each client actually captured
/// when `conn` is available.
pub fn render_doctor_reports(mcp_stdio_ok: bool, conn: Option<&rusqlite::Connection>) -> String {
    let mut reports = vec![
        claude::doctor_report(mcp_stdio_ok),
        rho::doctor_report(mcp_stdio_ok),
        capture_doctor::doctor_report("Codex", "codex", "codex --capture", ".codex", mcp_stdio_ok),
        capture_doctor::doctor_report(
            "Cursor",
            "cursor",
            "cursor --capture",
            ".cursor",
            mcp_stdio_ok,
        ),
        capture_doctor::doctor_report(
            "Gemini CLI",
            "gemini",
            "gemini --capture",
            ".gemini",
            mcp_stdio_ok,
        ),
    ];
    if let Some(conn) = conn {
        for report in reports.iter_mut().filter(|r| r.detected) {
            report.observed = observed(conn, report.client);
        }
    }
    report::render_reports(&reports)
}

/// The newest captured command from `agent` (the client name is its agent
/// identifier) and how many of its recent rows carried an exit code.
fn observed(conn: &rusqlite::Connection, agent: &str) -> Option<report::Observed> {
    let last: Option<String> = conn
        .query_row(
            "SELECT created_at FROM command_runs WHERE agent = ?1 ORDER BY id DESC LIMIT 1",
            [agent],
            |r| r.get(0),
        )
        .ok();
    let Some(last) = last else {
        return Some(report::Observed::Never);
    };
    let (recent, with_exit): (u32, u32) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(exit_code IS NOT NULL), 0) FROM
               (SELECT exit_code FROM command_runs WHERE agent = ?1 ORDER BY id DESC LIMIT 20)",
            [agent],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok()?;
    Some(report::Observed::Seen {
        age: crate::relative_age(&last),
        recent,
        with_exit,
    })
}

#[cfg(test)]
mod observed_tests {
    use super::*;

    #[test]
    fn observed_separates_configured_from_seen() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::storage::initialize(&conn).unwrap();
        assert_eq!(observed(&conn, "codex"), Some(report::Observed::Never));

        let now = chrono::Utc::now().to_rfc3339();
        for exit in [Some(1), None, Some(0)] {
            conn.execute(
                "INSERT INTO command_runs (command, argv_json, exit_code, created_at, agent)
                 VALUES ('x', '[\"x\"]', ?1, ?2, 'gemini')",
                rusqlite::params![exit, now],
            )
            .unwrap();
        }
        let Some(report::Observed::Seen {
            age,
            recent,
            with_exit,
        }) = observed(&conn, "gemini")
        else {
            panic!("gemini rows not seen");
        };
        assert_eq!((age.as_str(), recent, with_exit), ("just now", 3, 2));

        let mut never = report::IntegrationReport::capture_client(
            "Codex",
            "codex",
            "codex --capture",
            report::McpStatus::NotConfigured,
            report::PieceStatus::Installed,
        );
        never.observed = Some(report::Observed::Never);
        assert!(never
            .render()
            .contains("trust the hook with /hooks in Codex"));
    }
}
