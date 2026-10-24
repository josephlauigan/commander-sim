//! The server against a fake session: routes, seats by cookie, SSE with reconnect, 403 for other computers,
//! path traversal. (The HTTP suite shared with the Python server is practice-httptests'.)

use practice::server::api::{Info, SessionApi, SessionFactory};
use practice::server::{Config, ServerHandle, start_with};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Default)]
struct Fake {
    events: Mutex<Vec<Value>>,
    cond: Condvar,
    answers: Mutex<Vec<(String, Value)>>,
    started: Mutex<bool>,
    closed: Mutex<bool>,
    deck: String,
    partner: Option<String>,
}

impl Fake {
    fn push(&self, ev: Value) {
        self.events.lock().unwrap().push(ev);
        self.cond.notify_all();
    }
}

impl SessionApi for Fake {
    fn info(&self) -> Info {
        Info {
            deck: self.deck.clone(),
            tier: "t1".into(),
            seed: 7,
            seats: vec![self.deck.clone(), "a".into(), "b".into(), "c".into()],
            profile: "loose".into(),
            ai: "adaptive".into(),
            humans: vec![self.deck.clone()],
            names: vec![(self.deck.clone(), "Me".into())],
            partner: self.partner.clone(),
            waiting: true,
            finished: false,
        }
    }
    fn autopass_of(&self, _: &str) -> String {
        "respond".into()
    }
    fn events_after(&self, cursor: u64, timeout: Duration) -> (u64, Vec<Value>) {
        let mut g = self.events.lock().unwrap();
        if g.len() as u64 <= cursor {
            g = self.cond.wait_timeout(g, timeout).unwrap().0;
        }
        (g.len() as u64, g[cursor as usize..].to_vec())
    }
    fn start(&self) {
        *self.started.lock().unwrap() = true;
        self.push(json!({"kind": "request", "seat": self.deck, "request": {"kind": "main", "prompt": "Your move", "choices": [],
            "data": {"view": {"turn": 1}, "x": 1}}}));
    }
    fn answer(&self, seat: &str, value: Value) {
        self.answers.lock().unwrap().push((seat.into(), value));
    }
    fn undo(&self, _: i64, _: &str) -> Option<String> {
        Some("Nothing to undo yet.".into())
    }
    fn try_it(&self, _: &Value, _: &str) -> Option<String> {
        Some("That decision has no AI choice to try.".into())
    }
    fn undo_point(&self, _: i64, _: &str) -> Option<usize> {
        None
    }
    fn answer_seats(&self) -> Vec<String> {
        vec![]
    }
    fn has_try_entry(&self, _: &Value, _: &str) -> bool {
        false
    }
    fn set_autopass(&self, _: &str, mode: &Value) -> Option<String> {
        (!matches!(mode.as_str(), Some("respond" | "stack" | "all")))
            .then(|| "auto-pass: one of respond, stack, all".to_string())
    }
    fn hint(&self, _: &str) -> Result<Value, String> {
        Ok(json!({"text": "pass", "detail": "", "choice": null}))
    }
    fn review(&self, _: &str) -> Result<Value, String> {
        Err("no review".into())
    }
    fn saved(&self, _: Option<Duration>) -> Result<Value, String> {
        Ok(json!({"deck": self.deck, "tier": "t1", "seed": 7, "round": 2, "finished": false}))
    }
    fn close(&self, _: Duration) {
        *self.closed.lock().unwrap() = true;
    }
}

struct Factory(Mutex<Vec<Arc<Fake>>>);

impl SessionFactory for Factory {
    fn new_session(&self, opts: &Value) -> Result<Arc<dyn SessionApi>, String> {
        let f = Arc::new(Fake { deck: opts["deck"].as_str().unwrap().into(), ..Default::default() });
        self.0.lock().unwrap().push(f.clone());
        Ok(f)
    }
    fn check_two(&self, _: &Value, _: &str) -> Result<(), String> {
        Ok(())
    }
    fn load_session(&self, data: &Value, _: bool) -> Result<Arc<dyn SessionApi>, String> {
        let f = Arc::new(Fake { deck: data["deck"].as_str().ok_or("deck")?.into(), ..Default::default() });
        self.0.lock().unwrap().push(f.clone());
        Ok(f)
    }
    fn card_names(&self, _: &Info) -> Vec<String> {
        vec![]
    }
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("practice-srv-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

struct Rig {
    h: ServerHandle,
    f: Arc<Factory>,
    saves: PathBuf,
    root: PathBuf,
}

fn rig(name: &str, lan: bool) -> Rig {
    let root = temp(name);
    std::fs::create_dir_all(root.join("static")).unwrap();
    std::fs::write(root.join("static/index.html"), "<html>page</html>").unwrap();
    std::fs::write(root.join("static/app.js"), "var x;").unwrap();
    std::fs::write(root.join("secret.txt"), "secret").unwrap();
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let saves = root.join("saves");
    let f = Arc::new(Factory(Mutex::new(vec![])));
    let h = start_with(
        Config { data, saves: saves.clone(), static_dir: root.join("static"), lan, app: false, port: 0 },
        f.clone(),
    )
    .unwrap();
    Rig { h, f, saves, root }
}

impl Drop for Rig {
    fn drop(&mut self) {
        self.h.stop();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// one request on a fresh connection: (status, headers, body)
fn call(port: u16, method: &str, path: &str, cookie: Option<&str>, body: Option<&Value>) -> (u16, String, Value) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let b = body.map(|b| b.to_string()).unwrap_or_default();
    let ck = cookie.map(|c| format!("Cookie: practice_seat={c}\r\n")).unwrap_or_default();
    write!(s, "{method} {path} HTTP/1.1\r\nHost: x\r\n{ck}Content-Length: {}\r\nConnection: close\r\n\r\n{b}", b.len())
        .unwrap();
    let mut raw = String::new();
    s.read_to_string(&mut raw).unwrap();
    let (head, body) = raw.split_once("\r\n\r\n").unwrap();
    let status = head.split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, head.to_string(), serde_json::from_str(body).unwrap_or(Value::String(body.to_string())))
}

fn cookie_of(head: &str) -> Option<String> {
    head.lines()
        .find_map(|l| l.strip_prefix("Set-Cookie: practice_seat="))
        .map(|v| v.split(';').next().unwrap().to_string())
}

fn wait_for(mut f: impl FnMut() -> bool) {
    for _ in 0..200 {
        if f() {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out");
}

#[test]
fn page_cookie_and_static_files() {
    let r = rig("static", false);
    let (st, head, body) = call(r.h.port, "GET", "/", None, None);
    assert_eq!((st, body.as_str()), (200, Some("<html>page</html>")));
    assert!(head.contains("text/html") && head.contains("Cache-Control: no-store"));
    assert!(cookie_of(&head).is_some_and(|c| c.len() >= 16));
    let (_, head2, _) = call(r.h.port, "GET", "/", Some("mine"), None);
    assert!(cookie_of(&head2).is_none());
    let (st, head, _) = call(r.h.port, "GET", "/static/app.js", None, None);
    assert!(st == 200 && head.contains("text/javascript") && cookie_of(&head).is_none());
    for bad in [
        "/static/../secret.txt",
        "/static/a/../../secret.txt",
        "/static//etc/passwd",
        "/static/",
        "/static/nothing",
        "/images/../secret.txt",
        "/nope",
    ] {
        let (st, _, body) = call(r.h.port, "GET", bad, None, None);
        assert_eq!((st, body["error"].as_str()), (404, Some("not found")), "{bad}");
    }
}

#[test]
fn options_and_saves_shapes() {
    let r = rig("options", false);
    let (st, _, o) = call(r.h.port, "GET", "/api/options", None, None);
    assert_eq!(st, 200);
    assert_eq!((o["lan"].clone(), o["host"].clone(), o["app"].clone()), (json!(false), json!(true), json!(false)));
    assert_eq!(o["decks"].as_array().unwrap().len(), 15);
    assert_eq!(o["tiers"].as_array().unwrap().len(), 7);
    assert!(o["images"].is_object());
    let (_, _, s) = call(r.h.port, "GET", "/api/saves", None, None);
    assert_eq!(s, json!({"saves": [], "autosave": null}));
}

#[test]
fn a_game_through_the_routes() {
    let r = rig("game", false);
    let p = r.h.port;
    let (st, _, b) = call(p, "POST", "/api/new", Some("me"), Some(&json!({"deck": "nope", "tier": "t1"})));
    assert_eq!(
        (st, b["error"].as_str()),
        (400, Some("deck: one of seph, veyran, sauron, yshtola, galadriel, alela, kefka, omnath, sheoldred, cloud, thrasios, sephfs, godo, karn, yshtola-cedh"))
    );
    assert_eq!(
        call(p, "POST", "/api/new", Some("me"), Some(&json!({"deck": "seph", "tier": "t9"}))).2["error"],
        "tier: one of t0, t1, t2, t3, t4, t5, t6"
    );
    assert_eq!(
        call(p, "POST", "/api/new", Some("me"), Some(&json!({"deck": "seph", "tier": "t1", "autopass": "x"}))).2["error"],
        "autopass: one of respond, stack, all"
    );
    assert_eq!(
        call(p, "POST", "/api/new", Some("me"), Some(&json!({"deck": "seph", "tier": "t1", "seat": "2"}))).2["error"],
        "seat: 1 to 4"
    );
    assert_eq!(
        call(p, "POST", "/api/new", Some("me"), Some(&json!({"deck": "seph", "tier": "t1", "two": true}))).0,
        409
    );
    let mut s = TcpStream::connect(("127.0.0.1", p)).unwrap();
    s.write_all(b"POST /api/new HTTP/1.1\r\nContent-Length: 3\r\nConnection: close\r\n\r\n{{{").unwrap();
    let mut raw = String::new();
    s.read_to_string(&mut raw).unwrap();
    assert!(raw.starts_with("HTTP/1.1 400") && raw.contains("the request body is not JSON"));

    let (st, _, b) = call(p, "POST", "/api/new", Some("me"), Some(&json!({"deck": "seph", "tier": "t1"})));
    assert_eq!((st, b["ok"].clone(), b["seed"].clone()), (200, json!(true), json!(7)));
    let fake = r.f.0.lock().unwrap()[0].clone();
    wait_for(|| *fake.started.lock().unwrap());
    // my seat: the cookie names it; the request's view moved into views
    wait_for(|| call(p, "GET", "/api/state", Some("me"), None).2["pending"].is_object());
    let (_, _, state) = call(p, "GET", "/api/state", Some("me"), None);
    assert_eq!(state["game"]["you"], "seph");
    assert_eq!(state["view"], json!({"turn": 1}));
    let pend = &state["pending"];
    assert_eq!((pend["who"].clone(), pend["request"]["data"].clone()), (json!("Me"), json!({"x": 1})));
    // another browser on this computer is the host: it plays the first seat; busy never shows for it
    assert_eq!(call(p, "GET", "/api/state", Some("other"), None).2["game"]["you"], "seph");
    // a stale id is refused, the right one is taken once
    let id = pend["id"].clone();
    let (st, _, b) = call(p, "POST", "/api/answer", Some("me"), Some(&json!({"id": 999, "answer": 1})));
    assert_eq!((st, b["error"].as_str()), (409, Some("That decision has already been answered.")));
    assert_eq!(call(p, "POST", "/api/answer", Some("me"), Some(&json!({"id": id, "answer": {"pass": true}}))).0, 200);
    assert_eq!(fake.answers.lock().unwrap().clone(), vec![("seph".to_string(), json!({"pass": true}))]);
    assert_eq!(call(p, "POST", "/api/answer", Some("me"), Some(&json!({"id": id, "answer": 2}))).0, 409);
    assert_eq!(call(p, "POST", "/api/undo", Some("me"), Some(&json!({"n": 1}))).2["error"], "Nothing to undo yet.");
    assert_eq!(call(p, "POST", "/api/autopass", Some("me"), Some(&json!({"mode": "all"}))).0, 200);
    assert_eq!(call(p, "POST", "/api/autopass", Some("me"), Some(&json!({"mode": "x"}))).0, 409);
    assert_eq!(call(p, "POST", "/api/hint", Some("me"), None).2["text"], "pass");
    assert_eq!(
        call(p, "POST", "/api/approve", Some("me"), Some(&json!({"id": 1, "yes": true}))).2["error"],
        "That request is no longer open."
    );
    // saving, listing, loading
    let (st, _, b) = call(p, "POST", "/api/save", Some("me"), None);
    assert_eq!(st, 200);
    let name = b["name"].as_str().unwrap().to_string();
    assert!(name.ends_with("-seph-t1-7.json"), "{name}");
    let (_, _, l) = call(p, "GET", "/api/saves", None, None);
    assert_eq!((l["saves"][0]["name"].clone(), l["saves"][0]["round"].clone()), (json!(name), json!(2)));
    assert_eq!(
        call(p, "POST", "/api/load", Some("me"), Some(&json!({"name": "../x.json"}))).2["error"],
        "No such saved game."
    );
    assert_eq!(call(p, "POST", "/api/load", Some("me"), Some(&json!({"name": name, "images": false}))).0, 200);
    assert!(*fake.closed.lock().unwrap(), "loading a game ends the one running");
    assert_eq!(r.f.0.lock().unwrap().len(), 2);
    // autosave: a one-player game waiting on you saves itself; quitting removes it
    wait_for(|| r.saves.join("autosave.json").is_file());
    let (_, _, l) = call(p, "GET", "/api/saves", None, None);
    assert_eq!(l["autosave"]["deck"], "seph");
    assert_eq!(l["saves"].as_array().unwrap().len(), 1);
    assert_eq!(call(p, "POST", "/api/quit", None, None).0, 200);
    assert!(!r.saves.join("autosave.json").exists());
    assert_eq!(call(p, "GET", "/api/review", Some("me"), None).2["error"], "There is no game.");
}

#[test]
fn events_stream_and_reconnect() {
    let r = rig("sse", false);
    let p = r.h.port;
    call(p, "POST", "/api/new", Some("me"), Some(&json!({"deck": "seph", "tier": "t1", "images": false})));
    let fake = r.f.0.lock().unwrap()[0].clone();
    wait_for(|| *fake.started.lock().unwrap());
    fake.push(json!({"kind": "log", "text": "one", "view": {"turn": 2}}));
    fake.push(json!({"kind": "log", "text": "two", "seat": "someone-else"}));
    fake.push(json!({"kind": "log", "text": "three"}));

    let open = |extra: &str, path: &str| {
        let mut s = TcpStream::connect(("127.0.0.1", p)).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        write!(s, "GET {path} HTTP/1.1\r\nHost: x\r\nCookie: practice_seat=me\r\n{extra}\r\n").unwrap();
        s
    };
    let read_until = |s: &mut TcpStream, want: &str| {
        let mut got = String::new();
        let mut buf = [0u8; 4096];
        while !got.contains(want) {
            let n = s.read(&mut buf).expect("stream stalled");
            assert!(n > 0, "closed early: {got}");
            got.push_str(&String::from_utf8_lossy(&buf[..n]));
        }
        got
    };
    let mut s = open("", "/api/events");
    let got = read_until(&mut s, "three");
    assert!(
        got.starts_with("HTTP/1.1 200 OK") && got.contains("text/event-stream") && got.contains("Connection: close")
    );
    assert!(got.contains("\"one\"") && !got.contains("\"two\""), "another seat's messages are left out: {got}");
    let ids: Vec<u64> = got.lines().filter_map(|l| l.strip_prefix("id: ")).map(|n| n.parse().unwrap()).collect();
    assert!(ids.windows(2).all(|w| w[0] < w[1]) && ids.len() >= 3, "{ids:?}");
    // live events arrive on the open stream
    fake.push(json!({"kind": "log", "text": "four"}));
    read_until(&mut s, "four");
    drop(s);
    // a reconnect with Last-Event-ID resumes after it; ?since= works too
    let last = ids[ids.len() - 1];
    let mut s2 = open(&format!("Last-Event-ID: {}\r\n", last), "/api/events");
    let got = read_until(&mut s2, "four");
    assert!(!got.contains("\"three\"") && got.contains(&format!("id: {}", last + 1)), "{got}");
    let mut s3 = open("", &format!("/api/events?since={}", last));
    assert!(!read_until(&mut s3, "four").contains("\"three\""));
    // a browser with no seat gets nothing but the head
    let mut s4 = TcpStream::connect(("127.0.0.1", p)).unwrap();
    s4.write_all(b"GET /api/events HTTP/1.1\r\n\r\n").unwrap();
    s4.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    assert!(read_until(&mut s4, "\r\n\r\n").starts_with("HTTP/1.1 200"));
}

#[test]
fn only_the_host_computer_can_do_host_things() {
    use practice::server::http::{Conn, Request};
    use practice::server::routes::{Reply, Site, dispatch};
    let r = rig("host", false);
    let site = Site::new(
        r.h.hub.clone(),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data"),
        r.root.join("static"),
    );
    let req = |method: &str, path: &str, peer: &str| -> Request {
        let mut c = Conn::new(std::io::Cursor::new(Vec::<u8>::new()));
        let _ = &mut c;
        Request {
            method: method.into(),
            path: path.into(),
            query: String::new(),
            headers: vec![],
            body: b"{}".to_vec(),
            peer: Some(peer.parse().unwrap()),
        }
    };
    for (m, p) in [
        ("POST", "/api/new"),
        ("POST", "/api/load"),
        ("POST", "/api/quit"),
        ("POST", "/api/save"),
        ("GET", "/api/saves"),
    ] {
        let Reply::Respond(resp) = dispatch(&site, &req(m, p, "192.168.1.20:4000")) else { panic!() };
        assert_eq!(resp.status, 403, "{p}");
        assert_eq!(
            serde_json::from_slice::<Value>(&resp.body).unwrap()["error"],
            "Only the host computer can do that."
        );
    }
    let Reply::Respond(resp) = dispatch(&site, &req("GET", "/api/saves", "127.0.0.1:4000")) else { panic!() };
    assert_eq!(resp.status, 200);
    // another computer sees the state of a game with no seat as busy
    let Reply::Respond(resp) = dispatch(&site, &req("GET", "/api/state", "192.168.1.20:4000")) else { panic!() };
    let v: Value = serde_json::from_slice(&resp.body).unwrap();
    assert_eq!((v["host"].clone(), v["game"].clone()), (json!(false), Value::Null));
    assert!(resp.cookie.is_some(), "a browser without a cookie gets one when its seat is looked up");
}

#[test]
fn lobby_and_join_with_lan() {
    let r = rig("lan", true);
    let p = r.h.port;
    let (st, _, b) =
        call(p, "POST", "/api/new", Some("host"), Some(&json!({"deck": "seph", "tier": "t1", "two": true})));
    assert_eq!(st, 200);
    let code = b["code"].as_str().unwrap().to_string();
    assert_eq!(code.len(), 6);
    let (_, _, s) = call(p, "GET", "/api/state", Some("host"), None);
    assert_eq!((s["lobby"]["deck"].clone(), s["lobby"]["code"].clone()), (json!("seph"), json!(code)));
    let join = |cookie: &str, code: &str, deck: &str| {
        call(p, "POST", "/api/join", Some(cookie), Some(&json!({"code": code, "deck": deck}))).2
    };
    assert_eq!(join("guest", "000000x", "veyran")["error"], "That code is not right: the host's screen shows it.");
    assert_eq!(join("guest", &code, "seph")["error"], "The host plays that deck: pick another.");
    assert_eq!(join("host", &code, "veyran")["error"], "This browser is the host's: join from the other computer.");
    assert_eq!(
        join("guest", &code, "bogus")["error"],
        "deck: one of seph, veyran, sauron, yshtola, galadriel, alela, kefka, omnath, sheoldred, cloud, thrasios, sephfs, godo, karn, yshtola-cedh"
    );
    assert_eq!(join("guest", &code, "veyran")["ok"], true);
    assert_eq!(join("guest", &code, "veyran")["error"], "There is no game waiting for a second player.");
    let (_, _, s) = call(p, "GET", "/api/state", Some("guest"), None);
    assert_eq!(s["game"]["you"], "veyran");
}
