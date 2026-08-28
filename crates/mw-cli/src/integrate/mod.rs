//! Agent integrations installed by `mw integrate`.

use std::path::PathBuf;

mod files;

pub mod claude;
pub mod hermes;
pub mod rho;

pub(crate) const SKILL: &str = include_str!("../../integrate/SKILL.md");

/// The independently detectable state of one integration component.
#[derive(Debug, Clone)]
pub enum ComponentStatus {
    Healthy,
    Missing,
    Invalid(String),
}

/// The independently detectable pieces of an agent integration.
#[derive(Debug, Clone)]
pub struct IntegrationDiagnostics {
    pub config_dir: PathBuf,
    pub mcp: ComponentStatus,
    pub auto_capture: ComponentStatus,
    pub skill: ComponentStatus,
}
