//! Shared token files for `mw-serve` LAN auth and Rho Streamable HTTP.
//!
//! `serve.token` is the raw secret the server demands. `mcp-authorization` is
//! the `Bearer …` header value a client process should export as
//! `MEMORYWHALE_AUTHORIZATION`. They are allowed to disagree: one is "what I
//! demand," the other is "what I send to a remote server."

use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

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

/// Token `mw-serve` will actually use: an explicit value (CLI/env already
/// folded in by the caller), else the file, else a newly minted file.
pub fn load_or_mint_serve_token(explicit: &str) -> Result<String, String> {
    let explicit = explicit.trim();
    if !explicit.is_empty() {
        return Ok(explicit.to_string());
    }
    let path = serve_token_path()?;
    if path.exists() {
        return read_secret_file(&path);
    }
    let token = mint_token()?;
    write_secret_file(&path, &token)?;
    Ok(token)
}

/// Persist `Bearer <raw>` for the Rho client hook.
pub fn write_mcp_authorization(raw_token: &str) -> Result<PathBuf, String> {
    let raw_token = raw_token.trim();
    if raw_token.is_empty() {
        return Err("token must not be empty".to_string());
    }
    let path = mcp_authorization_path()?;
    write_secret_file(&path, &format!("Bearer {raw_token}"))?;
    Ok(path)
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

fn read_secret_file(path: &Path) -> Result<String, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let token = text.trim();
    if token.is_empty() {
        return Err(format!("{} is empty", path.display()));
    }
    Ok(token.to_string())
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
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_data_dir<T>(name: &str, body: impl FnOnce(&std::path::Path) -> T) -> T {
        let _guard = ENV_LOCK.lock().expect("env lock");
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
        let result = body(&dir);
        match previous {
            Some(value) => std::env::set_var("MEMORYWHALE_DATA_DIR", value),
            None => std::env::remove_var("MEMORYWHALE_DATA_DIR"),
        }
        let _ = fs::remove_dir_all(&dir);
        result
    }

    #[test]
    fn explicit_token_wins_over_file() {
        with_data_dir("explicit", |_| {
            let minted = load_or_mint_serve_token("").unwrap();
            assert_eq!(minted.len(), TOKEN_BYTES * 2);
            assert_eq!(load_or_mint_serve_token("from-cli").unwrap(), "from-cli");
            assert_eq!(load_or_mint_serve_token("").unwrap(), minted);
        });
    }

    #[test]
    fn authorization_file_is_bearer_prefixed() {
        with_data_dir("authz", |dir| {
            let path = write_mcp_authorization("abc").unwrap();
            assert_eq!(path, dir.join(MCP_AUTHORIZATION_FILE));
            assert_eq!(fs::read_to_string(&path).unwrap().trim(), "Bearer abc");
        });
    }
}
