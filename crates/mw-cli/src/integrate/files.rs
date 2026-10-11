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

/// Read `path` if it exists. `Ok(None)` means the file is absent; `Err` means
/// it existed but could not be read.
pub(crate) fn read_existing(path: &Path) -> Result<Option<String>, std::io::Error> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

pub(crate) fn read_or_empty(path: &Path) -> Result<String, String> {
    match read_existing(path) {
        Ok(Some(text)) => Ok(text),
        Ok(None) => Ok(String::new()),
        Err(err) => Err(format!("failed to read {}: {err}", path.display())),
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

/// Absolute path to `mw-remember` from the same install as this `mw`.
pub(crate) fn mw_remember_executable() -> Result<PathBuf, String> {
    stable_executable("mw-remember")
}

/// Absolute path to helper `name` from the same install as this `mw`,
/// preferring a PATH entry for the same file (Homebrew's `/opt/homebrew/bin`
/// symlink) over its resolved, versioned location. Hook and MCP commands that
/// name the versioned directory break once an upgrade removes it.
pub(crate) fn stable_executable(name: &str) -> Result<PathBuf, String> {
    let name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    let on_path: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|path| {
            std::env::split_paths(&path)
                .filter(|dir| dir.is_absolute())
                .map(|dir| dir.join(&name))
                .filter(|candidate| candidate.is_file())
                .collect()
        })
        .unwrap_or_default();
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(&name)))
        .filter(|candidate| candidate.is_file());
    prefer_stable(beside, on_path).ok_or_else(|| format!("{name} not found next to mw or on PATH"))
}

/// True when a configured helper path may be replaced by `new` without asking:
/// it no longer exists (an upgrade removed it), is a versioned Homebrew
/// install of MemoryWhale (kept after an upgrade without cleanup), or names
/// the same file.
pub(crate) fn replaceable_executable(old: &Path, new: Option<PathBuf>) -> bool {
    !old.exists()
        || old.to_string_lossy().contains("/Cellar/memorywhale/")
        || new.is_some_and(|new| {
            fs::canonicalize(old).ok().is_some()
                && fs::canonicalize(old).ok() == fs::canonicalize(new).ok()
        })
}

/// The helper beside `mw`, named by a PATH entry that resolves to the same
/// file when there is one; else the first PATH match.
fn prefer_stable(beside: Option<PathBuf>, on_path: Vec<PathBuf>) -> Option<PathBuf> {
    let Some(beside) = beside else {
        return on_path.into_iter().next();
    };
    let target = fs::canonicalize(&beside).ok()?;
    Some(
        on_path
            .into_iter()
            .find(|candidate| fs::canonicalize(candidate).ok().as_ref() == Some(&target))
            .unwrap_or(beside),
    )
}

#[cfg(all(test, unix))]
mod stable_path_tests {
    use super::*;

    #[test]
    fn prefers_a_path_symlink_over_the_versioned_install() {
        let root = std::env::temp_dir().join(format!("mw-stable-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let cellar = root.join("Cellar/memorywhale/0.18.0/bin");
        let bin = root.join("bin");
        let other = root.join("other");
        for dir in [&cellar, &bin, &other] {
            fs::create_dir_all(dir).unwrap();
        }
        let real = cellar.join("mw-remember");
        fs::write(&real, "").unwrap();
        std::os::unix::fs::symlink(&real, bin.join("mw-remember")).unwrap();
        fs::write(other.join("mw-remember"), "").unwrap();

        // A different install earlier on PATH is skipped; the symlink to the
        // running install wins over its versioned location.
        assert_eq!(
            prefer_stable(
                Some(real.clone()),
                vec![other.join("mw-remember"), bin.join("mw-remember")]
            ),
            Some(bin.join("mw-remember"))
        );
        // No PATH name for it: keep the path beside mw.
        assert_eq!(
            prefer_stable(Some(real.clone()), vec![other.join("mw-remember")]),
            Some(real)
        );
        // Nothing beside mw: first PATH match.
        assert_eq!(
            prefer_stable(None, vec![other.join("mw-remember")]),
            Some(other.join("mw-remember"))
        );
        // Replace without asking: a removed path, a versioned Homebrew
        // install, or the same file. Keep: a different, existing helper.
        let stable = Some(bin.join("mw-remember"));
        assert!(replaceable_executable(
            &root.join("gone/mw-remember"),
            stable.clone()
        ));
        assert!(replaceable_executable(
            &cellar.join("mw-remember"),
            stable.clone()
        ));
        assert!(!replaceable_executable(&other.join("mw-remember"), stable));
        let _ = fs::remove_dir_all(&root);
    }
}

/// True when `name` (or `name.exe` on Windows) exists as a file on PATH.
/// Looks at the filesystem only; does not execute the binary.
pub(crate) fn command_on_path(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    for dir in std::env::split_paths(&path) {
        if dir.join(name).is_file() {
            return true;
        }
        if cfg!(windows) && dir.join(format!("{name}.exe")).is_file() {
            return true;
        }
    }
    false
}

/// Whether the bundled skill file is present and nonempty.
///
/// `Err` means the file exists but could not be read (permissions or invalid
/// UTF-8). Callers must not treat that as absence.
pub(crate) fn skill_is_installed(config_dir: &Path) -> Result<bool, std::io::Error> {
    match read_existing(&BundledLayout::from_config_dir(config_dir.to_path_buf()).skill_path) {
        Ok(Some(text)) => Ok(!text.trim().is_empty()),
        Ok(None) => Ok(false),
        Err(err) => Err(err),
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
