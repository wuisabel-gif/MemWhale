//! Agent integrations installed by `mw integrate`.

mod files;
mod report;
mod skill_files;

pub mod claude;
pub mod cursor_capture;
pub mod hermes;
pub mod mcp_client;
pub mod portable;
pub mod rho;

pub(crate) const SKILL: &str = include_str!("../../integrate/SKILL.md");

/// Claude Code and Rho integration status for `mw doctor`.
pub fn render_doctor_reports(mcp_stdio_ok: bool) -> String {
    report::render_reports(&[
        claude::doctor_report(mcp_stdio_ok),
        rho::doctor_report(mcp_stdio_ok),
    ])
}
