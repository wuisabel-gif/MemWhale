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

use chrono::{DateTime, Utc};
use memorywhale_core::engine::MemoryEngine;
use regex::Regex;
use rusqlite::{params, Connection};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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
        Some("list") => return list_sessions(&raw_args[1..]),
        Some("mark") => return mark_bookmark(&raw_args[1..]),
        Some("remember") => return remember_cmd(&raw_args[1..]),
        Some("rm") => return rm_memory(&raw_args[1..]),
        Some("prune") => return prune_cmd(&raw_args[1..]),
        Some("audit") => return audit_cmd(),
        Some("share") => return share_cmd(&raw_args[1..]),
        Some("discard") => return discard_cmd(),
        Some("replay") => return replay_command(&raw_args[1..]),
        Some("demo") => return seed_demo(),
        Some("export") => return export_memory(&raw_args[1..]),
        Some("import") => return import_memory(&raw_args[1..]),
        Some("push") => return push_memory(&raw_args[1..]),
        Some("pull") => return pull_memory(&raw_args[1..]),
        Some("context") => return context_cmd(&raw_args[1..]),
        Some("agent") => return agent_cmd(&raw_args[1..]),
        Some("ask") => return ask_cmd(&raw_args[1..]),
        Some("search") => return search_memory(&raw_args[1..]),
        Some("tui") => return memorywhale_cli::tui::run(),
        Some("sync-mempalace") => return sync_mempalace(&raw_args[1..]),
        Some("git-fix") => return git_fix_cmd(&raw_args[1..]),
        Some("doctor") => return doctor(),
        Some("global") => return global_cmd(&raw_args[1..]),
        Some("status") => return global_status(),
        Some("hooks") => return hooks_cmd(&raw_args[1..]),
        Some("--help") | Some("-h") => {
            print_help();
            return Ok(());
        }
        _ => {}
    }

    if raw_args.is_empty() {
        first_run_welcome()?;
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
    record_session(append_environment_tags(notes), live)
}

fn record_session(notes: String, live: bool) -> Result<(), String> {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let cwd = env::current_dir()
        .ok()
        .and_then(|path| path.to_str().map(ToOwned::to_owned));

    // Capture gate for this directory, resolved before anything is recorded.
    let gate = memorywhale_cli::capture_rule_for(cwd.as_deref());
    if !gate.mode.stores_anything() {
        return run_unrecorded_shell(&shell, &gate.source);
    }
    let store_output = gate.mode.stores_output();

    let started_at = Utc::now().to_rfc3339();
    let sessions_dir = sessions_dir()?;
    fs::create_dir_all(&sessions_dir)
        .map_err(|err| format!("failed to create sessions dir: {err}"))?;
    // commands-only: `script` still needs somewhere to write, but it goes to a
    // scratch file outside the memory directory and is deleted on exit, so no
    // output survives anywhere.
    let transcript_path = if store_output {
        sessions_dir.join(format!("session-{}.log", started_at.replace(':', "-")))
    } else {
        env::temp_dir().join(format!("mw-scratch-{}.log", started_at.replace(':', "-")))
    };
    let transcript_str = transcript_path
        .to_str()
        .ok_or_else(|| "transcript path is not valid UTF-8".to_string())?
        .to_string();

    if store_output {
        eprintln!("mw: recording session to {transcript_str}");
    } else {
        eprintln!("mw: capture is commands-only here ({}) — no output will be stored.", gate.source);
    }
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
            store_output,
        })?;
        // If our parent dies while `script` is still running, finalize this row
        // as `interrupted` immediately instead of leaving it stranded as
        // `recording` until the dashboard's next-startup recovery. Installed
        // before any other thread so the SIGTERM mask (Linux) covers them.
        #[cfg(unix)]
        {
            let death_path = transcript_path.clone();
            // Honours the directory's capture mode: under commands-only the
            // interrupted row is finalized without storing transcript output,
            // same as the normal finish path.
            memorywhale_cli::guard_parent_death(move || {
                let ended_at = Utc::now().to_rfc3339();
                let _ = update_session_from_transcript(
                    id,
                    &death_path,
                    &ended_at,
                    "interrupted",
                    store_output,
                );
            });
        }
        let sync = start_live_sync(id, transcript_path.clone(), store_output);
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
        // Let `mw discard`, run inside the recorded shell, find and remove this
        // transcript so the session is thrown away on exit.
        .env("MW_TRANSCRIPT", &transcript_str)
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

    // `mw discard`, run inside the shell, leaves a marker next to the transcript.
    let discard_marker = PathBuf::from(format!("{transcript_str}.discarded"));
    if discard_marker.exists() {
        let _ = fs::remove_file(&discard_marker);
        let _ = fs::remove_file(&transcript_path);
        if let Some(id) = live_session {
            if let Ok(conn) = open_session_db() {
                let _ = conn.execute("DELETE FROM sessions WHERE id = ?1", params![id]);
            }
        }
        eprintln!("mw: session discarded — nothing saved.");
        return Ok(());
    }

    if !transcript_path.exists() {
        // `script` never produced a transcript (e.g. it failed to launch).
        if let Some(id) = live_session {
            if let Ok(conn) = open_session_db() {
                let _ = conn.execute("DELETE FROM sessions WHERE id = ?1", params![id]);
            }
        }
        eprintln!("mw: recording produced no transcript — nothing saved.");
        return Ok(());
    }
    let (id, byte_count) = if let Some(id) = live_session {
        let byte_count =
            update_session_from_transcript(id, &transcript_path, &ended_at, "finished", store_output)?;
        (id, byte_count)
    } else {
        insert_finished_session(
            &SessionDraft {
                shell: &shell,
                cwd: cwd.as_deref(),
                transcript_path: &transcript_str,
                notes: &notes,
                started_at: &started_at,
                store_output,
            },
            &transcript_path,
            &ended_at,
        )?
    };
    if !store_output {
        let _ = fs::remove_file(&transcript_path);
    }

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
         mw list [--project X] [--machine Y] [--since 7d]  list recorded sessions\n\
         mw show <id>             print the full faithful transcript of a session\n\
         mw mark <text>           bookmark the current debugging moment\n\
         mw remember <text>       save a lesson/conclusion, e.g. \"the fix was passing --features vendored-ssl\"\n\
         mw rm [session|command] <id>  delete a saved item and its transcript\n\
         mw prune [--min-bytes N] [--dry-run]  delete empty auto-recorded sessions (noise cleanup)\n\
         mw prune --older-than <7d|24h|2w> [--dry-run]  delete sessions and command runs older than a window\n\
         mw audit                 report capture policy, retained volume, and high-volume sources\n\
         mw status                print the effective capture mode for this directory and why\n\
         mw share [session|command] <id> [-o file]  write a self-contained HTML page to send to someone\n\
         mw discard               inside a recording: throw the current session away — nothing saved\n\
         mw replay <run-id>       rerun a saved command from command_runs\n\
         mw demo                  seed a small demo terminal-memory dataset\n\
         mw export [project:name] export memory to Markdown + JSON\n\
         mw import <bundle|sqlite> merge another machine's exported memory into this one\n\
         mw push <ssh-host>       send this machine's memory to a teammate (scp + remote mw import)\n\
         mw pull <ssh-host> [path] copy another machine's memory here and merge it (scp + import)\n\
         mw search <text> [--explain] [--project X] [--machine Y] [--since 7d]  rank commands, sessions, and notes by relevance (--explain shows why)\n\
         mw tui                   interactive terminal browser: type to search, arrow keys to move, Enter to reveal the command\n\
         mw sync-mempalace [--wing NAME] [--limit N] [--dry-run]  sync local memories into a running MemPalace server, idempotent by memory id (needs mempalace_command in config)\n\
         mw git-fix [id]          diagnose the last failed git command (or one by id): what happened, the fix, seen before?\n\
         mw context [project:name] [--last-error] [--limit N]  print a compact digest to paste into an AI agent\n\
         mw agent [session-id]    export a full session as agent-ready text to paste later (default: latest)\n\
         mw ask [question] [--chat chatgpt|claude|gemini|URL] [--session] [--no-open]  package the last failure for your chat AI\n\
         mw doctor                check the install: data dir, database, `script`, and hook status\n\
         mw global on|off|status  auto-record every new terminal by wiring a shell startup hook\n\
         mw hooks install|uninstall  always-on lightweight capture: command, cwd, exit code, duration (no output)\n\
         \n\
         Records every command + output, stored locally and never uploaded.\n\
         Raw transcript: <data_local>/MemoryWhale/sessions/\n\
         Metadata + cleaned transcript: <data_local>/MemoryWhale/memorywhale.sqlite3 (sessions table)"
    );
}

/// Shown only on a genuine cold start: no hook wired and nothing recorded yet.
/// Explains `mw` and offers to enable auto-recording; on "no" it falls through
/// to recording this one session so bare `mw` still works as documented.
fn first_run_welcome() -> Result<(), String> {
    use std::io::{IsTerminal, Write};

    // Existing user or scripted call → behave exactly as before (record).
    if global_enabled_path().map(|p| p.exists()).unwrap_or(false) {
        return Ok(());
    }
    if !std::io::stdin().is_terminal() {
        return Ok(());
    }
    let recorded_before = open_session_db()
        .and_then(|conn| {
            conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get::<_, i64>(0))
                .map_err(|err| err.to_string())
        })
        .map(|count| count > 0)
        .unwrap_or(false);
    if recorded_before {
        return Ok(());
    }

    println!(
        "🐬 Welcome to MemoryWhale.\n\
         \n\
         It records your terminal commands, output, and errors into a local\n\
         SQLite database so debugging context survives crashes, SSH drops, and\n\
         switching machines. Nothing is ever uploaded.\n\
         \n\
         The easiest way to use it is to auto-record every new terminal — no\n\
         need to type `mw` each time. This adds one line to your shell startup\n\
         file (`mw global off` undoes it).\n"
    );
    print!("Enable auto-recording in every new terminal now? [y/N] ");
    let _ = std::io::stdout().flush();

    let mut answer = String::new();
    std::io::stdin()
        .read_line(&mut answer)
        .map_err(|err| format!("failed to read input: {err}"))?;

    if matches!(answer.trim(), "y" | "Y" | "yes" | "Yes") {
        global_on()?;
        std::process::exit(0);
    }

    println!("\nNo problem — recording just this one session. Type `exit` to stop.");
    println!("Run `mw --help` to see everything, or `mw global on` later.\n");
    Ok(())
}

struct SessionDraft<'a> {
    shell: &'a str,
    cwd: Option<&'a str>,
    transcript_path: &'a str,
    notes: &'a str,
    started_at: &'a str,
    /// False under `commands-only`: session metadata is kept, the transcript
    /// (and its on-disk path) never reaches the database.
    store_output: bool,
}

/// `capture = "off"` for this directory: hand the user a plain interactive
/// shell. `mw` is often `exec`d by the global hook, so it must still leave a
/// usable shell behind — it just records nothing, anywhere.
fn run_unrecorded_shell(shell: &str, source: &str) -> Result<(), String> {
    eprintln!("mw: capture is OFF for this directory ({source}) — nothing will be recorded.");
    Command::new(shell)
        .env("MW_RECORDING", "1")
        .status()
        .map_err(|err| format!("failed to launch {shell}: {err}"))?;
    Ok(())
}

impl SessionDraft<'_> {
    /// Under `commands-only` the scratch transcript is deleted on exit, so the
    /// database must not point at it.
    fn stored_transcript_path(&self) -> &str {
        if self.store_output {
            self.transcript_path
        } else {
            ""
        }
    }
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
            (shell, cwd, transcript_path, transcript, notes, started_at, ended_at, byte_count,
             status, project, machine)
        VALUES (?1, ?2, ?3, '', ?4, ?5, ?5, 0, 'recording', ?6, ?7)
        ",
        params![
            draft.shell,
            draft.cwd,
            draft.stored_transcript_path(),
            draft.notes,
            draft.started_at,
            memorywhale_cli::project_of(draft.notes),
            memorywhale_cli::machine_name()
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
    let cleaned = if draft.store_output {
        clean_transcript(&String::from_utf8_lossy(&raw))
    } else {
        String::new()
    };
    let conn = open_session_db()?;
    conn.execute(
        "
        INSERT INTO sessions
            (shell, cwd, transcript_path, transcript, notes, started_at, ended_at, byte_count,
             status, project, machine)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'finished', ?9, ?10)
        ",
        params![
            draft.shell,
            draft.cwd,
            draft.stored_transcript_path(),
            cleaned,
            draft.notes,
            draft.started_at,
            ended_at,
            byte_count,
            memorywhale_cli::project_of(draft.notes),
            memorywhale_cli::machine_name()
        ],
    )
    .map_err(|err| format!("failed to insert session: {err}"))?;
    Ok((conn.last_insert_rowid(), byte_count))
}

/// Delete a saved session (default) or command run by id, including a session's
/// transcript file so the dashboard can't re-recover it.
fn rm_memory(args: &[String]) -> Result<(), String> {
    let (kind, id): (&str, i64) = match args {
        [k, id] if k == "session" || k == "command" => (
            k.as_str(),
            id.parse().map_err(|_| format!("invalid id {id:?}"))?,
        ),
        [id] => (
            "session",
            id.parse().map_err(|_| format!("invalid id {id:?}"))?,
        ),
        _ => return Err("usage: mw rm [session|command] <id>".to_string()),
    };
    let conn = open_session_db()?;
    if kind == "session" {
        match conn.query_row(
            "SELECT transcript_path FROM sessions WHERE id = ?1",
            params![id],
            |r| r.get::<_, Option<String>>(0),
        ) {
            Ok(path) => {
                if let Some(p) = path {
                    let _ = fs::remove_file(&p);
                }
                conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])
                    .map_err(|e| format!("failed to delete session: {e}"))?;
                println!("mw: removed session #{id} (and its transcript).");
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err(format!("no session #{id}")),
            Err(e) => return Err(format!("failed to look up session: {e}")),
        }
    } else {
        match conn.query_row("SELECT 1 FROM command_runs WHERE id = ?1", params![id], |_| Ok(())) {
            Ok(()) => {
                let _ = conn.execute(
                    "DELETE FROM command_arguments WHERE command_run_id = ?1",
                    params![id],
                );
                conn.execute("DELETE FROM command_runs WHERE id = ?1", params![id])
                    .map_err(|e| format!("failed to delete command run: {e}"))?;
                println!("mw: removed command run #{id}.");
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(format!("no command run #{id}"))
            }
            Err(e) => return Err(format!("failed to look up command run: {e}")),
        }
    }
    Ok(())
}

/// Summarize what is retained so users can spot unexpectedly sensitive or
/// high-volume capture before deciding what to prune or remove.
fn audit_cmd() -> Result<(), String> {
    let cwd = env::current_dir().map_err(|e| format!("failed to resolve current directory: {e}"))?;
    let rule = memorywhale_cli::capture_rule(&cwd);
    let db_path = database_path()?;
    let conn = open_session_db()?;
    let db_bytes = fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
    let sessions: (i64, i64) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(byte_count), 0) FROM sessions",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("failed to audit sessions: {e}"))?;
    let commands: (i64, i64) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(length(stdout) + length(stderr)), 0) FROM command_runs",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("failed to audit command runs: {e}"))?;

    println!("MemoryWhale privacy audit");
    println!("  current directory: {}", cwd.display());
    println!("  capture mode: {} ({})", rule.mode.as_str(), rule.source);
    println!(
        "  per-field limit: {} bytes (MEMORYWHALE_MAX_CAPTURE_BYTES)",
        memorywhale_cli::max_capture_bytes()
    );
    println!("  database: {} ({} bytes)", db_path.display(), db_bytes);
    println!("  sessions: {} ({} raw bytes)", sessions.0, sessions.1);
    println!("  command runs: {} ({} stored output bytes)", commands.0, commands.1);

    println!("\nHighest-volume session sources:");
    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(NULLIF(cwd, ''), '(unknown)'), COUNT(*), COALESCE(SUM(byte_count), 0)
             FROM sessions GROUP BY cwd ORDER BY 3 DESC LIMIT 10",
        )
        .map_err(|e| format!("failed to group session sources: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| format!("failed to read session sources: {e}"))?;
    let mut found = false;
    for row in rows {
        let (source, count, bytes) = row.map_err(|e| format!("failed to read audit row: {e}"))?;
        println!("  {bytes:>10} bytes  {count:>5} sessions  {source}");
        found = true;
    }
    if !found {
        println!("  no session captures");
    }

    println!("\nCleanup: `mw rm <id>` deletes one session and transcript; use");
    println!("`mw prune --older-than 30d --dry-run` to preview retention cleanup.");
    Ok(())
}

/// Run inside a recorded shell to throw the current session away. Drops a marker
/// file next to the transcript; the parent `mw` sees it on exit and saves nothing.
fn discard_cmd() -> Result<(), String> {
    let path = env::var("MW_TRANSCRIPT").map_err(|_| {
        "mw discard only works inside a recorded session (started by `mw` or auto-record)."
            .to_string()
    })?;
    fs::write(format!("{path}.discarded"), b"1")
        .map_err(|e| format!("failed to mark session for discard: {e}"))?;
    println!("mw: this session will NOT be saved. Type `exit` (or Ctrl-D) to close the shell.");
    Ok(())
}

/// Delete empty/near-empty auto-recorded sessions — the noise left by opening a
/// terminal and closing it without doing anything. Only touches `interrupted`
/// sessions below the byte threshold; deliberate `finished` sessions are safe.
fn prune_cmd(args: &[String]) -> Result<(), String> {
    let mut min_bytes: i64 = 200;
    let mut dry = false;
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        match a.as_str() {
            "--min-bytes" => {
                min_bytes = iter
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or_else(|| "--min-bytes needs a number".to_string())?;
            }
            "--dry-run" => dry = true,
            "--older-than" => {
                let spec = iter
                    .next()
                    .ok_or_else(|| "--older-than needs a duration, e.g. 30d".to_string())?;
                return prune_older_than(spec, dry || args.iter().any(|a| a == "--dry-run"));
            }
            other => {
                return Err(format!(
                    "unexpected argument {other:?}; usage: mw prune [--min-bytes N] [--older-than 30d] [--dry-run]"
                ))
            }
        }
    }
    let conn = open_session_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, transcript_path FROM sessions
             WHERE status = 'interrupted' AND byte_count < ?1",
        )
        .map_err(|err| format!("failed to query sessions: {err}"))?;
    let targets: Vec<(i64, Option<String>)> = stmt
        .query_map(params![min_bytes], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|err| format!("query error: {err}"))?
        .filter_map(Result::ok)
        .collect();

    if targets.is_empty() {
        println!("mw: nothing to prune (no interrupted sessions under {min_bytes} bytes).");
        return Ok(());
    }
    if dry {
        println!(
            "mw: would remove {} interrupted session(s) under {min_bytes} bytes (dry run).",
            targets.len()
        );
        return Ok(());
    }
    let mut n = 0;
    for (id, transcript_path) in &targets {
        if let Some(p) = transcript_path {
            let _ = fs::remove_file(p);
        }
        let _ = conn.execute("DELETE FROM sessions WHERE id = ?1", params![id]);
        n += 1;
    }
    println!("mw: pruned {n} empty/interrupted session(s) (and their transcripts).");
    Ok(())
}

/// Delete sessions (and their transcripts) and command runs older than a
/// relative window like `30d`. Reuses the `--since` duration parser.
fn prune_older_than(spec: &str, dry: bool) -> Result<(), String> {
    let cutoff = (Utc::now() - memorywhale_cli::parse_since(spec)?).to_rfc3339();
    let conn = open_session_db()?;

    let mut stmt = conn
        .prepare("SELECT id, transcript_path FROM sessions WHERE started_at < ?1")
        .map_err(|err| format!("failed to query sessions: {err}"))?;
    let sessions: Vec<(i64, Option<String>)> = stmt
        .query_map(params![cutoff], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|err| format!("query error: {err}"))?
        .filter_map(Result::ok)
        .collect();
    let runs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM command_runs WHERE created_at < ?1",
            params![cutoff],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if sessions.is_empty() && runs == 0 {
        println!("mw: nothing older than {spec}.");
        return Ok(());
    }
    if dry {
        println!(
            "mw: would remove {} session(s) and {runs} command run(s) older than {spec} (dry run).",
            sessions.len()
        );
        for (id, path) in &sessions {
            println!("  session #{id}{}", path.as_deref().map(|p| format!(" -> {p}")).unwrap_or_default());
        }
        return Ok(());
    }
    for (id, transcript_path) in &sessions {
        if let Some(p) = transcript_path.as_deref().filter(|p| !p.is_empty()) {
            let _ = fs::remove_file(p);
        }
        let _ = conn.execute("DELETE FROM sessions WHERE id = ?1", params![id]);
    }
    let _ = conn.execute("DELETE FROM command_runs WHERE created_at < ?1", params![cutoff]);
    println!(
        "mw: pruned {} session(s) and {runs} command run(s) older than {spec}.",
        sessions.len()
    );
    Ok(())
}

/// Locate a sibling MemoryWhale binary (installed next to this one), falling
/// back to the bare name on PATH.
fn sibling_binary(name: &str) -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(name)))
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from(name))
}

/// Write a self-contained, redacted HTML page for one session or command run,
/// to hand to a teammate. Reuses `mw-view`'s renderer.
fn share_cmd(args: &[String]) -> Result<(), String> {
    let mut kind: Option<&str> = None;
    let mut id: Option<String> = None;
    let mut output: Option<String> = None;
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        match a.as_str() {
            "session" | "command" => kind = Some(if a == "session" { "session" } else { "command" }),
            "-o" | "--output" => {
                output = Some(
                    iter.next()
                        .cloned()
                        .ok_or_else(|| "-o needs a file path".to_string())?,
                )
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option {other:?}; usage: mw share <id> [-o file.html]"))
            }
            other => id = Some(other.to_string()),
        }
    }
    let id = id.ok_or_else(|| "usage: mw share [session|command] <id> [-o file.html]".to_string())?;
    let output = output.unwrap_or_else(|| format!("memory-{id}.html"));

    let mut cmd = Command::new(sibling_binary("mw-view"));
    if let Some(k) = kind {
        cmd.arg(k);
    }
    cmd.arg(&id).arg("--no-open").arg("-o").arg(&output);
    let result = cmd
        .output()
        .map_err(|err| format!("failed to run mw-view (is MemoryWhale installed?): {err}"))?;
    if !result.status.success() {
        return Err(format!(
            "could not render #{id}: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }
    println!("mw: wrote shareable page to {output}");
    println!("    It's self-contained and local — send the file; nothing was uploaded.");
    Ok(())
}

fn start_live_sync(id: i64, transcript_path: PathBuf, store_output: bool) -> LiveSync {
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let handle = thread::spawn(move || {
        while !thread_stop.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(LIVE_SYNC_INTERVAL_SECS));
            if thread_stop.load(Ordering::SeqCst) {
                break;
            }
            let ended_at = Utc::now().to_rfc3339();
            let _ =
                update_session_from_transcript(id, &transcript_path, &ended_at, "recording", store_output);
        }
    });
    LiveSync { stop, handle }
}

fn update_session_from_transcript(
    id: i64,
    transcript_path: &PathBuf,
    ended_at: &str,
    status: &str,
    store_output: bool,
) -> Result<i64, String> {
    let raw = match fs::read(transcript_path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(format!("failed to read transcript: {err}")),
    };
    let byte_count = raw.len() as i64;
    let cleaned = if store_output {
        clean_transcript(&String::from_utf8_lossy(&raw))
    } else {
        String::new()
    };
    let conn = open_session_db()?;
    conn.execute(
        "
        UPDATE sessions
        SET transcript = ?1, ended_at = ?2, byte_count = ?3, status = ?4
        WHERE id = ?5
        ",
        params![cleaned, ended_at, byte_count, status, id],
    )
    .map_err(|err| format!("failed to autosave session: {err}"))?;
    Ok(byte_count)
}

fn open_session_db() -> Result<Connection, String> {
    let db_path = database_path()?;
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("failed to create data dir: {err}"))?;
        memorywhale_cli::restrict_path_permissions(parent, true)?;
    }
    let conn = memorywhale_cli::storage::open_path(&db_path)?;
    memorywhale_cli::restrict_path_permissions(&db_path, false)?;
    Ok(conn)
}

fn mark_bookmark(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("usage: mw mark <text>".to_string());
    }
    let label = args.join(" ");
    let cwd = env::current_dir()
        .ok()
        .and_then(|path| path.to_str().map(ToOwned::to_owned));
    let id = memorywhale_cli::remember(&label, cwd.as_deref())?;
    println!("mw: marked bookmark #{id}");
    Ok(())
}

/// Save a freeform lesson/conclusion (not tied to a specific command), so you
/// or an agent can search it back out later with `mw search`/`mw context` or
/// the MCP `search_memory` tool. Shares storage with `mw mark`.
fn remember_cmd(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("usage: mw remember <text>".to_string());
    }
    let text = args.join(" ");
    let cwd = env::current_dir()
        .ok()
        .and_then(|path| path.to_str().map(ToOwned::to_owned));
    let id = memorywhale_cli::remember(&text, cwd.as_deref())?;
    // Echo back what was actually stored (redacted), not the raw input.
    println!("mw: remembered #{id}: {}", memorywhale_cli::redact(&text));
    Ok(())
}

fn replay_command(args: &[String]) -> Result<(), String> {
    let id: i64 = args
        .first()
        .ok_or_else(|| "usage: mw replay <command-run-id>".to_string())?
        .parse()
        .map_err(|_| "command-run-id must be a number".to_string())?;
    let conn = open_session_db()?;
    let (argv_json, cwd): (String, Option<String>) = conn
        .query_row(
            "SELECT argv_json, cwd FROM command_runs WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|err| format!("failed to read command run #{id}: {err}"))?;
    let argv: Vec<String> =
        serde_json::from_str(&argv_json).map_err(|err| format!("bad stored argv: {err}"))?;
    if argv.is_empty() {
        return Err(format!("command run #{id} has no argv"));
    }

    println!("mw: replaying #{}: {}", id, argv.join(" "));
    let mut command = Command::new(&argv[0]);
    command.args(&argv[1..]);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let status = command
        .status()
        .map_err(|err| format!("failed to replay command: {err}"))?;
    println!("mw: replay exited with {}", status);
    Ok(())
}

fn seed_demo() -> Result<(), String> {
    let conn = open_session_db()?;
    let now = Utc::now().to_rfc3339();
    let demo_notes = "project:demo host:jetson runtime:host";
    conn.execute(
        "INSERT INTO command_runs (command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            "cargo",
            serde_json::to_string(&vec!["cargo", "check"]).unwrap(),
            "/demo/MemoryWhale",
            101_i64,
            "",
            "error: failed to build\\nNo package 'libsoup-3.0' found\\n",
            demo_notes,
            now
        ],
    )
    .map_err(|err| format!("failed to insert demo command: {err}"))?;
    let run_id = conn.last_insert_rowid();
    for (position, value) in ["cargo", "check"].iter().enumerate() {
        conn.execute(
            "INSERT INTO command_arguments (command_run_id, position, value) VALUES (?1, ?2, ?3)",
            params![run_id, position as i64, value],
        )
        .map_err(|err| format!("failed to insert demo argument: {err}"))?;
    }
    conn.execute(
        "INSERT INTO bookmarks (label, cwd, created_at) VALUES (?1, ?2, ?3)",
        params![
            "Tauri build failed here; install missing Linux packages.",
            "/demo/MemoryWhale",
            Utc::now().to_rfc3339()
        ],
    )
    .map_err(|err| format!("failed to insert demo bookmark: {err}"))?;
    println!("mw: demo memory inserted. Run `mw-serve` and search for project:demo.");
    Ok(())
}

fn export_memory(args: &[String]) -> Result<(), String> {
    let project = args.first().cloned();
    let export_dir = memorywhale_dir()?.join("exports");
    fs::create_dir_all(&export_dir)
        .map_err(|err| format!("failed to create exports dir: {err}"))?;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let base = project
        .as_deref()
        .unwrap_or("all")
        .replace([':', '/', '\\', ' '], "-");
    let bundle_dir = export_dir.join(format!("{base}-{stamp}"));
    let transcripts_dir = bundle_dir.join("transcripts");
    fs::create_dir_all(&transcripts_dir)
        .map_err(|err| format!("failed to create bundle dir: {err}"))?;
    let markdown_path = bundle_dir.join("memory.md");
    let json_path = bundle_dir.join("memory.json");
    let sqlite_path = bundle_dir.join("memorywhale.sqlite3");
    let conn = open_session_db()?;
    let like = project.as_ref().map(|p| format!("%{p}%"));

    let mut md = String::from("# MemoryWhale Debug Bundle\n\n");
    let mut commands = Vec::new();
    let mut sessions = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT id, command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at
             FROM command_runs
             WHERE ?1 IS NULL OR notes LIKE ?1
             ORDER BY id",
        )
        .map_err(|err| format!("failed to prepare command export: {err}"))?;
    let rows = stmt
        .query_map(params![like.as_deref()], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, String>(8)?,
            ))
        })
        .map_err(|err| format!("failed to export commands: {err}"))?;
    for row in rows {
        let (id, command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at) =
            row.map_err(|err| format!("command row error: {err}"))?;
        let argv: Vec<String> = serde_json::from_str(&argv_json).unwrap_or_default();
        md.push_str(&format!(
            "## Command #{id}: `{}`\n\n- when: `{created_at}`\n- cwd: `{}`\n- exit: `{:?}`\n- notes: {}\n\n```text\n{}\n{}\n```\n\n",
            argv.join(" "),
            cwd.clone().unwrap_or_default(),
            exit_code,
            notes,
            stdout,
            stderr
        ));
        commands.push(serde_json::json!({
            "id": id,
            "command": command,
            "argv": argv,
            "cwd": cwd,
            "exit_code": exit_code,
            "stdout": stdout,
            "stderr": stderr,
            "notes": notes,
            "created_at": created_at
        }));
    }

    let mut stmt = conn
        .prepare(
            "SELECT id, transcript_path, transcript, notes, started_at, ended_at, byte_count, status
             FROM sessions
             WHERE ?1 IS NULL OR notes LIKE ?1
             ORDER BY id",
        )
        .map_err(|err| format!("failed to prepare session export: {err}"))?;
    let rows = stmt
        .query_map(params![like.as_deref()], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, String>(7)?,
            ))
        })
        .map_err(|err| format!("failed to export sessions: {err}"))?;
    for row in rows {
        let (id, transcript_path, transcript, notes, started_at, ended_at, byte_count, status) =
            row.map_err(|err| format!("session row error: {err}"))?;
        let transcript_file = transcripts_dir.join(format!("session-{id}.txt"));
        fs::write(&transcript_file, &transcript)
            .map_err(|err| format!("failed to write transcript export: {err}"))?;
        md.push_str(&format!(
            "## Session #{id}\n\n- started: `{started_at}`\n- ended: `{ended_at}`\n- status: `{status}`\n- bytes: `{byte_count}`\n- notes: {notes}\n- transcript: `{}`\n\n",
            transcript_file
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("transcript.txt")
        ));
        sessions.push(serde_json::json!({
            "id": id,
            "transcript_path": transcript_path,
            "exported_transcript": transcript_file.file_name().and_then(|name| name.to_str()).unwrap_or("transcript.txt"),
            "notes": notes,
            "started_at": started_at,
            "ended_at": ended_at,
            "byte_count": byte_count,
            "status": status
        }));
    }

    fs::write(&markdown_path, md)
        .map_err(|err| format!("failed to write markdown export: {err}"))?;
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "project": project,
            "commands": commands,
            "sessions": sessions
        }))
        .map_err(|err| format!("failed to encode JSON export: {err}"))?,
    )
    .map_err(|err| format!("failed to write JSON export: {err}"))?;
    let db_path = database_path()?;
    if db_path.exists() {
        fs::copy(&db_path, &sqlite_path)
            .map_err(|err| format!("failed to copy SQLite backup: {err}"))?;
    }
    println!("mw: exported debug bundle {}", bundle_dir.display());
    Ok(())
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

// ---------------------------------------------------------------------------
// `mw hooks` — the lightweight always-on capture tier.
//
// Supports bash, zsh, fish, and PowerShell (`mw hooks install pwsh`).
// ---------------------------------------------------------------------------

// In-crate so they ship inside the published package (an include_str! that
// reached ../../../../linux/ broke `cargo publish`, which only packages crate
// files). `linux/install.sh` and the .deb reference this same copy.
const HOOK_SH: &str = include_str!("../../shell/memorywhale.sh");
const HOOK_FISH: &str = include_str!("../../shell/memorywhale.fish");
const HOOK_PS1: &str = include_str!("../../shell/memorywhale.ps1");
const HOOK_BEGIN: &str = "# >>> memorywhale shell hooks >>>";
const HOOK_END: &str = "# <<< memorywhale shell hooks <<<";

/// The PowerShell profile (`$PROFILE`, current-user/current-host). Windows
/// keeps it under Documents/PowerShell; PowerShell Core on Unix under
/// ~/.config/powershell. `#` starts a comment in PowerShell too, so the shared
/// HOOK_BEGIN/HOOK_END markers work as-is.
fn powershell_profile(home: &Path) -> PathBuf {
    let file = "Microsoft.PowerShell_profile.ps1";
    if cfg!(windows) {
        home.join("Documents").join("PowerShell").join(file)
    } else {
        home.join(".config").join("powershell").join(file)
    }
}

/// (shell name, rc file, generated hook file, hook script body).
/// `explicit` is an optional shell argument (`mw hooks install pwsh`); when
/// absent we fall back to `$SHELL` detection. PowerShell is never in `$SHELL`,
/// so it must be requested explicitly.
fn hook_target(explicit: Option<&str>) -> Result<(&'static str, PathBuf, PathBuf, &'static str), String> {
    let home = dirs::home_dir().ok_or_else(|| "could not resolve home directory".to_string())?;
    let hooks_dir = memorywhale_dir()?;
    let name = match explicit {
        Some(arg) => arg.to_string(),
        None => {
            let shell = env::var("SHELL").unwrap_or_default();
            shell.rsplit('/').next().unwrap_or_default().to_string()
        }
    };
    if name.contains("pwsh") || name.contains("powershell") {
        Ok((
            "powershell",
            powershell_profile(&home),
            hooks_dir.join("memorywhale.ps1"),
            HOOK_PS1,
        ))
    } else if name.contains("fish") {
        Ok((
            "fish",
            home.join(".config").join("fish").join("config.fish"),
            hooks_dir.join("memorywhale.fish"),
            HOOK_FISH,
        ))
    } else if name.contains("zsh") {
        Ok(("zsh", home.join(".zshrc"), hooks_dir.join("memorywhale.sh"), HOOK_SH))
    } else if explicit.map_or(false, |a| !a.contains("bash")) {
        Err(format!(
            "unknown shell {name:?}; supported: bash, zsh, fish, pwsh/powershell"
        ))
    } else {
        Ok(("bash", home.join(".bashrc"), hooks_dir.join("memorywhale.sh"), HOOK_SH))
    }
}

fn hook_block(shell: &str, hook_path: &str) -> String {
    let source_line = match shell {
        "fish" => format!("test -f \"{hook_path}\"; and source \"{hook_path}\""),
        // PowerShell: dot-source the generated hook when it exists.
        "powershell" => format!("if (Test-Path \"{hook_path}\") {{ . \"{hook_path}\" }}"),
        _ => format!("[ -f \"{hook_path}\" ] && . \"{hook_path}\""),
    };
    format!("{HOOK_BEGIN}\n# Managed by `mw hooks` — edit above/below, not inside.\n{source_line}\n{HOOK_END}\n")
}

/// Drop the managed block (and only that) from an rc file's text.
fn strip_hook_block(rc: &str) -> String {
    let mut out = String::with_capacity(rc.len());
    let mut skipping = false;
    for line in rc.lines() {
        if line.trim() == HOOK_BEGIN {
            skipping = true;
            continue;
        }
        if skipping {
            if line.trim() == HOOK_END {
                skipping = false;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn hooks_cmd(args: &[String]) -> Result<(), String> {
    // Optional trailing shell arg: `mw hooks install pwsh`. PowerShell isn't in
    // $SHELL, so it must be named explicitly.
    let shell_arg = args.get(1).map(String::as_str);
    match args.first().map(String::as_str) {
        Some("install") => hooks_install(shell_arg),
        Some("uninstall") | Some("remove") => hooks_uninstall(shell_arg),
        Some(other) => Err(format!(
            "unknown subcommand {other:?}; usage: mw hooks [install|uninstall] [pwsh]"
        )),
        None => Err("usage: mw hooks [install|uninstall] [pwsh]".to_string()),
    }
}

fn hooks_install(shell_arg: Option<&str>) -> Result<(), String> {
    let (shell, rc_path, hook_path, body) = hook_target(shell_arg)?;
    if let Some(parent) = hook_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create data dir: {e}"))?;
    }
    fs::write(&hook_path, body).map_err(|e| format!("failed to write hook script: {e}"))?;
    let hook_str = hook_path
        .to_str()
        .ok_or_else(|| "hook path is not valid UTF-8".to_string())?;

    if let Some(parent) = rc_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    let existing = fs::read_to_string(&rc_path).unwrap_or_default();
    // Idempotent: strip any previous block first, then append exactly one.
    let mut updated = strip_hook_block(&existing);
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(&hook_block(shell, hook_str));
    fs::write(&rc_path, updated).map_err(|e| format!("failed to update {}: {e}", rc_path.display()))?;

    println!("mw: lightweight shell hooks INSTALLED ({shell}).");
    println!("  rc file: {}", rc_path.display());
    println!("  hook:    {hook_str}");
    println!("  Records command, cwd, exit code and duration. No output — use `mw --live` for that.");
    println!("  Open a NEW terminal to start capturing.");
    Ok(())
}

fn hooks_uninstall(shell_arg: Option<&str>) -> Result<(), String> {
    let (shell, rc_path, hook_path, _) = hook_target(shell_arg)?;
    let existing = fs::read_to_string(&rc_path).unwrap_or_default();
    let updated = strip_hook_block(&existing);
    let changed = updated != existing;
    if changed {
        fs::write(&rc_path, updated)
            .map_err(|e| format!("failed to update {}: {e}", rc_path.display()))?;
    }
    let _ = fs::remove_file(&hook_path);
    if changed {
        println!("mw: shell hooks REMOVED from {} ({shell}).", rc_path.display());
    } else {
        println!("mw: no shell hook block found in {} — nothing to do.", rc_path.display());
    }
    Ok(())
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

    let cwd = env::current_dir().unwrap_or_default();
    let gate = memorywhale_cli::capture_rule(&cwd);
    println!();
    println!("capture mode here ({}): {}", cwd.display(), gate.mode.as_str());
    println!("  rule: {}", gate.source);
    Ok(())
}

fn show_session(args: &[String]) -> Result<(), String> {
    let id: i64 = match args.first() {
        Some(value) => value
            .parse()
            .map_err(|_| format!("invalid session id {value:?}; usage: mw show <id>"))?,
        None => return Err("usage: mw show <id>".to_string()),
    };

    let conn = memorywhale_cli::storage::open()?;

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

fn list_sessions(args: &[String]) -> Result<(), String> {
    let (scope, rest) = Scope::take(args)?;
    if let Some(other) = rest.first() {
        return Err(format!("unexpected argument {other:?}; run mw --help"));
    }
    let conn = open_session_db()?;
    let since = scope.cutoff(Utc::now()).map(|c| c.to_rfc3339());

    let mut stmt = conn
        .prepare(
            "SELECT id, started_at, byte_count, notes FROM sessions
             WHERE (?1 IS NULL OR project = ?1)
               AND (?2 IS NULL OR machine = ?2)
               AND (?3 IS NULL OR started_at >= ?3)
             ORDER BY id",
        )
        .map_err(|err| format!("failed to query sessions: {err}"))?;
    let rows = stmt
        .query_map(
            params![scope.project.as_deref(), scope.machine.as_deref(), since.as_deref()],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                ))
            },
        )
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
    let cleaned = ctrl.replace_all(&s, "").into_owned();
    // Scrub secrets before the transcript is stored (env dumps, pasted tokens).
    memorywhale_cli::sanitize_capture(&cleaned)
}

/// Send this machine's memory to a teammate: make a clean DB snapshot, scp it
/// over, and run `mw import` on the far side. Uses ssh/scp you already have —
/// no server, nothing uploaded to a third party.
fn push_memory(args: &[String]) -> Result<(), String> {
    let host = args
        .first()
        .ok_or_else(|| "usage: mw push <ssh-host>  (e.g. mw push jetson)".to_string())?;

    // Clean, standalone snapshot (folds in any WAL) via VACUUM INTO.
    let stamp = Utc::now().format("%Y%m%d-%H%M%S");
    let snapshot = std::env::temp_dir().join(format!("mw-push-{stamp}.sqlite3"));
    let snapshot_str = snapshot
        .to_str()
        .ok_or_else(|| "temp path is not valid UTF-8".to_string())?;
    {
        let conn = open_session_db()?;
        conn.execute("VACUUM INTO ?1", params![snapshot_str])
            .map_err(|err| format!("failed to snapshot database: {err}"))?;
    }

    let remote_path = format!("/tmp/mw-push-{stamp}.sqlite3");
    println!("mw: copying snapshot to {host}:{remote_path} …");
    let scp = Command::new("scp")
        .arg(snapshot_str)
        .arg(format!("{host}:{remote_path}"))
        .status()
        .map_err(|err| format!("failed to run scp (is it installed?): {err}"))?;
    let _ = fs::remove_file(&snapshot);
    if !scp.success() {
        return Err(format!("scp to {host} failed"));
    }

    println!("mw: running `mw import` on {host} …");
    let ssh = Command::new("ssh")
        .arg(host)
        // Login shell so ~/.local/bin (where the installer puts mw) is on PATH.
        .arg(format!(
            "sh -lc 'mw import {remote_path}; rm -f {remote_path}'"
        ))
        .status()
        .map_err(|err| format!("failed to run ssh: {err}"))?;
    if !ssh.success() {
        return Err(format!(
            "remote import on {host} failed (is `mw` installed and on PATH there?)"
        ));
    }
    println!("mw: pushed memory to {host}.");
    Ok(())
}

/// Pull a teammate's (or your other machine's) memory over SSH and merge it in:
/// copy their database with scp, then import it locally. The mirror of `mw push`.
fn pull_memory(args: &[String]) -> Result<(), String> {
    let host = args
        .first()
        .ok_or_else(|| "usage: mw pull <ssh-host> [remote-db-path]".to_string())?;
    // Default to the Linux/Jetson data dir; override with an explicit path arg.
    let remote = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("~/.local/share/MemoryWhale/memorywhale.sqlite3");

    let stamp = Utc::now().format("%Y%m%d-%H%M%S");
    let local = std::env::temp_dir().join(format!("mw-pull-{stamp}.sqlite3"));
    let local_str = local
        .to_str()
        .ok_or_else(|| "temp path is not valid UTF-8".to_string())?;

    println!("mw: copying {host}:{remote} …");
    let scp = Command::new("scp")
        .arg(format!("{host}:{remote}"))
        .arg(local_str)
        .status()
        .map_err(|err| format!("failed to run scp (is it installed?): {err}"))?;
    if !scp.success() {
        return Err(format!(
            "scp from {host} failed (is MemoryWhale installed there? try `mw pull {host} <path-to-its-memorywhale.sqlite3>`)"
        ));
    }

    let result = import_sqlite(&local);
    let _ = fs::remove_file(&local);
    result
}

/// Merge another machine's exported memory (a bundle dir or a raw .sqlite3)
/// into the local database, skipping rows that are already present.
fn import_memory(args: &[String]) -> Result<(), String> {
    let raw = args
        .first()
        .ok_or_else(|| "usage: mw import <bundle-dir|memorywhale.sqlite3>".to_string())?;
    let path = PathBuf::from(raw);
    let src = if path.is_dir() {
        path.join("memorywhale.sqlite3")
    } else {
        path
    };
    if !src.exists() {
        return Err(format!("no SQLite database found at {}", src.display()));
    }
    import_sqlite(&src)
}

/// Attach a source SQLite database and merge its command runs, sessions, and
/// bookmarks into the local database (skipping duplicates). Shared by
/// `mw import` and `mw pull`.
fn import_sqlite(src: &std::path::Path) -> Result<(), String> {
    let src_str = src
        .to_str()
        .ok_or_else(|| "import path is not valid UTF-8".to_string())?;

    let conn = open_session_db()?;
    conn.execute("ATTACH DATABASE ?1 AS src", params![src_str])
        .map_err(|err| format!("failed to attach {src_str}: {err}"))?;

    // A DB written by one CLI binary may only have the tables that binary uses.
    let src_has = |table: &str| -> bool {
        conn.query_row(
            "SELECT 1 FROM src.sqlite_master WHERE type='table' AND name=?1",
            params![table],
            |_| Ok(()),
        )
        .is_ok()
    };

    let before_runs: i64 = conn
        .query_row("SELECT COUNT(*) FROM command_runs", [], |r| r.get(0))
        .unwrap_or(0);
    let before_sessions: i64 = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
        .unwrap_or(0);

    // A source DB written by an older `mw` may be missing columns added later
    // (e.g. sessions.status). Select `s.<col>` when present, else a default, so
    // cross-version imports don't fail.
    let sel = |cols: &std::collections::HashSet<String>, col: &str, default: &str| -> String {
        if cols.contains(col) {
            format!("s.{col}")
        } else {
            default.to_string()
        }
    };

    // command_runs: skip rows already present (same command, argv, and timestamp).
    if src_has("command_runs") {
        let c = src_columns(&conn, "command_runs");
        if c.contains("command") && c.contains("argv_json") && c.contains("created_at") {
            let sql = format!(
                "INSERT INTO command_runs (command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at)
                 SELECT {}, {}, {}, {}, {}, {}, {}, {}
                 FROM src.command_runs s
                 WHERE NOT EXISTS (
                   SELECT 1 FROM command_runs m
                   WHERE m.command = s.command AND m.argv_json = s.argv_json AND m.created_at = s.created_at
                 )",
                sel(&c, "command", "''"), sel(&c, "argv_json", "'[]'"), sel(&c, "cwd", "NULL"),
                sel(&c, "exit_code", "NULL"), sel(&c, "stdout", "''"), sel(&c, "stderr", "''"),
                sel(&c, "notes", "''"), sel(&c, "created_at", "''")
            );
            conn.execute(&sql, [])
                .map_err(|err| format!("failed to merge command runs: {err}"))?;
        }
    }

    // sessions: skip rows with the same start time and transcript path.
    if src_has("sessions") {
        let c = src_columns(&conn, "sessions");
        if c.contains("started_at") && c.contains("transcript_path") {
            let sql = format!(
                "INSERT INTO sessions (shell, cwd, transcript_path, transcript, notes, started_at, ended_at, byte_count, status)
                 SELECT {}, {}, {}, {}, {}, {}, {}, {}, {}
                 FROM src.sessions s
                 WHERE NOT EXISTS (
                   SELECT 1 FROM sessions m
                   WHERE m.started_at = s.started_at AND m.transcript_path = s.transcript_path
                 )",
                sel(&c, "shell", "''"), sel(&c, "cwd", "NULL"), sel(&c, "transcript_path", "''"),
                sel(&c, "transcript", "''"), sel(&c, "notes", "''"), sel(&c, "started_at", "''"),
                sel(&c, "ended_at", "''"), sel(&c, "byte_count", "0"), sel(&c, "status", "'finished'")
            );
            conn.execute(&sql, [])
                .map_err(|err| format!("failed to merge sessions: {err}"))?;
        }
    }

    // bookmarks: skip same label + timestamp.
    if src_has("bookmarks") {
        let _ = conn.execute(
            "INSERT INTO bookmarks (label, cwd, created_at)
             SELECT s.label, s.cwd, s.created_at FROM src.bookmarks s
             WHERE NOT EXISTS (
               SELECT 1 FROM bookmarks m WHERE m.label = s.label AND m.created_at = s.created_at
             )",
            [],
        );
    }

    conn.execute("DETACH DATABASE src", [])
        .map_err(|err| format!("failed to detach source: {err}"))?;

    // Rebuild the searchable argument rows for any newly imported command runs.
    rebuild_missing_arguments(&conn)?;

    let after_runs: i64 = conn
        .query_row("SELECT COUNT(*) FROM command_runs", [], |r| r.get(0))
        .unwrap_or(before_runs);
    let after_sessions: i64 = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
        .unwrap_or(before_sessions);
    println!(
        "mw: imported {} new command run(s) and {} new session(s) from {}",
        after_runs - before_runs,
        after_sessions - before_sessions,
        src.display()
    );
    Ok(())
}

/// Column names present in an attached-source table (`src.<table>`).
fn src_columns(conn: &Connection, table: &str) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    if let Ok(mut stmt) = conn.prepare("SELECT name FROM pragma_table_info(?1, 'src')") {
        if let Ok(it) = stmt.query_map(params![table], |r| r.get::<_, String>(0)) {
            for c in it.flatten() {
                set.insert(c);
            }
        }
    }
    set
}

/// Split argv into the searchable command_arguments table for any command_run
/// that has none yet (imported rows arrive without their argument rows).
fn rebuild_missing_arguments(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, argv_json FROM command_runs
             WHERE id NOT IN (SELECT DISTINCT command_run_id FROM command_arguments)",
        )
        .map_err(|err| format!("failed to find runs missing arguments: {err}"))?;
    let rows: Vec<(i64, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|err| format!("query error: {err}"))?
        .filter_map(Result::ok)
        .collect();
    for (id, argv_json) in rows {
        let argv: Vec<String> = serde_json::from_str(&argv_json).unwrap_or_default();
        for (position, value) in argv.iter().enumerate() {
            conn.execute(
                "INSERT INTO command_arguments (command_run_id, position, value) VALUES (?1, ?2, ?3)",
                params![id, position as i64, value],
            )
            .map_err(|err| format!("failed to insert argument: {err}"))?;
        }
    }
    Ok(())
}

/// Search remembered commands, sessions, and notes — ranked by the explainable
/// engine (`memorywhale-core`) rather than a raw LIKE scan, so the best match rises to
/// the top with a score and, under `--explain`, the per-signal breakdown behind
/// it. The everyday "where did I see that error?" command.
///
/// The CLI runs the engine in its lexical mode (term overlap) — no Ollama, no
/// embedding cache — so it stays dependency-light and works fully offline. The
/// desktop app attaches semantic embeddings over the same engine + loader.
/// Provenance for one remembered note, e.g. "remembered by Claude Code on
/// 2026-07-12 during session #41". Looked up per hit — only note-sourced
/// results have one, and a search returns at most 20.
fn note_provenance(conn: &Connection, id: i64) -> Option<String> {
    conn.query_row(
        "SELECT author_kind, author_name, created_at, source_session_id
         FROM bookmarks WHERE id = ?1",
        params![id],
        |r| {
            Ok(memorywhale_cli::provenance_label(
                &r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?.as_deref(),
                &r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                r.get::<_, Option<i64>>(3)?,
            ))
        },
    )
    .ok()
}

/// `--project` / `--machine` / `--since` as parsed off a command line.
#[derive(Default)]
struct Scope {
    project: Option<String>,
    machine: Option<String>,
    since: Option<String>,
}

impl Scope {
    /// Pull the scope flags out of `args`, returning them plus the leftovers.
    fn take(args: &[String]) -> Result<(Scope, Vec<String>), String> {
        let mut scope = Scope::default();
        let mut rest = Vec::new();
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            let slot = match arg.as_str() {
                "--project" => &mut scope.project,
                "--machine" => &mut scope.machine,
                "--since" => &mut scope.since,
                _ => {
                    rest.push(arg.clone());
                    continue;
                }
            };
            *slot = Some(
                iter.next()
                    .cloned()
                    .ok_or_else(|| format!("{arg} requires a value"))?,
            );
        }
        if let Some(spec) = &scope.since {
            memorywhale_cli::parse_since(spec)?; // fail fast on a bad window
        }
        // `--project project:demo` and `--project demo` mean the same thing.
        scope.project = scope
            .project
            .map(|p| p.trim_start_matches("project:").to_string())
            .filter(|p| !p.is_empty());
        Ok((scope, rest))
    }

    /// Cutoff timestamp for `--since`, relative to `now`.
    fn cutoff(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let spec = self.since.as_ref()?;
        Some(now - memorywhale_cli::parse_since(spec).ok()?)
    }

    /// The scope as engine task tags, so task-relevance scoring can fire on
    /// memories that mention the project or machine.
    fn task_tags(&self) -> Vec<String> {
        [self.project.as_ref(), self.machine.as_ref()]
            .into_iter()
            .flatten()
            .cloned()
            .collect()
    }
}

/// Render hits from an external engine (MemPalace) — no local ids, so no
/// `mw replay/show` action or provenance lookup; the reason string already
/// carries the source ("mempalace semantic score 0.87").
fn print_external_hits(query: &str, hits: &[memorywhale_core::ScoredMemory], explain: bool) {
    println!("# matches for {query:?}  (mempalace)\n");
    if hits.is_empty() {
        println!("(none)");
        return;
    }
    for sm in hits {
        let snippet: String = sm
            .memory
            .text
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim()
            .chars()
            .take(100)
            .collect();
        println!("{:>3}%  [mempalace] {}", sm.percent(), snippet);
        if explain {
            for reason in sm.reasons() {
                println!("      {reason}");
            }
        }
    }
}

/// Push local memories into a running MemPalace server so it can search your
/// terminal history semantically. Maps each memory to a MemPalace drawer:
/// `--wing` (default "memorywhale") groups them, and the source kind (command /
/// session / note / …) becomes the room. `mempalace_checkpoint` on the server
/// side semantic-dedups, so re-running is safe and only files what's new.
fn sync_mempalace(args: &[String]) -> Result<(), String> {
    let mut wing = "memorywhale".to_string();
    let mut limit: usize = 0; // 0 = all
    let mut dry_run = false;
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        match a.as_str() {
            "--wing" => wing = iter.next().ok_or("--wing needs a value")?.clone(),
            "--limit" => limit = iter.next().and_then(|v| v.parse().ok()).ok_or("--limit needs a number")?,
            "--dry-run" => dry_run = true,
            other => return Err(format!("unexpected argument {other:?}; run mw --help")),
        }
    }

    let argv = memorywhale_cli::mempalace_argv()
        .ok_or("no mempalace_command configured in config.toml")?;
    let (cmd, rest) = argv.split_first().expect("mempalace_argv is non-empty");

    let mut conn = open_session_db()?;
    memorywhale_cli::migrate(&conn)?; // brings up the mempalace_sync table (v5)
    let mut mems = memorywhale_core::sqlite::load_memories(&conn);
    if limit > 0 && mems.len() > limit {
        mems.truncate(limit);
    }
    // Render each memory into its target drawer, carrying provenance into the
    // content and computing the change-key hash over the final string.
    let desired: Vec<memorywhale_cli::DesiredDrawer> = mems
        .iter()
        .filter(|m| !m.text.trim().is_empty())
        .map(|m| {
            let (source, real_id) = memorywhale_core::sqlite::decode_id(m.id);
            let (content, added_by) = drawer_content(&conn, m, source, real_id);
            memorywhale_cli::DesiredDrawer {
                mw_id: m.id,
                wing: wing.clone(),
                room: source.tag().to_string(),
                content_hash: memorywhale_cli::content_hash(&content),
                content,
                added_by,
            }
        })
        .collect();

    if desired.is_empty() {
        println!("Nothing to sync (no memories found).");
        return Ok(());
    }

    let existing = memorywhale_cli::load_sync_map(&conn)?;
    let plan = memorywhale_cli::plan_sync(&existing, &desired);

    if dry_run {
        println!(
            "Plan for wing {wing:?}: {} to add, {} to update (delete+add), {} unchanged.",
            plan.added(),
            plan.updated(),
            plan.unchanged
        );
        return Ok(());
    }

    if plan.items.is_empty() {
        println!(
            "Already up to date (wing {wing:?}): 0 added, 0 updated, {} unchanged.",
            plan.unchanged
        );
        return Ok(());
    }

    // Build the op stream: each update deletes its old drawer, then every item
    // adds a fresh one. Adds come back in item order (deletes carry no result).
    let mut ops: Vec<memorywhale_core::engine::SyncOp> = Vec::new();
    for it in &plan.items {
        if let Some(old) = &it.old_drawer_id {
            ops.push(memorywhale_core::engine::SyncOp::Delete { drawer_id: old.clone() });
        }
        ops.push(memorywhale_core::engine::SyncOp::Add {
            wing: it.wing.clone(),
            room: it.room.clone(),
            content: it.content.clone(),
            added_by: it.added_by.clone(),
        });
    }

    let add_tool = memorywhale_cli::mempalace_add_tool();
    let delete_tool = memorywhale_cli::mempalace_delete_tool();
    let results = memorywhale_core::engine::sync_ops(cmd, rest, &add_tool, &delete_tool, &ops)
        .map_err(|e| format!("mempalace sync failed: {e:#}"))?;

    // New drawer_ids, in the same order as plan.items (one Add each).
    let new_ids: Vec<String> = results
        .into_iter()
        .filter_map(|r| match r {
            memorywhale_core::engine::SyncResult::Added { drawer_id } => Some(drawer_id),
            memorywhale_core::engine::SyncResult::Deleted => None,
        })
        .collect();
    if new_ids.len() != plan.items.len() {
        return Err(format!(
            "mempalace returned {} drawer id(s) for {} add(s)",
            new_ids.len(),
            plan.items.len()
        ));
    }

    let synced_at = Utc::now().to_rfc3339();
    let rows: Vec<(i64, String, String, String)> = plan
        .items
        .iter()
        .zip(&new_ids)
        .map(|(it, id)| (it.mw_id, it.wing.clone(), id.clone(), it.content_hash.clone()))
        .collect();
    memorywhale_cli::record_sync(&mut conn, &rows, &synced_at)?;

    println!(
        "Synced to MemPalace (wing {wing:?}): {} added, {} updated, {} unchanged, {} deleted.",
        plan.added(),
        plan.updated(),
        plan.unchanged,
        plan.updated()
    );
    Ok(())
}

/// Render one memory into its MemPalace drawer content + `added_by`. Prefixes a
/// stable `[memorywhale <source> #<realid>]` provenance tag; for notes it also
/// appends the human-readable provenance line and attributes `added_by` to the
/// note's author.
fn drawer_content(
    conn: &Connection,
    m: &memorywhale_core::Memory,
    source: memorywhale_core::sqlite::Source,
    real_id: i64,
) -> (String, String) {
    let tag = format!("[memorywhale {} #{}]", source.tag(), real_id);
    let mut content = format!("{tag}\n{}", m.text);
    let mut added_by = "memorywhale".to_string();
    if source == memorywhale_core::sqlite::Source::Note {
        if let Some((label, author)) = note_meta(conn, real_id) {
            content.push_str(&format!("\n({label})"));
            added_by = author;
        }
    }
    (content, added_by)
}

/// A note's provenance label plus its `added_by` attribution (author name, else
/// author kind). `None` if the row is gone or predates the provenance columns.
fn note_meta(conn: &Connection, id: i64) -> Option<(String, String)> {
    conn.query_row(
        "SELECT author_kind, author_name, created_at, source_session_id
         FROM bookmarks WHERE id = ?1",
        params![id],
        |r| {
            let kind: String = r.get(0)?;
            let name: Option<String> = r.get(1)?;
            let created: String = r.get::<_, Option<String>>(2)?.unwrap_or_default();
            let sid: Option<i64> = r.get(3)?;
            let label = memorywhale_cli::provenance_label(&kind, name.as_deref(), &created, sid);
            let added_by = name.filter(|n| !n.is_empty()).unwrap_or(kind);
            Ok((label, added_by))
        },
    )
    .ok()
}

fn search_memory(args: &[String]) -> Result<(), String> {
    let (scope, args) = Scope::take(args)?;
    let explain = args.iter().any(|a| a == "--explain");
    let terms: Vec<&String> = args.iter().filter(|a| a.as_str() != "--explain").collect();
    if terms.is_empty() {
        return Err(
            "usage: mw search <text> [--explain] [--project X] [--machine Y] [--since 7d]"
                .to_string(),
        );
    }
    let query = terms.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ");

    // External semantic engine (MemPalace), if the user opted in via config. It
    // ranks server-side over its own corpus, so the local scope filters don't
    // apply. If the server is unreachable we print a notice and fall through to
    // the builtin engine over local memory, so a misconfig never hard-fails.
    if let Some(argv) = memorywhale_cli::mempalace_command() {
        let (cmd, rest) = argv.split_first().expect("mempalace_command is non-empty");
        let eng = memorywhale_core::engine::MemPalaceEngine::new(cmd.clone(), rest.to_vec())
            .with_tool(memorywhale_cli::mempalace_search_tool());
        match eng.try_retrieve(&memorywhale_core::Query::new(&query, Utc::now()), 20) {
            Ok(hits) => {
                print_external_hits(&query, &hits, explain);
                return Ok(());
            }
            Err(e) => {
                eprintln!("mw: mempalace unavailable ({e:#}); using the local engine");
            }
        }
    }

    // Opening the db migrates it, so the provenance and scope columns exist
    // before the loader reads them; the loader also skips unapproved notes.
    let conn = open_session_db()?;

    // One loader, one engine — the same code path the desktop Recall panel uses.
    // "now" is supplied here by the caller so scoring stays deterministic.
    let now = Utc::now();
    let mems = memorywhale_core::sqlite::load_memories(&conn);
    // No flags => `mems` comes back untouched, i.e. exactly the old behaviour.
    let mems = memorywhale_cli::scope_memories(
        &conn,
        mems,
        scope.project.as_deref(),
        scope.machine.as_deref(),
        scope.cutoff(now),
    );
    let engine = memorywhale_core::engine::BuiltinEngine::new(mems);
    let mut q = memorywhale_core::Query::new(&query, now);
    let tags = scope.task_tags();
    if !tags.is_empty() {
        q = q.with_task(tags);
    }
    let hits = engine.retrieve(&q, 20);

    println!("# matches for {query:?}  (ranked)\n");
    if hits.is_empty() {
        println!("(none)");
        return Ok(());
    }
    for sm in &hits {
        let (source, real_id) = memorywhale_core::sqlite::decode_id(sm.memory.id);
        let action = match source {
            memorywhale_core::sqlite::Source::Command => format!("  — `mw replay {real_id}`"),
            memorywhale_core::sqlite::Source::Session => format!("  — `mw show {real_id}`"),
            _ => String::new(),
        };
        let snippet = sm
            .memory
            .text
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim();
        let snippet: String = snippet.chars().take(100).collect();
        // Only remembered notes carry provenance — who wrote this lesson, and when.
        let prov = match source {
            memorywhale_core::sqlite::Source::Note => note_provenance(&conn, real_id)
                .map(|p| format!("  ({p})"))
                .unwrap_or_default(),
            _ => String::new(),
        };
        println!(
            "{:>3}%  [{}] {}{}{}",
            sm.percent(),
            source.tag(),
            snippet,
            action,
            prov
        );
        if explain {
            // Full per-signal breakdown + reasons (reuses the engine's view).
            for line in sm.explain().lines() {
                println!("      {line}");
            }
            println!();
        }
    }
    Ok(())
}

/// One recognizable git failure shape: how to spot it (substrings to look for
/// in the captured stderr/stdout, matched case-insensitively), what it means,
/// the fix, and the terms to use when checking whether this has happened
/// before.
struct GitPattern {
    label: &'static str,
    matches: &'static [&'static str],
    explain: &'static str,
    fix: &'static [&'static str],
    search_terms: &'static [&'static str],
}

const GIT_PATTERNS: &[GitPattern] = &[
    GitPattern {
        label: "push rejected — the remote has commits you don't have",
        matches: &["[rejected]", "non-fast-forward", "failed to push some refs"],
        explain: "Someone (or another one of your machines) pushed to this branch since you last fetched. Git refuses to push over commits you haven't seen, so nothing was lost.",
        fix: &[
            "git fetch origin",
            "git pull --rebase        # replay your commits on top of theirs (or: git pull, for a merge commit)",
            "# resolve any conflicts the rebase reports, then:",
            "git push",
        ],
        search_terms: &["non-fast-forward", "rejected"],
    },
    GitPattern {
        label: "merge conflict",
        matches: &["conflict (", "automatic merge failed", "fix conflicts and then commit"],
        explain: "Two changes touched the same lines and git couldn't reconcile them automatically. Nothing is broken — the conflicted files just need a decision.",
        fix: &[
            "git status                    # lists the conflicted files",
            "# open each one; resolve the <<<<<<< / ======= / >>>>>>> markers",
            "git add <file>                 # mark each one resolved",
            "git rebase --continue          # (or `git commit`, if this was a merge not a rebase)",
            "# stuck? `git rebase --abort` / `git merge --abort` undoes everything, no risk",
        ],
        search_terms: &["conflict", "automatic merge failed"],
    },
    GitPattern {
        label: "local changes would be overwritten (dirty working tree)",
        matches: &[
            "please commit your changes or stash them",
            "would be overwritten by",
            "your local changes to the following files would be overwritten",
        ],
        explain: "The operation (checkout/rebase/pull/merge) needs a clean working tree, but you have uncommitted edits in the way.",
        fix: &[
            "git stash                     # shelve your uncommitted changes",
            "# retry the original command",
            "git stash pop                 # bring your changes back",
        ],
        search_terms: &["commit your changes or stash them"],
    },
    GitPattern {
        label: "diverged branches — no reconcile strategy configured",
        matches: &["divergent branches", "need to specify how to reconcile"],
        explain: "Your branch and its remote have both moved since the last common commit, and git wants to know whether to merge or rebase before it'll pull.",
        fix: &[
            "git config --global pull.rebase true   # make `git pull` always rebase (recommended)",
            "git pull",
        ],
        search_terms: &["divergent branches", "reconcile"],
    },
    GitPattern {
        label: "refusing to merge unrelated histories",
        matches: &["refusing to merge unrelated histories"],
        explain: "The two branches don't share a common ancestor commit — usually from re-initializing a repo or pulling into a freshly `git init`'d directory.",
        fix: &["git pull origin <branch> --allow-unrelated-histories   # only if this merge is actually intentional"],
        search_terms: &["unrelated histories"],
    },
    GitPattern {
        label: "SSH authentication failure",
        matches: &["permission denied (publickey)", "could not read from remote repository"],
        explain: "The remote rejected your SSH key — it's missing, not loaded in the agent, or not added to your GitHub/GitLab account.",
        fix: &[
            "ssh -T git@github.com          # confirms whether auth actually works",
            "ssh-add -l                     # is your key loaded in the agent?",
            "ssh-add ~/.ssh/id_ed25519       # (or your key's path) load it",
        ],
        search_terms: &["permission denied (publickey)"],
    },
];

fn classify_git_failure(text: &str) -> Option<&'static GitPattern> {
    let hay = text.to_lowercase();
    GIT_PATTERNS
        .iter()
        .find(|p| p.matches.iter().any(|m| hay.contains(m)))
}

/// Diagnose the last failed `git` command (or a specific `command_run` id):
/// what the error means, the fix, and whether this exact class of failure has
/// come up before — in past command runs or a remembered lesson.
fn git_fix_cmd(args: &[String]) -> Result<(), String> {
    let conn = open_session_db()?;

    let row: Option<(i64, String, Option<i64>, String, String, String)> = if let Some(id_str) =
        args.first()
    {
        let id: i64 = id_str
            .parse()
            .map_err(|_| format!("invalid id {id_str:?}"))?;
        match conn.query_row(
            "SELECT id, argv_json, exit_code, stdout, stderr, created_at FROM command_runs WHERE id = ?1",
            params![id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            },
        ) {
            Ok(r) => Some(r),
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err(format!("no command run #{id}")),
            Err(e) => return Err(format!("failed to read command run: {e}")),
        }
    } else {
        conn.query_row(
            "SELECT id, argv_json, exit_code, stdout, stderr, created_at FROM command_runs
             WHERE command LIKE 'git%' AND exit_code IS NOT NULL AND exit_code != 0
             ORDER BY id DESC LIMIT 1",
            [],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            },
        )
        .ok()
    };

    let Some((id, argv_json, exit_code, stdout, stderr, created_at)) = row else {
        println!(
            "mw: no recent failed git commands found.\n\
             Give a specific id with `mw git-fix <id>`, or make sure MemoryWhale is\n\
             capturing your shell — `mw doctor` checks the install, `mw global on`\n\
             turns on auto-recording."
        );
        return Ok(());
    };

    let argv: Vec<String> = serde_json::from_str(&argv_json).unwrap_or_default();
    println!(
        "# git-fix: command_run #{id} — `{}` (exit {}, {created_at})\n",
        argv.join(" "),
        exit_code.unwrap_or(-1)
    );

    let haystack = format!("{stderr}\n{stdout}");
    match classify_git_failure(&haystack) {
        Some(pattern) => {
            println!("## {}\n", pattern.label);
            println!("{}\n", pattern.explain);
            println!("Fix:");
            for step in pattern.fix {
                println!("  {step}");
            }

            // Has this exact class of failure shown up before (or been solved)?
            let like_params: Vec<String> =
                pattern.search_terms.iter().map(|term| format!("%{term}%")).collect();
            let cmd_clauses: Vec<String> = (1..=like_params.len())
                .map(|i| format!("(stderr LIKE ?{i} OR stdout LIKE ?{i})"))
                .collect();
            let cmd_sql = format!(
                "SELECT id, created_at FROM command_runs
                 WHERE command LIKE 'git%' AND id != {id} AND ({})
                 ORDER BY id DESC LIMIT 3",
                cmd_clauses.join(" OR ")
            );
            let mut past_ids = Vec::new();
            if let Ok(mut stmt) = conn.prepare(&cmd_sql) {
                if let Ok(rows) = stmt.query_map(rusqlite::params_from_iter(like_params.iter()), |r| {
                    Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
                }) {
                    past_ids.extend(rows.flatten());
                }
            }

            let bm_clauses: Vec<String> = (1..=like_params.len())
                .map(|i| format!("label LIKE ?{i}"))
                .collect();
            let bm_sql = format!(
                "SELECT label, created_at FROM bookmarks WHERE {} ORDER BY id DESC LIMIT 3",
                bm_clauses.join(" OR ")
            );
            let mut lessons = Vec::new();
            if let Ok(mut stmt) = conn.prepare(&bm_sql) {
                if let Ok(rows) = stmt.query_map(rusqlite::params_from_iter(like_params.iter()), |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                }) {
                    lessons.extend(rows.flatten());
                }
            }

            println!();
            if !lessons.is_empty() {
                println!("You've saved a lesson about this before:");
                for (label, at) in &lessons {
                    println!("  - ({at}): {label}");
                }
            } else if !past_ids.is_empty() {
                println!(
                    "You've hit this before ({} time(s)) but never saved the fix:",
                    past_ids.len()
                );
                for (pid, at) in &past_ids {
                    println!("  - command_run #{pid} ({at}) — `mw git-fix {pid}` to see it, `mw show {pid}` for detail");
                }
            } else {
                println!("First time this has shown up in your recorded memory.");
            }
            println!("\nOnce it's resolved, save the fix so this is instant next time:");
            println!("  mw remember \"<what actually fixed it>\"");
        }
        None => {
            println!(
                "Didn't recognize this failure pattern. Raw error:\n\n  {}\n",
                stderr
                    .lines()
                    .rev()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or(stderr.trim())
                    .trim()
            );
            println!("`git status` is usually the fastest way to see what git wants next.");
            println!("Once you've fixed it, save it: `mw remember \"<what fixed it>\"` — the next unrecognized case gets easier to add to `mw git-fix` too.");
        }
    }
    Ok(())
}

/// Export a recorded session as agent-ready Markdown on stdout — pipe it to a
/// file or the clipboard and paste it into an AI agent later. With an id it
/// exports that session; otherwise the most recent one. Also appends a few
/// recent failures for context.
fn agent_cmd(args: &[String]) -> Result<(), String> {
    let conn = open_session_db()?;

    let session = if let Some(id_str) = args.first() {
        let id: i64 = id_str
            .parse()
            .map_err(|_| format!("invalid session id {id_str:?}"))?;
        match conn.query_row(
            "SELECT id, notes, started_at, transcript FROM sessions WHERE id = ?1",
            params![id],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            },
        ) {
            Ok(row) => Some(row),
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err(format!("no session #{id}")),
            Err(e) => return Err(format!("failed to read session: {e}")),
        }
    } else {
        conn.query_row(
            "SELECT id, notes, started_at, transcript FROM sessions
             ORDER BY started_at DESC LIMIT 1",
            [],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            },
        )
        .ok()
    };

    // Everything below goes to stdout so `mw agent | pbcopy` / `> file.md` is clean.
    println!("# Terminal history from MemoryWhale\n");
    println!(
        "Below is recorded terminal activity — commands, output, and errors. \
         Please help me understand or debug what happened.\n"
    );

    if let Some((id, notes, started_at, transcript)) = session {
        println!("## Session #{id} — {started_at}");
        if !notes.trim().is_empty() {
            println!("notes: {}", notes.trim());
        }
        println!("\n```text\n{}\n```\n", transcript.trim());
    } else {
        println!("_(no recorded sessions yet)_\n");
    }

    // A few recent failures for extra context.
    let mut stmt = conn
        .prepare(
            "SELECT argv_json, exit_code, stderr, created_at FROM command_runs
             WHERE exit_code IS NOT NULL AND exit_code != 0
             ORDER BY id DESC LIMIT 5",
        )
        .map_err(|err| format!("failed to prepare command query: {err}"))?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<i64>>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(|err| format!("failed to read command runs: {err}"))?;
    let mut any = false;
    for row in rows {
        let (argv_json, code, stderr, at) = row.map_err(|err| format!("row error: {err}"))?;
        if !any {
            println!("## Recent failed commands\n");
            any = true;
        }
        let argv: Vec<String> = serde_json::from_str(&argv_json).unwrap_or_default();
        println!("- `{}` (exit {}, {})", argv.join(" "), code.unwrap_or(-1), at);
        let tail = stderr
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim();
        if !tail.is_empty() {
            println!("  ```\n  {tail}\n  ```");
        }
    }

    eprintln!("\nmw: tip — `mw agent > session.md` to save, or `mw agent | pbcopy` to copy.");
    Ok(())
}

/// Copy text to the system clipboard via whichever tool exists. Returns false
/// if none is available (caller prints the payload instead).
fn copy_to_clipboard(text: &str) -> bool {
    use std::io::Write as _;
    for (cmd, args) in [
        ("pbcopy", &[][..]),
        ("xclip", &["-selection", "clipboard"][..]),
        ("wl-copy", &[][..]),
    ] {
        let child = Command::new(cmd)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        if let Ok(mut child) = child {
            if let Some(stdin) = child.stdin.as_mut() {
                if stdin.write_all(text.as_bytes()).is_ok() {
                    if matches!(child.wait(), Ok(s) if s.success()) {
                        return true;
                    }
                    continue;
                }
            }
            let _ = child.wait();
        }
    }
    false
}

/// Map a chat target name to its URL. Accepts a full http(s) URL verbatim, so
/// any chat provider works: `mw ask --chat https://example.com/chat`.
fn resolve_chat_url(spec: &str) -> String {
    match spec.to_lowercase().as_str() {
        "chatgpt" | "gpt" | "openai" => "https://chatgpt.com".to_string(),
        "claude" => "https://claude.ai/new".to_string(),
        "gemini" => "https://gemini.google.com/app".to_string(),
        s if s.starts_with("http://") || s.starts_with("https://") => spec.to_string(),
        other => {
            eprintln!("mw: unknown chat {other:?} (know: chatgpt, claude, gemini, or a URL) — using chatgpt");
            "https://chatgpt.com".to_string()
        }
    }
}

fn open_url(url: &str) {
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    let _ = Command::new(opener)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

/// Package the most recent failure — exact error, similar past failures, saved
/// lessons — into one debugging prompt on the clipboard, and open a chat. The
/// "bring your own AI" bridge: works with the user's ChatGPT/Claude/Gemini
/// subscription, no API key; the human paste is the wire.
fn ask_cmd(args: &[String]) -> Result<(), String> {
    let mut include_session = false;
    let mut no_open = false;
    // Which chat to open: --chat flag beats the MEMORYWHALE_CHAT env default.
    let mut chat = std::env::var("MEMORYWHALE_CHAT").unwrap_or_else(|_| "chatgpt".to_string());
    let mut question_words: Vec<String> = Vec::new();
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        match a.as_str() {
            "--session" => include_session = true,
            "--no-open" => no_open = true,
            "--chat" => {
                chat = iter
                    .next()
                    .cloned()
                    .ok_or_else(|| "--chat needs a value (chatgpt, claude, gemini, or a URL)".to_string())?;
            }
            other if other.starts_with("--") => {
                return Err(format!(
                    "unknown option {other:?}; usage: mw ask [question] [--chat chatgpt|claude|gemini|URL] [--session] [--no-open]"
                ))
            }
            other => question_words.push(other.to_string()),
        }
    }
    let chat_url = resolve_chat_url(&chat);
    let question = question_words.join(" ");
    let conn = open_session_db()?;

    // Last non-empty line of a text, char-capped.
    let tail = |text: &str, max: usize| -> String {
        let t = text
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim();
        let chars: Vec<char> = t.chars().collect();
        if chars.len() > max {
            format!("…{}", chars[chars.len() - max..].iter().collect::<String>())
        } else {
            t.to_string()
        }
    };
    // Cap a whole blob to its last `max` chars (keep the end — errors live there).
    let cap_blob = |text: &str, max: usize| -> String {
        let chars: Vec<char> = text.trim().chars().collect();
        if chars.len() > max {
            format!("…{}", chars[chars.len() - max..].iter().collect::<String>())
        } else {
            text.trim().to_string()
        }
    };

    // The most recent failed command.
    let failure = conn
        .query_row(
            "SELECT id, argv_json, cwd, exit_code, stderr, stdout, notes, created_at
             FROM command_runs
             WHERE exit_code IS NOT NULL AND exit_code != 0
             ORDER BY id DESC LIMIT 1",
            [],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                ))
            },
        )
        .ok();

    if failure.is_none() && question.is_empty() {
        return Err(
            "no failed commands in memory and no question given.\n\
             Usage: mw ask [question]. Record failures with auto-record (`mw global on`)\n\
             or `mw-run -- <command>`, then `mw ask` packages the latest one."
                .to_string(),
        );
    }

    // Words to find related history/lessons: from the error tail + the question.
    let mut terms: Vec<String> = Vec::new();
    if let Some((_, _, _, _, stderr, _, _, _)) = &failure {
        terms.extend(
            tail(stderr, 200)
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() >= 4)
                .take(6)
                .map(|w| w.to_lowercase()),
        );
    }
    terms.extend(
        question
            .split_whitespace()
            .filter(|w| w.len() >= 4)
            .take(4)
            .map(|w| w.to_lowercase()),
    );
    terms.dedup();

    // Similar past failures (FTS if available, else LIKE on the first term).
    let mut similar: Vec<(String, Option<i64>, String, Option<String>, String)> = Vec::new();
    let exclude_id = failure.as_ref().map(|f| f.0).unwrap_or(-1);
    if !terms.is_empty() {
        let _ = memorywhale_cli::ensure_fts(&conn);
        let collect = |sql: &str, param: &str| -> Vec<(String, Option<i64>, String, Option<String>, String)> {
            let mut out = Vec::new();
            if let Ok(mut stmt) = conn.prepare(sql) {
                if let Ok(rows) = stmt.query_map(params![param, exclude_id], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, Option<i64>>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, Option<String>>(3)?,
                        r.get::<_, String>(4)?,
                    ))
                }) {
                    out.extend(rows.flatten());
                }
            }
            out
        };
        // OR-match any term: one strong term is enough to surface history.
        let fts_or = terms
            .iter()
            .map(|t| format!("\"{}\"*", t.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" OR ");
        similar = collect(
            "SELECT argv_json, exit_code, stderr, cwd, created_at FROM command_runs
             WHERE exit_code IS NOT NULL AND exit_code != 0 AND id != ?2
               AND id IN (SELECT rowid FROM command_fts WHERE command_fts MATCH ?1)
             ORDER BY id DESC LIMIT 3",
            &fts_or,
        );
        if similar.is_empty() {
            if let Some(t) = terms.first() {
                similar = collect(
                    "SELECT argv_json, exit_code, stderr, cwd, created_at FROM command_runs
                     WHERE exit_code IS NOT NULL AND exit_code != 0 AND id != ?2
                       AND (stderr LIKE ?1 OR stdout LIKE ?1)
                     ORDER BY id DESC LIMIT 3",
                    &format!("%{t}%"),
                );
            }
        }
    }

    // Saved lessons matching any term.
    let mut lessons: Vec<(String, String)> = Vec::new();
    for t in &terms {
        if lessons.len() >= 3 {
            break;
        }
        if let Ok(mut stmt) = conn.prepare(
            "SELECT label, created_at FROM bookmarks WHERE label LIKE ?1 ORDER BY id DESC LIMIT 3",
        ) {
            if let Ok(rows) =
                stmt.query_map(params![format!("%{t}%")], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            {
                for row in rows.flatten() {
                    if !lessons.iter().any(|(l, _)| l == &row.0) {
                        lessons.push(row);
                    }
                }
            }
        }
    }
    lessons.truncate(3);

    // When did this exact command last succeed?
    let last_success: Option<String> = failure.as_ref().and_then(|f| {
        conn.query_row(
            "SELECT created_at FROM command_runs
             WHERE argv_json = ?1 AND exit_code = 0 ORDER BY id DESC LIMIT 1",
            params![f.1],
            |r| r.get(0),
        )
        .ok()
    });

    // ── assemble the prompt ──────────────────────────────────────────────
    let mut p = String::from(
        "Help me debug a terminal failure. Below is the failing command, its exact\n\
         output, and relevant history from my local memory (past attempts + lessons).\n\n",
    );
    if let Some((_, argv_json, cwd, exit_code, stderr, stdout, notes, created_at)) = &failure {
        let argv: Vec<String> = serde_json::from_str(argv_json).unwrap_or_default();
        p.push_str("## The failure (just now)\n");
        p.push_str(&format!("Command:   {}\n", argv.join(" ")));
        p.push_str(&format!("Directory: {}\n", cwd.clone().unwrap_or_default()));
        let tags: Vec<&str> = notes
            .split_whitespace()
            .filter(|w| w.contains(':') && !w.starts_with("http"))
            .collect();
        if !tags.is_empty() {
            p.push_str(&format!("Machine:   {}\n", tags.join(" ")));
        }
        p.push_str(&format!("Exit code: {}\n", exit_code.unwrap_or(-1)));
        p.push_str(&format!("When:      {created_at}\n\n"));
        let err_blob = if stderr.trim().is_empty() { stdout } else { stderr };
        p.push_str(&format!("```\n{}\n```\n\n", cap_blob(err_blob, 4000)));
    }
    if !similar.is_empty() {
        p.push_str("## I've hit similar errors before\n");
        for (argv_json, code, stderr, cwd, at) in &similar {
            let argv: Vec<String> = serde_json::from_str(argv_json).unwrap_or_default();
            p.push_str(&format!(
                "- `{}` (exit {}, {}, cwd {})\n  err: {}\n",
                argv.join(" "),
                code.unwrap_or(-1),
                at,
                cwd.clone().unwrap_or_default(),
                tail(stderr, 200)
            ));
        }
        p.push('\n');
    }
    if !lessons.is_empty() {
        p.push_str("## Lessons I saved from past fixes\n");
        for (label, at) in &lessons {
            p.push_str(&format!("- ({at}): {label}\n"));
        }
        p.push('\n');
    }
    if let Some(ts) = &last_success {
        p.push_str("## What I've established\n");
        p.push_str(&format!("- This exact command last succeeded on {ts}\n\n"));
    }
    if include_session {
        if let Ok((id, transcript)) = conn.query_row(
            "SELECT id, transcript FROM sessions ORDER BY started_at DESC LIMIT 1",
            [],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
        ) {
            let lines: Vec<&str> = transcript.lines().collect();
            let tail_lines = lines.iter().rev().take(60).rev().cloned().collect::<Vec<_>>().join("\n");
            p.push_str(&format!(
                "## Tail of my current session (#{id})\n```\n{}\n```\n\n",
                cap_blob(&tail_lines, 3000)
            ));
        }
    }
    if !question.is_empty() {
        p.push_str(&format!("## My question\n{question}\n\n"));
    }
    p.push_str(
        "Please: identify the root cause, tell me if this matches a saved lesson\n\
         above, and give the exact fix.\n",
    );

    // ── deliver ──────────────────────────────────────────────────────────
    let approx_tokens = p.chars().count() / 4;
    eprintln!(
        "mw: packaged {}{} similar failure(s), {} saved lesson(s){}",
        if failure.is_some() { "the last failure, " } else { "" },
        similar.len(),
        lessons.len(),
        if include_session { ", + session tail" } else { "" }
    );
    if copy_to_clipboard(&p) {
        eprintln!("mw: ~{approx_tokens} tokens copied to clipboard — paste at {chat_url} (Cmd-V)");
        if !no_open {
            eprintln!("mw: opening {chat_url} …");
            open_url(&chat_url);
        }
    } else {
        eprintln!("mw: no clipboard tool found (pbcopy/xclip/wl-copy) — printing instead:\n");
        println!("{p}");
    }
    Ok(())
}

/// Print a compact, token-budgeted digest of recent memory for an AI agent to
/// read: recent failed commands (with short error tails) and recent sessions.
fn context_cmd(args: &[String]) -> Result<(), String> {
    let mut project: Option<String> = None;
    let mut last_error = false;
    let mut limit: i64 = 8;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--last-error" => last_error = true,
            "--limit" => {
                limit = iter
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or_else(|| "--limit requires a number".to_string())?;
            }
            other if other.starts_with("project:") => project = Some(other.to_string()),
            other => return Err(format!("unexpected argument {other:?}; run mw --help")),
        }
    }
    let like = project.as_ref().map(|p| format!("%{p}%"));
    let conn = open_session_db()?;

    // One-line tail of the most useful error text, length-capped for token budget.
    let tail = |text: &str, max: usize| -> String {
        let t = text.trim();
        let t = t
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or(t)
            .trim();
        let chars: Vec<char> = t.chars().collect();
        if chars.len() > max {
            format!("…{}", chars[chars.len() - max..].iter().collect::<String>())
        } else {
            t.to_string()
        }
    };

    if let Some(p) = &project {
        println!("# MemoryWhale context ({p})\n");
    } else {
        println!("# MemoryWhale context\n");
    }

    // Failed commands are the highest-signal thing for a debugging agent.
    let mut stmt = conn
        .prepare(
            "SELECT argv_json, cwd, exit_code, stderr, notes, created_at
             FROM command_runs
             WHERE (exit_code IS NOT NULL AND exit_code != 0)
               AND (?1 IS NULL OR notes LIKE ?1)
             ORDER BY id DESC LIMIT ?2",
        )
        .map_err(|err| format!("failed to prepare context query: {err}"))?;
    let rows = stmt
        .query_map(params![like.as_deref(), if last_error { 1 } else { limit }], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        })
        .map_err(|err| format!("failed to read command runs: {err}"))?;

    let mut any = false;
    println!("## Recent failed commands");
    for row in rows {
        let (argv_json, cwd, exit_code, stderr, notes, created_at) =
            row.map_err(|err| format!("row error: {err}"))?;
        let argv: Vec<String> = serde_json::from_str(&argv_json).unwrap_or_default();
        any = true;
        println!(
            "- `{}` (exit {}, {})\n  cwd: {}\n  err: {}{}",
            argv.join(" "),
            exit_code.unwrap_or(-1),
            created_at,
            cwd.unwrap_or_default(),
            tail(&stderr, 200),
            if notes.trim().is_empty() {
                String::new()
            } else {
                format!("\n  note: {}", tail(&notes, 160))
            }
        );
    }
    if !any {
        println!("(none)");
    }

    if last_error {
        // Outcome history for the single failure we just printed: how often this
        // exact error has recurred, and whether it ever resolved. This is the
        // evidence-grounded answer to "have I seen this before, and did the fix
        // work" — computed from observed exit codes, not remembered claims.
        if let Ok(Some((command, stderr))) = conn.query_row(
            "SELECT command, stderr FROM command_runs
             WHERE exit_code IS NOT NULL AND exit_code != 0
             ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok(Some((r.get::<_, String>(0)?, r.get::<_, String>(1)?))),
        ) {
            if let Some(fp) = memorywhale_cli::error_fingerprint(&command, &stderr) {
                if let Ok(insight) = memorywhale_cli::error_insight(&conn, &fp) {
                    println!("\n## This error");
                    println!("{}", insight.summary());
                    if insight.occurrences > 1 && insight.resolutions == 0 {
                        println!("No recorded fix yet — once you resolve it, run `mw remember \"<the fix>\"`.");
                    }
                }
            }
        }
        return Ok(());
    }

    // A few recent sessions, for the "what was I doing" picture.
    let mut stmt = conn
        .prepare(
            "SELECT id, notes, started_at, byte_count
             FROM sessions
             WHERE ?1 IS NULL OR notes LIKE ?1
             ORDER BY id DESC LIMIT ?2",
        )
        .map_err(|err| format!("failed to prepare session query: {err}"))?;
    let rows = stmt
        .query_map(params![like.as_deref(), limit], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })
        .map_err(|err| format!("failed to read sessions: {err}"))?;
    println!("\n## Recent sessions");
    let mut any = false;
    for row in rows {
        let (id, notes, started_at, byte_count) = row.map_err(|err| format!("row error: {err}"))?;
        any = true;
        println!(
            "- #{id} {started_at} ({byte_count} bytes){}  — replay with `mw show {id}`",
            if notes.trim().is_empty() {
                String::new()
            } else {
                format!(": {}", tail(&notes, 160))
            }
        );
    }
    if !any {
        println!("(none)");
    }

    // Remembered lessons (`mw mark` / `mw remember`) — conclusions worth an
    // agent seeing before it re-derives them from scratch.
    let mut stmt = conn
        .prepare(
            "SELECT label, created_at FROM bookmarks
             WHERE ?1 IS NULL OR label LIKE ?1
             ORDER BY id DESC LIMIT ?2",
        )
        .map_err(|err| format!("failed to prepare notes query: {err}"))?;
    let rows = stmt
        .query_map(params![like.as_deref(), limit], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(|err| format!("failed to read notes: {err}"))?;
    println!("\n## Remembered lessons");
    let mut any = false;
    for row in rows {
        let (label, created_at) = row.map_err(|err| format!("row error: {err}"))?;
        any = true;
        println!("- {created_at}: {}", tail(&label, 200));
    }
    if !any {
        println!("(none)");
    }
    Ok(())
}

/// Self-check the install so a confused user (or agent) can see what's wrong.
fn doctor() -> Result<(), String> {
    let ok = |label: &str, detail: String| println!("  ok   {label}: {detail}");
    let warn = |label: &str, detail: String| println!("  WARN {label}: {detail}");

    println!("MemoryWhale doctor\n");

    // Data dir writable?
    match memorywhale_dir() {
        Ok(dir) => {
            let writable = fs::create_dir_all(&dir).is_ok()
                && {
                    let probe = dir.join(".doctor-write-test");
                    let r = fs::write(&probe, b"ok").is_ok();
                    let _ = fs::remove_file(&probe);
                    r
                };
            if writable {
                ok("data dir", dir.display().to_string());
            } else {
                warn("data dir", format!("{} (not writable)", dir.display()));
            }
        }
        Err(err) => warn("data dir", err),
    }

    // Database opens + row counts.
    match open_session_db() {
        Ok(conn) => {
            let count = |table: &str| -> i64 {
                conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                    .unwrap_or(-1)
            };
            ok(
                "database",
                format!(
                    "{} sessions, {} command runs, {} bookmarks",
                    count("sessions"),
                    count("command_runs"),
                    count("bookmarks")
                ),
            );
        }
        Err(err) => warn("database", err),
    }

    // `script` is required for `mw` session recording.
    match Command::new("script").arg("--version").output() {
        Ok(_) => ok("recording", "`script` is available".to_string()),
        Err(_) => warn(
            "recording",
            "`script` not found — session recording needs util-linux/bsdutils `script`".to_string(),
        ),
    }

    // Global hook status.
    let enabled = global_enabled_path().map(|p| p.exists()).unwrap_or(false);
    let wired = shell_rc_path()
        .ok()
        .and_then(|rc| fs::read_to_string(rc).ok())
        .map(|c| c.contains(RC_MARKER))
        .unwrap_or(false);
    if enabled && wired {
        ok("auto-record", "on and wired into your shell".to_string());
    } else {
        warn(
            "auto-record",
            format!(
                "off (enabled: {enabled}, wired: {wired}) — run `mw global on` to enable"
            ),
        );
    }

    Ok(())
}

fn append_environment_tags(notes: String) -> String {
    let mut tags = Vec::new();
    tags.push(format!("os:{}", env::consts::OS));
    if PathBuf::from("/.dockerenv").exists() || env::var_os("container").is_some() {
        tags.push("runtime:container".to_string());
    } else {
        tags.push("runtime:host".to_string());
    }
    if env::var_os("SSH_CONNECTION").is_some() || env::var_os("SSH_CLIENT").is_some() {
        tags.push("session:ssh".to_string());
    }
    if PathBuf::from("/etc/nv_tegra_release").exists() {
        tags.push("host:jetson".to_string());
    }

    if notes.trim().is_empty() {
        tags.join(" ")
    } else {
        format!("{} {}", notes.trim(), tags.join(" "))
    }
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
