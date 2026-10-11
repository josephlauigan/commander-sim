//! A thin HTTP/1.1 layer on `std::net` + `httparse`: parse a request (headers, `Content-Length` body, 1 MB cap),
//! write a response, static files with a path-traversal check. A thread serves each connection; keep-alive is
//! supported; no chunked request bodies (the page never sends them).

use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr};
use std::path::{Component, Path, PathBuf};

pub const MAX_BODY: usize = 1 << 20;
const MAX_HEAD: usize = 64 << 10;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    /// the path, without the query
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub peer: Option<SocketAddr>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }

    /// the cookie `name`, if sent
    pub fn cookie(&self, name: &str) -> Option<String> {
        let mut found = None;
        for h in self.headers.iter().filter(|(k, _)| k.eq_ignore_ascii_case("cookie")) {
            for part in h.1.split(';') {
                if let Some((k, v)) = part.trim().split_once('=')
                    && k == name
                    && !v.is_empty()
                {
                    found = Some(v.to_string()); // the last one wins, as in Python
                }
            }
        }
        found
    }

    pub fn query_param(&self, name: &str) -> Option<String> {
        self.query.split('&').find_map(|kv| {
            let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
            (k == name).then(|| v.to_string())
        })
    }

    /// the request comes from the computer running the server (the host)
    pub fn local(&self) -> bool {
        match self.peer.map(|p| p.ip()) {
            Some(IpAddr::V4(a)) => a.is_loopback(),
            Some(IpAddr::V6(a)) => a.is_loopback() || a.to_ipv4_mapped().is_some_and(|v| v.is_loopback()),
            None => false,
        }
    }

    pub fn keep_alive(&self) -> bool {
        !self.header("connection").is_some_and(|c| c.eq_ignore_ascii_case("close"))
    }
}

/// why a request could not be read
#[derive(Debug, PartialEq)]
pub enum ReadError {
    /// the peer closed the connection (or timed out) between requests
    Closed,
    /// answer with this status and message, then close
    Bad(u16, &'static str),
}

/// A connection with its read buffer.
pub struct Conn<S> {
    pub stream: S,
    buf: Vec<u8>,
}

impl<S: Read + Write> Conn<S> {
    pub fn new(stream: S) -> Conn<S> {
        Conn { stream, buf: Vec::new() }
    }

    fn fill(&mut self) -> Result<usize, ReadError> {
        let mut chunk = [0u8; 8192];
        match self.stream.read(&mut chunk) {
            Ok(0) | Err(_) => Err(ReadError::Closed),
            Ok(n) => {
                self.buf.extend_from_slice(&chunk[..n]);
                Ok(n)
            }
        }
    }

    pub fn read_request(&mut self, peer: Option<SocketAddr>) -> Result<Request, ReadError> {
        let (head_len, mut req, clen) = loop {
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut r = httparse::Request::new(&mut headers);
            match r.parse(&self.buf) {
                Ok(httparse::Status::Complete(n)) => {
                    let target = r.path.unwrap_or("/");
                    let (path, query) = target.split_once('?').unwrap_or((target, ""));
                    let hs: Vec<(String, String)> = r
                        .headers
                        .iter()
                        .map(|h| (h.name.to_string(), String::from_utf8_lossy(h.value).trim().to_string()))
                        .collect();
                    let req = Request {
                        method: r.method.unwrap_or("GET").to_string(),
                        path: path.to_string(),
                        query: query.to_string(),
                        headers: hs,
                        body: vec![],
                        peer,
                    };
                    let clen = match req.header("content-length") {
                        None => 0,
                        Some(v) => v.parse::<usize>().map_err(|_| ReadError::Bad(400, "bad Content-Length"))?,
                    };
                    break (n, req, clen);
                }
                Ok(httparse::Status::Partial) => {
                    if self.buf.len() > MAX_HEAD {
                        return Err(ReadError::Bad(431, "request headers too large"));
                    }
                    self.fill()?;
                }
                Err(_) => return Err(ReadError::Bad(400, "bad request")),
            }
        };
        if req.header("transfer-encoding").is_some() {
            return Err(ReadError::Bad(411, "chunked request bodies are not supported"));
        }
        if clen > MAX_BODY {
            return Err(ReadError::Bad(413, "request body too large"));
        }
        while self.buf.len() < head_len + clen {
            self.fill()?;
        }
        req.body = self.buf[head_len..head_len + clen].to_vec();
        self.buf.drain(..head_len + clen);
        Ok(req)
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub ctype: String,
    pub body: Vec<u8>,
    pub cookie: Option<String>,
}

pub const COOKIE: &str = "practice_seat";

pub fn set_cookie(token: &str) -> String {
    format!("{COOKIE}={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age=31536000")
}

fn reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        409 => "Conflict",
        411 => "Length Required",
        413 => "Payload Too Large",
        431 => "Request Header Fields Too Large",
        _ => "Error",
    }
}

impl Response {
    pub fn json(status: u16, v: &serde_json::Value) -> Response {
        Response { status, ctype: "application/json".into(), body: v.to_string().into_bytes(), cookie: None }
    }

    pub fn bytes(status: u16, ctype: &str, body: Vec<u8>) -> Response {
        Response { status, ctype: ctype.into(), body, cookie: None }
    }

    pub fn write_to(&self, w: &mut impl Write, keep_alive: bool) -> std::io::Result<()> {
        let mut head = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n",
            self.status,
            reason(self.status),
            self.ctype,
            self.body.len()
        );
        if let Some(c) = &self.cookie {
            head += &format!("Set-Cookie: {c}\r\n");
        }
        head += if keep_alive { "Connection: keep-alive\r\n\r\n" } else { "Connection: close\r\n\r\n" };
        w.write_all(head.as_bytes())?;
        w.write_all(&self.body)?;
        w.flush()
    }
}

pub fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).map(|e| format!(".{e}")).as_deref() {
        Some(".html") => "text/html; charset=utf-8",
        Some(".js") => "text/javascript; charset=utf-8",
        Some(".css") => "text/css; charset=utf-8",
        Some(".gif") => "image/gif",
        Some(".png") => "image/png",
        Some(".jpg") => "image/jpeg",
        Some(".svg") => "image/svg+xml",
        Some(".json") => "application/json",
        _ => "application/octet-stream",
    }
}

/// `root/name` normalised (`..` and `.` resolved without touching the disk); `None` when it leaves `root`
pub fn safe_join(root: &Path, name: &str) -> Option<PathBuf> {
    let mut out = root.to_path_buf();
    for c in Path::new(name.trim_start_matches('/')).components() {
        match c {
            Component::Normal(p) => out.push(p),
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
                if !out.starts_with(root) {
                    return None;
                }
            }
            _ => return None,
        }
    }
    (out != root && out.starts_with(root)).then_some(out)
}

/// a file under `root` (404 JSON when it is missing or outside)
pub fn static_file(root: &Path, name: &str) -> Response {
    let not_found = || Response::json(404, &serde_json::json!({"error": "not found"}));
    let Some(path) = safe_join(root, name) else { return not_found() };
    if !path.is_file() {
        return not_found();
    }
    match std::fs::read(&path) {
        Ok(data) => Response::bytes(200, content_type(&path), data),
        Err(_) => not_found(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct Pipe {
        input: Cursor<Vec<u8>>,
        out: Vec<u8>,
    }
    impl Read for Pipe {
        fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
            self.input.read(b)
        }
    }
    impl Write for Pipe {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.out.extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    fn conn(s: &str) -> Conn<Pipe> {
        Conn::new(Pipe { input: Cursor::new(s.as_bytes().to_vec()), out: vec![] })
    }

    #[test]
    fn parses_a_post_with_body_and_pipelined_next() {
        let mut c = conn(
            "POST /api/answer?x=1 HTTP/1.1\r\nHost: a\r\nContent-Length: 7\r\nCookie: a=b; practice_seat=tok\r\n\r\n{\"a\":1}GET /next HTTP/1.1\r\n\r\n",
        );
        let r = c.read_request(None).unwrap();
        assert_eq!((r.method.as_str(), r.path.as_str(), r.query.as_str()), ("POST", "/api/answer", "x=1"));
        assert_eq!(r.body, b"{\"a\":1}");
        assert_eq!(r.cookie(COOKIE).as_deref(), Some("tok"));
        let r2 = c.read_request(None).unwrap();
        assert_eq!(r2.path, "/next");
        assert_eq!(c.read_request(None).unwrap_err(), ReadError::Closed);
    }

    #[test]
    fn refuses_big_bodies_and_garbage() {
        let mut c = conn(&format!("POST / HTTP/1.1\r\nContent-Length: {}\r\n\r\n", MAX_BODY + 1));
        assert_eq!(c.read_request(None).unwrap_err(), ReadError::Bad(413, "request body too large"));
        let mut c = conn("\x01\x02 nonsense\r\n\r\n");
        assert!(matches!(c.read_request(None), Err(ReadError::Bad(400, _))));
        let mut c = conn("GET / HTTP/1.1\r\nContent-Length: 10\r\n\r\nabc");
        assert_eq!(c.read_request(None).unwrap_err(), ReadError::Closed);
    }

    #[test]
    fn local_means_loopback() {
        let mut r = conn("GET / HTTP/1.1\r\n\r\n").read_request(Some("127.0.0.1:5".parse().unwrap())).unwrap();
        assert!(r.local());
        r.peer = Some("[::1]:5".parse().unwrap());
        assert!(r.local());
        r.peer = Some("[::ffff:127.0.0.1]:5".parse().unwrap());
        assert!(r.local());
        r.peer = Some("192.168.1.9:5".parse().unwrap());
        assert!(!r.local());
    }

    #[test]
    fn traversal_is_refused() {
        let root = Path::new("/srv/static");
        assert_eq!(safe_join(root, "app.js"), Some(PathBuf::from("/srv/static/app.js")));
        assert_eq!(safe_join(root, "a/../b.css"), Some(PathBuf::from("/srv/static/b.css")));
        assert_eq!(safe_join(root, "../secret"), None);
        assert_eq!(safe_join(root, "a/../../secret"), None);
        assert_eq!(safe_join(root, "../static2/x"), None);
        assert_eq!(safe_join(root, ""), None);
        assert_eq!(safe_join(root, "."), None);
    }

    #[test]
    fn response_has_the_headers_the_page_needs() {
        let mut r = Response::json(409, &serde_json::json!({"error": "x"}));
        r.cookie = Some(set_cookie("t"));
        let mut out = vec![];
        r.write_to(&mut out, false).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.starts_with("HTTP/1.1 409 Conflict\r\n") && s.contains("Content-Length: 13\r\n"));
        assert!(s.contains("Set-Cookie: practice_seat=t; Path=/; HttpOnly; SameSite=Strict; Max-Age=31536000"));
        assert!(s.ends_with("{\"error\":\"x\"}"));
    }
}
