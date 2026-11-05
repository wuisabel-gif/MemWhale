//! The CLI surface of recall hints (#425): `mw hint` wording and the `Recall:`
//! line of `mw context --last-error`, against the real binaries and a
//! temporary store. The ranking logic has unit tests in `src/lib.rs`.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const ERR: &str = "error: linker `cc` not found";

fn store(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mw-recall-hints-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(bin: &str, dir: &Path, args: &[&str]) -> Output {
    let out = Command::new(bin)
        .env("MEMORYWHALE_DATA_DIR", dir)
        .args(args)
        .output()
        .expect("run binary");
    assert!(out.status.success(), "{bin} {args:?} failed: {out:?}");
    out
}

fn remember(dir: &Path, exit: &str, stderr: &str, argv: &[&str]) {
    let mut args = vec![
        "--cwd",
        "/work/api",
        "--exit-code",
        exit,
        "--stderr",
        stderr,
        "--",
    ];
    args.extend_from_slice(argv);
    run(env!("CARGO_BIN_EXE_mw-remember"), dir, &args);
}

fn fail(dir: &Path) {
    remember(dir, "101", ERR, &["cargo", "build"]);
}

/// The fix passes, then the failing command passes.
fn fix_and_rerun(dir: &Path) {
    remember(dir, "0", "", &["xcode-select", "--install"]);
    remember(dir, "0", "", &["cargo", "build"]);
}

fn hint(dir: &Path) -> String {
    let out = run(env!("CARGO_BIN_EXE_mw"), dir, &["hint", "--", ERR]);
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn an_error_without_a_fix_says_so() {
    let dir = store("none");
    fail(&dir);
    let hint = hint(&dir);
    assert!(hint.ends_with("no fix recorded yet"), "{hint}");
}

#[test]
fn one_episode_is_a_possible_fix() {
    let dir = store("once");
    fail(&dir);
    fix_and_rerun(&dir);
    let hint = hint(&dir);
    assert!(hint.contains("worked once"), "{hint}");
    assert!(hint.contains("run in a terminal"), "{hint}");
    assert!(
        hint.ends_with("possible fix: xcode-select --install"),
        "{hint}"
    );
}

#[test]
fn two_episodes_are_a_confirmed_fix_and_reach_context() {
    let dir = store("twice");
    for _ in 0..2 {
        fail(&dir);
        fix_and_rerun(&dir);
    }
    let hint = hint(&dir);
    assert!(hint.contains("worked 2 of 2 times"), "{hint}");
    assert!(
        hint.ends_with("confirmed fix: xcode-select --install"),
        "{hint}"
    );

    fail(&dir);
    let out = run(env!("CARGO_BIN_EXE_mw"), &dir, &["context", "--last-error"]);
    let text = String::from_utf8_lossy(&out.stdout);
    let recall = text
        .lines()
        .find(|l| l.starts_with("Recall:"))
        .unwrap_or_else(|| panic!("no Recall: line:\n{text}"));
    assert!(
        recall.contains("confirmed fix: xcode-select --install"),
        "{recall}"
    );
}

#[test]
fn a_hook_failure_and_a_terminal_fix_are_one_episode() {
    let dir = store("hook");
    let mut child = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .env("MEMORYWHALE_DATA_DIR", &dir)
        .args(["--from-hook", "claude"])
        .stdin(Stdio::piped())
        .spawn()
        .expect("spawn mw-remember --from-hook");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            br#"{"hook_event_name":"PostToolUse","tool_name":"Bash","cwd":"/work/api","tool_input":{"command":"cargo build"},"tool_response":{"stdout":"","stderr":"error: linker `cc` not found","exit_code":101}}"#,
        )
        .unwrap();
    assert!(child.wait().unwrap().success());
    fix_and_rerun(&dir);
    let hint = hint(&dir);
    assert!(hint.contains("seen once; worked once"), "{hint}");
    assert!(
        hint.ends_with("possible fix: xcode-select --install"),
        "{hint}"
    );
}
