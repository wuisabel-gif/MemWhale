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
                let name = args
                    .next()
                    .ok_or_else(|| "mw-remember --from-hook requires claude or rho".to_string())?;
                from_hook =
                    Some(Agent::parse(&name).ok_or_else(|| {
                        format!("unknown hook client {name:?}; use claude or rho")
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
    let mut buf = Vec::new();
    if io::stdin().read_to_end(&mut buf).is_err() {
        return;
    }
    let Some(record) = memorywhale_cli::agent_hook::record_from_slice(&buf, agent) else {
        return;
    };
    let _ = memorywhale_cli::remember::remember_command(record);
}

fn print_help() {
    println!(
        "mw-remember --cwd <path> --exit-code <code> --stdout <text> --stderr <text> --notes <text> --capture-kind <full|hook> -- <command> [args...]\n\
         mw-remember --from-hook claude|rho   read that client's hook JSON from stdin"
    );
}
