//! End-to-end auth boundary checks for the local dashboard.

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn request(address: &str, request: &str) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn wait_for_server(address: &str) {
    for _ in 0..50 {
        if TcpStream::connect(address).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("dashboard did not start at {address}");
}

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn dashboard_auth_cannot_be_bypassed_by_query_and_cookie_flow_works() {
    let data_dir = std::env::temp_dir().join(format!("mw-serve-auth-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data_dir);
    std::fs::create_dir_all(&data_dir).unwrap();
    let port = free_port();
    let address = format!("127.0.0.1:{port}");
    let child = Command::new(env!("CARGO_BIN_EXE_mw-serve"))
        .env("MEMORYWHALE_DATA_DIR", &data_dir)
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
            "--token",
            "shared-secret",
            "--api",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let _server = Server(child);
    wait_for_server(&address);

    let unauthenticated = request(&address, "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert!(unauthenticated.starts_with("HTTP/1.1 401 Unauthorized"));

    let unauthenticated_api = request(
        &address,
        "GET /api/v1/health HTTP/1.1\r\nHost: localhost\r\n\r\n",
    );
    assert!(unauthenticated_api.starts_with("HTTP/1.1 401 Unauthorized"));
    assert!(unauthenticated_api.contains("Content-Type: application/json"));
    assert!(unauthenticated_api.contains("\"code\":\"unauthorized\""));
    assert!(unauthenticated_api.contains("WWW-Authenticate: Bearer"));

    let bearer_api = request(
        &address,
        "GET /api/v1/health HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer shared-secret\r\n\r\n",
    );
    assert!(bearer_api.starts_with("HTTP/1.1 200 OK"));
    assert!(bearer_api.contains("Content-Type: application/json"));

    let query_token = request(
        &address,
        "GET /?token=shared-secret HTTP/1.1\r\nHost: localhost\r\n\r\n",
    );
    assert!(query_token.starts_with("HTTP/1.1 401 Unauthorized"));

    let body = "token=wrong";
    let wrong = request(
        &address,
        &format!(
            "POST /login HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        ),
    );
    assert!(wrong.starts_with("HTTP/1.1 401 Unauthorized"));
    assert!(!wrong.contains("Set-Cookie: mw_token="));

    let body = "token=shared-secret";
    let login = request(
        &address,
        &format!(
            "POST /login HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        ),
    );
    assert!(login.starts_with("HTTP/1.1 303 See Other"));
    let cookie = login
        .lines()
        .find_map(|line| line.strip_prefix("Set-Cookie: "))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    let authenticated = request(
        &address,
        &format!("GET / HTTP/1.1\r\nHost: localhost\r\nCookie: {cookie}\r\n\r\n"),
    );
    assert!(authenticated.starts_with("HTTP/1.1 200 OK"));
    let _ = std::fs::remove_dir_all(data_dir);
}
