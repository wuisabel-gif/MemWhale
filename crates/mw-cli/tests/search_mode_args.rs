use std::process::Command;

fn mw_search(args: &[&str], sandbox: &str) -> std::process::Output {
    let data_dir =
        std::env::temp_dir().join(format!("mw-search-mode-{sandbox}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data_dir);
    Command::new(env!("CARGO_BIN_EXE_mw"))
        .arg("search")
        .args(args)
        .env("MEMORYWHALE_DATA_DIR", &data_dir)
        .output()
        .expect("run mw search")
}

#[test]
fn bare_mode_flag_is_a_usage_error() {
    let out = mw_search(&["linker", "--mode"], "bare");
    assert!(!out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("--mode needs a value"));
}

#[test]
fn duplicate_mode_flags_are_rejected() {
    let out = mw_search(&["linker", "--mode", "lessons", "--mode=evidence"], "dup");
    assert!(!out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("more than once"));
}
