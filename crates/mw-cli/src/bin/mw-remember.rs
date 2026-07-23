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
    let mut capture_kind = "full".to_string();

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
            "--capture-kind" => capture_kind = args.next().unwrap_or_else(|| "full".to_string()),
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

    // Capture gate: decided before the database is even opened, so an `off`
    // directory never produces a row.
    let gate = memorywhale_cli::capture_rule_for(cwd.as_deref());
    if !gate.mode.stores_anything() {
        return Ok(());
    }
    if !gate.mode.stores_output() {
        stdout.clear();
        stderr.clear();
    }

    notes = append_environment_tags(notes);
    let command = command_parts[0].clone();
    let argv_json = serde_json::to_string(&command_parts)
        .map_err(|err| format!("failed to encode argv: {err}"))?;
    let created_at = Utc::now().to_rfc3339();
    let db_path = database_path()?;
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("failed to create data dir: {err}"))?;
    }

    let conn = open_ready(&db_path)?;
    memorywhale_cli::restrict_path_permissions(&db_path, false)?;
    conn.execute(
        "
        INSERT INTO command_runs (command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at, capture_kind)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ",
        params![
            command,
            argv_json,
            cwd,
            exit_code,
            memorywhale_cli::sanitize_capture(&stdout),
            memorywhale_cli::sanitize_capture(&stderr),
            memorywhale_cli::sanitize_capture(&notes),
            created_at,
            capture_kind
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

    println!("remembered command run #{run_id}");
    Ok(())
}

/// Open the database and make sure the schema is usable.
///
/// Shell hooks fire one writer per command, so several can be creating or
/// upgrading a brand-new database at the same instant. Schema initialization
/// can briefly lose a race even with the shared busy timeout.
/// Retry briefly rather than dropping the row.
fn open_ready(db_path: &std::path::Path) -> Result<Connection, String> {
    let mut last = String::new();
    for attempt in 0..5 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(80 * attempt));
        }
        match memorywhale_cli::storage::open_path(db_path) {
            Ok(conn) => return Ok(conn),
            Err(err) => last = err,
        }
    }
    Err(last)
}

fn print_help() {
    println!(
        "mw-remember --cwd <path> --exit-code <code> --stdout <text> --stderr <text> --notes <text> --capture-kind <full|hook> -- <command> [args...]"
    );
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

fn database_path() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("MEMORYWHALE_DATA_DIR") {
        return Ok(PathBuf::from(path).join("memorywhale.sqlite3"));
    }

    let base = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "could not resolve local data directory".to_string())?;
    Ok(base.join("MemoryWhale").join("memorywhale.sqlite3"))
}
