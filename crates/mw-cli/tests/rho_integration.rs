use std::process::Command;

fn mw_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mw"));
    cmd.env("PATH", "");
    cmd
}

fn sandbox(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "mw-rho-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn user_can_install_memorywhale_into_a_fresh_rho_home() {
    let rho_dir = sandbox("fresh");

    let output = mw_cmd()
        .args(["integrate", "rho"])
        .env("RHO_HOME", &rho_dir)
        .output()
        .expect("run Rho integration command");

    assert!(output.status.success(), "command failed: {output:?}");
    assert!(!rho_dir.join("hooks/mw-record.py").exists());
    assert!(rho_dir.join("skills/memorywhale/SKILL.md").is_file());

    let hooks = std::fs::read_to_string(rho_dir.join("hooks.toml")).unwrap();
    assert!(hooks.contains("id = \"memorywhale-record\""));
    assert!(hooks.contains("after_tool_use"));
    assert!(hooks.contains("mw-remember"));
    assert!(hooks.contains("--from-hook"));
    assert!(hooks.contains("\"rho\""));
    assert!(!hooks.contains("python3"));

    let config = std::fs::read_to_string(rho_dir.join("config.toml")).unwrap();
    assert!(config.contains("[mcp.servers.memorywhale]"));
    assert!(config.contains("transport = \"stdio\""));
    assert!(config.contains("command = \"mw-mcp\""));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Rho"),
        "missing success message: {output:?}"
    );
}

#[test]
fn installing_memorywhale_preserves_existing_rho_settings() {
    let rho_dir = sandbox("existing");
    std::fs::write(
        rho_dir.join("hooks.toml"),
        r#"version = 1

[[hook]]
id = "fmt-rust"
on = "after_tool_use"
tools = ["edit"]
command = ["./fmt"]
timeout = "5s"
"#,
    )
    .unwrap();
    std::fs::write(
        rho_dir.join("config.toml"),
        r#"# keep me
[model]
provider = "openai"
"#,
    )
    .unwrap();

    let output = mw_cmd()
        .args(["integrate", "rho"])
        .env("RHO_HOME", &rho_dir)
        .output()
        .expect("run Rho integration command");

    assert!(output.status.success(), "command failed: {output:?}");
    let hooks = std::fs::read_to_string(rho_dir.join("hooks.toml")).unwrap();
    assert!(hooks.contains("id = \"fmt-rust\""));
    assert!(hooks.contains("id = \"memorywhale-record\""));
    let config = std::fs::read_to_string(rho_dir.join("config.toml")).unwrap();
    assert!(config.contains("# keep me"));
    assert!(config.contains("provider = \"openai\""));
    assert!(config.contains("memorywhale"));
}

#[test]
fn installing_memorywhale_twice_does_not_duplicate_the_hook() {
    let rho_dir = sandbox("idempotent");

    for _ in 0..2 {
        let output = mw_cmd()
            .args(["integrate", "rho"])
            .env("RHO_HOME", &rho_dir)
            .output()
            .expect("run Rho integration command");
        assert!(output.status.success(), "command failed: {output:?}");
    }

    let hooks = std::fs::read_to_string(rho_dir.join("hooks.toml")).unwrap();
    assert_eq!(hooks.matches("memorywhale-record").count(), 1);
    let config = std::fs::read_to_string(rho_dir.join("config.toml")).unwrap();
    assert_eq!(config.matches("memorywhale").count(), 1);
}

#[test]
fn invalid_rho_hooks_are_left_untouched() {
    let rho_dir = sandbox("invalid-hooks");
    let hooks_path = rho_dir.join("hooks.toml");
    let original = "version = [";
    std::fs::write(&hooks_path, original).unwrap();

    let output = mw_cmd()
        .args(["integrate", "rho"])
        .env("RHO_HOME", &rho_dir)
        .output()
        .expect("run Rho integration command");

    assert!(
        !output.status.success(),
        "invalid TOML was accepted: {output:?}"
    );
    assert_eq!(std::fs::read_to_string(hooks_path).unwrap(), original);
    assert!(
        !rho_dir.join("hooks/mw-record.py").exists(),
        "legacy hook was written despite invalid hooks.toml: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid Rho hooks.toml"),
        "unexpected error: {output:?}"
    );
}

#[test]
fn invalid_rho_config_is_left_untouched() {
    let rho_dir = sandbox("invalid-config");
    let config_path = rho_dir.join("config.toml");
    let original = "model = [";
    std::fs::write(&config_path, original).unwrap();

    let output = mw_cmd()
        .args(["integrate", "rho"])
        .env("RHO_HOME", &rho_dir)
        .output()
        .expect("run Rho integration command");

    assert!(
        !output.status.success(),
        "invalid TOML was accepted: {output:?}"
    );
    assert_eq!(std::fs::read_to_string(config_path).unwrap(), original);
    assert!(
        !rho_dir.join("hooks/mw-record.py").exists(),
        "legacy hook was written despite invalid config.toml: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid Rho config.toml"),
        "unexpected error: {output:?}"
    );
}

#[test]
fn revert_removes_installed_rho_integration() {
    let rho_dir = sandbox("revert");

    let install = mw_cmd()
        .args(["integrate", "rho"])
        .env("RHO_HOME", &rho_dir)
        .output()
        .expect("run Rho install");
    assert!(install.status.success(), "install failed: {install:?}");

    let revert = mw_cmd()
        .args(["integrate", "rho", "--revert"])
        .env("RHO_HOME", &rho_dir)
        .output()
        .expect("run Rho revert");
    assert!(revert.status.success(), "revert failed: {revert:?}");

    assert!(!rho_dir.join("hooks/mw-record.py").exists());
    assert!(!rho_dir.join("skills/memorywhale/SKILL.md").exists());
    assert!(!rho_dir.join("hooks.toml").exists());
    assert!(!rho_dir.join("config.toml").exists());
    assert!(
        String::from_utf8_lossy(&revert.stdout).contains("removed from Rho"),
        "missing revert message: {revert:?}"
    );
}

#[test]
fn revert_preserves_unrelated_rho_settings() {
    let rho_dir = sandbox("revert-preserve");
    std::fs::write(
        rho_dir.join("hooks.toml"),
        r#"version = 1

[[hook]]
id = "fmt-rust"
on = "after_tool_use"
command = ["./fmt"]
timeout = "5s"
"#,
    )
    .unwrap();
    std::fs::write(
        rho_dir.join("config.toml"),
        r#"[model]
provider = "openai"
"#,
    )
    .unwrap();

    let output = mw_cmd()
        .args(["integrate", "rho"])
        .env("RHO_HOME", &rho_dir)
        .output()
        .expect("run Rho install");
    assert!(output.status.success(), "install failed: {output:?}");

    let revert = mw_cmd()
        .args(["integrate", "rho", "--revert"])
        .env("RHO_HOME", &rho_dir)
        .output()
        .expect("run Rho revert");
    assert!(revert.status.success(), "revert failed: {revert:?}");

    let hooks = std::fs::read_to_string(rho_dir.join("hooks.toml")).unwrap();
    assert!(hooks.contains("id = \"fmt-rust\""));
    assert!(!hooks.contains("memorywhale-record"));
    let config = std::fs::read_to_string(rho_dir.join("config.toml")).unwrap();
    assert!(config.contains("provider = \"openai\""));
    assert!(!config.contains("memorywhale"));
}

#[test]
fn revert_leaves_invalid_rho_hooks_untouched() {
    let rho_dir = sandbox("revert-invalid");
    std::fs::create_dir_all(rho_dir.join("hooks")).unwrap();
    std::fs::write(rho_dir.join("hooks/mw-record.py"), "hook").unwrap();
    let original = "version = [";
    std::fs::write(rho_dir.join("hooks.toml"), original).unwrap();

    let output = mw_cmd()
        .args(["integrate", "rho", "--revert"])
        .env("RHO_HOME", &rho_dir)
        .output()
        .expect("run Rho revert");

    assert!(
        !output.status.success(),
        "invalid TOML revert was accepted: {output:?}"
    );
    assert_eq!(
        std::fs::read_to_string(rho_dir.join("hooks.toml")).unwrap(),
        original
    );
    assert!(rho_dir.join("hooks/mw-record.py").exists());
}

#[test]
fn revert_without_memorywhale_installed_is_a_noop_for_settings() {
    let rho_dir = sandbox("revert-noop");
    let original = "[model]\nprovider = \"openai\"\n";
    std::fs::write(rho_dir.join("config.toml"), original).unwrap();

    let output = mw_cmd()
        .args(["integrate", "rho", "--revert"])
        .env("RHO_HOME", &rho_dir)
        .output()
        .expect("run Rho revert");

    assert!(output.status.success(), "revert failed: {output:?}");
    assert_eq!(
        std::fs::read_to_string(rho_dir.join("config.toml")).unwrap(),
        original
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("mcp:      memorywhale unregistered"),
        "unexpected mcp message: {output:?}"
    );
}
