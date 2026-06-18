use chrono::Utc;
use rusqlite::{params, Connection};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;

fn main() {
    match run() {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(err) => {
            eprintln!("mw: {err}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<i32, String> {
    let mut cwd = env::current_dir().map_err(|err| format!("failed to read cwd: {err}"))?;
    let mut notes = String::new();
    let mut command_parts = Vec::new();

    let mut args = env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(0);
            }
            "--cwd" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--cwd requires a path".to_string())?;
                cwd = PathBuf::from(value);
            }
            "--notes" => {
                notes = args
                    .next()
                    .ok_or_else(|| "--notes requires text".to_string())?;
            }
            "--" => {
                command_parts.extend(args);
                break;
            }
            value if value.starts_with("--") => {
                return Err(format!("unknown option {value:?}; run mw --help"));
            }
            value => {
                command_parts.push(value.to_string());
                command_parts.extend(args);
                break;
            }
        }
    }

    if command_parts.is_empty() {
        return Err("missing command; run mw --help".to_string());
    }

    let command = command_parts[0].clone();
    let args = &command_parts[1..];
    let output = run_and_capture(&command, args, &cwd)?;
    let run_id = remember_command(
        &command_parts,
        &cwd,
        output.exit_code,
        &output.stdout,
        &output.stderr,
        &notes,
    )?;

    eprintln!("mw: remembered command run #{run_id}");
    Ok(output.exit_code.unwrap_or(1) as i32)
}

fn run_and_capture(
    command: &str,
    args: &[String],
    cwd: &PathBuf,
) -> Result<CapturedOutput, String> {
    let mut child = Command::new(command)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("failed to start command {command:?}: {err}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "failed to capture stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "failed to capture stderr".to_string())?;

    let stdout_handle = thread::spawn(move || tee_reader(stdout, StreamKind::Stdout));
    let stderr_handle = thread::spawn(move || tee_reader(stderr, StreamKind::Stderr));

    let status = child
        .wait()
        .map_err(|err| format!("failed while waiting for command: {err}"))?;

    let stdout = stdout_handle
        .join()
        .map_err(|_| "stdout capture thread panicked".to_string())??;
    let stderr = stderr_handle
        .join()
        .map_err(|_| "stderr capture thread panicked".to_string())??;

    Ok(CapturedOutput {
        exit_code: status.code().map(i64::from),
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    })
}

fn tee_reader<R: Read>(mut reader: R, stream: StreamKind) -> Result<Vec<u8>, String> {
    let mut captured = Vec::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|err| format!("failed to read command output: {err}"))?;
        if count == 0 {
            break;
        }

        captured.extend_from_slice(&buffer[..count]);
        match stream {
            StreamKind::Stdout => {
                io::stdout()
                    .write_all(&buffer[..count])
                    .and_then(|_| io::stdout().flush())
                    .map_err(|err| format!("failed to write stdout: {err}"))?;
            }
            StreamKind::Stderr => {
                io::stderr()
                    .write_all(&buffer[..count])
                    .and_then(|_| io::stderr().flush())
                    .map_err(|err| format!("failed to write stderr: {err}"))?;
            }
        }
    }

    Ok(captured)
}

fn remember_command(
    command_parts: &[String],
    cwd: &PathBuf,
    exit_code: Option<i64>,
    stdout: &str,
    stderr: &str,
    notes: &str,
) -> Result<i64, String> {
    let command = command_parts[0].clone();
    let argv_json = serde_json::to_string(command_parts)
        .map_err(|err| format!("failed to encode argv: {err}"))?;
    let created_at = Utc::now().to_rfc3339();
    let db_path = database_path()?;
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("failed to create data dir: {err}"))?;
    }

    let conn = Connection::open(db_path).map_err(|err| format!("failed to open db: {err}"))?;
    init_schema(&conn)?;
    conn.execute(
        "
        INSERT INTO command_runs (command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ",
        params![
            command,
            argv_json,
            cwd.to_string_lossy(),
            exit_code,
            stdout,
            stderr,
            notes,
            created_at
        ],
    )
    .map_err(|err| format!("failed to insert command run: {err}"))?;
    let run_id = conn.last_insert_rowid();

    for (position, value) in command_parts.iter().enumerate() {
        conn.execute(
            "
            INSERT INTO command_arguments (command_run_id, position, value)
            VALUES (?1, ?2, ?3)
            ",
            params![run_id, position as i64, value],
        )
        .map_err(|err| format!("failed to insert argument: {err}"))?;
    }

    Ok(run_id)
}

fn print_help() {
    println!(
        "mw [--cwd <path>] [--notes <text>] -- <command> [args...]\n\
         mw runs a command, shows its output, and saves command/argv/cwd/exit code/stdout/stderr to MemoryWhale."
    );
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS command_runs (
            id INTEGER PRIMARY KEY,
            command TEXT NOT NULL,
            argv_json TEXT NOT NULL,
            cwd TEXT,
            exit_code INTEGER,
            stdout TEXT NOT NULL DEFAULT '',
            stderr TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS command_arguments (
            id INTEGER PRIMARY KEY,
            command_run_id INTEGER NOT NULL,
            position INTEGER NOT NULL,
            value TEXT NOT NULL,
            FOREIGN KEY(command_run_id) REFERENCES command_runs(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_command_runs_command ON command_runs(command);
        CREATE INDEX IF NOT EXISTS idx_command_runs_exit_code ON command_runs(exit_code);
        CREATE INDEX IF NOT EXISTS idx_command_arguments_value ON command_arguments(value);
        ",
    )
    .map_err(|err| format!("failed to initialize schema: {err}"))
}

fn database_path() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("MEMORYWHALE_DATA_DIR") {
        return Ok(PathBuf::from(path).join("memorywhale.sqlite3"));
    }

    let base = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "could not resolve local data directory".to_string())?;
    Ok(base.join("MemoryWhale").join("memorywhale.sqlite3"))
}

struct CapturedOutput {
    exit_code: Option<i64>,
    stdout: String,
    stderr: String,
}

enum StreamKind {
    Stdout,
    Stderr,
}
