//! Agent integrations installed by `mw integrate`.

mod files;

pub mod claude;
pub mod hermes;
pub mod rho;

pub(crate) const SKILL: &str = include_str!("../../integrate/SKILL.md");
