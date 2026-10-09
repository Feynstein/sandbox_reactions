//! The desktop's ready signal (m0_contrat.md §3.4): `--status-port <port>` serves `GET /status` on 127.0.0.1 only, as
//! plain HTTP/1.0 on std::net, with `{"status": "booting" | "ready" | "error", "frame", "step", "sim_time", "adapter",
//! "fps"}`. "ready" is raised by the app once its first frame is presented, never when the window is created. Off
//! without the flag; never in the web build (the module is native-only).

use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Booting,
    Ready,
    Error,
}

impl State {
    fn as_str(self) -> &'static str {
        match self {
            State::Booting => "booting",
            State::Ready => "ready",
            State::Error => "error",
        }
    }
}

/// What `/status` reports; the app writes it every frame, the server thread reads it per request.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub state: State,
    pub frame: u64,
    pub step: u64,
    pub sim_time: f64,
    pub adapter: String,
    pub fps: f64,
}

impl Snapshot {
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "status": self.state.as_str(),
            "frame": self.frame,
            "step": self.step,
            "sim_time": self.sim_time,
            "adapter": self.adapter,
            "fps": self.fps,
        })
        .to_string()
    }
}

/// The shared snapshot behind the endpoint.
pub struct Status(Mutex<Snapshot>);

impl Status {
    pub fn update(&self, f: impl FnOnce(&mut Snapshot)) {
        f(&mut self.0.lock().unwrap_or_else(PoisonError::into_inner));
    }

    pub fn snapshot(&self) -> Snapshot {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

/// Binds 127.0.0.1:`port` now — so a poller sees "booting" while the window and the device come up — and answers on
/// a background thread for the life of the process.
pub fn serve(port: u16) -> std::io::Result<Arc<Status>> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
    let status = Arc::new(Status(Mutex::new(Snapshot {
        state: State::Booting,
        frame: 0,
        step: 0,
        sim_time: 0.0,
        adapter: String::new(),
        fps: 0.0,
    })));
    let shared = Arc::clone(&status);
    std::thread::Builder::new().name("sr-status".into()).spawn(move || {
        for stream in listener.incoming().flatten() {
            // One request at a time: a poller asks a few times a second, and a stalled client times out below.
            let _ = answer(stream, &shared);
        }
    })?;
    Ok(status)
}

fn answer(mut stream: TcpStream, status: &Status) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    // The request line and headers: read until the blank line (or 8 KiB, or the client stops sending).
    let mut head = Vec::with_capacity(512);
    let mut buf = [0u8; 512];
    while !head.windows(4).any(|w| w == b"\r\n\r\n") && !head.windows(2).any(|w| w == b"\n\n") && head.len() < 8192 {
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => head.extend_from_slice(&buf[..n]),
        }
    }
    let line = String::from_utf8_lossy(&head);
    let mut parts = line.lines().next().unwrap_or("").split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    let (code, body) = match (method, target.split('?').next().unwrap_or("")) {
        ("GET", "/status") => ("200 OK", status.snapshot().to_json()),
        ("GET", _) => ("404 Not Found", r#"{"error": "not found; GET /status"}"#.to_string()),
        _ => ("405 Method Not Allowed", r#"{"error": "GET /status only"}"#.to_string()),
    };
    let reply = format!(
        "HTTP/1.0 {code}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nCache-Control: no-store\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(reply.as_bytes())?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(port: u16, path: &str) -> String {
        let mut s = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        write!(s, "GET {path} HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n").unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        out
    }

    #[test]
    fn status_booting_then_ready() {
        let port = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap().local_addr().unwrap().port();
        let status = serve(port).unwrap();
        let first = get(port, "/status");
        assert!(first.starts_with("HTTP/1.0 200 OK\r\n"), "{first}");
        let body: serde_json::Value = serde_json::from_str(first.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["status"], "booting");
        for key in ["frame", "step", "sim_time", "adapter", "fps"] {
            assert!(body.get(key).is_some(), "missing {key}: {body}");
        }
        status.update(|s| {
            s.state = State::Ready;
            s.frame = 3;
        });
        let body: serde_json::Value =
            serde_json::from_str(get(port, "/status").split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!((body["status"].as_str(), body["frame"].as_u64()), (Some("ready"), Some(3)));
        assert!(get(port, "/other").starts_with("HTTP/1.0 404"));
    }
}
