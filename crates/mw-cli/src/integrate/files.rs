use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Shared skill layout used by Claude Code and Rho.
pub(crate) struct BundledLayout {
    pub config_dir: PathBuf,
    pub skill_path: PathBuf,
    pub skill_dir: PathBuf,
}

impl BundledLayout {
    pub(crate) fn from_config_dir(config_dir: PathBuf) -> Self {
        let skill_dir = config_dir.join("skills/memorywhale");
        Self {
            skill_path: skill_dir.join("SKILL.md"),
            skill_dir,
            config_dir,
        }
    }
}

pub(crate) fn parse_revert(args: &[String], usage: &str) -> Result<bool, String> {
    let mut revert = false;
    for arg in args {
        match arg.as_str() {
            "--revert" => revert = true,
            _ => return Err(usage.to_string()),
        }
    }
    Ok(revert)
}

pub(crate) fn read_or_empty(path: &Path) -> Result<String, String> {
    if path.exists() {
        fs::read_to_string(path).map_err(|err| format!("failed to read {}: {err}", path.display()))
    } else {
        Ok(String::new())
    }
}

pub(crate) fn atomic_write(path: &Path, contents: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("path has no file name: {}", path.display()))?;
    let tmp = parent.join(format!(
        ".{}.tmp.{}",
        file_name.to_string_lossy(),
        std::process::id()
    ));
    fs::write(&tmp, contents).map_err(|err| format!("failed to write {}: {err}", tmp.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = path
            .metadata()
            .map(|meta| meta.permissions().mode())
            .unwrap_or(0o600);
        fs::set_permissions(&tmp, fs::Permissions::from_mode(mode))
            .map_err(|err| format!("failed to set permissions on {}: {err}", tmp.display()))?;
    }
    if let Err(err) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(format!("failed to write {}: {err}", path.display()));
    }
    Ok(())
}

pub(crate) fn write_or_remove(path: &Path, contents: &str) -> Result<(), String> {
    if contents.trim().is_empty() {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
            Err(err) => Err(format!("failed to remove {}: {err}", path.display())),
        }
    } else {
        atomic_write(path, contents)
    }
}

pub(crate) fn install_skill(layout: &BundledLayout, skill: &str) -> Result<(), String> {
    fs::create_dir_all(&layout.skill_dir)
        .map_err(|err| format!("failed to create {}: {err}", layout.skill_dir.display()))?;
    fs::write(&layout.skill_path, skill)
        .map_err(|err| format!("failed to write {}: {err}", layout.skill_path.display()))?;
    Ok(())
}

pub(crate) fn remove_legacy_python_hook(config_dir: &Path) -> Result<bool, String> {
    let path = config_dir.join("hooks/mw-record.py");
    match fs::remove_file(&path) {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(false),
        Err(err) => Err(format!("failed to remove {}: {err}", path.display())),
    }
}

/// Absolute path to `mw-remember` next to this `mw` binary, else on PATH.
pub(crate) fn mw_remember_executable() -> Result<PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|err| format!("failed to resolve current executable: {err}"))?;
    if let Some(dir) = exe.parent() {
        let candidate = remember_name(dir);
        if candidate.is_file() {
            return Ok(fs::canonicalize(&candidate).unwrap_or(candidate));
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = remember_name(&dir);
            if candidate.is_file() {
                return Ok(fs::canonicalize(&candidate).unwrap_or(candidate));
            }
        }
    }
    Err("mw-remember not found next to mw or on PATH".to_string())
}

fn remember_name(dir: &Path) -> PathBuf {
    if cfg!(windows) {
        dir.join("mw-remember.exe")
    } else {
        dir.join("mw-remember")
    }
}

pub(crate) fn remove_skill(layout: &BundledLayout) -> Result<bool, String> {
    if layout.skill_path.is_file() {
        fs::remove_file(&layout.skill_path)
            .map_err(|err| format!("failed to remove {}: {err}", layout.skill_path.display()))?;
        let _ = fs::remove_dir(&layout.skill_dir);
        Ok(true)
    } else {
        Ok(false)
    }
}
