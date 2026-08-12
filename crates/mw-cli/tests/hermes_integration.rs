use std::process::Command;

fn sandbox(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "mw-hermes-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn user_can_install_memorywhale_into_a_fresh_hermes_config() {
    let hermes_home = sandbox("fresh");

    let output = Command::new(env!("CARGO_BIN_EXE_mw"))
        .args(["integrate", "hermes"])
        .env("HERMES_HOME", &hermes_home)
        .output()
        .expect("run Hermes integration command");

    assert!(output.status.success(), "command failed: {output:?}");
    assert_eq!(
        std::fs::read_to_string(hermes_home.join("config.yaml")).unwrap(),
        "mcp_servers:\n  memorywhale:\n    command: \"mw-mcp\"\n"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Hermes Agent"),
        "missing success message: {output:?}"
    );
}

#[test]
fn installing_memorywhale_preserves_existing_hermes_settings_and_servers() {
    let hermes_home = sandbox("existing");
    let config_path = hermes_home.join("config.yaml");
    std::fs::write(
        &config_path,
        "model: kimi-k3\nmcp_servers:\n  filesystem:\n    command: \"npx\"\ntoolsets:\n  - hermes-cli\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_mw"))
        .args(["integrate", "hermes"])
        .env("HERMES_HOME", &hermes_home)
        .output()
        .expect("run Hermes integration command");

    assert!(output.status.success(), "command failed: {output:?}");
    let updated = std::fs::read_to_string(config_path).unwrap();
    assert!(updated.contains("model: kimi-k3\n"), "{updated}");
    assert!(
        updated.contains("  filesystem:\n    command: \"npx\"\n"),
        "{updated}"
    );
    assert!(
        updated.contains("  memorywhale:\n    command: \"mw-mcp\"\n"),
        "{updated}"
    );
    assert!(updated.contains("toolsets:\n  - hermes-cli\n"), "{updated}");
}

#[test]
fn installing_memorywhale_twice_does_not_duplicate_the_server() {
    let hermes_home = sandbox("idempotent");

    for _ in 0..2 {
        let output = Command::new(env!("CARGO_BIN_EXE_mw"))
            .args(["integrate", "hermes"])
            .env("HERMES_HOME", &hermes_home)
            .output()
            .expect("run Hermes integration command");
        assert!(output.status.success(), "command failed: {output:?}");
    }

    let updated = std::fs::read_to_string(hermes_home.join("config.yaml")).unwrap();
    assert_eq!(updated.matches("memorywhale:").count(), 1, "{updated}");
}

#[test]
fn invalid_hermes_yaml_is_left_untouched() {
    let hermes_home = sandbox("invalid");
    let config_path = hermes_home.join("config.yaml");
    let original = "mcp_servers: [\n";
    std::fs::write(&config_path, original).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_mw"))
        .args(["integrate", "hermes"])
        .env("HERMES_HOME", &hermes_home)
        .output()
        .expect("run Hermes integration command");

    assert!(
        !output.status.success(),
        "invalid YAML was accepted: {output:?}"
    );
    assert_eq!(std::fs::read_to_string(config_path).unwrap(), original);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid Hermes config"),
        "unexpected error: {output:?}"
    );
}
