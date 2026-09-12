use std::env;
use std::io::{self, Read};

use memorywhale_cli::agent_hook::Agent;

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
    let mut from_hook: Option<Agent> = None;
    let mut record_flags = false;

    let mut args = env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--from-hook" => {
                let name = args.next().ok_or_else(|| {
                    "mw-remember --from-hook requires claude, rho, or cursor".to_string()
                })?;
                from_hook = Some(Agent::parse(&name).ok_or_else(|| {
                    format!("unknown hook client {name:?}; use claude, rho, or cursor")
                })?);
            }
            "--cwd" => {
                record_flags = true;
                cwd = args.next();
            }
            "--exit-code" | "--exit" => {
                record_flags = true;
                exit_code = args.next().and_then(|value| value.parse::<i64>().ok());
            }
            "--stdout" => {
                record_flags = true;
                stdout = args.next().unwrap_or_default();
            }
            "--stderr" => {
                record_flags = true;
                stderr = args.next().unwrap_or_default();
            }
            "--notes" => {
                record_flags = true;
                notes = args.next().unwrap_or_default();
            }
            "--capture-kind" => {
                record_flags = true;
                capture_kind = args.next().unwrap_or_else(|| "full".to_string());
            }
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

    if let Some(agent) = from_hook {
        if record_flags || !command_parts.is_empty() {
            return Err("mw-remember --from-hook cannot be mixed with other options".to_string());
        }
        run_from_hook(agent);
        return Ok(());
    }

    let run_id =
        memorywhale_cli::remember::remember_command(memorywhale_cli::remember::CommandRecord {
            cwd,
            exit_code,
            stdout,
            stderr,
            notes,
            command_parts,
            capture_kind,
            agent: None,
        })?;
    if let Some(run_id) = run_id {
        println!("remembered command run #{run_id}");
    }
    Ok(())
}

/// Agent hooks must never fail the tool call. Parse stdin JSON and record
/// what we can; ignore empty, unknown, or broken payloads.
fn run_from_hook(agent: Agent) {
    if agent == Agent::Cursor {
        run_cursor_hook();
        return;
    }
    let mut buf = Vec::new();
    if io::stdin().read_to_end(&mut buf).is_err() {
        return;
    }
    let Some(record) = memorywhale_cli::agent_hook::record_from_slice(&buf, agent) else {
        return;
    };
    let _ = memorywhale_cli::remember::remember_command(record);
}

fn run_cursor_hook() {
    use memorywhale_cli::agent_hook::{cursor_record_from_slice, MAX_CURSOR_HOOK_BYTES};
    let mut bytes = Vec::new();
    if io::stdin()
        .take(MAX_CURSOR_HOOK_BYTES + 1)
        .read_to_end(&mut bytes)
        .is_err()
    {
        cursor_diagnostic("could not read Cursor hook input; event skipped");
        return;
    }
    match cursor_record_from_slice(&bytes) {
        Ok(Some(record)) => {
            if !record
                .cwd
                .as_deref()
                .is_some_and(|cwd| std::path::Path::new(cwd).is_dir())
            {
                cursor_diagnostic(
                    "Cursor cwd is unavailable locally; event skipped to preserve capture policy",
                );
                return;
            }
            if memorywhale_cli::remember::remember_command(record).is_err() {
                cursor_diagnostic("Cursor event could not be recorded");
            }
        }
        Ok(None) => (),
        Err(message) => cursor_diagnostic(message),
    }
}

// Normal observation stays silent. Debug output is explicit and contains only
// fixed diagnostics, never the captured command, output, path, or storage error.
fn cursor_diagnostic(message: &'static str) {
    if env::var_os("MEMORYWHALE_HOOK_DIAGNOSTICS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        eprintln!("mw-remember: {message}");
    }
}

fn print_help() {
    println!(
        "mw-remember --cwd <path> --exit-code <code> --stdout <text> --stderr <text> --notes <text> --capture-kind <full|hook> -- <command> [args...]\n\
         mw-remember --from-hook claude|rho|cursor   read that client's hook JSON from stdin"
    );
}
