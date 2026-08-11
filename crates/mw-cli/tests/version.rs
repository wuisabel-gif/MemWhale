use std::process::Command;

fn assert_version_flag(flag: &str, sandbox_name: &str) {
    let data_dir =
        std::env::temp_dir().join(format!("mw-version-{sandbox_name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data_dir);

    let output = Command::new(env!("CARGO_BIN_EXE_mw"))
        .arg(flag)
        .env("MEMORYWHALE_DATA_DIR", &data_dir)
        .output()
        .expect("run mw version command");

    assert!(output.status.success(), "{flag} failed: {output:?}");
    let stdout = std::str::from_utf8(&output.stdout).expect("version output is UTF-8");
    assert_eq!(
        stdout.trim_end(),
        format!("mw {}", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(
        stdout.lines().count(),
        1,
        "unexpected version output: {stdout:?}"
    );
    assert!(output.stderr.is_empty(), "unexpected stderr: {output:?}");
    assert!(
        !data_dir.exists(),
        "{flag} created the MemoryWhale data directory"
    );
}

#[test]
fn long_version_flag_prints_version_without_initializing_database() {
    assert_version_flag("--version", "long");
}

#[test]
fn short_version_flag_prints_version_without_initializing_database() {
    assert_version_flag("-V", "short");
}
