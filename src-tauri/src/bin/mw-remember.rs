use chrono::Utc;
use rusqlite::{params, Connection};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    if let Err(err) = run() {
        eprintln!("mw-remember: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut cwd: Option<String> = env::current_dir()
        .ok()
        .and_then(|path| path.to_str().map(ToOwned::to_owned));
    let mut exit_code: Option<i64> = None;
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut notes = String::new();
    let mut command_parts = Vec::new();

    let mut args = env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--cwd" => cwd = args.next(),
            "--exit-code" | "--exit" => {
                exit_code = args.next().and_then(|value| value.parse::<i64>().ok());
            }
            "--stdout" => stdout = args.next().unwrap_or_default(),
            "--stderr" => stderr = args.next().unwrap_or_default(),
            "--notes" => notes = args.next().unwrap_or_default(),
            "--" => {
                command_parts.extend(args);
                break;
            }
            value if value.starts_with("--") => {
                return Err(format!("unknown option {value:?}; run mw-remember --help"));
            }
            value => command_parts.push(value.to_string()),
        }
    }

    if command_parts.is_empty() {
        return Err("missing command; pass it after --".to_string());
    }

    let command = command_parts[0].clone();
    let argv_json = serde_json::to_string(&command_parts)
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
        params![command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at],
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

    println!("remembered command run #{run_id}");
    Ok(())
}

fn print_help() {
    println!(
        "mw-remember --cwd <path> --exit-code <code> --stdout <text> --stderr <text> --notes <text> -- <command> [args...]"
    );
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;

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
