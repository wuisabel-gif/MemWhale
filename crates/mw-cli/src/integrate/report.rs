//! Shared doctor status for thin client integrations.

fn next_step(detail: &str, client: &str) -> String {
    format!("{detail}; run `mw integrate {client}`")
}

/// MCP registration as doctor should print it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum McpStatus {
    Configured { reachable: bool },
    NotConfigured,
    Stale,
    Unreadable,
}

impl McpStatus {
    fn phrase(self, client: &str) -> String {
        match self {
            Self::Configured { reachable: true } => "configured and reachable".to_string(),
            Self::Configured { reachable: false } => "configured".to_string(),
            Self::NotConfigured => next_step("not configured", client),
            Self::Stale => next_step("stale", client),
            Self::Unreadable => next_step("unreadable", client),
        }
    }
}

/// Hook or skill as doctor should print it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PieceStatus {
    Installed,
    NotInstalled,
    Stale,
    Unreadable,
}

impl PieceStatus {
    pub(crate) fn skill(result: Result<bool, std::io::Error>) -> Self {
        match result {
            Ok(true) => Self::Installed,
            Ok(false) => Self::NotInstalled,
            Err(_) => Self::Unreadable,
        }
    }

    fn phrase(self, client: &str) -> String {
        match self {
            Self::Installed => "installed".to_string(),
            Self::NotInstalled => next_step("not installed", client),
            Self::Stale => next_step("stale", client),
            Self::Unreadable => next_step("unreadable", client),
        }
    }
}

/// What the client config says about MemoryWhale MCP, before doctor maps in the
/// generic stdio probe. Never carries URLs, headers, or tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum McpFact {
    Absent,
    Stdio,
    Http,
    Stale,
    Unreadable,
}

impl McpFact {
    pub(crate) fn into_status(self, stdio_ok: bool) -> McpStatus {
        match self {
            Self::Absent => McpStatus::NotConfigured,
            Self::Stdio if stdio_ok => McpStatus::Configured { reachable: true },
            Self::Stdio | Self::Http => McpStatus::Configured { reachable: false },
            Self::Stale => McpStatus::Stale,
            Self::Unreadable => McpStatus::Unreadable,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntegrationReport {
    pub title: &'static str,
    pub client: &'static str,
    pub detected: bool,
    pub mcp: McpStatus,
    pub hook: PieceStatus,
    /// `None` for clients whose skill install doctor does not track.
    pub skill: Option<PieceStatus>,
    /// What to pass to `mw integrate` to fix the hook (e.g. `codex --capture`).
    pub hook_fix: &'static str,
}

impl IntegrationReport {
    pub fn not_detected(title: &'static str, client: &'static str) -> Self {
        Self {
            title,
            client,
            detected: false,
            mcp: McpStatus::NotConfigured,
            hook: PieceStatus::NotInstalled,
            skill: Some(PieceStatus::NotInstalled),
            hook_fix: client,
        }
    }

    pub fn detected(
        title: &'static str,
        client: &'static str,
        mcp: McpStatus,
        hook: PieceStatus,
        skill: PieceStatus,
    ) -> Self {
        Self {
            title,
            client,
            detected: true,
            mcp,
            hook,
            skill: Some(skill),
            hook_fix: client,
        }
    }

    /// A client with opt-in capture (`mw integrate <client> --capture`) and no
    /// tracked skill, such as Codex or Cursor.
    pub fn capture_client(
        title: &'static str,
        client: &'static str,
        hook_fix: &'static str,
        mcp: McpStatus,
        hook: PieceStatus,
    ) -> Self {
        Self {
            title,
            client,
            detected: true,
            mcp,
            hook,
            skill: None,
            hook_fix,
        }
    }

    pub fn render(&self) -> String {
        let mut out = format!("  {}\n", self.title);
        if !self.detected {
            out.push_str("    not detected\n");
            return out;
        }
        out.push_str(&line("MCP", self.mcp.phrase(self.client)));
        out.push_str(&line("auto-capture hook", self.hook.phrase(self.hook_fix)));
        if let Some(skill) = self.skill {
            out.push_str(&line("skill", skill.phrase(self.client)));
        }
        out
    }
}

fn line(label: &str, phrase: String) -> String {
    format!("    {label:<19} {phrase}\n")
}

pub(crate) fn render_reports(reports: &[IntegrationReport]) -> String {
    let mut out = String::from("\nIntegrations\n");
    for report in reports {
        out.push_str(&report.render());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_separates_not_detected_from_missing_pieces() {
        let undetected = IntegrationReport::not_detected("Claude Code", "claude");
        assert_eq!(undetected.render(), "  Claude Code\n    not detected\n");

        let missing = IntegrationReport::detected(
            "Rho",
            "rho",
            McpStatus::NotConfigured,
            PieceStatus::NotInstalled,
            PieceStatus::Installed,
        );
        let rendered = missing.render();
        assert!(rendered.contains("MCP                 not configured; run `mw integrate rho`"));
        assert!(rendered.contains("auto-capture hook   not installed; run `mw integrate rho`"));
        assert!(rendered.contains("skill               installed"));
        assert!(!rendered.contains("not detected"));
    }

    #[test]
    fn render_reports_lists_each_client() {
        let text = render_reports(&[
            IntegrationReport::detected(
                "Claude Code",
                "claude",
                McpStatus::Configured { reachable: true },
                PieceStatus::Installed,
                PieceStatus::Installed,
            ),
            IntegrationReport::detected(
                "Rho",
                "rho",
                McpStatus::Configured { reachable: false },
                PieceStatus::Stale,
                PieceStatus::Installed,
            ),
        ]);
        assert!(text.starts_with("\nIntegrations\n"));
        assert!(text.contains("configured and reachable"));
        assert!(text.contains("stale; run `mw integrate rho`"));
    }
}
