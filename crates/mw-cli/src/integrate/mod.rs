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

/// Claude Code, Rho, Codex, and Cursor integration status for `mw doctor`.
pub fn render_doctor_reports(mcp_stdio_ok: bool) -> String {
    report::render_reports(&[
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
    ])
}
