// mw-screenshot: opt-in screenshot capture for MemoryWhale terminal memory.
//
// Privacy: capture only happens when you run this command (never automatic).
// Image files are stored locally under <data_local>/MemoryWhale/screenshots/
// and metadata is stored in the same SQLite DB as mw-remember
// (<data_local>/MemoryWhale/memorywhale.sqlite3). Screenshots are never uploaded.
//
// Usage:
//   mw-screenshot --notes "VS Code showed the TypeScript warning"
//   mw-screenshot --command-run-id 12 --notes "Screen after failed cargo check"

use chrono::Utc;
use rusqlite::params;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    if let Err(err) = run() {
        eprintln!("mw-screenshot: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut cwd: Option<String> = env::current_dir()
        .ok()
        .and_then(|path| path.to_str().map(ToOwned::to_owned));
    let mut command_run_id: Option<i64> = None;
    let mut notes = String::new();
    let mut output_override: Option<String> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--notes" => notes = args.next().unwrap_or_default(),
            "--command-run-id" | "--run-id" => {
                command_run_id = match args.next() {
                    Some(value) => Some(
                        value
                            .parse::<i64>()
                            .map_err(|_| format!("invalid --command-run-id {value:?}"))?,
                    ),
                    None => return Err("--command-run-id requires a value".to_string()),
                };
            }
            "--cwd" => cwd = args.next(),
            "--output" | "-o" => output_override = args.next(),
            value if value.starts_with("--") => {
                return Err(format!(
                    "unknown option {value:?}; run mw-screenshot --help"
                ));
            }
            value => {
                return Err(format!(
                    "unexpected argument {value:?}; run mw-screenshot --help"
                ));
            }
        }
    }

    let captured_at = Utc::now().to_rfc3339();

    // Open the DB and validate any link target *before* capturing, so an invalid
    // --command-run-id never triggers a screenshot of the screen.
    let db_path = database_path()?;
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("failed to create data dir: {err}"))?;
    }

    let conn = memorywhale_cli::storage::open_path(&db_path)?;

    if let Some(run_id) = command_run_id {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(1) FROM command_runs WHERE id = ?1",
                params![run_id],
                |row| row.get(0),
            )
            .map_err(|err| format!("failed to check command run: {err}"))?;
        if exists == 0 {
            return Err(format!(
                "no command run #{run_id} exists; record it first with mw-remember"
            ));
        }
    }

    let shots_dir = screenshots_dir()?;
    fs::create_dir_all(&shots_dir)
        .map_err(|err| format!("failed to create screenshots dir: {err}"))?;

    let file_path = match output_override {
        Some(path) => PathBuf::from(path),
        None => {
            // Colons are not filesystem-friendly; keep the timestamp readable.
            let stamp = captured_at.replace(':', "-");
            shots_dir.join(format!("screenshot-{stamp}.png"))
        }
    };

    capture_screenshot(&file_path)?;

    conn.execute(
        "
        INSERT INTO screenshots (command_run_id, file_path, cwd, notes, captured_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ",
        params![
            command_run_id,
            file_path.to_string_lossy(),
            cwd,
            notes,
            captured_at
        ],
    )
    .map_err(|err| format!("failed to insert screenshot: {err}"))?;
    let id = conn.last_insert_rowid();

    match command_run_id {
        Some(run_id) => println!(
            "saved screenshot #{id} (linked to command run #{run_id}) -> {}",
            file_path.display()
        ),
        None => println!("saved screenshot #{id} -> {}", file_path.display()),
    }
    Ok(())
}

fn print_help() {
    println!(
        "mw-screenshot [--command-run-id <id>] [--notes <text>] [--cwd <path>] [--output <file>]\n\
         \n\
         Captures the current screen (opt-in) and records it in MemoryWhale.\n\
         Image files: <data_local>/MemoryWhale/screenshots/\n\
         Metadata:    <data_local>/MemoryWhale/memorywhale.sqlite3 (screenshots table)\n\
         Screenshots are stored locally and never uploaded."
    );
}

/// Capture the screen to `path` using the platform's screenshot tool.
/// Returns a descriptive error (rather than panicking) when no display or
/// tool is available — e.g. a headless Jetson with no GTK display.
fn capture_screenshot(path: &Path) -> Result<(), String> {
    let path_str = path
        .to_str()
        .ok_or_else(|| "screenshot path is not valid UTF-8".to_string())?;

    // Ordered list of (tool, args) to try per OS. First one that succeeds wins.
    let attempts: Vec<(&str, Vec<String>)> = match env::consts::OS {
        "macos" => vec![("screencapture", vec!["-x".into(), path_str.into()])],
        "linux" => vec![
            ("gnome-screenshot", vec!["-f".into(), path_str.into()]),
            (
                "spectacle",
                vec!["-b".into(), "-n".into(), "-o".into(), path_str.into()],
            ),
            ("scrot", vec![path_str.into()]),
            (
                "import",
                vec!["-window".into(), "root".into(), path_str.into()],
            ),
            ("grim", vec![path_str.into()]),
        ],
        "windows" => vec![],
        _ => vec![],
    };

    if attempts.is_empty() {
        return Err(format!(
            "no screenshot tool configured for OS {:?}",
            env::consts::OS
        ));
    }

    let mut last_err = String::from("no screenshot tool available");
    for (tool, tool_args) in &attempts {
        match Command::new(tool).args(tool_args).status() {
            Ok(status) if status.success() && path.exists() => return Ok(()),
            Ok(status) => last_err = format!("{tool} exited with {status}"),
            Err(err) => last_err = format!("{tool} not available: {err}"),
        }
    }

    Err(format!(
        "screenshot capture failed ({last_err}); on a headless machine with no display this is expected — \
         record the terminal context with mw-remember instead"
    ))
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

fn screenshots_dir() -> Result<PathBuf, String> {
    Ok(memorywhale_dir()?.join("screenshots"))
}

fn database_path() -> Result<PathBuf, String> {
    Ok(memorywhale_dir()?.join("memorywhale.sqlite3"))
}
