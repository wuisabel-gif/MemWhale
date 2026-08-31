// mw-view: render a MemoryWhale memory as a friendly local web page.
//
// Reads the local SQLite store and writes a self-contained HTML page (inline CSS,
// no network) for a command run or a recorded session, then opens it in the
// browser. The page shows what happened and, from your own history, suggests
// next terminal steps. Everything is local; nothing is uploaded.
//
// Usage:
//   mw-view                 list memories you can open
//   mw-view list            same as above
//   mw-view <id>            auto-detect session or command run, render + open
//   mw-view session <id>    render a recorded session
//   mw-view command <id>    render a command run
//   mw-view <id> --no-open  write the HTML but don't launch a browser

use rusqlite::{params, Connection, OptionalExtension};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    if let Err(err) = run() {
        eprintln!("mw-view: {err}");
        std::process::exit(1);
    }
}

enum Kind {
    Auto,
    Session,
    CommandRun,
}

fn run() -> Result<(), String> {
    let mut kind = Kind::Auto;
    let mut no_open = false;
    let mut output: Option<String> = None;
    let mut positional: Vec<String> = Vec::new();

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "list" => return list_all(),
            "session" => kind = Kind::Session,
            "command" => kind = Kind::CommandRun,
            "--no-open" => no_open = true,
            "--output" | "-o" => output = args.next(),
            value if value.starts_with("--") => {
                return Err(format!("unknown option {value:?}; run mw-view --help"));
            }
            value => positional.push(value.to_string()),
        }
    }

    if positional.is_empty() {
        return list_all();
    }
    let id: i64 = positional[0]
        .parse()
        .map_err(|_| format!("invalid id {:?}; run mw-view --help", positional[0]))?;

    let conn = open_db()?;
    let html = match kind {
        Kind::Session => render_session(&conn, id)?,
        Kind::CommandRun => render_command(&conn, id)?,
        Kind::Auto => {
            if exists(&conn, "sessions", id)? {
                render_session(&conn, id)?
            } else if exists(&conn, "command_runs", id)? {
                render_command(&conn, id)?
            } else {
                return Err(format!(
                    "no session or command run #{id}; run `mw-view list` to see what's available"
                ));
            }
        }
    };

    let path = match output {
        Some(p) => PathBuf::from(p),
        None => {
            let dir = views_dir()?;
            fs::create_dir_all(&dir).map_err(|err| format!("failed to create views dir: {err}"))?;
            dir.join(format!("memory-{id}.html"))
        }
    };
    fs::write(&path, html).map_err(|err| format!("failed to write HTML: {err}"))?;
    println!("mw-view: wrote {}", path.display());
    if !no_open {
        open_in_browser(&path)?;
    }
    Ok(())
}

fn print_help() {
    println!(
        "mw-view                 list memories you can open\n\
         mw-view <id>            render a session or command run as a local web page + open it\n\
         mw-view session <id>    render a recorded session\n\
         mw-view command <id>    render a command run\n\
         mw-view <id> --no-open  write the HTML without launching a browser\n\
         \n\
         Reads the local SQLite store and writes a self-contained page. Local only; never uploaded."
    );
}

fn list_all() -> Result<(), String> {
    let conn = open_db()?;

    println!("Sessions (open with `mw-view session <id>`):");
    let mut s = conn
        .prepare("SELECT id, started_at, byte_count, notes FROM sessions ORDER BY id DESC LIMIT 20")
        .map_err(|e| format!("query sessions: {e}"))?;
    let mut any = false;
    let rows = s
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| format!("read sessions: {e}"))?;
    for row in rows {
        let (id, at, bytes, notes) = row.map_err(|e| format!("row: {e}"))?;
        println!("  #{id}\t{at}\t{bytes} bytes\t{notes}");
        any = true;
    }
    if !any {
        println!("  (none)");
    }

    println!("\nCommand runs (open with `mw-view command <id>`):");
    let mut c = conn
        .prepare("SELECT id, command, exit_code, created_at, notes, agent FROM command_runs ORDER BY id DESC LIMIT 20")
        .map_err(|e| format!("query command_runs: {e}"))?;
    let mut any2 = false;
    let rows = c
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<String>>(5)?,
            ))
        })
        .map_err(|e| format!("read command_runs: {e}"))?;
    for row in rows {
        let (id, cmd, code, at, notes, agent) = row.map_err(|e| format!("row: {e}"))?;
        if !memorywhale_core::provenance::is_valid(agent.as_deref()) {
            continue;
        }
        let code = code.map(|c| c.to_string()).unwrap_or_else(|| "-".into());
        println!(
            "  #{id}\t{cmd}\texit {code}\t{at}\tagent: {}\t{notes}",
            memorywhale_core::provenance::label(agent.as_deref())
        );
        any2 = true;
    }
    if !any2 {
        println!("  (none)");
    }
    Ok(())
}

fn render_command(conn: &Connection, id: i64) -> Result<String, String> {
    let row = conn
        .query_row(
            "SELECT command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at, agent
             FROM command_runs WHERE id = ?1",
            params![id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                    r.get::<_, Option<String>>(8)?,
                ))
            },
        )
        .optional()
        .map_err(|e| format!("read command run: {e}"))?
        .ok_or_else(|| format!("no command run #{id}"))?;
    let (command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at, agent) = row;
    if !memorywhale_core::provenance::is_valid(agent.as_deref()) {
        return Err("invalid stored agent provenance".to_string());
    }

    let argv: Vec<String> =
        serde_json::from_str(&argv_json).unwrap_or_else(|_| vec![command.clone()]);
    let full_cmd = argv.join(" ");
    let ok = exit_code == Some(0);

    let mut body = String::new();
    body.push_str(&format!(
        "<div class=\"eyebrow\">command run · #{id}</div>\n<h1>{}</h1>\n",
        esc(&command)
    ));
    body.push_str(&format!(
        "<div class=\"badge {}\">{}</div>\n",
        if ok { "ok" } else { "bad" },
        match exit_code {
            Some(0) => "exit 0 · success".to_string(),
            Some(c) => format!("exit {c} · failed"),
            None => "no exit code".to_string(),
        }
    ));

    body.push_str("<div class=\"meta\">");
    if let Some(cwd) = &cwd {
        body.push_str(&format!("<div><span>cwd</span>{}</div>", esc(cwd)));
    }
    body.push_str(&format!(
        "<div><span>agent</span>{}</div>",
        esc(memorywhale_core::provenance::label(agent.as_deref()))
    ));
    body.push_str(&format!("<div><span>when</span>{}</div>", esc(&created_at)));
    body.push_str("</div>\n");

    body.push_str("<h2>Command</h2>\n");
    body.push_str(&code_block_with_copy(&full_cmd));

    if !stdout.trim().is_empty() {
        body.push_str("<h2>Output</h2>\n");
        body.push_str(&format!("<pre class=\"out\">{}</pre>\n", esc(&stdout)));
    }
    if !stderr.trim().is_empty() {
        body.push_str("<h2>Error log</h2>\n");
        body.push_str(&format!("<pre class=\"err\">{}</pre>\n", esc(&stderr)));
    }
    if !notes.trim().is_empty() {
        body.push_str("<h2>Note</h2>\n");
        body.push_str(&format!("<p class=\"note\">{}</p>\n", esc(&notes)));
    }

    let hints = command_hints(conn, id, &command, ok)?;
    body.push_str(&render_hints(&hints));

    Ok(page(&format!("{} · MemoryWhale", command), &body))
}

struct Hint {
    text: String,
    snippet: Option<String>,
}

fn command_hints(conn: &Connection, id: i64, command: &str, ok: bool) -> Result<Vec<Hint>, String> {
    let mut hints = Vec::new();

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM command_runs WHERE command = ?1",
            params![command],
            |r| r.get(0),
        )
        .map_err(|e| format!("count runs: {e}"))?;
    let failures: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM command_runs WHERE command = ?1 AND exit_code <> 0",
            params![command],
            |r| r.get(0),
        )
        .map_err(|e| format!("count failures: {e}"))?;
    if total > 1 {
        hints.push(Hint {
            text: format!(
                "You've run `{command}` {total} time(s) — {} succeeded, {failures} failed.",
                total - failures
            ),
            snippet: None,
        });
    }

    if !ok {
        // The most recent successful run of the same command — try that exact line.
        if let Some(argv_json) = conn
            .query_row(
                "SELECT argv_json FROM command_runs
                 WHERE command = ?1 AND exit_code = 0 AND id <> ?2
                 ORDER BY created_at DESC LIMIT 1",
                params![command, id],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| format!("find success: {e}"))?
        {
            let argv: Vec<String> = serde_json::from_str(&argv_json).unwrap_or_default();
            if !argv.is_empty() {
                hints.push(Hint {
                    text: format!(
                        "A previous run of `{command}` succeeded — try that exact command:"
                    ),
                    snippet: Some(argv.join(" ")),
                });
            }
        }

        // What you ran right after a past failure of the same command (a likely fix).
        if let Some(prev_at) = conn
            .query_row(
                "SELECT created_at FROM command_runs
                 WHERE command = ?1 AND exit_code <> 0 AND id <> ?2
                 ORDER BY created_at DESC LIMIT 1",
                params![command, id],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| format!("find prev failure: {e}"))?
        {
            if let Some((next_cmd, next_argv)) = conn
                .query_row(
                    "SELECT command, argv_json FROM command_runs
                     WHERE created_at > ?1 ORDER BY created_at ASC LIMIT 1",
                    params![prev_at],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
                )
                .optional()
                .map_err(|e| format!("find next: {e}"))?
            {
                let argv: Vec<String> = serde_json::from_str(&next_argv).unwrap_or_default();
                let line = if argv.is_empty() {
                    next_cmd
                } else {
                    argv.join(" ")
                };
                hints.push(Hint {
                    text: "Last time this command failed, the next thing you ran was:".to_string(),
                    snippet: Some(line),
                });
            }
        }
    }

    Ok(hints)
}

fn render_session(conn: &Connection, id: i64) -> Result<String, String> {
    let row = conn
        .query_row(
            "SELECT shell, cwd, notes, started_at, ended_at, byte_count, transcript
             FROM sessions WHERE id = ?1",
            params![id],
            |r| {
                Ok((
                    r.get::<_, Option<String>>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, String>(6)?,
                ))
            },
        )
        .optional()
        .map_err(|e| format!("read session: {e}"))?
        .ok_or_else(|| format!("no session #{id}"))?;
    let (shell, cwd, notes, started_at, _ended_at, byte_count, transcript) = row;

    let mut body = String::new();
    body.push_str(&format!(
        "<div class=\"eyebrow\">recorded session · #{id}</div>\n<h1>Session {id}</h1>\n"
    ));
    body.push_str("<div class=\"meta\">");
    if let Some(shell) = &shell {
        body.push_str(&format!("<div><span>shell</span>{}</div>", esc(shell)));
    }
    if let Some(cwd) = &cwd {
        body.push_str(&format!("<div><span>cwd</span>{}</div>", esc(cwd)));
    }
    body.push_str(&format!(
        "<div><span>started</span>{}</div>",
        esc(&started_at)
    ));
    body.push_str(&format!("<div><span>size</span>{byte_count} bytes</div>"));
    body.push_str("</div>\n");

    if !notes.trim().is_empty() {
        body.push_str(&format!("<p class=\"note\">{}</p>\n", esc(&notes)));
    }

    body.push_str("<h2>Transcript</h2>\n");
    body.push_str(&format!("<pre class=\"out\">{}</pre>\n", esc(&transcript)));

    // Light hint: other recent sessions in the same directory.
    let mut hints = Vec::new();
    if let Some(cwd) = &cwd {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE cwd = ?1 AND id <> ?2",
                params![cwd, id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if count > 0 {
            hints.push(Hint {
                text: format!("{count} other session(s) recorded in this directory — `mw-view list` to browse."),
                snippet: None,
            });
        }
    }
    body.push_str(&render_hints(&hints));

    Ok(page(&format!("Session {id} · MemoryWhale"), &body))
}

fn render_hints(hints: &[Hint]) -> String {
    if hints.is_empty() {
        return String::new();
    }
    let mut s = String::from("<h2>Suggested next steps</h2>\n<div class=\"hints\">\n");
    for h in hints {
        s.push_str("<div class=\"hint\">");
        s.push_str(&format!("<p>{}</p>", esc(&h.text)));
        if let Some(snippet) = &h.snippet {
            s.push_str(&code_block_with_copy(snippet));
        }
        s.push_str("</div>\n");
    }
    s.push_str("</div>\n");
    s
}

fn code_block_with_copy(text: &str) -> String {
    format!(
        "<div class=\"cmd\"><code>{}</code><button onclick=\"navigator.clipboard.writeText(this.previousElementSibling.textContent)\">copy</button></div>\n",
        esc(text)
    )
}

fn page(title: &str, body: &str) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"/>\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"/>\
<title>{title}</title>\n<style>{CSS}</style></head>\n<body><main>{body}\
<footer>MemoryWhale — local memory · this page is stored on your machine and never uploaded</footer>\
</main></body></html>\n",
        title = esc(title),
        body = body,
        CSS = CSS
    )
}

const CSS: &str = r#"
:root{--ink:#0f1722;--muted:#566273;--line:#e5ebf2;--azure:#2b43dd;--cyan:#10b6c6;
--ok:#168a69;--bad:#e9663a;--bg:#f3f7fb;--card:#fff;}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--ink);
font-family:"Hanken Grotesk",system-ui,-apple-system,"Segoe UI",sans-serif;line-height:1.55}
main{max-width:860px;margin:0 auto;padding:40px 24px 80px}
.eyebrow{font:600 .72rem/1 ui-monospace,monospace;letter-spacing:.16em;text-transform:uppercase;color:var(--azure);margin-bottom:10px}
h1{font-size:2rem;margin:.1em 0 .4em;letter-spacing:-.02em}
h2{font-size:1rem;margin:1.8em 0 .6em;text-transform:uppercase;letter-spacing:.08em;color:var(--muted)}
.badge{display:inline-block;font:600 .8rem ui-monospace,monospace;padding:4px 12px;border-radius:999px}
.badge.ok{background:#e6f6ef;color:var(--ok)}
.badge.bad{background:#fceee7;color:var(--bad)}
.meta{display:flex;flex-wrap:wrap;gap:8px 24px;margin:18px 0;font-size:.9rem;color:var(--muted)}
.meta span{display:block;font:600 .7rem ui-monospace,monospace;text-transform:uppercase;letter-spacing:.08em;color:var(--azure)}
pre{background:#0b1c25;color:#e3f2f4;padding:16px;border-radius:10px;overflow:auto;font:.85rem/1.5 ui-monospace,monospace;white-space:pre-wrap;word-break:break-word}
pre.err{color:#ffd9c9}
.note{background:var(--card);border:1px solid var(--line);border-left:3px solid var(--cyan);padding:12px 16px;border-radius:8px}
.cmd{display:flex;align-items:center;gap:8px;background:var(--card);border:1px solid var(--line);border-radius:8px;padding:10px 12px;margin:8px 0}
.cmd code{flex:1;font:.9rem ui-monospace,monospace;white-space:pre-wrap;word-break:break-word}
.cmd button{border:1px solid var(--line);background:#fff;border-radius:6px;padding:4px 10px;font:600 .75rem ui-monospace,monospace;cursor:pointer;color:var(--azure)}
.cmd button:hover{border-color:var(--azure)}
.hints{display:flex;flex-direction:column;gap:10px}
.hint{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:14px 16px}
.hint p{margin:0 0 6px}
footer{margin-top:60px;padding-top:20px;border-top:1px solid var(--line);font:.75rem ui-monospace,monospace;color:var(--muted)}
"#;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn exists(conn: &Connection, table: &str, id: i64) -> Result<bool, String> {
    let sql = format!("SELECT COUNT(1) FROM {table} WHERE id = ?1");
    let n: i64 = conn
        .query_row(&sql, params![id], |r| r.get(0))
        .map_err(|e| format!("check {table}: {e}"))?;
    Ok(n > 0)
}

fn open_in_browser(path: &Path) -> Result<(), String> {
    let opener = match env::consts::OS {
        "macos" => "open",
        "windows" => "explorer",
        _ => "xdg-open",
    };
    Command::new(opener)
        .arg(path)
        .status()
        .map_err(|e| format!("failed to open browser ({opener}): {e}"))?;
    Ok(())
}

fn open_db() -> Result<Connection, String> {
    memorywhale_cli::storage::open_path(&database_path()?)
}

fn views_dir() -> Result<PathBuf, String> {
    Ok(memorywhale_dir()?.join("views"))
}

fn database_path() -> Result<PathBuf, String> {
    Ok(memorywhale_dir()?.join("memorywhale.sqlite3"))
}

fn data_base() -> Result<PathBuf, String> {
    dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "could not resolve local data directory".to_string())
}

fn memorywhale_dir() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("MEMORYWHALE_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }
    Ok(data_base()?.join("MemoryWhale"))
}
