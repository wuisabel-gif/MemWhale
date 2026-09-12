use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
// Compile the adapter independently as well as testing its public CLI dispatch.
#[path = "../src/integrate/mcp_client.rs"]
mod adapter;

#[test]
fn rejects_missing_client_and_invalid_options() {
    assert!(adapter::cli(&[]).is_err());
    assert!(adapter::cli(&["other".into()]).is_err());
    assert!(adapter::cli(&["codex".into(), "--config".into()]).is_err());
    assert!(adapter::cli(&["cursor".into(), "--check".into(), "--revert".into()]).is_err());
}

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "mw-mcp-client-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&p).unwrap();
        Self(fs::canonicalize(p).unwrap())
    }
    fn run(&self, client: &str, args: &[&str]) -> Output {
        self.run_with_data_dir(client, args, &self.0.join("data"))
    }
    fn run_with_data_dir(&self, client: &str, args: &[&str], data_dir: &Path) -> Output {
        assert!(Path::new(env!("CARGO_BIN_EXE_mw-mcp")).is_file());
        Command::new(env!("CARGO_BIN_EXE_mw"))
            .args(["integrate", client])
            .args(args)
            .env("HOME", &self.0)
            .env("CODEX_HOME", self.0.join("codex"))
            .env("RHO_HOME", self.0.join("rho"))
            .env("MEMORYWHALE_DATA_DIR", data_dir)
            .env("PATH", "")
            .current_dir(&self.0)
            .output()
            .unwrap()
    }
    fn config(&self, client: &str) -> PathBuf {
        self.0.join(if client == "codex" {
            "codex/config.toml"
        } else {
            ".cursor/mcp.json"
        })
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn success(o: Output) {
    assert!(
        o.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
}
fn failure(o: Output) {
    assert!(
        !o.status.success(),
        "unexpected success: {}",
        String::from_utf8_lossy(&o.stdout)
    );
}
fn owner(p: &Path) -> PathBuf {
    PathBuf::from(format!("{}.memorywhale-mcp-owner.json", p.display()))
}

// Model an earlier owned installation without moving/chmodding shared test binaries.
fn set_owned_command(config: &Path, client: &str, command: &Path) {
    let text = fs::read_to_string(config).unwrap();
    let snapshot = if client == "codex" {
        let mut doc = text.parse::<toml_edit::DocumentMut>().unwrap();
        doc["mcp_servers"]["memorywhale"]["command"] = toml_edit::value(command.to_str().unwrap());
        fs::write(config, doc.to_string()).unwrap();
        let doc = fs::read_to_string(config)
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        serde_json::Value::String(doc["mcp_servers"]["memorywhale"].to_string())
    } else {
        let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
        doc["mcpServers"]["memorywhale"]["command"] = serde_json::json!(command);
        fs::write(config, serde_json::to_vec_pretty(&doc).unwrap()).unwrap();
        doc["mcpServers"]["memorywhale"].clone()
    };
    let mut journal: serde_json::Value =
        serde_json::from_slice(&fs::read(owner(config)).unwrap()).unwrap();
    journal["entry"] = snapshot;
    fs::write(owner(config), serde_json::to_vec(&journal).unwrap()).unwrap();
}

#[test]
fn moved_or_missing_owned_executable_is_diagnosed_without_mutation() {
    for client in ["codex", "cursor"] {
        let s = Sandbox::new();
        success(s.run(client, &[]));
        let old = s.0.join("old-mw-mcp");
        fs::write(&old, "old executable fixture; must never run").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&old, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let config = s.config(client);
        set_owned_command(&config, client, &old);
        success(s.run(client, &["--check"]));
        let before = fs::read(&config).unwrap();
        let journal = fs::read(owner(&config)).unwrap();
        for mode in [vec![], vec!["--dry-run"]] {
            let result = s.run(client, &mode);
            assert!(String::from_utf8_lossy(&result.stderr).contains("--revert"));
            failure(result);
            assert_eq!(before, fs::read(&config).unwrap());
            assert_eq!(journal, fs::read(owner(&config)).unwrap());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&old, fs::Permissions::from_mode(0o600)).unwrap();
            failure(s.run(client, &["--check"]));
        }
        fs::remove_file(&old).unwrap();
        failure(s.run(client, &["--check"]));
        assert_eq!(before, fs::read(&config).unwrap());
        success(s.run(client, &["--revert"]));
        success(s.run(client, &[]));
        success(s.run(client, &["--check"]));
    }
}

#[test]
fn changed_data_override_requires_explicit_reinstall() {
    for client in ["codex", "cursor"] {
        let s = Sandbox::new();
        success(s.run(client, &[]));
        let before = fs::read(s.config(client)).unwrap();
        let journal = fs::read(owner(&s.config(client))).unwrap();
        let next = s.0.join("different-data");
        for mode in [vec![], vec!["--dry-run"]] {
            let result = s.run_with_data_dir(client, &mode, &next);
            assert!(String::from_utf8_lossy(&result.stderr).contains("--revert"));
            failure(result);
            assert_eq!(before, fs::read(s.config(client)).unwrap());
            assert_eq!(journal, fs::read(owner(&s.config(client))).unwrap());
        }
        success(s.run_with_data_dir(client, &["--revert"], &next));
        success(s.run_with_data_dir(client, &[], &next));
        success(s.run_with_data_dir(client, &["--check"], &next));
    }
}

#[test]
fn relative_data_override_is_persisted_against_installer_cwd() {
    for client in ["codex", "cursor"] {
        let s = Sandbox::new();
        success(s.run_with_data_dir(client, &[], Path::new("relative-data")));
        let text = fs::read_to_string(s.config(client)).unwrap();
        let persisted = if client == "codex" {
            let doc = text.parse::<toml_edit::DocumentMut>().unwrap();
            doc["mcp_servers"]["memorywhale"]["env"]["MEMORYWHALE_DATA_DIR"]
                .as_str()
                .unwrap()
                .to_owned()
        } else {
            let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
            doc["mcpServers"]["memorywhale"]["env"]["MEMORYWHALE_DATA_DIR"]
                .as_str()
                .unwrap()
                .to_owned()
        };
        assert!(Path::new(&persisted).is_absolute());
        assert_eq!(PathBuf::from(persisted), s.0.join("relative-data"));
    }
}
#[test]
fn defaults_idempotence_and_revert_preserve_new_servers() {
    for client in ["codex", "cursor"] {
        let s = Sandbox::new();
        let p = s.config(client);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, if client == "codex" { "# keep this comment\nsecret = 'do-not-copy-this-token'\n[mcp_servers.other]\ncommand = 'other'\n" } else { r#"{"secret":"do-not-copy-this-token","mcpServers":{"other":{"command":"other"}}}"# }).unwrap();
        success(s.run(client, &[]));
        let installed = fs::read(&p).unwrap();
        let journal = fs::read_to_string(owner(&p)).unwrap();
        assert!(!journal.contains("do-not-copy-this-token"));
        assert!(!journal.contains("other"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(owner(&p)).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        success(s.run(client, &[]));
        assert_eq!(installed, fs::read(&p).unwrap());
        let check = s.run(client, &["--check"]);
        assert!(String::from_utf8_lossy(&check.stdout).contains("connectivity not tested"));
        success(check);
        if client == "codex" {
            let mut d = String::from_utf8(installed).unwrap();
            assert!(d.contains("# keep this comment"));
            let doc = d.parse::<toml_edit::DocumentMut>().unwrap();
            assert!(Path::new(
                doc["mcp_servers"]["memorywhale"]["command"]
                    .as_str()
                    .unwrap()
            )
            .is_absolute());
            assert_eq!(
                doc["mcp_servers"]["memorywhale"]["args"]
                    .as_array()
                    .unwrap()
                    .len(),
                0
            );
            d.push_str("\n[mcp_servers.later]\ncommand = 'later'\n");
            fs::write(&p, d).unwrap();
        } else {
            let mut d: serde_json::Value = serde_json::from_slice(&installed).unwrap();
            assert_eq!(d["mcpServers"]["memorywhale"]["type"], "stdio");
            assert!(
                Path::new(d["mcpServers"]["memorywhale"]["command"].as_str().unwrap())
                    .is_absolute()
            );
            d["mcpServers"]["later"] = serde_json::json!({"command":"later"});
            fs::write(&p, serde_json::to_vec(&d).unwrap()).unwrap();
        }
        success(s.run(client, &["--revert"]));
        let remaining = fs::read_to_string(&p).unwrap();
        assert!(remaining.contains("later"));
        assert!(remaining.contains("do-not-copy-this-token"));
        assert!(!remaining.contains("mw-mcp"));
        assert!(!owner(&p).exists());
        success(s.run(client, &["--revert"]));
    }
}
#[test]
fn custom_paths_and_read_only_modes() {
    for client in ["codex", "cursor"] {
        let s = Sandbox::new();
        let p = s.0.join("nested/custom.conf");
        let arg = p.to_str().unwrap();
        success(s.run(client, &["--config", arg, "--dry-run"]));
        assert!(!p.parent().unwrap().exists());
        failure(s.run(client, &["--config", arg, "--check"]));
        assert!(!p.parent().unwrap().exists());
        failure(s.run(client, &["--config", arg, "--dry-run", "--revert"]));
        assert!(!p.exists());
        success(s.run(client, &["--config", arg]));
        assert!(p.exists());
        assert!(!s.config(client).exists());
        let original = fs::read(&p).unwrap();
        let journal = fs::read(owner(&p)).unwrap();
        success(s.run(client, &["--config", arg, "--dry-run"]));
        success(s.run(client, &["--config", arg, "--check"]));
        assert_eq!(original, fs::read(&p).unwrap());
        assert_eq!(journal, fs::read(owner(&p)).unwrap());
    }
}
#[test]
fn conflicts_modified_entries_and_redacted_malformed_config() {
    for client in ["codex", "cursor"] {
        let s = Sandbox::new();
        let p = s.config(client);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        let conflict = if client == "codex" {
            "[mcp_servers.memorywhale]\ncommand='mine'\n"
        } else {
            r#"{"mcpServers":{"memorywhale":{"command":"mine"}}}"#
        };
        fs::write(&p, conflict).unwrap();
        failure(s.run(client, &[]));
        failure(s.run(client, &["--revert"]));
        assert_eq!(fs::read_to_string(&p).unwrap(), conflict);
        assert!(!owner(&p).exists());
        fs::write(&p, "TOP_SECRET_TOKEN [ malformed").unwrap();
        let result = s.run(client, &[]);
        assert!(!String::from_utf8_lossy(&result.stderr).contains("TOP_SECRET_TOKEN"));
        assert!(!String::from_utf8_lossy(&result.stdout).contains("TOP_SECRET_TOKEN"));
        failure(result);
        fs::write(&p, if client == "codex" { "" } else { "{}" }).unwrap();
        success(s.run(client, &[]));
        let modified = fs::read_to_string(&p)
            .unwrap()
            .replace("mw-mcp", "user-mcp");
        fs::write(&p, &modified).unwrap();
        for args in [&[][..], &["--check"][..], &["--revert"][..]] {
            failure(s.run(client, args));
            assert_eq!(fs::read_to_string(&p).unwrap(), modified);
        }
    }
}
#[test]
fn oversized_nonregular_and_pending_ownership_rejected() {
    for client in ["codex", "cursor"] {
        let s = Sandbox::new();
        let p = s.config(client);
        fs::create_dir_all(&p).unwrap();
        failure(s.run(client, &[]));
        fs::remove_dir(&p).unwrap();
        fs::write(&p, vec![b' '; 1024 * 1024 + 1]).unwrap();
        failure(s.run(client, &[]));
        assert!(!owner(&p).exists());
        fs::write(&p, if client == "codex" { "" } else { "{}" }).unwrap();
        success(s.run(client, &[]));
        let mut journal: serde_json::Value =
            serde_json::from_slice(&fs::read(owner(&p)).unwrap()).unwrap();
        journal["state"] = "pending".into();
        fs::write(owner(&p), serde_json::to_vec(&journal).unwrap()).unwrap();
        let original = fs::read(&p).unwrap();
        failure(s.run(client, &["--revert"]));
        assert_eq!(original, fs::read(&p).unwrap());
    }
}
#[cfg(unix)]
#[test]
fn symlinks_including_dangling_and_ancestors_rejected_and_mode_preserved() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    for client in ["codex", "cursor"] {
        let s = Sandbox::new();
        let p = s.config(client);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        let target = s.0.join("real");
        symlink(&target, &p).unwrap();
        failure(s.run(client, &[]));
        assert!(!target.exists());
        fs::write(&target, if client == "codex" { "" } else { "{}" }).unwrap();
        failure(s.run(client, &[]));
        fs::remove_file(&p).unwrap();
        fs::write(&p, if client == "codex" { "" } else { "{}" }).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o640)).unwrap();
        success(s.run(client, &[]));
        assert_eq!(
            fs::metadata(&p).unwrap().permissions().mode() & 0o777,
            0o640
        );
        let link = s.0.join("linked");
        symlink(p.parent().unwrap(), &link).unwrap();
        failure(s.run(
            client,
            &[
                "--config",
                link.join(p.file_name().unwrap()).to_str().unwrap(),
                "--check",
            ],
        ));
        let dangling = s.0.join("dangling");
        symlink(s.0.join("absent"), &dangling).unwrap();
        failure(s.run(
            client,
            &[
                "--config",
                dangling.join("new").to_str().unwrap(),
                "--dry-run",
            ],
        ));
    }
}
#[test]
fn codex_home_fallback_and_optional_environment() {
    let s = Sandbox::new();
    let o = Command::new(env!("CARGO_BIN_EXE_mw"))
        .args(["integrate", "codex"])
        .env("HOME", &s.0)
        .env_remove("CODEX_HOME")
        .env_remove("MEMORYWHALE_DATA_DIR")
        .env("RHO_HOME", s.0.join("rho"))
        .env("PATH", "")
        .current_dir(&s.0)
        .output()
        .unwrap();
    success(o);
    let d = fs::read_to_string(s.0.join(".codex/config.toml"))
        .unwrap()
        .parse::<toml_edit::DocumentMut>()
        .unwrap();
    assert!(d["mcp_servers"]["memorywhale"].get("env").is_none());
}

#[test]
fn cursor_duplicate_keys_are_rejected_without_leaking_or_writing() {
    let s = Sandbox::new();
    let p = s.config("cursor");
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    let text = r#"{"token":"PRIVATE_SENTINEL","token":"second","mcpServers":{}}"#;
    fs::write(&p, text).unwrap();
    let output = s.run("cursor", &[]);
    assert!(!String::from_utf8_lossy(&output.stderr).contains("PRIVATE_SENTINEL"));
    failure(output);
    assert_eq!(fs::read_to_string(&p).unwrap(), text);
    assert!(!owner(&p).exists());
}

#[cfg(unix)]
#[test]
fn ownership_and_lock_symlinks_are_rejected_even_when_dangling() {
    use std::os::unix::fs::symlink;
    for client in ["codex", "cursor"] {
        for suffix in [".memorywhale-mcp-owner.json", ".memorywhale-mcp.lock"] {
            let s = Sandbox::new();
            let p = s.config(client);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            let link = PathBuf::from(format!("{}{suffix}", p.display()));
            let target = s.0.join("absent");
            symlink(&target, &link).unwrap();
            failure(s.run(client, &["--dry-run"]));
            failure(s.run(client, &[]));
            assert!(!p.exists());
            assert!(!target.exists());
        }
    }
}
