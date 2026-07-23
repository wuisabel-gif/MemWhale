// mw-serve: serve MemoryWhale's local memory as a web dashboard.
//
// Starts a small HTTP server (no external dependencies) that reads the local
// SQLite store and serves a browsable page of your previous command runs and
// recorded sessions. Designed for headless machines (e.g. a Jetson): run it on
// the machine that has the data, then open it from a laptop browser over the LAN
// at http://<machine-ip>:<port>/. Everything stays local; nothing is uploaded.
//
// Usage:
//   mw-serve                 serve on 127.0.0.1:7071
//   MEMORYWHALE_TOKEN=... mw-serve --lan  serve on the LAN with authentication
//   mw-serve --port 8080     serve on a different port

use chrono::{DateTime, FixedOffset, Local, Utc};
use regex::Regex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::SystemTime;

static STARTUP_NOTICE: OnceLock<String> = OnceLock::new();
// Optional shared token gating the dashboard. Empty = open (no auth).
static AUTH_TOKEN: OnceLock<String> = OnceLock::new();

fn main() {
    if let Err(err) = run() {
        eprintln!("mw-serve: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_server_args(std::env::args().skip(1))?;
    if config.help {
        println!(
            "mw-serve [--lan | --host <addr>] [--port <n>] [--token <secret>]  — serve memory as a web dashboard"
        );
        return Ok(());
    }
    validate_server_config(&config)?;
    if !config.token.is_empty() {
        let _ = AUTH_TOKEN.set(config.token.clone());
    }

    let db = database_path()?;

    // Self-heal: import any session transcripts whose recording was interrupted
    // before it could write its database row.
    match recover_orphans() {
        Ok(report) => {
            if report.recovered > 0 {
                println!(
                    "Recovered {} interrupted session(s) from transcripts.",
                    report.recovered
                );
            }
            if report.deleted_empty > 0 {
                println!(
                    "Removed {} empty 0-byte transcript(s).",
                    report.deleted_empty
                );
            }
            if report.recovered > 0 || report.deleted_empty > 0 {
                let mut parts = Vec::new();
                if report.recovered > 0 {
                    parts.push(format!(
                        "{} interrupted session(s) recovered",
                        report.recovered
                    ));
                }
                if report.deleted_empty > 0 {
                    parts.push(format!(
                        "{} empty 0-byte transcript(s) cleaned",
                        report.deleted_empty
                    ));
                }
                let _ = STARTUP_NOTICE.set(parts.join(" · "));
            }
        }
        Err(e) => eprintln!("mw-serve: recovery skipped: {e}"),
    }

    let listener = TcpListener::bind((config.host.as_str(), config.port))
        .map_err(|e| format!("failed to bind {}:{}: {e}", config.host, config.port))?;

    println!("MemoryWhale dashboard serving from {}", db.display());
    println!("  local:   http://localhost:{}/", config.port);
    if !is_loopback_host(&config.host) {
        println!(
            "  network: http://<this-machine-ip>:{}/  (find it with: hostname -I)",
            config.port
        );
    }
    if AUTH_TOKEN.get().is_some() {
        println!("  auth:    token required — enter it in the dashboard sign-in form");
    }
    println!("Press Ctrl-C to stop.");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                std::thread::spawn(move || handle(s));
            }
            Err(e) => eprintln!("mw-serve: connection error: {e}"),
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct ServerConfig {
    host: String,
    port: u16,
    token: String,
    help: bool,
}

fn parse_server_args<I>(args: I) -> Result<ServerConfig, String>
where
    I: IntoIterator<Item = String>,
{
    let mut host = "127.0.0.1".to_string();
    let mut port = 7071;
    let mut token = std::env::var("MEMORYWHALE_TOKEN").unwrap_or_default();
    let mut help = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                help = true;
                break;
            }
            "--lan" => host = "0.0.0.0".to_string(),
            "--host" => host = args.next().ok_or("--host needs an address")?,
            "--port" => {
                port = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or("--port needs a number")?;
            }
            "--token" => token = args.next().unwrap_or_default(),
            other => return Err(format!("unknown option {other:?}; run mw-serve --help")),
        }
    }
    Ok(ServerConfig {
        host,
        port,
        token,
        help,
    })
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "::1" | "localhost")
}

fn validate_server_config(config: &ServerConfig) -> Result<(), String> {
    if !config.help && !is_loopback_host(&config.host) && config.token.is_empty() {
        return Err(
            "refusing unauthenticated non-loopback bind; set MEMORYWHALE_TOKEN or --token"
                .to_string(),
        );
    }
    Ok(())
}

fn handle(mut stream: TcpStream) {
    let read_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut reader = BufReader::new(read_stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or("GET").to_string();
    let raw_path = request_parts.next().unwrap_or("/").to_string();

    // Read the cookie and body length; stop at the blank line.
    let mut cookie = String::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line == "\r\n" || line == "\n" || line.is_empty()
        {
            break;
        }
        if let Some(rest) = line
            .split_once(':')
            .filter(|(k, _)| k.eq_ignore_ascii_case("cookie"))
        {
            cookie = rest.1.trim().to_string();
        }
        if let Some(rest) = line
            .split_once(':')
            .filter(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        {
            content_length = rest.1.trim().parse().unwrap_or(0).min(4096);
        }
    }
    let mut request_body = vec![0; content_length];
    if reader.read_exact(&mut request_body).is_err() {
        return;
    }
    let request_body = String::from_utf8_lossy(&request_body);

    let mut cookies: Vec<String> = Vec::new();

    // Display timezone: `?tz=` selects it (and remembers it in a cookie);
    // otherwise fall back to the cookie, else the server's local time.
    let cookie_tz = cookie
        .split(';')
        .find_map(|c| c.trim().strip_prefix("mw_tz=").map(str::to_string));
    match query_param(&raw_path, "tz") {
        Some(tz) => {
            set_display_tz(parse_tz(&tz));
            cookies.push(format!(
                "mw_tz={tz}; Path=/; SameSite=Strict; Max-Age=31536000"
            ));
        }
        None => set_display_tz(
            cookie_tz
                .as_deref()
                .map(parse_tz)
                .unwrap_or(DisplayTz::Local),
        ),
    }

    // Optional shared-token gate. Sign in with a POST body so the token never
    // appears in browser history, server logs, or copied dashboard URLs.
    if let Some(want) = AUTH_TOKEN.get() {
        let via_cookie = cookie
            .split(';')
            .filter_map(|c| c.trim().strip_prefix("mw_token="))
            .any(|v| v == want);
        let login_attempt = method == "POST" && raw_path == "/login";
        let supplied = form_param(&request_body, "token");
        if login_attempt && supplied.as_deref() == Some(want.as_str()) {
            cookies.push(format!(
                "mw_token={want}; Path=/; HttpOnly; SameSite=Strict"
            ));
            let response = format!(
                "HTTP/1.1 303 See Other\r\nLocation: /\r\nSet-Cookie: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                cookies.last().unwrap()
            );
            let _ = stream.write_all(response.as_bytes());
            return;
        } else if !via_cookie {
            let message = if login_attempt {
                "<p>That token was not accepted.</p>"
            } else {
                "<p>This dashboard requires a token.</p>"
            };
            let body = page(
                "Sign in",
                &format!(
                    "{message}<form method=\"post\" action=\"/login\"><label>Shared token <input type=\"password\" name=\"token\" autocomplete=\"current-password\" required></label> <button type=\"submit\">Sign in</button></form>"
                ),
            );
            let response = format!(
                "HTTP/1.1 401 Unauthorized\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.as_bytes().len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.write_all(body.as_bytes());
            return;
        }
    }

    let (status, body) = route(&raw_path);
    let cookie_header: String = cookies
        .iter()
        .map(|c| format!("Set-Cookie: {c}\r\n"))
        .collect();
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n{cookie_header}Connection: close\r\n\r\n",
        body.as_bytes().len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.write_all(body.as_bytes());
}

fn form_param(body: &str, key: &str) -> Option<String> {
    query_param(&format!("?{body}"), key)
}

fn route(raw_path: &str) -> (&'static str, String) {
    let path = raw_path.split('?').next().unwrap_or("/");
    if path == "/" {
        return (
            "200 OK",
            dashboard(&query_param(raw_path, "q").unwrap_or_default()),
        );
    }
    if path == "/graph" {
        return ("200 OK", graph_page());
    }
    if let Some(rest) = path.strip_prefix("/project/") {
        return ("200 OK", project_page(rest));
    }
    if let Some(rest) = path.strip_prefix("/repo/") {
        return ("200 OK", repo_page(rest));
    }
    if let Some(rest) = path.strip_prefix("/runs/") {
        return ("200 OK", runs_page(rest));
    }
    if path == "/favicon.ico" {
        return ("204 No Content", String::new());
    }
    if let Some(rest) = path.strip_prefix("/command/") {
        if let Ok(id) = rest.parse::<i64>() {
            return match command_page(id) {
                Ok(html) => ("200 OK", html),
                Err(e) => (
                    "404 Not Found",
                    page("Not found", &format!("<p>{}</p>", esc(&e))),
                ),
            };
        }
    }
    if let Some(rest) = path.strip_prefix("/session/") {
        if let Ok(id) = rest.parse::<i64>() {
            return match session_page(id) {
                Ok(html) => ("200 OK", html),
                Err(e) => (
                    "404 Not Found",
                    page("Not found", &format!("<p>{}</p>", esc(&e))),
                ),
            };
        }
    }
    (
        "404 Not Found",
        page(
            "Not found",
            "<p>Nothing here. <a href=\"/\">Back to dashboard</a></p>",
        ),
    )
}

fn dashboard(query: &str) -> String {
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => {
            return page(
                "MemoryWhale",
                &format!("<p>Could not open database: {}</p>", esc(&e)),
            )
        }
    };
    let _ = init_min_schema(&conn);

    let mut body =
        String::from("<div class=\"eyebrow\">MemoryWhale</div>\n<h1>Terminal memory</h1>\n");
    if let Some(notice) = STARTUP_NOTICE.get() {
        body.push_str(&format!("<div class=\"notice\">{}</div>\n", esc(notice)));
    }
    body.push_str("<p class=\"sub\">Your previous commands and recorded sessions, served locally. <a class=\"glink\" href=\"/graph\">open graph view →</a></p>\n");
    body.push_str(&format!(
        "<form class=\"search\" method=\"get\" action=\"/\"><input name=\"q\" value=\"{}\" placeholder=\"Search commands, logs, notes, sessions, cwd, tags\"/><button type=\"submit\">Search</button></form>\n",
        esc(query)
    ));
    body.push_str(&tz_selector());

    if !query.trim().is_empty() {
        body.push_str(&search_results(&conn, query));
    }

    let repos = repo_counts(&conn);
    if !repos.is_empty() {
        let mut names: Vec<(&String, &i64)> = repos.iter().collect();
        names.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        body.push_str("<h2>Repos</h2>\n<div class=\"chips\">\n");
        for (name, n) in names {
            body.push_str(&format!(
                "<a class=\"chip\" href=\"/repo/{}\">{} <span>{}</span></a>\n",
                esc(name),
                esc(name),
                n
            ));
        }
        body.push_str("</div>\n");
    }

    body.push_str("<h2>Command runs</h2>\n");
    let mut rows = 0;
    let mut cur_date = String::new();
    let mut group = String::new();
    let mut gcount = 0usize;
    let mut first_group = true;
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, command, argv_json, exit_code, created_at, notes FROM command_runs ORDER BY created_at DESC, id DESC LIMIT 200",
    ) {
        if let Ok(iter) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        }) {
            for row in iter.flatten() {
                let (id, cmd, argv_json, code, at, notes) = row;
                let day = fmt_date(&at);
                if day != cur_date {
                    if !cur_date.is_empty() {
                        push_day_group(&mut body, &cur_date, &group, gcount, first_group);
                        first_group = false;
                        group.clear();
                        gcount = 0;
                    }
                    cur_date = day.to_string();
                }
                let full = full_command(&argv_json, &cmd);
                let ok = code == Some(0);
                group.push_str(&format!(
                    "<a class=\"row\" href=\"/command/{id}\"><span class=\"badge {}\">{}</span>\
                     <span class=\"cmd\">{}</span><span class=\"when\">{}</span><span class=\"note\">{}</span></a>\n",
                    if ok { "ok" } else { "bad" },
                    match code { Some(c) => format!("exit {c}"), None => "—".into() },
                    esc_redacted(&full),
                    esc(&fmt_time(&at)),
                    esc_redacted(&notes)
                ));
                gcount += 1;
                rows += 1;
            }
        }
    }
    if !cur_date.is_empty() {
        push_day_group(&mut body, &cur_date, &group, gcount, first_group);
    }
    if rows == 0 {
        body.push_str("<div class=\"list\"><p class=\"empty\">No command runs yet. Record one with <code>mw-remember</code>.</p></div>\n");
    }

    body.push_str("<h2>Sessions</h2>\n");
    let mut srows = 0;
    let mut cur_date = String::new();
    let mut group = String::new();
    let mut gcount = 0usize;
    let mut first_group = true;
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, started_at, ended_at, byte_count, notes, status FROM sessions ORDER BY started_at DESC, id DESC LIMIT 200",
    ) {
        if let Ok(iter) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        }) {
            for row in iter.flatten() {
                let (id, at, ended_at, bytes, notes, status) = row;
                let day = fmt_date(&at);
                if day != cur_date {
                    if !cur_date.is_empty() {
                        push_day_group(&mut body, &cur_date, &group, gcount, first_group);
                        first_group = false;
                        group.clear();
                        gcount = 0;
                    }
                    cur_date = day.to_string();
                }
                group.push_str(&session_row(id, &at, &ended_at, bytes, &notes, &status));
                gcount += 1;
                srows += 1;
            }
        }
    }
    if !cur_date.is_empty() {
        push_day_group(&mut body, &cur_date, &group, gcount, first_group);
    }
    if srows == 0 {
        body.push_str("<div class=\"list\"><p class=\"empty\">No sessions yet. Record one with <code>mw</code>.</p></div>\n");
    }

    body.push_str("<h2>Bookmarks</h2>\n<div class=\"list\">\n");
    let mut brows = 0;
    if let Ok(mut stmt) =
        conn.prepare("SELECT id, label, cwd, created_at FROM bookmarks ORDER BY id DESC LIMIT 80")
    {
        if let Ok(iter) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
            ))
        }) {
            for (id, label, cwd, created_at) in iter.flatten() {
                body.push_str(&format!(
                    "<div class=\"row\"><span class=\"badge sess\">mark</span><span class=\"cmd\">#{id}</span><span class=\"when\">{}</span><span class=\"note\">{} {}</span></div>\n",
                    esc(&fmt_datetime(&created_at)),
                    esc_redacted(&label),
                    cwd.map(|c| format!("· {}", esc_redacted(&c))).unwrap_or_default()
                ));
                brows += 1;
            }
        }
    }
    if brows == 0 {
        body.push_str("<p class=\"empty\">No bookmarks yet. Mark one with <code>mw mark \"important moment\"</code>.</p>\n");
    }
    body.push_str("</div>\n");

    page("MemoryWhale — terminal memory", &body)
}

/// Extract a `project:<name>` tag from a notes string, if present.
/// Nearest ancestor of `cwd` that is a git repository root (contains `.git`),
/// as (root_path, basename). Filesystem-based, so it only resolves for paths
/// that still exist on the machine running the dashboard.
fn repo_of(cwd: &str) -> Option<(String, String)> {
    if cwd.trim().is_empty() {
        return None;
    }
    let mut dir: Option<&std::path::Path> = Some(std::path::Path::new(cwd));
    while let Some(d) = dir {
        if d.join(".git").exists() {
            let name = d.file_name()?.to_string_lossy().into_owned();
            return Some((d.to_string_lossy().into_owned(), name));
        }
        dir = d.parent();
    }
    None
}

/// Unique git repo roots discovered across all recorded working directories.
fn discovered_repo_roots(conn: &Connection) -> Vec<(String, String)> {
    let mut seen: HashMap<String, String> = HashMap::new(); // root_path -> basename
    for sql in ["SELECT cwd FROM command_runs", "SELECT cwd FROM sessions"] {
        if let Ok(mut stmt) = conn.prepare(sql) {
            if let Ok(it) = stmt.query_map([], |r| r.get::<_, Option<String>>(0)) {
                for cwd in it.flatten().flatten() {
                    if let Some((root, name)) = repo_of(&cwd) {
                        seen.entry(root).or_insert(name);
                    }
                }
            }
        }
    }
    seen.into_iter().collect()
}

/// The repos a session touched: the repo of its start directory, plus any repo
/// whose root path appears in the transcript. This is what lets a session that
/// `cd`-ed between repos show up under each of them.
fn session_repos(
    cwd: &Option<String>,
    transcript: &str,
    roots: &[(String, String)],
) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    if let Some(c) = cwd {
        if let Some((_, name)) = repo_of(c) {
            set.insert(name);
        }
    }
    for (root, name) in roots {
        if transcript.contains(root.as_str()) {
            set.insert(name.clone());
        }
    }
    set
}

/// Command-runs + sessions per repo (a session can count under several repos).
fn repo_counts(conn: &Connection) -> HashMap<String, i64> {
    let mut counts: HashMap<String, i64> = HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT cwd FROM command_runs") {
        if let Ok(it) = stmt.query_map([], |r| r.get::<_, Option<String>>(0)) {
            for cwd in it.flatten().flatten() {
                if let Some((_, name)) = repo_of(&cwd) {
                    *counts.entry(name).or_insert(0) += 1;
                }
            }
        }
    }
    let roots = discovered_repo_roots(conn);
    if let Ok(mut stmt) = conn.prepare("SELECT cwd, transcript FROM sessions") {
        if let Ok(it) = stmt.query_map([], |r| {
            Ok((r.get::<_, Option<String>>(0)?, r.get::<_, String>(1)?))
        }) {
            for (cwd, transcript) in it.flatten() {
                for name in session_repos(&cwd, &transcript, &roots) {
                    *counts.entry(name).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

fn project_of(notes: &str) -> Option<String> {
    let re = Regex::new(r"project:([\w.\-]+)").ok()?;
    re.captures(notes).map(|c| c[1].to_string())
}

/// Count how many command runs + sessions belong to each project tag.

fn search_results(conn: &Connection, query: &str) -> String {
    let needle = format!("%{}%", query.trim());
    let mut out = String::new();
    let mut rows = 0;
    out.push_str("<h2>Search results</h2>\n<div class=\"list\">\n");

    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, command, argv_json, exit_code, created_at, notes, stdout, stderr
         FROM command_runs
         WHERE command LIKE ?1 OR argv_json LIKE ?1 OR IFNULL(cwd, '') LIKE ?1
            OR stdout LIKE ?1 OR stderr LIKE ?1 OR notes LIKE ?1
         ORDER BY id DESC LIMIT 40",
    ) {
        if let Ok(iter) = stmt.query_map(params![needle.as_str()], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, String>(7)?,
            ))
        }) {
            for row in iter.flatten() {
                let (id, cmd, argv_json, code, at, notes, stdout, stderr) = row;
                let ok = code == Some(0);
                let tags = error_tags(&format!("{stdout}\n{stderr}"));
                out.push_str(&format!(
                    "<a class=\"row\" href=\"/command/{id}\"><span class=\"badge {}\">{}</span>\
                     <span class=\"cmd\">{}</span><span class=\"when\">{}</span><span class=\"note\">{} {}</span></a>\n",
                    if ok { "ok" } else { "bad" },
                    match code { Some(c) => format!("exit {c}"), None => "—".into() },
                    esc_redacted(&full_command(&argv_json, &cmd)),
                    esc(&fmt_datetime(&at)),
                    tag_pills(&tags),
                    esc_redacted(&notes)
                ));
                rows += 1;
            }
        }
    }

    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, started_at, ended_at, byte_count, notes, status, transcript
         FROM sessions
         WHERE IFNULL(shell, '') LIKE ?1 OR IFNULL(cwd, '') LIKE ?1 OR transcript LIKE ?1
            OR notes LIKE ?1 OR started_at LIKE ?1 OR status LIKE ?1
         ORDER BY id DESC LIMIT 40",
    ) {
        if let Ok(iter) = stmt.query_map(params![needle.as_str()], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
            ))
        }) {
            for row in iter.flatten() {
                let (id, started_at, ended_at, bytes, notes, status, transcript) = row;
                let tags = error_tags(&transcript);
                let mut row_html = session_row(id, &started_at, &ended_at, bytes, &notes, &status);
                if !tags.is_empty() {
                    row_html = row_html
                        .replace("</span></a>", &format!(" {}</span></a>", tag_pills(&tags)));
                }
                out.push_str(&row_html);
                rows += 1;
            }
        }
    }

    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, label, cwd, created_at FROM bookmarks
         WHERE label LIKE ?1 OR IFNULL(cwd, '') LIKE ?1 OR created_at LIKE ?1
         ORDER BY id DESC LIMIT 40",
    ) {
        if let Ok(iter) = stmt.query_map(params![needle.as_str()], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
            ))
        }) {
            for (id, label, cwd, created_at) in iter.flatten() {
                out.push_str(&format!(
                    "<div class=\"row\"><span class=\"badge sess\">mark</span><span class=\"cmd\">#{id}</span><span class=\"when\">{}</span><span class=\"note\">{} {}</span></div>\n",
                    esc(&fmt_datetime(&created_at)),
                    esc_redacted(&label),
                    cwd.map(|c| format!("· {}", esc_redacted(&c))).unwrap_or_default()
                ));
                rows += 1;
            }
        }
    }

    if rows == 0 {
        out.push_str("<p class=\"empty\">No matching terminal memory found.</p>\n");
    }
    out.push_str("</div>\n");
    out
}

/// Timezone the dashboard renders timestamps in, per-request (thread-local so it
/// doesn't have to be threaded through every render function).
#[derive(Clone, Copy)]
enum DisplayTz {
    Local,
    Fixed(FixedOffset),
}

thread_local! {
    static DISPLAY_TZ: std::cell::Cell<DisplayTz> = std::cell::Cell::new(DisplayTz::Local);
}

fn set_display_tz(tz: DisplayTz) {
    DISPLAY_TZ.with(|c| c.set(tz));
}
fn cur_tz() -> DisplayTz {
    DISPLAY_TZ.with(|c| c.get())
}

/// Parse a tz selection: "local", "utc", or a `±HH:MM` / `±HHMM` / `±H` offset.
fn parse_tz(s: &str) -> DisplayTz {
    let s = s.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("local") {
        return DisplayTz::Local;
    }
    if s.eq_ignore_ascii_case("utc") {
        return DisplayTz::Fixed(FixedOffset::east_opt(0).unwrap());
    }
    let sign = match s.as_bytes().first() {
        Some(b'+') => 1,
        Some(b'-') => -1,
        _ => return DisplayTz::Local,
    };
    let rest = &s[1..];
    let (h, m) = if let Some((h, m)) = rest.split_once(':') {
        (h.parse::<i32>().ok(), m.parse::<i32>().ok())
    } else if rest.len() == 4 {
        (rest[..2].parse().ok(), rest[2..].parse().ok())
    } else {
        (rest.parse::<i32>().ok(), Some(0))
    };
    match (h, m) {
        (Some(h), Some(m)) => FixedOffset::east_opt(sign * (h * 3600 + m * 60))
            .map(DisplayTz::Fixed)
            .unwrap_or(DisplayTz::Local),
        _ => DisplayTz::Local,
    }
}

fn parse_ts(ts: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(ts).ok()
}

/// Format a stored (UTC) timestamp in the request's display timezone.
fn fmt_with(ts: &str, pattern: &str, fallback: &str) -> String {
    match parse_ts(ts) {
        Some(dt) => match cur_tz() {
            DisplayTz::Local => dt.with_timezone(&Local).format(pattern).to_string(),
            DisplayTz::Fixed(o) => dt.with_timezone(&o).format(pattern).to_string(),
        },
        None => fallback.to_string(),
    }
}

/// Local calendar date `YYYY-MM-DD` (for grouping headers).
fn fmt_date(ts: &str) -> String {
    fmt_with(ts, "%Y-%m-%d", date_of(ts))
}
/// Local `YYYY-MM-DD HH:MM` for row timestamps.
fn fmt_datetime(ts: &str) -> String {
    fmt_with(ts, "%Y-%m-%d %H:%M", ts)
}
/// Local `HH:MM:SS` for rows shown under a date header.
fn fmt_time(ts: &str) -> String {
    fmt_with(ts, "%H:%M:%S", time_of(ts))
}

/// The value string for the active tz (matches a selector option).
fn cur_tz_value() -> String {
    match cur_tz() {
        DisplayTz::Local => "local".to_string(),
        DisplayTz::Fixed(o) if o.local_minus_utc() == 0 => "utc".to_string(),
        DisplayTz::Fixed(o) => {
            let secs = o.local_minus_utc();
            let sign = if secs < 0 { '-' } else { '+' };
            let a = secs.abs();
            format!("{sign}{:02}:{:02}", a / 3600, (a % 3600) / 60)
        }
    }
}

/// A no-JS timezone picker: pick a zone and hit Set. "Local" tracks your
/// computer's clock (with daylight saving); the rest are fixed UTC offsets.
fn tz_selector() -> String {
    const OPTS: &[(&str, &str)] = &[
        ("local", "Local (your computer)"),
        ("utc", "UTC"),
        ("-10:00", "UTC-10:00 · Hawaii"),
        ("-09:00", "UTC-09:00 · Alaska"),
        ("-08:00", "UTC-08:00 · US Pacific (PST)"),
        ("-07:00", "UTC-07:00 · US Pacific (PDT) / Mountain"),
        ("-06:00", "UTC-06:00 · US Central"),
        ("-05:00", "UTC-05:00 · US Eastern"),
        ("-03:00", "UTC-03:00"),
        ("+01:00", "UTC+01:00 · Central Europe"),
        ("+02:00", "UTC+02:00"),
        ("+03:00", "UTC+03:00"),
        ("+05:30", "UTC+05:30 · India"),
        ("+08:00", "UTC+08:00 · China / Singapore"),
        ("+09:00", "UTC+09:00 · Japan / Korea"),
        ("+10:00", "UTC+10:00 · Sydney"),
    ];
    let cur = cur_tz_value();
    let mut s = String::from(
        "<form class=\"tzbar\" method=\"get\" action=\"/\"><label for=\"tz\">times shown in</label> <select id=\"tz\" name=\"tz\">",
    );
    for (v, l) in OPTS {
        let sel = if *v == cur { " selected" } else { "" };
        s.push_str(&format!("<option value=\"{v}\"{sel}>{}</option>", esc(l)));
    }
    s.push_str("</select> <button type=\"submit\">Set</button></form>");
    s
}

/// The `YYYY-MM-DD` part of an RFC3339 timestamp, used to group rows by day.
/// Falls back to the whole string if it isn't a recognizable date.
fn date_of(ts: &str) -> &str {
    if ts.len() >= 10 && ts.as_bytes().get(4) == Some(&b'-') {
        &ts[..10]
    } else {
        ts
    }
}

/// The `HH:MM:SS` part of an RFC3339 timestamp (whole string if not one).
fn time_of(ts: &str) -> &str {
    if ts.len() >= 19 && ts.as_bytes().get(10) == Some(&b'T') {
        &ts[11..19]
    } else {
        ts
    }
}

/// Emit one collapsible day group: a `<details>` whose summary is the date and
/// count, wrapping the already-rendered rows. `open` expands it by default.
fn push_day_group(body: &mut String, date: &str, rows_html: &str, count: usize, open: bool) {
    body.push_str(&format!(
        "<details class=\"daygroup\"{}><summary class=\"datehead\">{} \
         <span class=\"gcount\">{}</span></summary>\n<div class=\"list\">\n{}</div>\n</details>\n",
        if open { " open" } else { "" },
        esc(date),
        count,
        rows_html
    ));
}

fn session_row(
    id: i64,
    started_at: &str,
    ended_at: &str,
    bytes: i64,
    notes: &str,
    status: &str,
) -> String {
    let badge_class = match status {
        "recording" if session_age_seconds(ended_at).map_or(false, |age| age <= 30) => "live",
        "recording" | "interrupted" => "warn",
        _ => "sess",
    };
    let label = session_label(status, ended_at);
    let badge_text = match badge_class {
        "live" => "live",
        "warn" => "interrupted",
        _ => "session",
    };
    format!(
        "<a class=\"row\" href=\"/session/{id}\"><span class=\"badge {badge_class}\">{}</span>\
         <span class=\"cmd\">#{id}</span><span class=\"when\">{}</span><span class=\"note\">{} · {bytes} bytes</span></a>\n",
        esc(badge_text),
        esc(&fmt_datetime(started_at)),
        esc_redacted(&format!("{label} · {notes}"))
    )
}

fn session_label(status: &str, ended_at: &str) -> String {
    match status {
        "recording" => match session_age_seconds(ended_at) {
            Some(age) if age <= 30 => format!("Recording now · last autosaved {age}s ago"),
            Some(age) => format!("Interrupted or stale · last autosaved {}", human_age(age)),
            None => "Recording now".to_string(),
        },
        "interrupted" => "Recovered interrupted".to_string(),
        _ => "session".to_string(),
    }
}

fn session_age_seconds(ended_at: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(ended_at)
        .ok()
        .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_seconds().max(0))
}

fn human_age(seconds: i64) -> String {
    if seconds < 60 {
        format!("{seconds}s ago")
    } else if seconds < 3600 {
        format!("{}m ago", seconds / 60)
    } else {
        format!("{}h ago", seconds / 3600)
    }
}

/// A combined, time-ordered view of every command run and session tagged with a
/// given project — even if they were recorded in different terminals.
fn project_page(raw_name: &str) -> String {
    let name = raw_name.trim_end_matches('/').to_string();
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => return page("Project", &format!("<p>{}</p>", esc(&e))),
    };
    let _ = init_min_schema(&conn);

    let mut items: Vec<(String, String)> = Vec::new(); // (timestamp, row html)

    if let Ok(mut stmt) = conn
        .prepare("SELECT id, command, argv_json, exit_code, created_at, notes FROM command_runs")
    {
        if let Ok(it) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        }) {
            for (id, cmd, argv_json, code, at, notes) in it.flatten() {
                if project_of(&notes).as_deref() != Some(name.as_str()) {
                    continue;
                }
                let ok = code == Some(0);
                let row = format!(
                    "<a class=\"row\" href=\"/command/{id}\"><span class=\"badge {}\">{}</span>\
                     <span class=\"cmd\">{}</span><span class=\"when\">{}</span><span class=\"note\">{}</span></a>",
                    if ok { "ok" } else { "bad" },
                    match code { Some(c) => format!("exit {c}"), None => "—".into() },
                    esc_redacted(&full_command(&argv_json, &cmd)), esc(&fmt_datetime(&at)), esc_redacted(&notes)
                );
                items.push((at, row));
            }
        }
    }

    if let Ok(mut stmt) =
        conn.prepare("SELECT id, started_at, ended_at, byte_count, notes, status FROM sessions")
    {
        if let Ok(it) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        }) {
            for (id, at, ended_at, bytes, notes, status) in it.flatten() {
                if project_of(&notes).as_deref() != Some(name.as_str()) {
                    continue;
                }
                let row = session_row(id, &at, &ended_at, bytes, &notes, &status);
                items.push((at, row));
            }
        }
    }

    items.sort_by(|a, b| b.0.cmp(&a.0)); // newest first

    let mut body = String::from("<a class=\"back\" href=\"/\">← all memory</a>\n");
    body.push_str(&format!(
        "<div class=\"eyebrow\">project</div>\n<h1>{}</h1>\n",
        esc(&name)
    ));
    body.push_str(&format!(
        "<p class=\"sub\">{} memory item(s) across all terminals, newest first.</p>\n",
        items.len()
    ));
    body.push_str("<div class=\"list\">\n");
    if items.is_empty() {
        body.push_str("<p class=\"empty\">No memory tagged <code>project:");
        body.push_str(&esc(&name));
        body.push_str("</code> yet. Record with <code>--notes \"project:");
        body.push_str(&esc(&name));
        body.push_str("\"</code>.</p>");
    }
    for (_, row) in items {
        body.push_str(&row);
        body.push('\n');
    }
    body.push_str("</div>\n");
    page(&format!("{} · MemoryWhale", name), &body)
}

/// Everything that happened in a given git repo — command runs whose working
/// directory is inside it, plus sessions that touched it (start dir or any repo
/// path seen in the transcript), newest first.
fn repo_page(raw_name: &str) -> String {
    let name = raw_name.trim_end_matches('/').to_string();
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => return page("Repo", &format!("<p>{}</p>", esc(&e))),
    };
    let _ = init_min_schema(&conn);
    let roots = discovered_repo_roots(&conn);

    let mut items: Vec<(String, String)> = Vec::new(); // (timestamp, row html)

    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, command, argv_json, exit_code, created_at, notes, cwd FROM command_runs",
    ) {
        if let Ok(it) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        }) {
            for (id, cmd, argv_json, code, at, notes, cwd) in it.flatten() {
                let in_repo = cwd.as_deref().and_then(repo_of).map(|(_, n)| n).as_deref()
                    == Some(name.as_str());
                if !in_repo {
                    continue;
                }
                let ok = code == Some(0);
                let row = format!(
                    "<a class=\"row\" href=\"/command/{id}\"><span class=\"badge {}\">{}</span>\
                     <span class=\"cmd\">{}</span><span class=\"when\">{}</span><span class=\"note\">{}</span></a>",
                    if ok { "ok" } else { "bad" },
                    match code { Some(c) => format!("exit {c}"), None => "—".into() },
                    esc_redacted(&full_command(&argv_json, &cmd)), esc(&fmt_datetime(&at)), esc_redacted(&notes)
                );
                items.push((at, row));
            }
        }
    }

    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, started_at, ended_at, byte_count, notes, status, cwd, transcript FROM sessions",
    ) {
        if let Ok(it) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, String>(7)?,
            ))
        }) {
            for (id, at, ended_at, bytes, notes, status, cwd, transcript) in it.flatten() {
                if !session_repos(&cwd, &transcript, &roots).contains(&name) {
                    continue;
                }
                items.push((
                    at.clone(),
                    session_row(id, &at, &ended_at, bytes, &notes, &status),
                ));
            }
        }
    }

    items.sort_by(|a, b| b.0.cmp(&a.0)); // newest first

    let mut body = String::from("<a class=\"back\" href=\"/\">← all memory</a>\n");
    body.push_str(&format!(
        "<div class=\"eyebrow\">repo</div>\n<h1>{}</h1>\n",
        esc(&name)
    ));
    body.push_str(&format!(
        "<p class=\"sub\">{} memory item(s) in this repository, newest first. A session that also touched another repo appears under that one too.</p>\n",
        items.len()
    ));
    body.push_str("<div class=\"list\">\n");
    if items.is_empty() {
        body.push_str(
            "<p class=\"empty\">Nothing recorded in a working directory under this repo yet.</p>",
        );
    }
    for (_, row) in items {
        body.push_str(&row);
        body.push('\n');
    }
    body.push_str("</div>\n");
    page(&format!("{} · MemoryWhale", name), &body)
}

fn command_page(id: i64) -> Result<String, String> {
    let conn = open_db()?;
    let row = conn
        .query_row(
            "SELECT command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at
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
                ))
            },
        )
        .optional()
        .map_err(|e| format!("read command run: {e}"))?
        .ok_or_else(|| format!("no command run #{id}"))?;
    let (command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at) = row;
    let argv: Vec<String> =
        serde_json::from_str(&argv_json).unwrap_or_else(|_| vec![command.clone()]);
    let ok = exit_code == Some(0);

    let mut body = String::from("<a class=\"back\" href=\"/\">← all memory</a>\n");
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
    body.push_str(&tag_pills(&error_tags(&format!(
        "{stdout}\n{stderr}\n{notes}"
    ))));
    body.push_str("<div class=\"meta\">");
    if let Some(cwd) = &cwd {
        body.push_str(&format!("<div><span>cwd</span>{}</div>", esc(cwd)));
    }
    body.push_str(&format!(
        "<div><span>when</span>{}</div></div>\n",
        esc(&fmt_datetime(&created_at))
    ));

    body.push_str("<h2>Command</h2>\n");
    body.push_str(&code_block(&argv.join(" ")));
    if !stdout.trim().is_empty() {
        body.push_str("<h2>Output</h2>\n");
        body.push_str(&format!(
            "<pre class=\"out\">{}</pre>\n",
            esc_redacted(&stdout)
        ));
    }
    if !stderr.trim().is_empty() {
        body.push_str("<h2>Error log</h2>\n");
        body.push_str(&format!(
            "<pre class=\"err\">{}</pre>\n",
            esc_redacted(&stderr)
        ));
    }
    if !notes.trim().is_empty() {
        body.push_str(&format!(
            "<h2>Note</h2>\n<p class=\"noteblock\">{}</p>\n",
            esc_redacted(&notes)
        ));
    }
    body.push_str(&debug_summary(&argv.join(" "), &stdout, &stderr, &notes));
    body.push_str(&hints(&conn, id, &command, ok));
    Ok(page(&format!("{} · MemoryWhale", command), &body))
}

fn session_page(id: i64) -> Result<String, String> {
    let conn = open_db()?;
    let row = conn
        .query_row(
            "SELECT shell, cwd, notes, started_at, ended_at, byte_count, transcript, status FROM sessions WHERE id = ?1",
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
                    r.get::<_, String>(7)?,
                ))
            },
        )
        .optional()
        .map_err(|e| format!("read session: {e}"))?
        .ok_or_else(|| format!("no session #{id}"))?;
    let (shell, cwd, notes, started_at, ended_at, byte_count, transcript, status) = row;

    let mut body = String::from("<a class=\"back\" href=\"/\">← all memory</a>\n");
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
        esc(&fmt_datetime(&started_at))
    ));
    body.push_str(&format!(
        "<div><span>status</span>{}</div>",
        esc(&session_label(&status, &ended_at))
    ));
    body.push_str(&format!(
        "<div><span>size</span>{byte_count} bytes</div></div>\n"
    ));
    if !notes.trim().is_empty() {
        body.push_str(&format!(
            "<p class=\"noteblock\">{}</p>\n",
            esc_redacted(&notes)
        ));
    }
    body.push_str(&tag_pills(&error_tags(&transcript)));
    body.push_str(&session_debug_summary(&transcript, &notes));
    body.push_str("<h2>Transcript</h2>\n");
    body.push_str(&format!(
        "<pre class=\"out\">{}</pre>\n",
        esc_redacted(&transcript)
    ));
    Ok(page(&format!("Session {id} · MemoryWhale"), &body))
}

fn hints(conn: &Connection, id: i64, command: &str, ok: bool) -> String {
    let mut out = String::new();
    let mut items: Vec<(String, Option<String>)> = Vec::new();

    if let Ok(total) = conn.query_row(
        "SELECT COUNT(*) FROM command_runs WHERE command = ?1",
        params![command],
        |r| r.get::<_, i64>(0),
    ) {
        if total > 1 {
            let failures: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM command_runs WHERE command = ?1 AND exit_code <> 0",
                    params![command],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            items.push((
                format!(
                    "You've run `{command}` {total} time(s) — {} succeeded, {failures} failed.",
                    total - failures
                ),
                None,
            ));
        }
    }

    if !ok {
        if let Ok(Some(argv_json)) = conn
            .query_row(
                "SELECT argv_json FROM command_runs WHERE command = ?1 AND exit_code = 0 AND id <> ?2 ORDER BY created_at DESC LIMIT 1",
                params![command, id],
                |r| r.get::<_, String>(0),
            )
            .optional()
        {
            let argv: Vec<String> = serde_json::from_str(&argv_json).unwrap_or_default();
            if !argv.is_empty() {
                items.push((format!("A previous run of `{command}` succeeded — try that exact command:"), Some(argv.join(" "))));
            }
        }
        if let Ok(Some(prev_at)) = conn
            .query_row(
                "SELECT created_at FROM command_runs WHERE command = ?1 AND exit_code <> 0 AND id <> ?2 ORDER BY created_at DESC LIMIT 1",
                params![command, id],
                |r| r.get::<_, String>(0),
            )
            .optional()
        {
            if let Ok(Some((next_cmd, next_argv))) = conn
                .query_row(
                    "SELECT command, argv_json FROM command_runs WHERE created_at > ?1 ORDER BY created_at ASC LIMIT 1",
                    params![prev_at],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
                )
                .optional()
            {
                let argv: Vec<String> = serde_json::from_str(&next_argv).unwrap_or_default();
                let line = if argv.is_empty() { next_cmd } else { argv.join(" ") };
                items.push(("Last time this command failed, the next thing you ran was:".to_string(), Some(line)));
            }
        }
    }

    if items.is_empty() {
        return out;
    }
    out.push_str("<h2>Suggested next steps</h2>\n<div class=\"hints\">\n");
    for (text, snippet) in items {
        out.push_str("<div class=\"hint\"><p>");
        out.push_str(&esc(&text));
        out.push_str("</p>");
        if let Some(s) = snippet {
            out.push_str(&code_block(&s));
        }
        out.push_str("</div>\n");
    }
    out.push_str("</div>\n");
    out
}

#[derive(Serialize)]
struct GNode {
    id: String,
    label: String,
    kind: String,
    weight: i64,
    name: Option<String>,
}

#[derive(Serialize)]
struct GLink {
    source: String,
    target: String,
}

#[derive(Serialize)]
struct Graph {
    nodes: Vec<GNode>,
    links: Vec<GLink>,
}

/// Build a graph aggregated by command name and argument value. Nodes carry a
/// weight (how often they appear); arguments used by two or more distinct
/// commands are marked as bridges. One node per command (not per run).
fn graph_json() -> Result<String, String> {
    let conn = open_db()?;
    let _ = init_min_schema(&conn);

    let mut run_cmd: HashMap<i64, String> = HashMap::new();
    let mut cmd_count: HashMap<String, i64> = HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT id, command FROM command_runs")
            .map_err(|e| format!("query runs: {e}"))?;
        let it = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| format!("read runs: {e}"))?;
        for (id, cmd) in it.flatten() {
            run_cmd.insert(id, cmd.clone());
            *cmd_count.entry(cmd).or_insert(0) += 1;
        }
    }

    let mut arg_count: HashMap<String, i64> = HashMap::new();
    let mut arg_cmds: HashMap<String, HashSet<String>> = HashMap::new();
    let mut pairs: HashSet<(String, String)> = HashSet::new();
    {
        let mut stmt = conn
            .prepare("SELECT command_run_id, value FROM command_arguments WHERE position >= 1")
            .map_err(|e| format!("query args: {e}"))?;
        let it = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| format!("read args: {e}"))?;
        for (run_id, value) in it.flatten() {
            if value.trim().is_empty() {
                continue;
            }
            let cmd = match run_cmd.get(&run_id) {
                Some(c) => c.clone(),
                None => continue,
            };
            *arg_count.entry(value.clone()).or_insert(0) += 1;
            arg_cmds
                .entry(value.clone())
                .or_default()
                .insert(cmd.clone());
            pairs.insert((cmd, value));
        }
    }

    let mut nodes: Vec<GNode> = Vec::new();
    for (cmd, count) in &cmd_count {
        nodes.push(GNode {
            id: format!("cmd:{cmd}"),
            label: cmd.clone(),
            kind: "cmd".into(),
            weight: *count,
            name: Some(cmd.clone()),
        });
    }
    for (val, count) in &arg_count {
        let shared = arg_cmds.get(val).map(|s| s.len() >= 2).unwrap_or(false);
        nodes.push(GNode {
            id: format!("arg:{val}"),
            label: val.clone(),
            kind: if shared {
                "bridge".into()
            } else {
                "arg".into()
            },
            weight: *count,
            name: None,
        });
    }
    let mut links: Vec<GLink> = Vec::new();
    for (cmd, val) in &pairs {
        links.push(GLink {
            source: format!("cmd:{cmd}"),
            target: format!("arg:{val}"),
        });
    }

    serde_json::to_string(&Graph { nodes, links }).map_err(|e| format!("serialize graph: {e}"))
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn query_param(raw_path: &str, key: &str) -> Option<String> {
    let query = raw_path.split_once('?')?.1;
    for pair in query.split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        if percent_decode(k) == key {
            return Some(percent_decode(&v.replace('+', " ")));
        }
    }
    None
}

/// List all command runs for one command name (where a graph command node links).
fn runs_page(raw: &str) -> String {
    let name = percent_decode(raw.trim_end_matches('/'));
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => return page("Runs", &format!("<p>{}</p>", esc(&e))),
    };
    let _ = init_min_schema(&conn);

    let mut body = String::from("<a class=\"back\" href=\"/graph\">← graph</a>\n");
    body.push_str(&format!(
        "<div class=\"eyebrow\">command</div>\n<h1>{}</h1>\n",
        esc(&name)
    ));
    body.push_str("<div class=\"list\">\n");
    let mut rows = 0;
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, argv_json, exit_code, created_at, notes FROM command_runs WHERE command = ?1 ORDER BY id DESC",
    ) {
        if let Ok(it) = stmt.query_map(params![name], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        }) {
            for (id, argv_json, code, at, notes) in it.flatten() {
                let ok = code == Some(0);
                body.push_str(&format!(
                    "<a class=\"row\" href=\"/command/{id}\"><span class=\"badge {}\">{}</span>\
                     <span class=\"cmd\">{}</span><span class=\"when\">{}</span><span class=\"note\">{}</span></a>\n",
                    if ok { "ok" } else { "bad" },
                    match code { Some(c) => format!("exit {c}"), None => "—".into() },
                    esc_redacted(&full_command(&argv_json, &name)), esc(&fmt_datetime(&at)), esc_redacted(&notes)
                ));
                rows += 1;
            }
        }
    }
    if rows == 0 {
        body.push_str("<p class=\"empty\">No runs found for this command.</p>");
    }
    body.push_str("</div>\n");
    page(&format!("{} runs · MemoryWhale", name), &body)
}

fn graph_page() -> String {
    let data = match graph_json() {
        Ok(d) => d.replace("</", "<\\/"), // keep an arg value of "</script>" from breaking the page
        Err(e) => return page("Graph", &format!("<p>{}</p>", esc(&e))),
    };
    let mut body = String::new();
    body.push_str("<a class=\"back\" href=\"/\">← all memory</a>\n");
    body.push_str("<div class=\"eyebrow\">knowledge graph</div>\n<h1>Command graph</h1>\n");
    body.push_str("<p class=\"sub\">Commands sized by how often you ran them, linked to their arguments. Orange arguments are shared by two or more commands (bridges). Click a command to see its runs.</p>\n");
    body.push_str("<div class=\"legend\"><span class=\"dot run\"></span>command<span class=\"dot arg\"></span>argument<span class=\"dot bridge\"></span>shared</div>\n");
    body.push_str("<canvas id=\"g\" width=\"920\" height=\"560\"></canvas>\n");
    body.push_str("<script>\nconst DATA = ");
    body.push_str(&data);
    body.push_str(";\n");
    body.push_str(GRAPH_JS);
    body.push_str("\n</script>\n");
    page("Command graph · MemoryWhale", &body)
}

const GRAPH_JS: &str = r#"
const cv=document.getElementById('g'),cx=cv.getContext('2d');
const W=cv.width,H=cv.height,N=DATA.nodes,L=DATA.links;
if(!N.length){cx.fillStyle='#566273';cx.font='15px sans-serif';cx.fillText('No commands with arguments yet — record some with mw-remember.',24,40);}
else{
const idx={},maxW=Math.max(1,...N.map(n=>n.weight||1));
N.forEach(n=>{idx[n.id]=n;n.x=W/2+(Math.random()-.5)*260;n.y=H/2+(Math.random()-.5)*260;n.vx=0;n.vy=0;n.r=(n.kind==='cmd'?8:4)+14*Math.sqrt((n.weight||1)/maxW);});
L.forEach(l=>{l.s=idx[l.source];l.t=idx[l.target];});
function col(n){return n.kind==='cmd'?'#2b43dd':n.kind==='bridge'?'#e9663a':'#10b6c6';}
function step(){
 for(let i=0;i<N.length;i++)for(let j=i+1;j<N.length;j++){const a=N[i],b=N[j];let dx=a.x-b.x,dy=a.y-b.y,d=Math.hypot(dx,dy)||1;if(d<320){const f=2600/(d*d);a.vx+=dx/d*f;a.vy+=dy/d*f;b.vx-=dx/d*f;b.vy-=dy/d*f;}}
 L.forEach(l=>{if(!l.s||!l.t)return;let dx=l.t.x-l.s.x,dy=l.t.y-l.s.y,d=Math.hypot(dx,dy)||1,f=(d-84)*0.02;l.s.vx+=dx/d*f;l.s.vy+=dy/d*f;l.t.vx-=dx/d*f;l.t.vy-=dy/d*f;});
 N.forEach(n=>{n.vx+=(W/2-n.x)*0.002;n.vy+=(H/2-n.y)*0.002;n.vx*=0.86;n.vy*=0.86;n.x+=n.vx;n.y+=n.vy;n.x=Math.max(30,Math.min(W-30,n.x));n.y=Math.max(30,Math.min(H-30,n.y));});
}
function draw(){
 cx.clearRect(0,0,W,H);
 cx.strokeStyle='#d5dee9';cx.lineWidth=1;
 L.forEach(l=>{if(!l.s||!l.t)return;cx.beginPath();cx.moveTo(l.s.x,l.s.y);cx.lineTo(l.t.x,l.t.y);cx.stroke();});
 N.forEach(n=>{cx.beginPath();cx.arc(n.x,n.y,n.r,0,7);cx.fillStyle=col(n);cx.fill();cx.fillStyle='#0f1722';cx.font=(n.kind==='cmd'?'600 12px ':'11px ')+'ui-monospace,monospace';cx.fillText(n.label,n.x+n.r+4,n.y+4);});
}
let t=0;function loop(){for(let k=0;k<3;k++)step();draw();if(t++<800)requestAnimationFrame(loop);}
loop();
cv.style.cursor='pointer';
cv.onclick=e=>{const rc=cv.getBoundingClientRect(),sx=W/rc.width,sy=H/rc.height,mx=(e.clientX-rc.left)*sx,my=(e.clientY-rc.top)*sy;let best=null,bd=1e9;N.forEach(n=>{const d=(n.x-mx)**2+(n.y-my)**2;if(d<bd&&d<(n.r+12)*(n.r+12)){bd=d;best=n;}});if(best&&best.kind==='cmd'&&best.name)location.href='/runs/'+encodeURIComponent(best.name);};
}
"#;

fn code_block(text: &str) -> String {
    format!(
        "<div class=\"codeblock\"><code>{}</code></div>\n",
        esc_redacted(text)
    )
}

fn page(title: &str, body: &str) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"/>\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"/>\
<title>{}</title>\n<style>{CSS}</style></head>\n<body><main>{body}\
<footer>MemoryWhale — served locally from SQLite · nothing is uploaded</footer></main></body></html>\n",
        esc(title)
    )
}

const CSS: &str = r#"
:root{--ink:#0f1722;--muted:#566273;--line:#e5ebf2;--azure:#2b43dd;--cyan:#10b6c6;--ok:#168a69;--bad:#e9663a;--bg:#f3f7fb;--card:#fff;}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--ink);font-family:"Hanken Grotesk",system-ui,-apple-system,"Segoe UI",sans-serif;line-height:1.55}
main{max-width:920px;margin:0 auto;padding:40px 24px 80px}
a{color:inherit;text-decoration:none}
.eyebrow{font:600 .72rem/1 ui-monospace,monospace;letter-spacing:.16em;text-transform:uppercase;color:var(--azure);margin-bottom:10px}
.back{display:inline-block;margin-bottom:18px;color:var(--azure);font:600 .8rem ui-monospace,monospace}
h1{font-size:2rem;margin:.1em 0 .3em;letter-spacing:-.02em}
h2{font-size:.95rem;margin:1.8em 0 .6em;text-transform:uppercase;letter-spacing:.08em;color:var(--muted)}
.sub{color:var(--muted);margin:0 0 1em}
.search{display:flex;gap:8px;margin:18px 0 8px}
.search input{flex:1;border:1px solid var(--line);border-radius:10px;background:#fff;padding:10px 12px;font:600 .9rem ui-monospace,monospace;color:var(--ink)}
.search button{border:0;border-radius:10px;background:var(--azure);color:#fff;padding:10px 14px;font:700 .85rem ui-monospace,monospace;cursor:pointer}
.list{display:flex;flex-direction:column;gap:8px}
.tzbar{display:flex;align-items:center;gap:8px;margin:0 0 6px;font-size:.8rem;color:var(--muted)}
.tzbar select{font:inherit;padding:4px 8px;border:1px solid var(--line);border-radius:8px;background:var(--card);color:var(--ink)}
.tzbar button{font:inherit;padding:4px 10px;border:1px solid var(--line);border-radius:8px;background:var(--card);color:var(--ink);cursor:pointer}
.tzbar button:hover{border-color:var(--azure)}
.daygroup{margin:1.1em 0}
.datehead{margin:0 0 .5em;font:600 .8rem ui-monospace,monospace;letter-spacing:.04em;color:var(--muted);border-bottom:1px solid var(--line);padding-bottom:.3em;cursor:pointer;user-select:none}
.datehead:hover{color:var(--azure)}
.datehead .gcount{color:var(--muted);font-weight:400;opacity:.7}
.datehead .gcount::before{content:"· "}
.row{display:grid;grid-template-columns:90px 1fr 1.2fr 1.4fr;gap:14px;align-items:center;background:var(--card);border:1px solid var(--line);border-radius:10px;padding:12px 16px;transition:border-color .15s}
.row:hover{border-color:var(--azure)}
.row .cmd{font:600 .95rem ui-monospace,monospace}
.row .when{font:.78rem ui-monospace,monospace;color:var(--muted)}
.row .note{font-size:.85rem;color:var(--muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.badge{display:inline-block;font:600 .72rem ui-monospace,monospace;padding:4px 10px;border-radius:999px;text-align:center}
.badge.ok{background:#e6f6ef;color:var(--ok)}
.badge.bad{background:#fceee7;color:var(--bad)}
.badge.sess{background:#eaeefe;color:var(--azure)}
.badge.live{background:#dff9f4;color:#087260}
.badge.warn{background:#fff4d8;color:#9a5b00}
.notice{background:#e9fbf7;border:1px solid #b7ebe0;border-left:4px solid var(--cyan);color:#0f5e57;border-radius:10px;padding:12px 14px;margin:0 0 16px;font-weight:650}
.tags{display:flex;flex-wrap:wrap;gap:6px;margin:10px 0}
.tag{display:inline-flex;align-items:center;background:#fff4d8;color:#9a5b00;border:1px solid #f3d89a;border-radius:999px;padding:2px 8px;font:700 .7rem ui-monospace,monospace}
.meta{display:flex;flex-wrap:wrap;gap:8px 24px;margin:16px 0;font-size:.9rem;color:var(--muted)}
.meta span{display:block;font:600 .7rem ui-monospace,monospace;text-transform:uppercase;letter-spacing:.08em;color:var(--azure)}
pre{background:#0b1c25;color:#e3f2f4;padding:16px;border-radius:10px;overflow:auto;font:.85rem/1.5 ui-monospace,monospace;white-space:pre-wrap;word-break:break-word}
pre.err{color:#ffd9c9}
.noteblock{background:var(--card);border:1px solid var(--line);border-left:3px solid var(--cyan);padding:12px 16px;border-radius:8px}
.codeblock{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:10px 12px;margin:8px 0}
.codeblock code{font:.9rem ui-monospace,monospace;white-space:pre-wrap;word-break:break-word}
.hints{display:flex;flex-direction:column;gap:10px}
.hint{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:14px 16px}
.hint p{margin:0 0 6px}
.empty{color:var(--muted)}
.glink{color:var(--azure);font-weight:600}
.chips{display:flex;flex-wrap:wrap;gap:8px}
.chip{display:inline-flex;align-items:center;gap:8px;background:var(--card);border:1px solid var(--line);border-radius:999px;padding:7px 14px;font:600 .85rem ui-monospace,monospace;color:var(--azure)}
.chip span{background:#eaeefe;border-radius:999px;padding:1px 8px;font-size:.72rem}
.chip:hover{border-color:var(--azure)}
canvas{max-width:100%;background:#fff;border:1px solid var(--line);border-radius:12px;margin-top:8px}
.legend{display:flex;align-items:center;gap:8px;font:.8rem ui-monospace,monospace;color:var(--muted);margin:4px 0 0}
.legend .dot{width:11px;height:11px;border-radius:999px;display:inline-block;margin-left:14px}
.legend .dot.run{background:var(--azure)}
.legend .dot.arg{background:var(--cyan)}
.legend .dot.bridge{background:var(--bad)}
footer{margin-top:60px;padding-top:20px;border-top:1px solid var(--line);font:.75rem ui-monospace,monospace;color:var(--muted)}
"#;

/// The full command line from stored argv (e.g. "npm run tauri:dev"), falling
/// back to the bare command name if argv can't be parsed.
fn full_command(argv_json: &str, fallback: &str) -> String {
    serde_json::from_str::<Vec<String>>(argv_json)
        .ok()
        .filter(|a| !a.is_empty())
        .map(|a| a.join(" "))
        .unwrap_or_else(|| fallback.to_string())
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn esc_redacted(s: &str) -> String {
    esc(&redact_secrets(s))
}

fn redact_secrets(input: &str) -> String {
    let mut out = input.to_string();
    let patterns = [
        (
            r#"(?i)(api[_-]?key|token|password|passwd|secret|authorization)\s*[:=]\s*['"]?([^\s'"&]+)"#,
            "$1=<redacted>",
        ),
        (r"(?i)Bearer\s+[A-Za-z0-9._\-]+", "Bearer <redacted>"),
        (r"gh[pousr]_[A-Za-z0-9_]+", "<redacted-github-token>"),
        (r"sk-[A-Za-z0-9_\-]{20,}", "<redacted-api-key>"),
    ];
    for (pattern, replacement) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            out = re.replace_all(&out, replacement).into_owned();
        }
    }
    out
}

fn error_tags(text: &str) -> Vec<&'static str> {
    let lower = text.to_lowercase();
    let mut tags = Vec::new();
    let patterns = [
        ("command not found", "command not found"),
        ("permission denied", "permission denied"),
        ("package not found", "package not found"),
        ("no package", "package not found"),
        ("failed to build", "failed to build"),
        ("failed to compile", "failed to build"),
        ("could not compile", "failed to build"),
        ("address already in use", "port already in use"),
        ("port already in use", "port already in use"),
        ("no such file or directory", "no such file"),
        ("not found", "not found"),
    ];
    for (needle, label) in patterns {
        if lower.contains(needle) && !tags.contains(&label) {
            tags.push(label);
        }
    }
    tags
}

fn tag_pills(tags: &[&str]) -> String {
    if tags.is_empty() {
        return String::new();
    }
    let mut out = String::from("<span class=\"tags\">");
    for tag in tags {
        out.push_str(&format!("<span class=\"tag\">{}</span>", esc(tag)));
    }
    out.push_str("</span>");
    out
}

fn debug_summary(command: &str, stdout: &str, stderr: &str, notes: &str) -> String {
    let combined = format!("{stdout}\n{stderr}\n{notes}");
    let tags = error_tags(&combined);
    let mut next = Vec::new();
    if tags.contains(&"command not found") {
        next.push("Verify the command is installed and available on PATH.");
    }
    if tags.contains(&"permission denied") {
        next.push("Check file permissions, executable bits, or whether sudo is actually required.");
    }
    if tags.contains(&"package not found") {
        next.push("Update package indexes and confirm the package name for this OS/architecture.");
    }
    if tags.contains(&"failed to build") {
        next.push("Read the first compiler error above, then rerun the narrowest build command.");
    }
    if tags.contains(&"port already in use") {
        next.push("Find the process using the port or choose a different port.");
    }
    if next.is_empty() {
        next.push("Search for the most recent related failed command, then compare it with the next successful run.");
    }

    format!(
        "<h2>Local debug summary</h2><div class=\"hint\">\
         <p><strong>What happened?</strong> MemoryWhale recorded <code>{}</code> with its output, error log, notes, and timestamp.</p>\
         <p><strong>What failed?</strong> {}</p>\
         <p><strong>What was tried?</strong> {}</p>\
         <p><strong>Try next:</strong> {}</p></div>\n",
        esc_redacted(command),
        if tags.is_empty() { "No common failure pattern was detected.".to_string() } else { esc(&tags.join(", ")) },
        if notes.trim().is_empty() { "No note was attached to this run.".to_string() } else { esc_redacted(notes) },
        esc(&next.join(" "))
    )
}

fn session_debug_summary(transcript: &str, notes: &str) -> String {
    let tags = error_tags(transcript);
    format!(
        "<h2>Local debug summary</h2><div class=\"hint\">\
         <p><strong>What happened?</strong> MemoryWhale saved a terminal session transcript for later debugging.</p>\
         <p><strong>Detected issues:</strong> {}</p>\
         <p><strong>Context:</strong> {}</p></div>\n",
        if tags.is_empty() { "No common failure pattern was detected.".to_string() } else { esc(&tags.join(", ")) },
        if notes.trim().is_empty() { "No session note was attached.".to_string() } else { esc_redacted(notes) }
    )
}

fn init_min_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS command_runs (id INTEGER PRIMARY KEY, command TEXT NOT NULL,
            argv_json TEXT NOT NULL, cwd TEXT, exit_code INTEGER, stdout TEXT NOT NULL DEFAULT '',
            stderr TEXT NOT NULL DEFAULT '', notes TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS sessions (id INTEGER PRIMARY KEY, shell TEXT, cwd TEXT,
            transcript_path TEXT NOT NULL DEFAULT '', transcript TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '', started_at TEXT NOT NULL DEFAULT '',
            ended_at TEXT NOT NULL DEFAULT '', byte_count INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'finished');
         CREATE TABLE IF NOT EXISTS bookmarks (id INTEGER PRIMARY KEY, label TEXT NOT NULL,
            cwd TEXT, created_at TEXT NOT NULL, command_run_id INTEGER, session_id INTEGER);",
    )
    .map_err(|e| format!("init schema: {e}"))?;
    let _ = conn.execute(
        "ALTER TABLE sessions ADD COLUMN status TEXT NOT NULL DEFAULT 'finished'",
        [],
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_defaults_to_loopback() {
        let config = parse_server_args(Vec::<String>::new()).unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 7071);
        assert!(validate_server_config(&config).is_ok());
    }

    #[test]
    fn lan_requires_authentication() {
        let mut config = parse_server_args(["--lan".to_string()]).unwrap();
        config.token.clear();
        assert!(validate_server_config(&config).is_err());
        config.token = "shared-secret".to_string();
        assert!(validate_server_config(&config).is_ok());
    }

    #[test]
    fn token_is_read_from_form_body_not_query_string() {
        assert_eq!(
            form_param("token=shared%20secret", "token").as_deref(),
            Some("shared secret")
        );
        assert!(form_param("other=value", "token").is_none());
    }
}

/// Import every session `.log` that has no row yet (interrupted recordings).
struct RecoveryReport {
    recovered: usize,
    deleted_empty: usize,
}

fn recover_orphans() -> Result<RecoveryReport, String> {
    let sessions_dir = memorywhale_dir()?.join("sessions");
    if !sessions_dir.exists() {
        return Ok(RecoveryReport {
            recovered: 0,
            deleted_empty: 0,
        });
    }
    let conn = open_db()?;
    init_min_schema(&conn)?;

    let mut entries: Vec<PathBuf> = fs::read_dir(&sessions_dir)
        .map_err(|e| format!("read sessions dir: {e}"))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "log").unwrap_or(false))
        .collect();
    entries.sort();

    let mut recovered = 0;
    let mut deleted_empty = 0;
    for path in entries {
        let path_str = match path.to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        let already: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE transcript_path = ?1",
                params![path_str],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if already > 0 {
            continue;
        }
        let raw = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if raw.is_empty() {
            if fs::remove_file(&path).is_ok() {
                deleted_empty += 1;
            }
            continue;
        }
        let cleaned = clean_transcript(&String::from_utf8_lossy(&raw));
        let started = started_from_filename(&path).unwrap_or_else(|| mtime_rfc3339(&path));
        let ended = mtime_rfc3339(&path);
        conn.execute(
            "INSERT INTO sessions (shell, cwd, transcript_path, transcript, notes, started_at, ended_at, byte_count, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'interrupted')",
            params![
                Option::<String>::None, Option::<String>::None, path_str, cleaned,
                "recovered from transcript (recording was interrupted before saving)",
                started, ended, raw.len() as i64
            ],
        )
        .map_err(|e| format!("insert recovered session: {e}"))?;
        recovered += 1;
    }
    Ok(RecoveryReport {
        recovered,
        deleted_empty,
    })
}

fn started_from_filename(path: &Path) -> Option<String> {
    let stamp = path.file_stem()?.to_str()?.strip_prefix("session-")?;
    let re =
        Regex::new(r"^(\d{4}-\d{2}-\d{2})T(\d{2})-(\d{2})-(\d{2})(\.\d+)?([+-]\d{2})-(\d{2})$")
            .ok()?;
    let c = re.captures(stamp)?;
    Some(format!(
        "{}T{}:{}:{}{}{}:{}",
        &c[1],
        &c[2],
        &c[3],
        &c[4],
        c.get(5).map(|m| m.as_str()).unwrap_or(""),
        &c[6],
        &c[7]
    ))
}

fn mtime_rfc3339(path: &Path) -> String {
    let t = fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    DateTime::<Utc>::from(t).to_rfc3339()
}

fn clean_transcript(input: &str) -> String {
    let osc = Regex::new(r"\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)").unwrap();
    let csi = Regex::new(r"\x1b[@-Z\\-_]|\x1b\[[0-?]*[ -/]*[@-~]").unwrap();
    let ctrl = Regex::new(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]").unwrap();
    let s = osc.replace_all(input, "");
    let s = csi.replace_all(&s, "");
    let s = s.replace('\r', "");
    ctrl.replace_all(&s, "").into_owned()
}

fn open_db() -> Result<Connection, String> {
    let path = database_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create data dir: {e}"))?;
    }
    Connection::open(&path).map_err(|e| format!("open db {}: {e}", path.display()))
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
    if let Some(path) = std::env::var_os("MEMORYWHALE_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }
    Ok(data_base()?.join("MemoryWhale"))
}
