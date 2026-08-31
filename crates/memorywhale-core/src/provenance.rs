//! Canonical producing-agent vocabulary shared by storage, retrieval, and
//! interface renderers.

pub const AGENT_CLAUDE: &str = "claude";
pub const AGENT_RHO: &str = "rho";
pub const AGENT_TERMINAL: &str = "terminal";
pub const SUPPORTED_AGENTS: [&str; 3] = [AGENT_CLAUDE, AGENT_RHO, AGENT_TERMINAL];

/// Whether a stored optional value is one of the canonical agent identifiers.
/// NULL is valid and means terminal/manual or legacy provenance.
pub fn is_valid(agent: Option<&str>) -> bool {
    matches!(agent, None | Some(AGENT_CLAUDE) | Some(AGENT_RHO))
}

/// Render structured storage metadata without inspecting notes or payloads.
/// NULL is the deliberate terminal/manual representation.
pub fn label(agent: Option<&str>) -> &'static str {
    match agent {
        None => AGENT_TERMINAL,
        Some(AGENT_CLAUDE) => AGENT_CLAUDE,
        Some(AGENT_RHO) => AGENT_RHO,
        Some(_) => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_only_structured_values_and_null() {
        assert_eq!(label(Some(AGENT_CLAUDE)), AGENT_CLAUDE);
        assert_eq!(label(Some(AGENT_RHO)), AGENT_RHO);
        assert_eq!(label(None), AGENT_TERMINAL);
        assert_eq!(label(Some("agent:claude")), "unknown");
    }
}
