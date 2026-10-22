use super::{
    Snapshot, feedback,
    live::{Engine, Query, Scan},
    presentation, render_data,
};
use crate::language::Language;
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::mpsc,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

pub(super) type Capture = Box<dyn FnMut() -> Result<Snapshot, String> + Send>;

struct Session {
    snapshot: Snapshot,
    revision: String,
    data: Value,
    html: String,
    error: Option<String>,
}
fn fingerprint(snapshot: &Snapshot) -> String {
    let mut value = serde_json::to_value(snapshot).unwrap();
    value["id"] = Value::Null;
    value["captured_at"] = Value::Null;
    format!("{:x}", Sha256::digest(serde_json::to_vec(&value).unwrap()))
}
impl Session {
    fn new(snapshot: Snapshot, language: Language, live: bool, hot: bool) -> Result<Self, String> {
        let revision = fingerprint(&snapshot);
        let mut data = presentation(&snapshot, language, live)?;
        data["revision"] = revision.clone().into();
        data["hot"] = hot.into();
        data["server"] = true.into();
        let html = render_data(&data, language, true)?;
        Ok(Self {
            snapshot,
            revision,
            data,
            html,
            error: None,
        })
    }
}

pub(super) fn serve(
    snapshot: Snapshot,
    language: Language,
    port: u16,
    engine: Option<Engine>,
    mut capture: Option<Capture>,
    writer: &mut impl Write,
) -> Result<(), String> {
    let hot = capture.is_some();
    let live = engine.is_some();
    let session = Arc::new(RwLock::new(Session::new(snapshot, language, live, hot)?));
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))
        .map_err(|e| format!("Cannot start review server: {e}"))?;
    let address = listener.local_addr().map_err(|e| e.to_string())?;
    writeln!(writer, "http://{address}/\nCtrl-C to stop.").map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    if let Some(mut capture) = capture.take() {
        let session = session.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(2));
                let result = capture().and_then(|snapshot| {
                    if fingerprint(&snapshot) == session.read().unwrap().revision {
                        return Ok(None);
                    }
                    Session::new(snapshot, language, live, hot).map(Some)
                });
                let mut current = session.write().unwrap();
                match result {
                    Ok(Some(next)) => *current = next,
                    Ok(None) => current.error = None,
                    Err(error) => current.error = Some(error),
                }
            }
        });
    }
    let authority = Arc::new(address.to_string());
    let revision = session.read().unwrap().revision.clone();
    let engine = Arc::new(engine.map(|engine| Mutex::new((revision, engine))));
    let active = Arc::new(AtomicUsize::new(0));
    let (feedback_sender, feedback_receiver) = mpsc::sync_channel(8);
    let mut delivered = feedback::Seen::new();
    let feedback_registry = Arc::new(Mutex::new(feedback::Registry::default()));
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    loop {
        while let Ok(delivery) = feedback_receiver.try_recv() {
            feedback::deliver(
                delivery,
                &mut delivered,
                &mut feedback_registry.lock().unwrap(),
                writer,
            )?;
        }
        let mut stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(25));
                continue;
            }
            Err(error) => return Err(format!("Review server failed: {error}")),
        };
        stream
            .set_nonblocking(false)
            .map_err(|error| error.to_string())?;
        if active.load(Ordering::Relaxed) >= 8 {
            let _ = stream.shutdown(std::net::Shutdown::Both);
            continue;
        }
        active.fetch_add(1, Ordering::Relaxed);
        let (session, authority, engine, active) = (
            session.clone(),
            authority.clone(),
            engine.clone(),
            active.clone(),
        );
        let feedback_sender = feedback_sender.clone();
        let feedback_registry = feedback_registry.clone();
        std::thread::spawn(move || {
            let _ = respond(
                &mut stream,
                &authority,
                &session,
                engine.as_ref().as_ref(),
                language,
                hot,
                (&feedback_sender, &feedback_registry),
            );
            active.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

fn reply(
    stream: &mut TcpStream,
    status: &str,
    kind: &str,
    body: &str,
    head: bool,
) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {kind}; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    if !head {
        stream.write_all(body.as_bytes())?;
    }
    Ok(())
}
fn respond(
    stream: &mut TcpStream,
    authority: &str,
    session: &RwLock<Session>,
    engine: Option<&Mutex<(String, Engine)>>,
    language: Language,
    hot: bool,
    feedback: (&feedback::Sender, &Mutex<feedback::Registry>),
) -> std::io::Result<()> {
    let (feedback_sender, feedback_registry) = feedback;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut request = Vec::new();
    let mut byte = [0];
    while request.len() < 8192 {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::TimedOut))?;
        stream.set_read_timeout(Some(remaining))?;
        if stream.read(&mut byte)? == 0 {
            return Ok(());
        }
        request.push(byte[0]);
        if request.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let request = String::from_utf8_lossy(&request);
    let first: Vec<_> = request
        .split("\r\n")
        .next()
        .unwrap_or("")
        .split_whitespace()
        .collect();
    if first.first() != Some(&"POST") {
        let current = session.read().unwrap();
        let (status, body, head) = response(&request, authority, &current.html);
        return reply(stream, status, "text/html", body, head);
    }
    let error = |stream: &mut TcpStream, status: &str, message: &str| {
        reply(
            stream,
            status,
            "application/json",
            &json!({"error":message}).to_string(),
            false,
        )
    };
    if first.len() != 3 || first[2] != "HTTP/1.1" || !request.ends_with("\r\n\r\n") {
        return error(stream, "400 Bad Request", "Invalid request");
    }
    let headers: Vec<_> = request
        .split("\r\n")
        .skip(1)
        .filter_map(|line| line.split_once(':'))
        .collect();
    let values = |name: &str| {
        headers
            .iter()
            .filter(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.trim())
            .collect::<Vec<_>>()
    };
    if values("host") != [authority] || !values("transfer-encoding").is_empty() {
        return error(stream, "400 Bad Request", "Invalid headers");
    }
    let lengths = values("content-length");
    let length = if lengths.len() == 1 {
        lengths[0].parse::<usize>().ok().filter(|n| {
            *n <= if first[1] == "/api/feedback" {
                1024 * 1024
            } else {
                8192
            }
        })
    } else {
        None
    };
    let Some(length) = length else {
        return error(stream, "400 Bad Request", "Invalid request length");
    };
    let mut body = vec![0; length];
    let mut received = 0;
    while received < body.len() {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::TimedOut))?;
        stream.set_read_timeout(Some(remaining))?;
        let count = stream.read(&mut body[received..])?;
        if count == 0 {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        received += count;
    }
    if ![
        "/api/refresh",
        "/api/related",
        "/api/uncovered",
        "/api/feedback",
        "/api/feedback/resolve",
    ]
    .contains(&first[1])
    {
        return error(
            stream,
            "405 Method Not Allowed",
            "Live judgments are disabled",
        );
    }
    if values("content-type") != ["application/json"]
        || values("origin") != [format!("http://{authority}").as_str()]
    {
        return error(
            stream,
            "403 Forbidden",
            "Only same-origin JSON requests are accepted",
        );
    }
    if first[1] == "/api/feedback" {
        let current = session.read().unwrap();
        let (base, target) = (
            current.snapshot.base.clone(),
            current.snapshot.target.clone(),
        );
        drop(current);
        return match feedback::submit(&body, &base, &target, feedback_sender) {
            Ok(receipt) => reply(
                stream,
                "200 OK",
                "application/json",
                &receipt.to_string(),
                false,
            ),
            Err(message) => error(stream, "400 Bad Request", &message),
        };
    }
    if first[1] == "/api/feedback/resolve" {
        let current = session.read().unwrap();
        if current.error.is_some() {
            return error(
                stream,
                "409 Conflict",
                "Capture must recover before resolving comments",
            );
        }
        return match feedback_registry.lock().unwrap().resolve(
            &body,
            &current.snapshot,
            &current.revision,
        ) {
            Ok(result) => reply(
                stream,
                "200 OK",
                "application/json",
                &result.to_string(),
                false,
            ),
            Err(message) => error(stream, "400 Bad Request", &message),
        };
    }
    if first[1] == "/api/refresh" {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Refresh {
            revision: String,
        }
        let Ok(query) = serde_json::from_slice::<Refresh>(&body) else {
            return error(stream, "400 Bad Request", "Invalid refresh query");
        };
        let current = session.read().unwrap();
        let result = json!({"revision":current.revision,"error":current.error,
            "resolutions":feedback_registry.lock().unwrap().resolutions(),
            "data":if hot && query.revision != current.revision { Some(&current.data) } else { None }});
        return reply(
            stream,
            "200 OK",
            "application/json",
            &result.to_string(),
            false,
        );
    }
    let Some(engine) = engine else {
        return error(
            stream,
            "405 Method Not Allowed",
            "Live judgments are disabled",
        );
    };
    let Ok(mut query) = serde_json::from_slice::<Value>(&body) else {
        return error(stream, "400 Bad Request", "Invalid judgment query");
    };
    let revision = query
        .as_object_mut()
        .and_then(|query| query.remove("revision"));
    let mut engine = match engine.try_lock() {
        Ok(engine) => engine,
        Err(_) => {
            return error(
                stream,
                "503 Service Unavailable",
                "A judgment is in progress. Try again shortly.",
            );
        }
    };
    let current = session.read().unwrap();
    if (hot || revision.is_some())
        && revision.as_ref().and_then(Value::as_str) != Some(&current.revision)
    {
        return error(
            stream,
            "409 Conflict",
            "The comparison changed. Wait for the updated text and retry.",
        );
    }
    if current.error.is_some() && hot {
        return error(
            stream,
            "409 Conflict",
            "Cannot refresh the comparison. Judgments are paused until capture recovers.",
        );
    }
    if engine.0 != current.revision {
        engine.1.replace_inputs(&current.snapshot, language);
        engine.0 = current.revision.clone();
    }
    let used_revision = current.revision.clone();
    drop(current);
    let result = if first[1] == "/api/related" {
        serde_json::from_value::<Query>(query)
            .map_err(|_| "Invalid passage query".to_owned())
            .and_then(|query| engine.1.related(query))
    } else {
        serde_json::from_value::<Scan>(query)
            .map_err(|_| "Invalid scan query".to_owned())
            .and_then(|scan| engine.1.uncovered(scan))
    }
    .map(|mut value| {
        value["revision"] = used_revision.into();
        value
    });
    match result {
        Ok(value) => reply(
            stream,
            "200 OK",
            "application/json",
            &value.to_string(),
            false,
        ),
        Err(message) => error(stream, "400 Bad Request", &message),
    }
}
fn response<'a>(request: &str, authority: &str, html: &'a str) -> (&'static str, &'a str, bool) {
    let mut lines = request.split("\r\n");
    let parts: Vec<_> = lines.next().unwrap_or("").split_whitespace().collect();
    let head = parts.first() == Some(&"HEAD");
    if !request.ends_with("\r\n\r\n") || parts.len() != 3 || parts[2] != "HTTP/1.1" {
        return ("400 Bad Request", "Invalid request", head);
    }
    let hosts: Vec<_> = lines
        .filter_map(|line| line.split_once(':'))
        .filter(|(name, _)| name.eq_ignore_ascii_case("host"))
        .map(|(_, value)| value.trim())
        .collect();
    if hosts != [authority] {
        return ("400 Bad Request", "Invalid Host", head);
    }
    if !["GET", "HEAD"].contains(&parts[0]) {
        return ("405 Method Not Allowed", "Read-only review", head);
    }
    if parts[1] != "/" {
        return ("404 Not Found", "Not found", head);
    }
    ("200 OK", html, head)
}
#[cfg(test)]
mod tests {
    use super::response;
    #[test]
    fn only_the_artifact_is_served_to_the_bound_host() {
        let authority = "127.0.0.1:8765";
        let request = |line: &str, host: &str| format!("{line} HTTP/1.1\r\nHost: {host}\r\n\r\n");
        assert_eq!(
            response(&request("GET /", authority), authority, "html"),
            ("200 OK", "html", false)
        );
        assert_eq!(
            response(&request("HEAD /", authority), authority, "html"),
            ("200 OK", "html", true)
        );
        assert_eq!(
            response(&request("GET /src/main.rs", authority), authority, "html").0,
            "404 Not Found"
        );
        assert_eq!(
            response(&request("POST /", authority), authority, "html").0,
            "405 Method Not Allowed"
        );
        assert_eq!(
            response(&request("GET /", "attacker.invalid"), authority, "html").0,
            "400 Bad Request"
        );
    }
}
