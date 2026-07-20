//! A recorder child must finalize its in-progress session row as `interrupted`
//! the moment its parent dies — not wait for the dashboard's next-startup
//! recovery. These tests build a real 3-generation process tree (grandparent =
//! the test, middle = the parent we kill, recorder = the guarded child) so the
//! child's parent genuinely dies and the OS mechanism under test actually fires.
#![cfg(unix)]

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS sessions (id INTEGER PRIMARY KEY,
    shell TEXT, cwd TEXT, transcript_path TEXT NOT NULL DEFAULT '',
    transcript TEXT NOT NULL DEFAULT '', notes TEXT NOT NULL DEFAULT '',
    started_at TEXT NOT NULL DEFAULT '', ended_at TEXT NOT NULL DEFAULT '',
    byte_count INTEGER NOT NULL DEFAULT 0, status TEXT NOT NULL DEFAULT 'finished');";

/// Fresh temp data dir + a `recording` session row; returns (db_path, transcript, id).
fn setup(tag: &str) -> (PathBuf, PathBuf, i64) {
    let dir = std::env::temp_dir().join(format!("mw-pdeath-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("memorywhale.sqlite3");
    let transcript = dir.join("session.log");
    std::fs::write(&transcript, b"partial output before the crash\n").unwrap();

    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch(SCHEMA).unwrap();
    conn.execute(
        "INSERT INTO sessions (transcript_path, transcript, notes, started_at, ended_at, byte_count, status)
         VALUES (?1, '', '', 't', 't', 0, 'recording')",
        [transcript.to_str().unwrap()],
    )
    .unwrap();
    let id = conn.last_insert_rowid();
    (db_path, transcript, id)
}

/// Finalization the recorder runs on parent death — same effect as mw's real
/// `update_session_from_transcript(.., "interrupted")`: flip the row's status.
fn finalize_interrupted(db_path: &Path, id: i64) {
    let conn = Connection::open(db_path).unwrap();
    conn.execute("UPDATE sessions SET status = 'interrupted' WHERE id = ?1", [id])
        .unwrap();
}

fn poll_status(db_path: &Path, id: i64, want: &str, timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    let mut got = String::new();
    while Instant::now() < deadline {
        let conn = Connection::open(db_path).unwrap();
        got = conn
            .query_row("SELECT status FROM sessions WHERE id = ?1", [id], |r| r.get(0))
            .unwrap();
        if got == want {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    got
}

// safety: fork() in a test. The middle and recorder are single-threaded at their
// fork points (the second fork happens from the single-threaded middle before any
// thread spawns), so the guarded work (thread + SQLite) runs in a child forked
// from a single-threaded process.
unsafe fn fork() -> libc::pid_t {
    libc::fork()
}

/// Exercises the non-Linux pipe/EOF fallback: the middle holds the pipe's write
/// end; killing it closes that end; the recorder sees EOF and finalizes.
#[cfg(not(target_os = "linux"))]
#[test]
fn interrupted_on_parent_death_via_pipe_eof() {
    let (db_path, _transcript, id) = setup("pipe");

    let mut fds = [0 as libc::c_int; 2];
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0, "pipe() failed");
    let (read_fd, write_fd) = (fds[0], fds[1]);

    let middle = unsafe { fork() };
    assert!(middle >= 0, "first fork failed");
    if middle == 0 {
        // MIDDLE: single-threaded. Spawn the recorder, then hold the write end.
        let recorder = unsafe { fork() };
        if recorder == 0 {
            // RECORDER: keep the read end, drop the write end so only the middle
            // is a writer; when the middle dies we get EOF.
            unsafe { libc::close(write_fd) };
            std::env::set_var(memorywhale_cli::PDEATH_FD_ENV, read_fd.to_string());
            let dbp = db_path.clone();
            memorywhale_cli::guard_parent_death(move || finalize_interrupted(&dbp, id));
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        unsafe { libc::close(read_fd) };
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    // TEST (grandparent): drop both ends so the middle is the sole writer.
    unsafe {
        libc::close(read_fd);
        libc::close(write_fd);
    }
    std::thread::sleep(Duration::from_millis(300)); // let the recorder arm its watcher
    unsafe { libc::kill(middle, libc::SIGKILL) };
    let mut status = 0;
    unsafe { libc::waitpid(middle, &mut status, 0) };

    let got = poll_status(&db_path, id, "interrupted", Duration::from_secs(5));
    assert_eq!(got, "interrupted", "row must be finalized as interrupted after parent death (pipe EOF)");
    let _ = std::fs::remove_dir_all(db_path.parent().unwrap());
}

/// Exercises the Linux prctl(PR_SET_PDEATHSIG) path: the middle simply exits and
/// the kernel delivers SIGTERM to the recorder, which finalizes.
#[cfg(target_os = "linux")]
#[test]
fn interrupted_on_parent_death_via_prctl() {
    let (db_path, _transcript, id) = setup("prctl");

    let middle = unsafe { fork() };
    assert!(middle >= 0, "first fork failed");
    if middle == 0 {
        let recorder = unsafe { fork() };
        if recorder == 0 {
            // RECORDER: no pipe — rely on prctl PDEATHSIG (+ getppid race check).
            let dbp = db_path.clone();
            memorywhale_cli::guard_parent_death(move || finalize_interrupted(&dbp, id));
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        // MIDDLE: let the recorder arm prctl, then die so the kernel signals it.
        std::thread::sleep(Duration::from_millis(300));
        unsafe { libc::_exit(0) };
    }

    let got = poll_status(&db_path, id, "interrupted", Duration::from_secs(5));
    let mut status = 0;
    unsafe { libc::waitpid(middle, &mut status, 0) };
    assert_eq!(got, "interrupted", "row must be finalized as interrupted after parent death (prctl)");
    let _ = std::fs::remove_dir_all(db_path.parent().unwrap());
}
