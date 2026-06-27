// mw: automatic whole-session terminal recorder for MemoryWhale.
//
// Starts your $SHELL inside a recorded subshell (via the system `script` tool),
// captures every command and all output until you `exit`, then stores the
// session in the same local SQLite DB as mw-remember:
//   <data_local>/MemoryWhale/memorywhale.sqlite3   (sessions table + cleaned transcript)
//   <data_local>/MemoryWhale/sessions/             (raw transcript files)
//
// Everything is stored locally and never uploaded.
//
// Usage:
//   mw                                  # record a session, exit the subshell to stop
//   mw --notes "debugging the Jetson build"

use chrono::Utc;
use regex::Regex;
use rusqlite::{params, Connection};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

const LIVE_SYNC_INTERVAL_SECS: u64 = 2;

fn main() {
    if let Err(err) = run() {
        eprintln!("mw: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    match raw_args.first().map(String::as_str) {
        Some("show") => return show_session(&raw_args[1..]),
        Some("list") => return list_sessions(),
        Some("global") => return global_cmd(&raw_args[1..]),
        Some("--help") | Some("-h") => {
            print_help();
            return Ok(());
        }
        _ => {}
    }

    let mut notes = String::new();
    let mut live = false;
    let mut iter = raw_args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--notes" => notes = iter.next().unwrap_or_default(),
            "--live" | "--autosave" => live = true,
            value if value.starts_with("--") => {
                return Err(format!("unknown option {value:?}; run mw --help"));
            }
            value => return Err(format!("unexpected argument {value:?}; run mw --help")),
        }
    }
    record_session(notes, live)
}

fn record_session(notes: String, live: bool) -> Result<(), String> {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let cwd = env::current_dir()
        .ok()
        .and_then(|path| path.to_str().map(ToOwned::to_owned));

    let started_at = Utc::now().to_rfc3339();
    let sessions_dir = sessions_dir()?;
    fs::create_dir_all(&sessions_dir)
        .map_err(|err| format!("failed to create sessions dir: {err}"))?;
    let transcript_path =
        sessions_dir.join(format!("session-{}.log", started_at.replace(':', "-")));
    let transcript_str = transcript_path
        .to_str()
        .ok_or_else(|| "transcript path is not valid UTF-8".to_string())?
        .to_string();

    eprintln!("mw: recording session to {transcript_str}");
    if live {
        eprintln!(
            "mw: live autosave is on; the dashboard/SQLite row updates every {LIVE_SYNC_INTERVAL_SECS}s."
        );
    }
    eprintln!("mw: type `exit` (or Ctrl-D) to stop recording.\n");

    let live_session = if live {
        let id = insert_live_session(&SessionDraft {
            shell: &shell,
            cwd: cwd.as_deref(),
            transcript_path: &transcript_str,
            notes: &notes,
            started_at: &started_at,
        })?;
        let sync = start_live_sync(id, transcript_path.clone());
        Some((id, sync))
    } else {
        None
    };

    // `script -q <file>` runs $SHELL interactively and records all I/O to <file>
    // on both macOS (BSD script) and Linux (util-linux script). MW_RECORDING is
    // set so the recorded shell's global-recording hook sees the guard and does
    // not start a nested recording, however this session was launched.
    let mut script = Command::new("script");
    script.arg("-q");
    if live && env::consts::OS == "linux" {
        script.arg("-f");
    }
    let status = script
        .arg(&transcript_path)
        .env("MW_RECORDING", "1")
        .status()
        .map_err(|err| format!("failed to launch `script` (is it installed?): {err}"))?;

    let ended_at = Utc::now().to_rfc3339();
    let live_session = if let Some((id, sync)) = live_session {
        sync.stop.store(true, Ordering::SeqCst);
        let _ = sync.handle.join();
        Some(id)
    } else {
        None
    };

    if !transcript_path.exists() {
        return Err("recording produced no transcript (session not saved)".to_string());
    }
    let (id, byte_count) = if let Some(id) = live_session {
        let byte_count = update_session_from_transcript(id, &transcript_path, &ended_at)?;
        (id, byte_count)
    } else {
        insert_finished_session(
            &SessionDraft {
                shell: &shell,
                cwd: cwd.as_deref(),
                transcript_path: &transcript_str,
                notes: &notes,
                started_at: &started_at,
            },
            &transcript_path,
            &ended_at,
        )?
    };

    let exit_note = match status.code() {
        Some(code) => format!("shell exited with code {code}"),
        None => "shell terminated by signal".to_string(),
    };
    eprintln!("\nmw: recorded session #{id} ({byte_count} bytes, {exit_note}) -> {transcript_str}");
    Ok(())
}

fn print_help() {
    println!(
        "mw [--notes <text>]      record a whole shell session until you exit\n\
         mw --live [--notes <text>]  autosave the session to SQLite while it is still running\n\
         mw list                  list recorded sessions\n\
         mw show <id>             print the full faithful transcript of a session\n\
         mw global on|off|status  auto-record every new terminal by wiring a shell startup hook\n\
         \n\
         Records every command + output, stored locally and never uploaded.\n\
         Raw transcript: <data_local>/MemoryWhale/sessions/\n\
         Metadata + cleaned transcript: <data_local>/MemoryWhale/memorywhale.sqlite3 (sessions table)"
    );
}

struct SessionDraft<'a> {
    shell: &'a str,
    cwd: Option<&'a str>,
    transcript_path: &'a str,
    notes: &'a str,
    started_at: &'a str,
}

struct LiveSync {
    stop: Arc<AtomicBool>,
    handle: thread::JoinHandle<()>,
}

fn insert_live_session(draft: &SessionDraft<'_>) -> Result<i64, String> {
    let conn = open_session_db()?;
    conn.execute(
        "
        INSERT INTO sessions
            (shell, cwd, transcript_path, transcript, notes, started_at, ended_at, byte_count)
        VALUES (?1, ?2, ?3, '', ?4, ?5, ?5, 0)
        ",
        params![
            draft.shell,
            draft.cwd,
            draft.transcript_path,
            draft.notes,
            draft.started_at
        ],
    )
    .map_err(|err| format!("failed to create live session row: {err}"))?;
    Ok(conn.last_insert_rowid())
}

fn insert_finished_session(
    draft: &SessionDraft<'_>,
    transcript_path: &PathBuf,
    ended_at: &str,
) -> Result<(i64, i64), String> {
    let raw =
        fs::read(transcript_path).map_err(|err| format!("failed to read transcript: {err}"))?;
    let byte_count = raw.len() as i64;
    let cleaned = clean_transcript(&String::from_utf8_lossy(&raw));
    let conn = open_session_db()?;
    conn.execute(
        "
        INSERT INTO sessions
            (shell, cwd, transcript_path, transcript, notes, started_at, ended_at, byte_count)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ",
        params![
            draft.shell,
            draft.cwd,
            draft.transcript_path,
            cleaned,
            draft.notes,
            draft.started_at,
            ended_at,
            byte_count
        ],
    )
    .map_err(|err| format!("failed to insert session: {err}"))?;
    Ok((conn.last_insert_rowid(), byte_count))
}

fn start_live_sync(id: i64, transcript_path: PathBuf) -> LiveSync {
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let handle = thread::spawn(move || {
        while !thread_stop.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(LIVE_SYNC_INTERVAL_SECS));
            if thread_stop.load(Ordering::SeqCst) {
                break;
            }
            let ended_at = Utc::now().to_rfc3339();
            let _ = update_session_from_transcript(id, &transcript_path, &ended_at);
        }
    });
    LiveSync { stop, handle }
}

fn update_session_from_transcript(
    id: i64,
    transcript_path: &PathBuf,
    ended_at: &str,
) -> Result<i64, String> {
    let raw = match fs::read(transcript_path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(format!("failed to read transcript: {err}")),
    };
    let byte_count = raw.len() as i64;
    let cleaned = clean_transcript(&String::from_utf8_lossy(&raw));
    let conn = open_session_db()?;
    conn.execute(
        "
        UPDATE sessions
        SET transcript = ?1, ended_at = ?2, byte_count = ?3
        WHERE id = ?4
        ",
        params![cleaned, ended_at, byte_count, id],
    )
    .map_err(|err| format!("failed to autosave session: {err}"))?;
    Ok(byte_count)
}

fn open_session_db() -> Result<Connection, String> {
    let db_path = database_path()?;
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("failed to create data dir: {err}"))?;
    }
    let conn = Connection::open(db_path).map_err(|err| format!("failed to open db: {err}"))?;
    init_schema(&conn)?;
    Ok(conn)
}

fn global_cmd(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("on") => global_on(),
        Some("off") => global_off(),
        Some("status") | None => global_status(),
        Some(other) => Err(format!(
            "unknown subcommand {other:?}; usage: mw global [on|off|status]"
        )),
    }
}

/// Shell startup file to wire the hook into, chosen from $SHELL (zsh vs bash).
fn shell_rc_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "could not resolve home directory".to_string())?;
    let shell = env::var("SHELL").unwrap_or_default();
    let is_zsh = shell
        .rsplit('/')
        .next()
        .map_or(false, |name| name.contains("zsh"));
    Ok(home.join(if is_zsh { ".zshrc" } else { ".bashrc" }))
}

fn global_hook_path() -> Result<PathBuf, String> {
    Ok(memorywhale_dir()?.join("global-hook.sh"))
}

fn global_enabled_path() -> Result<PathBuf, String> {
    Ok(memorywhale_dir()?.join("global-enabled"))
}

const RC_MARKER: &str = "# memorywhale-global";

/// The POSIX-sh hook sourced by every interactive shell. It records only when
/// the enabled flag exists, `mw` is on PATH, the shell is interactive, and it
/// isn't already inside a recording (MW_RECORDING guard prevents any loop).
fn hook_contents(enabled_path: &str) -> String {
    format!(
        "# MemoryWhale global recording hook (managed by `mw global` — do not edit)\n\
         if [ -z \"$MW_RECORDING\" ] && [ -f \"{enabled_path}\" ] && command -v mw >/dev/null 2>&1 && case $- in *i*) true;; *) false;; esac && [ -t 0 ]; then\n\
         \x20   export MW_RECORDING=1\n\
         \x20   exec mw --notes \"auto session ($(basename \"$PWD\"))\"\n\
         fi\n"
    )
}

fn global_on() -> Result<(), String> {
    let hook_path = global_hook_path()?;
    let enabled_path = global_enabled_path()?;
    let rc_path = shell_rc_path()?;

    if let Some(parent) = hook_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("failed to create data dir: {err}"))?;
    }
    let hook_str = hook_path
        .to_str()
        .ok_or_else(|| "hook path is not valid UTF-8".to_string())?;
    let enabled_str = enabled_path
        .to_str()
        .ok_or_else(|| "enabled-flag path is not valid UTF-8".to_string())?;

    fs::write(&hook_path, hook_contents(enabled_str))
        .map_err(|err| format!("failed to write hook: {err}"))?;

    let existing = fs::read_to_string(&rc_path).unwrap_or_default();
    let already_wired = existing.contains(RC_MARKER);
    if !already_wired {
        use std::io::Write;
        let line = format!("\n[ -f \"{hook_str}\" ] && . \"{hook_str}\"  {RC_MARKER}\n");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rc_path)
            .map_err(|err| format!("failed to open {}: {err}", rc_path.display()))?;
        file.write_all(line.as_bytes())
            .map_err(|err| format!("failed to update {}: {err}", rc_path.display()))?;
    }

    fs::write(&enabled_path, "enabled\n")
        .map_err(|err| format!("failed to write enabled flag: {err}"))?;

    println!("mw: global recording ENABLED.");
    if !already_wired {
        println!("  wired into: {}", rc_path.display());
    } else {
        println!("  already wired into: {} (re-enabled)", rc_path.display());
    }
    println!("  hook: {hook_str}");
    println!(
        "  Open a NEW terminal (or run `source {}`) to start auto-recording.",
        rc_path.display()
    );
    Ok(())
}

fn global_off() -> Result<(), String> {
    let enabled_path = global_enabled_path()?;
    if enabled_path.exists() {
        fs::remove_file(&enabled_path)
            .map_err(|err| format!("failed to remove enabled flag: {err}"))?;
    }
    println!("mw: global recording DISABLED. New terminals will not auto-record.");
    println!("  (Any already-open recording sessions continue until you exit.)");
    println!("  Re-enable anytime with: mw global on");
    Ok(())
}

fn global_status() -> Result<(), String> {
    let enabled = global_enabled_path()?.exists();
    let hook_path = global_hook_path()?;
    let rc_path = shell_rc_path()?;
    let wired = fs::read_to_string(&rc_path)
        .unwrap_or_default()
        .contains(RC_MARKER);

    println!("global recording: {}", if enabled { "ON" } else { "OFF" });
    println!(
        "wired into {}: {}",
        rc_path.display(),
        if wired { "yes" } else { "no" }
    );
    println!(
        "hook file: {}",
        if hook_path.exists() {
            hook_path.display().to_string()
        } else {
            "(not installed yet)".to_string()
        }
    );
    if !wired {
        println!("run `mw global on` to set it up.");
    }
    Ok(())
}

fn show_session(args: &[String]) -> Result<(), String> {
    let id: i64 = match args.first() {
        Some(value) => value
            .parse()
            .map_err(|_| format!("invalid session id {value:?}; usage: mw show <id>"))?,
        None => return Err("usage: mw show <id>".to_string()),
    };

    let conn =
        Connection::open(database_path()?).map_err(|err| format!("failed to open db: {err}"))?;
    init_schema(&conn)?;

    let row = conn.query_row(
        "SELECT started_at, cwd, notes, transcript FROM sessions WHERE id = ?1",
        params![id],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        },
    );

    match row {
        Ok((started_at, cwd, notes, transcript)) => {
            println!("=== session #{id} ===");
            println!("started: {started_at}");
            if let Some(cwd) = cwd {
                println!("cwd:     {cwd}");
            }
            if !notes.is_empty() {
                println!("notes:   {notes}");
            }
            println!("----------------------------------------");
            print!("{transcript}");
            if !transcript.ends_with('\n') {
                println!();
            }
            Ok(())
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(format!(
            "no session #{id}; run `mw list` to see recorded sessions"
        )),
        Err(err) => Err(format!("failed to read session: {err}")),
    }
}

fn list_sessions() -> Result<(), String> {
    let conn =
        Connection::open(database_path()?).map_err(|err| format!("failed to open db: {err}"))?;
    init_schema(&conn)?;

    let mut stmt = conn
        .prepare("SELECT id, started_at, byte_count, notes FROM sessions ORDER BY id")
        .map_err(|err| format!("failed to query sessions: {err}"))?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(|err| format!("failed to read sessions: {err}"))?;

    let mut count = 0;
    for row in rows {
        let (id, started_at, byte_count, notes) = row.map_err(|err| format!("row error: {err}"))?;
        println!("#{id}\t{started_at}\t{byte_count} bytes\t{notes}");
        count += 1;
    }
    if count == 0 {
        println!("no sessions recorded yet; run `mw` to record one");
    }
    Ok(())
}

/// Strip terminal escape sequences and control characters so the stored
/// transcript is searchable plain text. The raw file is kept on disk untouched.
fn clean_transcript(input: &str) -> String {
    // OSC sequences: ESC ] ... BEL  (or ESC \)
    let osc = Regex::new(r"\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)").unwrap();
    // CSI / other ESC-introduced sequences.
    let csi = Regex::new(r"\x1b[@-Z\\-_]|\x1b\[[0-?]*[ -/]*[@-~]").unwrap();
    // Carriage returns (script logs are full of them) and stray control chars.
    let ctrl = Regex::new(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]").unwrap();

    let s = osc.replace_all(input, "");
    let s = csi.replace_all(&s, "");
    let s = s.replace('\r', "");
    ctrl.replace_all(&s, "").into_owned()
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;

        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY,
            shell TEXT,
            cwd TEXT,
            transcript_path TEXT NOT NULL,
            transcript TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            started_at TEXT NOT NULL,
            ended_at TEXT NOT NULL,
            byte_count INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions(started_at);
        ",
    )
    .map_err(|err| format!("failed to initialize schema: {err}"))
}

fn memorywhale_dir() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("MEMORYWHALE_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }

    let base = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "could not resolve local data directory".to_string())?;
    Ok(base.join("MemoryWhale"))
}

fn sessions_dir() -> Result<PathBuf, String> {
    Ok(memorywhale_dir()?.join("sessions"))
}

fn database_path() -> Result<PathBuf, String> {
    Ok(memorywhale_dir()?.join("memorywhale.sqlite3"))
}
