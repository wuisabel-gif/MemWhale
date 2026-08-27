//! End-to-end check of `mw-serve POST /mcp` and `mw-serve --print-token`.
#![cfg(unix)]

use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::time::Duration;

fn modern_meta() -> Value {
    json!({
        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientInfo": {"name": "http-it", "version": "1"},
        "io.modelcontextprotocol/clientCapabilities": {}
    })
}

fn sandbox(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "mw-mcp-http-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_for_port(port: u16) {
    for _ in 0..50 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("mw-serve did not bind 127.0.0.1:{port}");
}

fn post_mcp(port: u16, body: &str) -> String {
    let request = format!(
        "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

#[test]
fn print_token_mints_and_reuses_the_file() {
    let dir = sandbox("print-token");
    let bin = env!("CARGO_BIN_EXE_mw-serve");
    let first = Command::new(bin)
        .arg("--print-token")
        .env("MEMORYWHALE_DATA_DIR", &dir)
        .env_remove("MEMORYWHALE_TOKEN")
        .output()
        .expect("print-token");
    assert!(first.status.success(), "{first:?}");
    let token = String::from_utf8(first.stdout).unwrap();
    let token = token.trim();
    assert_eq!(token.len(), 64, "expected 32-byte hex, got {token:?}");
    let file = std::fs::read_to_string(dir.join("serve.token")).unwrap();
    assert_eq!(file.trim(), token);

    let second = Command::new(bin)
        .arg("--print-token")
        .env("MEMORYWHALE_DATA_DIR", &dir)
        .env_remove("MEMORYWHALE_TOKEN")
        .output()
        .expect("print-token again");
    assert!(second.status.success());
    assert_eq!(String::from_utf8_lossy(&second.stdout).trim(), token);
}

#[test]
fn serve_mcp_discovers_current_protocol() {
    let dir = sandbox("serve");
    let port = free_port();
    let mut child = Command::new(env!("CARGO_BIN_EXE_mw-serve"))
        .args(["--port", &port.to_string()])
        .env("MEMORYWHALE_DATA_DIR", &dir)
        .env_remove("MEMORYWHALE_TOKEN")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mw-serve");
    wait_for_port(port);

    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "server/discover",
        "params": {"_meta": modern_meta()}
    })
    .to_string();
    let response = post_mcp(port, &body);
    let _ = child.kill();
    let _ = child.wait();

    assert!(
        response.starts_with("HTTP/1.1 200 OK"),
        "unexpected response: {response}"
    );
    assert!(response.contains("2026-07-28"));
    assert!(
        response.contains("\"resultType\":\"complete\""),
        "unexpected discovery result: {response}"
    );
}

#[test]
fn serve_mcp_accepts_rho_initialize_handshake() {
    let dir = sandbox("serve-rho");
    let port = free_port();
    let mut child = Command::new(env!("CARGO_BIN_EXE_mw-serve"))
        .args(["--port", &port.to_string()])
        .env("MEMORYWHALE_DATA_DIR", &dir)
        .env_remove("MEMORYWHALE_TOKEN")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mw-serve");
    wait_for_port(port);

    let init = post_mcp(
        port,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {"roots": {"listChanged": false}},
                "clientInfo": {"name": "rho", "version": "2.2.0"}
            }
        })
        .to_string(),
    );
    assert!(init.starts_with("HTTP/1.1 200 OK"), "{init}");
    assert!(init.contains("2025-11-25"), "{init}");

    let listed = post_mcp(
        port,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        })
        .to_string(),
    );
    let _ = child.kill();
    let _ = child.wait();
    assert!(listed.starts_with("HTTP/1.1 200 OK"), "{listed}");
    assert!(listed.contains("recent_errors"), "{listed}");
}
