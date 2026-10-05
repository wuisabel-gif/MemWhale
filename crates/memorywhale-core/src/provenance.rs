//! Canonical producing-agent vocabulary shared by storage, retrieval, and
//! interface renderers.

pub const AGENT_CLAUDE: &str = "claude";
pub const AGENT_RHO: &str = "rho";
pub const AGENT_CURSOR: &str = "cursor";
pub const AGENT_CODEWHALE: &str = "codewhale";
pub const AGENT_CODEX: &str = "codex";
pub const AGENT_TERMINAL: &str = "terminal";
/// Every producing-agent identifier `label` can return, `terminal` last. A
/// slice, so adding an agent does not change this constant's type.
pub const SUPPORTED_AGENTS: &[&str] = &[
    AGENT_CLAUDE,
    AGENT_RHO,
    AGENT_CURSOR,
    AGENT_CODEWHALE,
    AGENT_CODEX,
    AGENT_TERMINAL,
];

/// Whether a stored optional value is one of the canonical agent identifiers.
/// NULL is valid and means terminal/manual or legacy provenance; `terminal`
/// itself is never stored.
pub fn is_valid(agent: Option<&str>) -> bool {
    agent.is_none_or(|agent| agent != AGENT_TERMINAL && SUPPORTED_AGENTS.contains(&agent))
}

/// Render structured storage metadata without inspecting notes or payloads.
/// NULL is the deliberate terminal/manual representation.
pub fn label(agent: Option<&str>) -> &'static str {
    match agent {
        None => AGENT_TERMINAL,
        Some(agent) if is_valid(Some(agent)) => SUPPORTED_AGENTS
            .iter()
            .find(|known| **known == agent)
            .copied()
            .unwrap_or("unknown"),
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
        assert_eq!(label(Some(AGENT_CURSOR)), AGENT_CURSOR);
        assert!(is_valid(Some(AGENT_CURSOR)));
        assert_eq!(label(Some(AGENT_CODEWHALE)), AGENT_CODEWHALE);
        assert!(is_valid(Some(AGENT_CODEWHALE)));
        assert!(SUPPORTED_AGENTS.contains(&AGENT_CODEWHALE));
        assert_eq!(label(Some(AGENT_CODEX)), AGENT_CODEX);
        assert!(is_valid(Some(AGENT_CODEX)));
        assert!(!is_valid(Some("Codewhale")));
        assert!(!is_valid(Some(AGENT_TERMINAL)));
        assert_eq!(label(None), AGENT_TERMINAL);
        assert_eq!(label(Some("agent:claude")), "unknown");
    }
}
