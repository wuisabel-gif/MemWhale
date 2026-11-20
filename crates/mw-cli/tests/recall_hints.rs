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

#[test]
fn a_saved_lesson_leads_context_instead_of_no_fix() {
    let dir = store("lesson");
    fail(&dir);
    run(
        env!("CARGO_BIN_EXE_mw"),
        &dir,
        &[
            "remember",
            "linker cc not found -> run: xcode-select --install",
        ],
    );
    fail(&dir);
    let out = run(env!("CARGO_BIN_EXE_mw"), &dir, &["context", "--last-error"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains(
            "Saved lesson (just now): linker cc not found -> run: xcode-select --install"
        ),
        "{text}"
    );
    assert!(!text.contains("no fix recorded yet"), "{text}");
    assert!(!text.contains("No recorded fix yet"), "{text}");
}

#[test]
fn a_saved_lesson_ranks_above_the_failures_it_explains() {
    let dir = store("rank");
    fail(&dir);
    run(
        env!("CARGO_BIN_EXE_mw"),
        &dir,
        &[
            "remember",
            "linker cc not found -> run: xcode-select --install",
        ],
    );
    fail(&dir);
    let out = run(
        env!("CARGO_BIN_EXE_mw"),
        &dir,
        &["search", "linker", "cc", "not", "found"],
    );
    let text = String::from_utf8_lossy(&out.stdout);
    let first = text
        .lines()
        .find(|l| l.contains('%'))
        .unwrap_or_else(|| panic!("no hits:\n{text}"));
    assert!(first.contains("[note"), "lesson should lead:\n{text}");
}

#[test]
fn a_pending_lesson_is_reviewed_from_the_terminal() {
    let dir = store("review");
    fail(&dir);
    let db = rusqlite::Connection::open(dir.join("memorywhale.sqlite3")).unwrap();
    db.execute(
        "INSERT INTO bookmarks (label, created_at, author_kind, author_name, approved)
         VALUES ('linker cc not found -> run: xcode-select --install',
                 '2026-01-01T00:00:00Z', 'agent', 'claude-code', 0)",
        [],
    )
    .unwrap();
    let mw = env!("CARGO_BIN_EXE_mw");
    let text = |args: &[&str]| String::from_utf8_lossy(&run(mw, &dir, args).stdout).into_owned();

    // Unapproved: listed for review, but not shown as a saved lesson.
    let pending = text(&["memory", "pending"]);
    assert!(
        pending.contains("#1") && pending.contains("claude-code"),
        "{pending}"
    );
    assert!(!text(&["context", "--last-error"]).contains("Saved lesson"));
    assert!(text(&["doctor"]).contains("1 waiting for review"));

    assert!(text(&["memory", "approve", "1"]).contains("approved"));
    assert!(text(&["context", "--last-error"]).contains("Saved lesson"));
    assert!(text(&["memory", "pending"]).contains("no lessons are waiting"));

    // Approve and reject touch only the review queue.
    let out = Command::new(mw)
        .env("MEMORYWHALE_DATA_DIR", &dir)
        .args(["memory", "reject", "1"])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "an approved lesson must not be deletable by reject"
    );
}

#[test]
fn a_lesson_saved_from_claude_codes_shell_names_the_agent_and_stays_live() {
    let dir = store("author");
    fail(&dir);
    let out = Command::new(env!("CARGO_BIN_EXE_mw"))
        .env("MEMORYWHALE_DATA_DIR", &dir)
        .env("CLAUDECODE", "1")
        .args([
            "remember",
            "linker cc not found -> run: xcode-select --install",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let db = rusqlite::Connection::open(dir.join("memorywhale.sqlite3")).unwrap();
    let row: (String, Option<String>, i64) = db
        .query_row(
            "SELECT author_kind, author_name, approved FROM bookmarks",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(row, ("agent".into(), Some("claude-code".into()), 1));
}

/// What the Claude Code pre-run hook prints for `command`.
fn before(dir: &Path, command: &str) -> String {
    before_as("claude", dir, command)
}

/// What `agent`'s pre-run hook prints for `command`.
fn before_as(agent: &str, dir: &Path, command: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .env("MEMORYWHALE_DATA_DIR", dir)
        .args(["--before-hook", agent])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn mw-remember --before-hook");
    let payload = serde_json::json!({
        "hook_event_name": "PreToolUse", "tool_name": "Bash", "cwd": "/somewhere/else",
        "tool_input": { "command": command }
    });
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn a_lesson_naming_command_text_stops_that_command_once() {
    let dir = store("stop");
    fail(&dir);
    run(
        env!("CARGO_BIN_EXE_mw"),
        &dir,
        &[
            "remember",
            "when `echo ===`: zsh reads =word as a path lookup; quote the separator",
        ],
    );
    // Never run here before, in a directory with no history: stopped anyway.
    let stopped = before(&dir, "ls src; echo ====; ls tests");
    assert!(
        stopped.contains(r#""permissionDecision":"deny""#),
        "{stopped}"
    );
    assert!(stopped.contains("quote the separator"), "{stopped}");
    // Sent again unchanged, it runs.
    assert_eq!(before(&dir, "ls src; echo ====; ls tests"), "");
    // Quoted, or only inside a heredoc body, it was never a match.
    assert_eq!(before(&dir, "ls src; echo '===='; ls tests"), "");
    assert_eq!(before(&dir, "cat <<'EOF'\necho ====\nEOF"), "");

    // Codex takes the same denial, for a command it has not been stopped on.
    let codex = before_as("codex", &dir, "echo ====; pwd");
    assert!(codex.contains(r#""permissionDecision":"deny""#), "{codex}");

    let doctor = run(env!("CARGO_BIN_EXE_mw"), &dir, &["doctor"]);
    let doctor = String::from_utf8_lossy(&doctor.stdout);
    assert!(doctor.contains("2 stopped before running"), "{doctor}");
}
