//! Shared pieces of the HTTP-level practice tests (`tests/http_*.rs`): a small blocking HTTP client over `TcpStream`
//! (cookie jar, JSON bodies, chunked or close-delimited responses), a server-sent-events reader, the bot that plays
//! a game, and the choice of server.
//!
//! The suite talks to the server named by `PRACTICE_URL` (for example the Python one:
//! `python3 -m commander_sim.play --no-browser --port 8799 --lan`). `PRACTICE_URL_HOST2` names the address a second
//! computer would use (default: this computer's LAN address and the same port), `PRACTICE_SAVES` / `PRACTICE_IMAGES` the
//! server's saves and image folders (default: the repo's `data/saves` and `data/images`, where the Python server
//! works). When `PRACTICE_URL` is unset the suite
//! should start the Rust server in this process; that server does not exist yet, see `start_rust`.
#![allow(dead_code)]

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------------------------- the server

/// What the tests know about the server under test. Holding it holds the lock that runs the tests one at a time (the
/// server plays one game at a time).
pub struct Server {
    pub addr: String,
    /// where a second computer reaches the same server (a non-loopback address), if there is one
    pub remote_addr: Option<String>,
    pub saves: PathBuf,
    pub images: PathBuf,
    pub rust: bool,
    pub lan: bool,
    _guard: MutexGuard<'static, ()>,
}

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

fn host_port(url: &str) -> String {
    url.trim().trim_start_matches("http://").trim_end_matches('/').to_string()
}

fn lan_ip() -> Option<String> {
    let u = UdpSocket::bind("0.0.0.0:0").ok()?;
    u.connect("10.255.255.255:1").ok()?; // no packet is sent: this only picks the interface
    let ip = u.local_addr().ok()?.ip().to_string();
    (!ip.starts_with("127.") && ip != "0.0.0.0").then_some(ip)
}

/// The server for this test, or `None` (with a note on stderr) when there is none to talk to yet.
pub fn server() -> Option<Server> {
    let guard = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let (addr, rust) = match std::env::var("PRACTICE_URL") {
        Ok(u) if !u.trim().is_empty() => (host_port(&u), false),
        _ => match start_rust() {
            Some(a) => (a, true),
            None => {
                eprintln!("SKIPPED: set PRACTICE_URL (the Rust server does not exist yet)");
                return None;
            }
        },
    };
    let port = addr.rsplit(':').next().unwrap_or("0").to_string();
    let remote_addr = match std::env::var("PRACTICE_URL_HOST2") {
        Ok(u) if !u.trim().is_empty() => Some(host_port(&u)),
        _ => lan_ip().map(|ip| format!("{ip}:{port}")),
    };
    let saves = std::env::var("PRACTICE_SAVES").map(PathBuf::from).unwrap_or_else(|_| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data/saves")
    });
    let images = std::env::var("PRACTICE_IMAGES").map(PathBuf::from).unwrap_or_else(|_| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data/images")
    });
    let mut s = Server { addr, remote_addr, saves, images, rust, lan: false, _guard: guard };
    let mut c = s.client();
    c.quit(); // whatever an earlier test left
    s.lan = c.get("/api/options").json()["lan"].as_bool().unwrap_or(false);
    Some(s)
}

/// Start the Rust server in this process on port 0 with `lan: true`, a temp saves folder and `ai` adaptive, and return
/// its `host:port`. PORT(practice-server): `practice::server::start(Config { port: 0, lan: true, saves: tempdir, .. })`
/// does not exist yet; until it does the suite needs `PRACTICE_URL`.
fn start_rust() -> Option<String> {
    None
}

impl Server {
    pub fn client(&self) -> Client {
        Client::new(&self.addr)
    }
    /// a browser on another computer (its own cookie jar, a non-loopback source address), or None
    pub fn remote_client(&self) -> Option<Client> {
        self.remote_addr.as_ref().map(|a| Client::new(a))
    }
    /// the saves folder's file `name` as JSON, if it exists
    pub fn save_file(&self, name: &str) -> Option<Value> {
        serde_json::from_slice(&std::fs::read(self.saves.join(name)).ok()?).ok()
    }
    pub fn remove_save(&self, name: &str) {
        let _ = std::fs::remove_file(self.saves.join(name));
    }
}

// ---------------------------------------------------------------------------------------------- the client

pub struct Resp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Resp {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body)
            .unwrap_or_else(|e| panic!("status {} body is not JSON ({e}): {:?}", self.status, String::from_utf8_lossy(&self.body)))
    }
    pub fn error(&self) -> String {
        self.json()["error"].as_str().unwrap_or("").to_string()
    }
}

/// One browser: an address and a cookie jar.
pub struct Client {
    pub addr: String,
    pub cookie: Option<String>,
}

fn read_head(r: &mut impl BufRead) -> (u16, Vec<(String, String)>) {
    let mut line = String::new();
    r.read_line(&mut line).expect("response status line");
    let status = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or_else(|| panic!("bad status line {line:?}"));
    let mut headers = Vec::new();
    loop {
        line.clear();
        if r.read_line(&mut line).expect("response header") == 0 || line.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    (status, headers)
}

fn header<'a>(h: &'a [(String, String)], name: &str) -> Option<&'a str> {
    h.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
}

/// A body reader that undoes `Transfer-Encoding: chunked` (or passes bytes through, up to `left` when a length is
/// known).
struct Body<R: BufRead> {
    r: R,
    chunked: bool,
    left: Option<usize>, // bytes left in the chunk (chunked) or in the body (content-length); None: until close
}

impl<R: BufRead> Read for Body<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.chunked && self.left == Some(0) {
            let mut line = String::new();
            self.r.read_line(&mut line)?;
            if line.trim().is_empty() {
                line.clear(); // the CRLF after the previous chunk
                self.r.read_line(&mut line)?;
            }
            let n = usize::from_str_radix(line.trim().split(';').next().unwrap_or("0"), 16).unwrap_or(0);
            if n == 0 {
                return Ok(0);
            }
            self.left = Some(n);
        }
        let want = match self.left {
            Some(n) => buf.len().min(n),
            None => buf.len(),
        };
        if want == 0 {
            return Ok(0);
        }
        let got = self.r.read(&mut buf[..want])?;
        if let Some(n) = self.left.as_mut() {
            *n -= got;
        }
        Ok(got)
    }
}

fn open(addr: &str, timeout: Duration) -> TcpStream {
    let s = TcpStream::connect(addr).unwrap_or_else(|e| panic!("cannot connect to {addr}: {e}"));
    s.set_read_timeout(Some(timeout)).unwrap();
    s.set_write_timeout(Some(Duration::from_secs(30))).unwrap();
    s
}

impl Client {
    pub fn new(addr: &str) -> Client {
        Client { addr: addr.to_string(), cookie: None }
    }

    fn request_bytes(&self, method: &str, path: &str, body: Option<&Value>, extra: &[(&str, String)]) -> Vec<u8> {
        let payload = body.map(|b| b.to_string());
        let mut h = format!("{method} {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n", self.addr);
        if let Some(c) = &self.cookie {
            h += &format!("Cookie: {c}\r\n");
        }
        for (k, v) in extra {
            h += &format!("{k}: {v}\r\n");
        }
        if let Some(p) = &payload {
            h += &format!("Content-Type: application/json\r\nContent-Length: {}\r\n", p.len());
        }
        h += "\r\n";
        let mut out = h.into_bytes();
        if let Some(p) = payload {
            out.extend(p.into_bytes());
        }
        out
    }

    fn remember_cookie(&mut self, headers: &[(String, String)]) {
        for (k, v) in headers {
            if k.eq_ignore_ascii_case("set-cookie") {
                self.cookie = Some(v.split(';').next().unwrap_or("").to_string());
            }
        }
    }

    pub fn call(&mut self, method: &str, path: &str, body: Option<Value>) -> Resp {
        let mut s = open(&self.addr, Duration::from_secs(120));
        s.write_all(&self.request_bytes(method, path, body.as_ref(), &[])).expect("send request");
        let mut r = BufReader::new(s);
        let (status, headers) = read_head(&mut r);
        let chunked = header(&headers, "transfer-encoding").is_some_and(|v| v.to_ascii_lowercase().contains("chunked"));
        let len = header(&headers, "content-length").and_then(|v| v.parse::<usize>().ok());
        let mut b = Body { r, chunked, left: if chunked { Some(0) } else { len } };
        let mut data = Vec::new();
        b.read_to_end(&mut data).expect("response body");
        self.remember_cookie(&headers);
        Resp { status, headers, body: data }
    }

    pub fn get(&mut self, path: &str) -> Resp {
        self.call("GET", path, None)
    }
    pub fn post(&mut self, path: &str, body: Value) -> Resp {
        self.call("POST", path, Some(body))
    }
    pub fn quit(&mut self) -> Resp {
        self.post("/api/quit", json!({}))
    }

    /// open the event stream (`since` is the `?since=` id; `last_event_id` the header a reconnecting page sends)
    pub fn stream(&mut self, since: u64, last_event_id: Option<u64>) -> Stream {
        let mut s = open(&self.addr, Duration::from_secs(5));
        let extra: Vec<(&str, String)> = last_event_id.map(|i| ("Last-Event-ID", i.to_string())).into_iter().collect();
        s.write_all(&self.request_bytes("GET", &format!("/api/events?since={since}"), None, &extra)).expect("send");
        let mut r = BufReader::new(s);
        let (status, headers) = read_head(&mut r);
        self.remember_cookie(&headers);
        let chunked = header(&headers, "transfer-encoding").is_some_and(|v| v.to_ascii_lowercase().contains("chunked"));
        let ctype = header(&headers, "content-type").unwrap_or("").to_string();
        Stream { status, content_type: ctype, body: BufReader::new(Body { r, chunked, left: if chunked { Some(0) } else { None } }) }
    }

    // --- the common calls
    pub fn new_game(&mut self, deck: &str, tier: &str, seed: u64) -> Resp {
        self.post("/api/new", json!({"deck": deck, "tier": tier, "seed": seed, "ai": "adaptive", "images": false}))
    }
    pub fn answer(&mut self, id: u64, answer: Value) -> Resp {
        self.post("/api/answer", json!({"id": id, "answer": answer}))
    }
    pub fn state(&mut self) -> Value {
        self.get("/api/state").json()
    }
}

// ---------------------------------------------------------------------------------------------- the event stream

/// A `text/event-stream` response being read. `next` returns the next `data:` event as JSON, skipping comments
/// (`: keep-alive`), or None when `wait` passes with nothing (or the stream ends).
pub struct Stream {
    pub status: u16,
    pub content_type: String,
    body: BufReader<Body<BufReader<TcpStream>>>,
}

impl Stream {
    /// the next event, with the SSE `id:` line checked against the event's own id
    pub fn next(&mut self, wait: Duration) -> Option<Value> {
        let end = Instant::now() + wait;
        let mut last_id: Option<u64> = None;
        loop {
            let mut line = String::new();
            match self.body.read_line(&mut line) {
                Ok(0) => return None,
                Ok(_) => {}
                Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {
                    if Instant::now() >= end {
                        return None;
                    }
                    continue;
                }
                Err(_) => return None,
            }
            let line = line.trim_end();
            if let Some(i) = line.strip_prefix("id: ") {
                last_id = i.trim().parse().ok();
            } else if let Some(d) = line.strip_prefix("data: ") {
                let ev: Value = serde_json::from_str(d).unwrap_or_else(|e| panic!("event is not JSON ({e}): {d}"));
                if let Some(id) = last_id {
                    assert_eq!(ev["id"].as_u64(), Some(id), "the SSE id line and the event's id differ");
                }
                return Some(ev);
            }
            if Instant::now() >= end {
                return None;
            }
        }
    }

    /// read until an event satisfies `pred` (the events before it are returned too, in order); panics after `secs`
    pub fn until(&mut self, secs: u64, mut pred: impl FnMut(&Value) -> bool) -> Vec<Value> {
        let end = Instant::now() + Duration::from_secs(secs);
        let mut seen = Vec::new();
        while Instant::now() < end {
            if let Some(ev) = self.next(Duration::from_secs(2)) {
                let hit = pred(&ev);
                seen.push(ev);
                if hit {
                    return seen;
                }
            }
        }
        panic!("no matching event within {secs}s; saw kinds {:?}", seen.iter().map(|e| e["kind"].clone()).collect::<Vec<_>>());
    }
}

/// open a stream and wait for the first event satisfying `pred` (as the Python tests' `wait_for`)
pub fn wait_for(c: &mut Client, since: u64, pred: impl FnMut(&Value) -> bool) -> Value {
    c.stream(since, None).until(120, pred).pop().unwrap()
}

pub fn is_request(e: &Value) -> bool {
    e["kind"] == "request"
}

// ---------------------------------------------------------------------------------------------- the bot

/// Answer a request the way the Python tests' `bot` does: land, tap a mana source, then pass; keep every hand;
/// the first choice otherwise. `played` holds the (seat, round) pairs already given a land. `ev` is the request
/// event (its `view` is the table from the asking seat).
pub fn bot(ev: &Value, played: &mut Vec<(String, i64)>) -> Value {
    let req = &ev["request"];
    match req["kind"].as_str().unwrap_or("") {
        "priority" => {}
        "mulligan" => return json!("keep"),
        "attack" => return json!([]),
        "choose" | "target" | "block" | "search" => return json!(0),
        _ => return json!(true),
    }
    let view = &ev["view"];
    let Some(me) = view["players"].as_array().and_then(|ps| ps.iter().find(|p| p["you"] == true)) else {
        return json!({"do": "pass"});
    };
    let key = me["key"].as_str().unwrap_or("").to_string();
    if view["active"] == me["key"] && view["step"] == "main1" {
        let round = view["round"].as_i64().unwrap_or(0);
        let lands: Vec<usize> = me["hand_land"]
            .as_array()
            .map(|a| a.iter().enumerate().filter(|(_, x)| x.as_bool().unwrap_or(false) || x.as_i64().unwrap_or(0) != 0).map(|(i, _)| i).collect())
            .unwrap_or_default();
        if let Some(&l) = lands.first() {
            if !played.contains(&(key.clone(), round)) {
                played.push((key, round));
                return json!({"do": "land", "card": l});
            }
        }
        if let Some(src) = me["mana_sources"].as_array().and_then(|a| a.first()) {
            return json!({"do": "tap", "source": src["id"]});
        }
    }
    json!({"do": "pass"})
}

/// What identifies a decision across two runs of the same game: kind, prompt, choices and the table.
pub fn frozen(ev: &Value) -> String {
    let r = &ev["request"];
    json!({"kind": r["kind"], "prompt": r["prompt"], "choices": r["choices"], "view": ev["view"]}).to_string()
}

/// Answer `n` requests with the bot, reading `c`'s own stream; returns the requests seen (frozen) and the last event id
/// read. The next request is left waiting (and is in the returned list's tail, unanswered).
pub fn play(c: &mut Client, n: usize, hints: bool) -> (Vec<String>, u64) {
    let mut st = c.stream(0, None);
    let mut played = Vec::new();
    let mut seen = Vec::new();
    let mut last;
    loop {
        let ev = st.until(300, is_request).pop().unwrap();
        last = ev["id"].as_u64().unwrap();
        seen.push(frozen(&ev));
        if seen.len() > n {
            return (seen, last);
        }
        if hints && matches!(ev["request"]["kind"].as_str(), Some("priority" | "attack")) {
            let _ = c.post("/api/hint", json!({}));
        }
        let a = bot(&ev, &mut played);
        let r = c.answer(last, a);
        assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    }
}
