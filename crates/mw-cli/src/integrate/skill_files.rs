//! Staged skill-file transactions in the Interfaces layer.
//! The caller validates absolute paths and holds the lock outside `dir`.
//! Private staging protects against ordinary editor renames, not writes through
//! already-open file descriptors or a hostile process running as the same user.
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const SKILL: &str = "SKILL.md";
const MARKER: &str = ".memorywhale-owned.json";
const LIMIT: u64 = 64 * 1024;
const SNAPSHOT_LIMIT: u64 = LIMIT * 6 + 2;

fn rename_exclusive(from: &Path, to: &Path) -> io::Result<()> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        let from = CString::new(from.as_os_str().as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid path"))?;
        let to = CString::new(to.as_os_str().as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid path"))?;
        // Both operations atomically refuse *any* existing destination, including
        // an empty directory or dangling symlink. There is no unsafe fallback.
        #[cfg(target_os = "linux")]
        let result = unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                libc::AT_FDCWD,
                from.as_ptr(),
                libc::AT_FDCWD,
                to.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        #[cfg(target_os = "macos")]
        let result = unsafe { libc::renamex_np(from.as_ptr(), to.as_ptr(), libc::RENAME_EXCL) };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (from, to);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "atomic no-replace rename unavailable",
        ))
    }
}

fn staging(dir: &Path) -> Result<PathBuf, String> {
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    return Err("atomic no-replace rename is unsupported on this platform".into());
    #[allow(unreachable_code)]
    {
        let parent = dir.parent().ok_or("skill directory needs a parent")?;
        for _ in 0..16 {
            let mut random = [0u8; 16];
            getrandom::getrandom(&mut random).map_err(|_| "cannot generate staging name")?;
            let name: String = random.iter().map(|b| format!("{b:02x}")).collect();
            let path = parent.join(format!(".memorywhale-skill-stage-{name}"));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => return Ok(path),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Err("cannot create private skill staging directory".into()),
            }
        }
        Err("cannot allocate unique skill staging directory".into())
    }
}

// Only install uses automatic cleanup: removal staging contains user data and
// must never be cleaned up on a failed restore.
struct InstallStage {
    path: PathBuf,
    created: Vec<&'static str>,
}
impl Drop for InstallStage {
    fn drop(&mut self) {
        for name in &self.created {
            let _ = fs::remove_file(self.path.join(name));
        }
        let _ = fs::remove_dir(&self.path);
    }
}

pub(super) fn install(dir: &Path, text: &str) -> Result<(), String> {
    install_with(dir, text, |_, _| Ok(()))
}

fn install_with(
    dir: &Path,
    text: &str,
    mut before_io: impl FnMut(usize, bool) -> io::Result<()>,
) -> Result<(), String> {
    if text.len() as u64 > LIMIT {
        return Err("skill exceeds size limit".into());
    }
    let snapshot = serde_json::to_string(text).map_err(|_| "cannot serialize ownership record")?;
    let mut stage = InstallStage {
        path: staging(dir)?,
        created: Vec::new(),
    };
    for (index, (name, content)) in [(MARKER, snapshot.as_str()), (SKILL, text)]
        .into_iter()
        .enumerate()
    {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(stage.path.join(name))
            .map_err(|_| "cannot create staged skill file")?;
        stage.created.push(name);
        before_io(index, false)
            .and_then(|_| file.write_all(content.as_bytes()))
            .and_then(|_| before_io(index, true))
            .and_then(|_| file.sync_all())
            .map_err(|_| "cannot persist staged skill file")?;
    }
    fs::File::open(&stage.path)
        .and_then(|file| file.sync_all())
        .map_err(|_| "cannot persist staged skill directory")?;
    rename_exclusive(&stage.path, dir).map_err(|_| {
        "cannot publish skill directory atomically; existing destination was not overwritten"
            .to_string()
    })
}

fn read_staged(path: &Path, limit: u64) -> Result<String, String> {
    let meta = fs::symlink_metadata(path).map_err(|_| "cannot inspect staged skill file")?;
    if !meta.is_file() || meta.len() > limit {
        return Err("staged skill files must be regular files within the size limit".into());
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options
        .open(path)
        .map_err(|_| "cannot open staged skill file")?;
    if !file
        .metadata()
        .map_err(|_| "cannot inspect opened skill file")?
        .is_file()
    {
        return Err("staged skill files must be regular files".into());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read staged skill file")?;
    if bytes.len() as u64 > limit {
        return Err("staged skill file exceeds size limit".into());
    }
    String::from_utf8(bytes).map_err(|_| "staged skill file must be UTF-8".into())
}

fn validate(stage: &Path, text: &str) -> Result<(), String> {
    let owned: String = serde_json::from_str(&read_staged(&stage.join(MARKER), SNAPSHOT_LIMIT)?)
        .map_err(|_| "invalid staged ownership record")?;
    if owned != text || read_staged(&stage.join(SKILL), LIMIT)? != owned {
        return Err("skill differs from the owned snapshot or supplied source; refusing to remove user changes".into());
    }
    Ok(())
}

fn restore(dir: &Path, stage: &Path, moved: &[&str], error: String) -> String {
    let mut retained = false;
    for name in moved.iter().rev() {
        if rename_exclusive(&stage.join(name), &dir.join(name)).is_err() {
            retained = true;
        }
    }
    if retained {
        format!("{error}; destination replacements were not overwritten; recover retained entries from {}", stage.display())
    } else if fs::remove_dir(stage).is_err() {
        format!(
            "{error}; entries restored; inspect staging directory {}",
            stage.display()
        )
    } else {
        format!("{error}; staged entries restored without overwriting existing files")
    }
}

pub(super) fn remove(dir: &Path, text: &str) -> Result<(), String> {
    remove_with(dir, text, |_| {})
}

fn remove_with(dir: &Path, text: &str, after_staging: impl FnOnce(&Path)) -> Result<(), String> {
    let stage = staging(dir)?;
    let mut moved = Vec::new();
    for name in [SKILL, MARKER] {
        if rename_exclusive(&dir.join(name), &stage.join(name)).is_err() {
            return Err(restore(
                dir,
                &stage,
                &moved,
                "cannot stage skill files for removal".into(),
            ));
        }
        moved.push(name);
    }
    after_staging(&stage);
    if let Err(error) = validate(&stage, text) {
        return Err(restore(dir, &stage, &moved, error));
    }
    for name in moved {
        fs::remove_file(stage.join(name)).map_err(|_| {
            format!(
                "cannot delete validated staged entry; recover remaining entries from {}",
                stage.display()
            )
        })?;
    }
    fs::remove_dir(&stage)
        .map_err(|_| format!("cannot remove empty staging directory {}", stage.display()))?;
    if [SKILL, MARKER]
        .iter()
        .any(|name| fs::symlink_metadata(dir.join(name)).is_ok())
    {
        return Err("owned files removed, but a concurrent replacement remains untouched; inspect the skill directory".into());
    }
    match fs::remove_dir(dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => match fs::read_dir(dir) {
            Ok(mut entries) => match entries.next() {
                Some(Ok(_)) => Ok(()),
                _ => Err("owned files removed, but cannot remove empty skill directory".into()),
            },
            _ => Err("owned files removed, but cannot remove empty skill directory".into()),
        },
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
#[path = "skill_files_tests.rs"]
mod tests;
