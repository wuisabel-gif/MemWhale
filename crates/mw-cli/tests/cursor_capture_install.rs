#![cfg(unix)]
// Also compile adapters independently of public CLI dispatch.
#[path = "../src/integrate/cursor_capture.rs"]
mod cursor_capture;
#[path = "../src/integrate/mcp_client.rs"]
#[allow(dead_code)]
mod mcp_client;

#[test]
fn rejects_invalid_capture_options() {
    assert!(cursor_capture::cli(&["--unknown".into()]).is_err());
}

use serde_json::{json, Value};
use std::{
    fs,
    os::unix::fs::{symlink, PermissionsExt},
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "mw-cursor-install-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&p).unwrap();
        Self(fs::canonicalize(p).unwrap())
    }
    fn command(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_mw"));
        c.args(["integrate", "cursor", "--capture"])
            .env("HOME", &self.0)
            .env("RHO_HOME", self.0.join("rho"))
            .env("MEMORYWHALE_DATA_DIR", self.0.join("data"))
            .env("PATH", "")
            .current_dir(&self.0);
        c
    }
    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }
    fn path(&self) -> PathBuf {
        self.0.join(".cursor/hooks.json")
    }
    fn put(&self, bytes: &[u8]) {
        fs::create_dir_all(self.path().parent().unwrap()).unwrap();
        fs::write(self.path(), bytes).unwrap();
    }
    fn doc(&self) -> Value {
        serde_json::from_slice(&fs::read(self.path()).unwrap()).unwrap()
    }
    fn owner(&self) -> PathBuf {
        self.0
            .join(".cursor/hooks.json.memorywhale-cursor-capture-owner.json")
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn ok(o: Output) {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}
fn bad(o: Output) {
    assert!(
        !o.status.success(),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
}

#[test]
fn merge_check_idempotent_revert_preserve_and_private_journal() {
    let s = Sandbox::new();
    let original = json!({"version":1,"private":"DO-NOT-COPY","extra":{"approval":"unchanged"},"hooks":{
        "postToolUse":[{"command":"other-hook","matcher":"Read"}],
        "afterShellExecution":[{"command":"/workspace/MemoryWhale/scripts/lint.sh"}], "futureEvent":[]}});
    s.put(&serde_json::to_vec(&original).unwrap());
    ok(s.run(&[]));
    let doc = s.doc();
    assert_eq!(doc["private"], original["private"]);
    assert_eq!(doc["extra"], original["extra"]);
    assert_eq!(
        doc["hooks"]["afterShellExecution"],
        original["hooks"]["afterShellExecution"]
    );
    assert_eq!(doc["hooks"]["postToolUse"].as_array().unwrap().len(), 2);
    for event in ["postToolUse", "postToolUseFailure"] {
        let entry = doc["hooks"][event].as_array().unwrap().last().unwrap();
        assert_eq!(entry["matcher"], "Shell");
        assert_eq!(entry["timeout"], 5);
        assert_eq!(entry["type"], "command");
        assert!(entry["command"]
            .as_str()
            .unwrap()
            .ends_with(" --from-hook cursor"));
    }
    let before = fs::read(s.path()).unwrap();
    ok(s.run(&[]));
    ok(s.run(&["--check"]));
    assert_eq!(before, fs::read(s.path()).unwrap());
    assert!(!fs::read_to_string(s.owner())
        .unwrap()
        .contains("DO-NOT-COPY"));
    assert_eq!(
        fs::metadata(s.owner()).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let mut changed = s.doc();
    changed["hooks"]["newEvent"] = json!([{"command":"new"}]);
    s.put(&serde_json::to_vec(&changed).unwrap());
    ok(s.run(&["--revert"]));
    ok(s.run(&["--revert"]));
    assert_eq!(
        s.doc()["hooks"]["postToolUse"],
        original["hooks"]["postToolUse"]
    );
    assert_eq!(s.doc()["hooks"]["newEvent"], changed["hooks"]["newEvent"]);
    assert!(!s.owner().exists());
    bad(s.run(&["--check"]));
}

#[test]
fn dry_run_custom_and_invalid_options_write_nothing() {
    let s = Sandbox::new();
    ok(s.run(&["--dry-run"]));
    assert!(!s.path().exists());
    assert!(!s.path().parent().unwrap().exists());
    bad(s.run(&["--check"]));
    bad(s.run(&["--check", "--revert"]));
    bad(s.run(&["--hooks-file"]));
    ok(s.run(&["--hooks-file", "custom/hooks.json", "--dry-run"]));
    assert!(!s.0.join("custom").exists());
    let o = s.run(&["--hooks-file", "custom/hooks.json"]);
    assert!(String::from_utf8_lossy(&o.stdout).contains("not automatically registered"));
    ok(o);
    assert!(!s.path().exists());
    ok(s.run(&["--hooks-file", "custom/hooks.json", "--check"]));
    ok(s.run(&["--hooks-file", "custom/hooks.json", "--revert"]));
}

#[test]
fn malformed_duplicate_unsupported_and_oversize_preserve_source() {
    let s = Sandbox::new();
    for bytes in [
        b"{\"version\":2,\"hooks\":{}}".to_vec(),
        b"{\"version\":1,\"version\":1}".to_vec(),
        b"{\"version\":1,\"hooks\":{\"postToolUse\":[],\"postToolUse\":[]}}".to_vec(),
        b"{\"version\":1,\"hooks\":[]}".to_vec(),
        b"{\"version\":1,\"hooks\":{\"postToolUse\":false}}".to_vec(),
        b"{secret-malformed".to_vec(),
        vec![b' '; 1024 * 1024 + 1],
    ] {
        s.put(&bytes);
        let o = s.run(&[]);
        assert!(!String::from_utf8_lossy(&o.stderr).contains("secret-malformed"));
        bad(o);
        assert_eq!(fs::read(s.path()).unwrap(), bytes);
        assert!(!s.owner().exists());
    }
}

#[test]
fn conflicts_and_modified_owned_entries_do_not_write() {
    let s = Sandbox::new();
    s.put(br#"{"version":1,"hooks":{"afterShellExecution":[{"command":"mw-remember --from-hook cursor"}]}}"#);
    let before = fs::read(s.path()).unwrap();
    bad(s.run(&[]));
    assert_eq!(before, fs::read(s.path()).unwrap());
    s.put(br#"{"version":1,"hooks":{}}"#);
    ok(s.run(&[]));
    let mut doc = s.doc();
    doc["hooks"]["postToolUse"][0]["timeout"] = json!(9);
    s.put(&serde_json::to_vec(&doc).unwrap());
    let before = fs::read(s.path()).unwrap();
    let owner = fs::read(s.owner()).unwrap();
    for args in [&[][..], &["--check"][..], &["--revert"][..]] {
        bad(s.run(args));
    }
    assert_eq!(before, fs::read(s.path()).unwrap());
    assert_eq!(owner, fs::read(s.owner()).unwrap());
}

#[test]
fn relative_data_directory_and_hostile_paths_are_quoted_and_stale_requires_revert() {
    let s = Sandbox::new();
    let data = "store ' $(touch SHOULD_NOT_EXIST); with spaces";
    ok(s.command()
        .env("MEMORYWHALE_DATA_DIR", data)
        .output()
        .unwrap());
    let command = s.doc()["hooks"]["postToolUse"][0]["command"]
        .as_str()
        .unwrap()
        .to_string();
    let anchored = s.0.join(data).to_str().unwrap().replace('\'', "'\"'\"'");
    assert!(command.starts_with(&format!("MEMORYWHALE_DATA_DIR='{anchored}' ")));
    assert!(!s.0.join("SHOULD_NOT_EXIST").exists());
    bad(s.run(&[]));
    bad(s.run(&["--check"]));
    ok(s.run(&["--revert"]));
    ok(s.command()
        .env_remove("MEMORYWHALE_DATA_DIR")
        .output()
        .unwrap());
    assert!(!s.doc()["hooks"]["postToolUse"][0]["command"]
        .as_str()
        .unwrap()
        .contains("MEMORYWHALE_DATA_DIR"));
}

#[test]
fn missing_executable_and_nonregular_or_symlink_paths_are_rejected() {
    let s = Sandbox::new();
    let isolated = s.0.join("mw");
    fs::copy(env!("CARGO_BIN_EXE_mw"), &isolated).unwrap();
    // Copying binaries on macOS requires ad-hoc re-signing; use a hard link instead.
    #[cfg(target_os = "macos")]
    {
        fs::remove_file(&isolated).unwrap();
        fs::hard_link(env!("CARGO_BIN_EXE_mw"), &isolated).unwrap();
    }
    let mut c = Command::new(&isolated);
    bad(c
        .args(["integrate", "cursor", "--capture"])
        .env("HOME", &s.0)
        .env("PATH", "")
        .env("MEMORYWHALE_DATA_DIR", s.0.join("data"))
        .current_dir(&s.0)
        .output()
        .unwrap());
    assert!(!s.path().exists());
    fs::create_dir_all(s.path().parent().unwrap()).unwrap();
    let target = s.0.join("untouched");
    fs::write(&target, b"private").unwrap();
    symlink(&target, s.path()).unwrap();
    bad(s.run(&[]));
    fs::remove_file(s.path()).unwrap();
    symlink(&target, s.owner()).unwrap();
    bad(s.run(&[]));
    fs::remove_file(s.owner()).unwrap();
    let lock =
        s.0.join(".cursor/hooks.json.memorywhale-cursor-capture.lock");
    symlink(&target, &lock).unwrap();
    bad(s.run(&[]));
    fs::remove_file(lock).unwrap();
    fs::create_dir(s.path()).unwrap();
    bad(s.run(&[]));
    fs::remove_dir(s.path()).unwrap();
    symlink(s.path().parent().unwrap(), s.0.join("alias")).unwrap();
    bad(s.run(&["--hooks-file", "alias/hooks.json"]));
    assert_eq!(fs::read(target).unwrap(), b"private");
}

#[test]
fn quoted_binary_paths_are_not_executed_and_stale_binary_is_detected() {
    let s = Sandbox::new();
    let bin = s.0.join("bin ' $(touch NEVER_EXECUTED)");
    fs::create_dir(&bin).unwrap();
    let mw = bin.join("mw");
    fs::hard_link(env!("CARGO_BIN_EXE_mw"), &mw).unwrap();
    let remember = bin.join("mw-remember");
    fs::write(&remember, b"#!/bin/sh\ntouch NEVER_EXECUTED\nexit 99\n").unwrap();
    fs::set_permissions(&remember, fs::Permissions::from_mode(0o700)).unwrap();
    let run = |args: &[&str]| {
        Command::new(&mw)
            .args(["integrate", "cursor", "--capture"])
            .args(args)
            .env("HOME", &s.0)
            .env("PATH", "")
            .env("MEMORYWHALE_DATA_DIR", s.0.join("data"))
            .current_dir(&s.0)
            .output()
            .unwrap()
    };
    ok(run(&[]));
    ok(run(&["--check"]));
    let command = s.doc()["hooks"]["postToolUse"][0]["command"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(command.contains(&format!(
        "'{}' --from-hook cursor",
        remember.to_str().unwrap().replace('\'', "'\"'\"'")
    )));
    assert!(!s.0.join("NEVER_EXECUTED").exists());
    fs::set_permissions(&remember, fs::Permissions::from_mode(0o600)).unwrap();
    let before = fs::read(s.path()).unwrap();
    bad(run(&["--check"]));
    bad(run(&[]));
    assert_eq!(before, fs::read(s.path()).unwrap());
    ok(run(&["--revert"]));
}

#[test]
fn installed_shell_command_records_into_the_quoted_store_without_executing_payload() {
    use std::io::Write;
    use std::process::Stdio;
    let s = Sandbox::new();
    let data = "store ' $(touch CONFIG_SHOULD_NOT_RUN)";
    ok(s.command()
        .env("MEMORYWHALE_DATA_DIR", data)
        .output()
        .unwrap());
    let command = s.doc()["hooks"]["postToolUse"][0]["command"]
        .as_str()
        .unwrap()
        .to_owned();
    // Execute the installed hook command, never the command described by its input.
    let mut child = Command::new("/bin/sh")
        .args(["-c", &command])
        .env_clear()
        .env("HOME", &s.0)
        .env("PATH", "")
        .current_dir(&s.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let event = json!({"hook_event_name":"postToolUse","tool_name":"Shell", "cwd":s.0,
        "tool_input":{"command":"touch RECORDED_COMMAND_SHOULD_NOT_RUN"},
        "tool_output":json!({"exitCode":0,"stdout":"synthetic installed-hook fixture"}).to_string()});
    child
        .stdin
        .take()
        .unwrap()
        .write_all(event.to_string().as_bytes())
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(result.stdout.is_empty());
    ok(result);
    assert!(!s.0.join("CONFIG_SHOULD_NOT_RUN").exists());
    assert!(!s.0.join("RECORDED_COMMAND_SHOULD_NOT_RUN").exists());
    let conn = rusqlite::Connection::open(s.0.join(data).join("memorywhale.sqlite3")).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM command_runs WHERE agent = 'cursor' AND exit_code = 0",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn duplicate_owned_entries_and_pending_or_ambiguous_journals_fail_closed() {
    let s = Sandbox::new();
    ok(s.run(&[]));
    let original = fs::read(s.path()).unwrap();
    let owner = fs::read(s.owner()).unwrap();
    let mut doc = s.doc();
    let entry = doc["hooks"]["postToolUse"][0].clone();
    doc["hooks"]["postToolUse"]
        .as_array_mut()
        .unwrap()
        .push(entry);
    s.put(&serde_json::to_vec(&doc).unwrap());
    let duplicate = fs::read(s.path()).unwrap();
    bad(s.run(&["--revert"]));
    assert_eq!(duplicate, fs::read(s.path()).unwrap());
    s.put(&original);
    let mut pending: Value = serde_json::from_slice(&owner).unwrap();
    pending["state"] = json!("pending");
    fs::write(s.owner(), serde_json::to_vec(&pending).unwrap()).unwrap();
    bad(s.run(&[]));
    bad(s.run(&["--revert"]));
    assert_eq!(original, fs::read(s.path()).unwrap());
    let ambiguous = String::from_utf8(owner).unwrap().replacen(
        "\"version\":1",
        "\"version\":1,\"version\":1",
        1,
    );
    fs::write(s.owner(), ambiguous.as_bytes()).unwrap();
    bad(s.run(&["--check"]));
    assert_eq!(original, fs::read(s.path()).unwrap());
}
