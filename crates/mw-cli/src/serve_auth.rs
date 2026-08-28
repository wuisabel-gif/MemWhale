//! Token files for `mw-serve` LAN auth and a Rho HTTP client copy.
//!
//! `serve.token` is the secret `mw-serve` demands. `mcp-authorization` is only
//! a client-side `Bearer …` copy written by `mw integrate rho --http --token`
//! so Rho can send `MEMORYWHALE_AUTHORIZATION`. `mw-serve` never reads it.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static EXCLUSIVE_TMP_SEQ: AtomicU64 = AtomicU64::new(0);

use crate::{data_dir, restrict_path_permissions};

pub const SERVE_TOKEN_FILE: &str = "serve.token";
pub const MCP_AUTHORIZATION_FILE: &str = "mcp-authorization";

/// 32 random bytes, hex-encoded. Same entropy as `openssl rand -hex 32`.
const TOKEN_BYTES: usize = 32;

pub fn serve_token_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join(SERVE_TOKEN_FILE))
}

pub fn mcp_authorization_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join(MCP_AUTHORIZATION_FILE))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenSource {
    Explicit,
    File,
    Minted,
}

pub struct LoadedToken {
    pub value: String,
    pub source: TokenSource,
}

/// Token `mw-serve` will actually use: an explicit value (CLI/env already
/// folded in by the caller), else the file, else a newly minted file.
pub fn load_or_mint_serve_token(explicit: &str) -> Result<LoadedToken, String> {
    let explicit = explicit.trim();
    if !explicit.is_empty() {
        return Ok(LoadedToken {
            value: explicit.to_string(),
            source: TokenSource::Explicit,
        });
    }
    let path = serve_token_path()?;
    if let Some(existing) = read_secret_file_if_present(&path)? {
        return Ok(LoadedToken {
            value: existing,
            source: TokenSource::File,
        });
    }
    let token = mint_token()?;
    if create_secret_file_exclusive(&path, &token)? {
        return Ok(LoadedToken {
            value: token,
            source: TokenSource::Minted,
        });
    }
    Ok(LoadedToken {
        value: read_secret_file(&path)?,
        source: TokenSource::File,
    })
}

/// Persist `Bearer <raw>` for a Rho HTTP client. Export
/// `MEMORYWHALE_AUTHORIZATION` from this file; the capture hook does not load it.
/// `mw integrate rho --revert` removes this copy. It does not touch `serve.token`.
pub fn write_mcp_authorization(raw_token: &str) -> Result<PathBuf, String> {
    let raw_token = raw_token.trim();
    if raw_token.is_empty() {
        return Err("token must not be empty".to_string());
    }
    let path = mcp_authorization_path()?;
    write_secret_file(&path, &format!("Bearer {raw_token}"))?;
    Ok(path)
}

/// Current `mcp-authorization` text, if the file exists.
pub fn snapshot_mcp_authorization() -> Result<Option<String>, String> {
    let path = mcp_authorization_path()?;
    match fs::read_to_string(&path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("failed to read {}: {e}", path.display())),
    }
}

/// Restore a snapshot from [`snapshot_mcp_authorization`].
pub fn restore_mcp_authorization(previous: Option<&str>) -> Result<(), String> {
    match previous {
        Some(text) => {
            let path = mcp_authorization_path()?;
            write_secret_file(&path, text.trim_end_matches(['\n', '\r']))?;
            Ok(())
        }
        None => {
            remove_mcp_authorization()?;
            Ok(())
        }
    }
}

/// Delete the client-side bearer copy. Returns whether a file was removed.
pub fn remove_mcp_authorization() -> Result<bool, String> {
    let path = mcp_authorization_path()?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("failed to remove {}: {e}", path.display())),
    }
}

fn mint_token() -> Result<String, String> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::getrandom(&mut bytes).map_err(|e| format!("failed to generate token: {e}"))?;
    Ok(bytes
        .iter()
        .fold(String::with_capacity(TOKEN_BYTES * 2), |mut hex, byte| {
            use std::fmt::Write;
            let _ = write!(hex, "{byte:02x}");
            hex
        }))
}

fn read_secret_file_if_present(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(text) => {
            let token = text.trim();
            if token.is_empty() {
                return Err(format!("{} is empty", path.display()));
            }
            Ok(Some(token.to_string()))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("failed to read {}: {e}", path.display())),
    }
}

fn read_secret_file(path: &Path) -> Result<String, String> {
    read_secret_file_if_present(path)?.ok_or_else(|| format!("{} is missing", path.display()))
}

/// Publish `contents` to `path` only if `path` does not exist.
///
/// Writes a unique temp file first, then `hard_link`s it into place so a
/// racing reader never sees a half-written token. Returns whether this
/// process created the file.
fn create_secret_file_exclusive(path: &Path, contents: &str) -> Result<bool, String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    restrict_path_permissions(parent, true)?;
    let tmp = parent.join(format!(
        ".{}.{}-{}-{}.tmp",
        path.file_name()
            .ok_or_else(|| format!("path has no file name: {}", path.display()))?
            .to_string_lossy(),
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        EXCLUSIVE_TMP_SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    {
        let mut file = fs::File::create(&tmp)
            .map_err(|e| format!("failed to create {}: {e}", tmp.display()))?;
        file.write_all(contents.as_bytes())
            .map_err(|e| format!("failed to write {}: {e}", tmp.display()))?;
        file.write_all(b"\n")
            .map_err(|e| format!("failed to write {}: {e}", tmp.display()))?;
        file.sync_all()
            .map_err(|e| format!("failed to sync {}: {e}", tmp.display()))?;
    }
    restrict_path_permissions(&tmp, false)?;
    match fs::hard_link(&tmp, path) {
        Ok(()) => {
            let _ = fs::remove_file(&tmp);
            restrict_path_permissions(path, false)?;
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let _ = fs::remove_file(&tmp);
            Ok(false)
        }
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            Err(format!("failed to create {}: {e}", path.display()))
        }
    }
}

fn write_secret_file(path: &Path, contents: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    restrict_path_permissions(parent, true)?;
    let tmp = parent.join(format!(
        ".{}.tmp",
        path.file_name()
            .ok_or_else(|| format!("path has no file name: {}", path.display()))?
            .to_string_lossy()
    ));
    {
        let mut file = fs::File::create(&tmp)
            .map_err(|e| format!("failed to create {}: {e}", tmp.display()))?;
        file.write_all(contents.as_bytes())
            .map_err(|e| format!("failed to write {}: {e}", tmp.display()))?;
        file.write_all(b"\n")
            .map_err(|e| format!("failed to write {}: {e}", tmp.display()))?;
        file.sync_all()
            .map_err(|e| format!("failed to sync {}: {e}", tmp.display()))?;
    }
    restrict_path_permissions(&tmp, false)?;
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("failed to replace {}: {e}", path.display())
    })?;
    restrict_path_permissions(path, false)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EnvDirGuard {
        dir: PathBuf,
        previous: Option<std::ffi::OsString>,
    }

    impl Drop for EnvDirGuard {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => std::env::set_var("MEMORYWHALE_DATA_DIR", value),
                None => std::env::remove_var("MEMORYWHALE_DATA_DIR"),
            }
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn with_data_dir<T>(name: &str, body: impl FnOnce(&std::path::Path) -> T) -> T {
        let _lock = crate::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!(
            "mw-serve-auth-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let previous = std::env::var_os("MEMORYWHALE_DATA_DIR");
        std::env::set_var("MEMORYWHALE_DATA_DIR", &dir);
        let _guard = EnvDirGuard {
            dir: dir.clone(),
            previous,
        };
        body(&dir)
    }

    #[test]
    fn explicit_token_wins_over_file() {
        with_data_dir("explicit", |_| {
            let minted = load_or_mint_serve_token("").unwrap();
            assert_eq!(minted.value.len(), TOKEN_BYTES * 2);
            assert_eq!(minted.source, TokenSource::Minted);
            assert_eq!(
                load_or_mint_serve_token("from-cli").unwrap().value,
                "from-cli"
            );
            let again = load_or_mint_serve_token("").unwrap();
            assert_eq!(again.value, minted.value);
            assert_eq!(again.source, TokenSource::File);
        });
    }

    #[test]
    fn authorization_file_is_bearer_prefixed() {
        with_data_dir("authz", |dir| {
            let path = write_mcp_authorization("abc").unwrap();
            assert_eq!(path, dir.join(MCP_AUTHORIZATION_FILE));
            assert_eq!(fs::read_to_string(&path).unwrap().trim(), "Bearer abc");
            let snapshot = snapshot_mcp_authorization().unwrap();
            write_mcp_authorization("replacement").unwrap();
            assert_eq!(
                fs::read_to_string(&path).unwrap().trim(),
                "Bearer replacement"
            );
            restore_mcp_authorization(snapshot.as_deref()).unwrap();
            assert_eq!(fs::read_to_string(&path).unwrap().trim(), "Bearer abc");
        });
    }

    #[test]
    fn exclusive_create_loser_leaves_the_winner() {
        let dir = std::env::temp_dir().join(format!(
            "mw-serve-auth-race-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(SERVE_TOKEN_FILE);
        let (first, second) = std::thread::scope(|scope| {
            let a = scope.spawn(|| {
                let token = mint_token().unwrap();
                let created = create_secret_file_exclusive(&path, &token).unwrap();
                (created, token)
            });
            let b = scope.spawn(|| {
                let token = mint_token().unwrap();
                let created = create_secret_file_exclusive(&path, &token).unwrap();
                (created, token)
            });
            (a.join().unwrap(), b.join().unwrap())
        });
        assert_ne!(first.0, second.0, "exactly one creator");
        let file = read_secret_file(&path).unwrap();
        let winner = if first.0 { &first.1 } else { &second.1 };
        assert_eq!(&file, winner);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn env_dir_guard_restores_env_after_panic() {
        let _lock = crate::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let previous = std::env::var_os("MEMORYWHALE_DATA_DIR");
        let dir = std::env::temp_dir().join(format!(
            "mw-serve-auth-panic-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        std::env::set_var("MEMORYWHALE_DATA_DIR", &dir);
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = EnvDirGuard {
                dir: dir.clone(),
                previous: previous.clone(),
            };
            panic!("boom");
        }));
        assert!(panicked.is_err());
        assert_eq!(std::env::var_os("MEMORYWHALE_DATA_DIR"), previous);
        assert!(!dir.exists());
    }
}
