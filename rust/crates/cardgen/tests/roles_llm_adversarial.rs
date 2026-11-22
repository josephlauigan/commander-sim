//! Adversarial tests for the local-LLM stage of the role classifier (phase 3L). Everything talks to a FAKE local
//! server written here with `std::net::TcpListener`; the real LLM server is never contacted and nothing leaves
//! localhost. Fixture cards are real Oracle text from `tests/fixtures/roles_oracle.json`.
//!
//! A test whose name starts with `defect_` fails because of a real defect in the code under test (see the reviewer's
//! handoff). Every other test passes today and stays as a regression test. Real-catalog tests are `#[ignore]`d:
//! run them with `cargo test --release -p cardgen --test roles_llm_adversarial -- --ignored`.
use cardgen::data::Project;
use cardgen::knowledge::{self, Index, Legal};
use cardgen::llm::{self, Kind, LlmCard, MergeEnv, Row};
use cardgen::roles::{self, Assignment, Evidence, RouteReason, Source, Taxonomy};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const TAXONOMY: &str = include_str!("../../../../data/knowledge/roles.toml");
const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/roles_oracle.json");
const EXTRA_CARDS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/adversarial_llm/extra_cards.json");
const FIVE: [&str; 5] = [
    "Skywhaler's Shot",
    "Ugin's Mastery",
    "Counterspell",
    "Agadeem's Awakening // Agadeem, the Undercrypt",
    "Elspeth, Sun's Champion",
];

fn taxonomy() -> Taxonomy {
    plain_variant();
    Taxonomy::parse(TAXONOMY).unwrap()
}
/// The tests' fake models answer in the `plain` shape: select it (the default is `think`) once for the whole test
/// process, before any test reads the variable.
fn plain_variant() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    // SAFETY: set once, before any run of this process reads it; every test calls this first
    ONCE.call_once(|| unsafe { std::env::set_var("CARDGEN_LLM_VARIANT", "plain") });
}
/// roles.toml as the scratch environments get it: the repository's, with the frozen `llm_add = false` lines taken out
/// (the fake models propose roles of every kind; the frozen flags have their own tests).
fn test_taxonomy() -> String {
    TAXONOMY.replace("llm_add = false\n", "")
}

// ------------------------------------------------------------------------------------------------ the fake server

/// What the fake server writes back to one request.
enum Wire {
    /// a 200 chat completion carrying this content (finish_reason stop)
    Content(String),
    /// a status line and a body sent as is (with a correct Content-Length)
    Json(u16, String),
    /// the whole HTTP response, bytes as given, then the connection closes
    Bytes(Vec<u8>),
    /// a 200 with chunked transfer encoding, one chunk per piece
    Chunked(Vec<Vec<u8>>),
    /// a 200 whose body arrives in pieces with a pause (ms) between them
    Slow(Vec<Vec<u8>>, u64),
    /// the head promises 100 bytes more than the body then delivers before the connection closes
    ShortBody(String),
    /// a 200 whose body is this many bytes
    Big(usize),
    /// the connection closes without a byte
    Drop,
}
type ChatFn = Box<dyn Fn(usize, &Value) -> Wire + Send + Sync>;
type ModelsFn = Box<dyn Fn() -> Wire + Send + Sync>;

struct Srv {
    /// `http://127.0.0.1:PORT/v1`
    url: String,
    addr: SocketAddr,
    model: Arc<Mutex<String>>,
    chats: Arc<AtomicUsize>,
    models: Arc<AtomicUsize>,
    bodies: Arc<Mutex<Vec<Value>>>,
    /// "METHOD /path" of every request, in order
    paths: Arc<Mutex<Vec<String>>>,
    /// the raw text of every chat request body, byte for byte
    raws: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}
struct Shared {
    chat: ChatFn,
    models_fn: Option<ModelsFn>,
    model: Arc<Mutex<String>>,
    chats: Arc<AtomicUsize>,
    models: Arc<AtomicUsize>,
    bodies: Arc<Mutex<Vec<Value>>>,
    paths: Arc<Mutex<Vec<String>>>,
    raws: Arc<Mutex<Vec<String>>>,
    prefix: String,
    delay_ms: u64,
}
impl Srv {
    fn start(chat: ChatFn) -> Srv {
        Srv::build(chat, None, "/v1", 0)
    }
    fn build(chat: ChatFn, models_fn: Option<ModelsFn>, prefix: &str, delay_ms: u64) -> Srv {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let shared = Arc::new(Shared {
            chat,
            models_fn,
            model: Arc::new(Mutex::new("fake-model-1".to_string())),
            chats: Arc::new(AtomicUsize::new(0)),
            models: Arc::new(AtomicUsize::new(0)),
            bodies: Arc::new(Mutex::new(vec![])),
            paths: Arc::new(Mutex::new(vec![])),
            raws: Arc::new(Mutex::new(vec![])),
            prefix: prefix.to_string(),
            delay_ms,
        });
        let stop = Arc::new(AtomicBool::new(false));
        let (sh, st) = (shared.clone(), stop.clone());
        let handle = std::thread::spawn(move || {
            for stream in listener.incoming() {
                if st.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(stream) = stream else { continue };
                let sh = sh.clone();
                std::thread::spawn(move || serve(stream, &sh));
            }
        });
        Srv {
            url: format!("http://{addr}{prefix}"),
            addr,
            model: shared.model.clone(),
            chats: shared.chats.clone(),
            models: shared.models.clone(),
            bodies: shared.bodies.clone(),
            paths: shared.paths.clone(),
            raws: shared.raws.clone(),
            stop,
            handle: Some(handle),
        }
    }
    fn chats(&self) -> usize {
        self.chats.load(Ordering::SeqCst)
    }
    fn models(&self) -> usize {
        self.models.load(Ordering::SeqCst)
    }
    fn set_model(&self, id: &str) {
        *self.model.lock().unwrap() = id.to_string();
    }
    fn bodies(&self) -> Vec<Value> {
        self.bodies.lock().unwrap().clone()
    }
    fn raws(&self) -> Vec<String> {
        self.raws.lock().unwrap().clone()
    }
}
impl Drop for Srv {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.addr);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}
fn head(status: u16, extra: &str) -> String {
    format!("HTTP/1.1 {status} X\r\nContent-Type: application/json\r\n{extra}Connection: close\r\n\r\n")
}
fn write_wire(stream: &mut TcpStream, wire: Wire, model: &str) {
    let completion = |content: &str| {
        let message = json!({"role": "assistant", "content": content});
        let choice = json!({"index": 0, "finish_reason": "stop", "message": message});
        json!({"id": "c", "object": "chat.completion", "model": model, "choices": [choice]}).to_string()
    };
    let whole = |stream: &mut TcpStream, status: u16, body: &str| {
        let _ = stream.write_all(head(status, &format!("Content-Length: {}\r\n", body.len())).as_bytes());
        let _ = stream.write_all(body.as_bytes());
    };
    match wire {
        Wire::Content(c) => whole(stream, 200, &completion(&c)),
        Wire::Json(s, b) => whole(stream, s, &b),
        Wire::Bytes(b) => {
            let _ = stream.write_all(&b);
        }
        Wire::Chunked(pieces) => {
            let _ = stream.write_all(head(200, "Transfer-Encoding: chunked\r\n").as_bytes());
            for p in pieces {
                let _ = write!(stream, "{:x}\r\n", p.len());
                let _ = stream.write_all(&p);
                let _ = stream.write_all(b"\r\n");
            }
            let _ = stream.write_all(b"0\r\n\r\n");
        }
        Wire::Slow(pieces, pause) => {
            let total: usize = pieces.iter().map(Vec::len).sum();
            let _ = stream.write_all(head(200, &format!("Content-Length: {total}\r\n")).as_bytes());
            for p in pieces {
                let _ = stream.write_all(&p);
                let _ = stream.flush();
                std::thread::sleep(Duration::from_millis(pause));
            }
        }
        Wire::ShortBody(s) => {
            let _ = stream.write_all(head(200, &format!("Content-Length: {}\r\n", s.len() + 100)).as_bytes());
            let _ = stream.write_all(s.as_bytes());
        }
        Wire::Big(n) => {
            let _ = stream.write_all(head(200, &format!("Content-Length: {n}\r\n")).as_bytes());
            let block = vec![b' '; 1 << 16];
            let mut left = n;
            while left > 0 {
                let k = left.min(block.len());
                if stream.write_all(&block[..k]).is_err() {
                    break;
                }
                left -= k;
            }
        }
        Wire::Drop => {}
    }
}
fn serve(mut stream: TcpStream, sh: &Shared) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request = String::new();
    if reader.read_line(&mut request).unwrap_or(0) == 0 {
        return;
    }
    let mut length = 0usize;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).unwrap_or(0) == 0 || header == "\r\n" {
            break;
        }
        if let Some(v) = header.to_lowercase().strip_prefix("content-length:") {
            length = v.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; length];
    let _ = reader.read_exact(&mut body);
    let (method, path) = {
        let mut it = request.split_whitespace();
        (it.next().unwrap_or("").to_string(), it.next().unwrap_or("").to_string())
    };
    sh.paths.lock().unwrap().push(format!("{method} {path}"));
    let model = sh.model.lock().unwrap().clone();
    if sh.delay_ms > 0 {
        std::thread::sleep(Duration::from_millis(sh.delay_ms));
    }
    if method == "GET" && path == format!("{}/models", sh.prefix) {
        sh.models.fetch_add(1, Ordering::SeqCst);
        let wire = match &sh.models_fn {
            Some(f) => f(),
            None => Wire::Json(200, json!({"object": "list", "data": [{"id": model, "object": "model"}]}).to_string()),
        };
        write_wire(&mut stream, wire, &model);
    } else if method == "POST" && path == format!("{}/chat/completions", sh.prefix) {
        let n = sh.chats.fetch_add(1, Ordering::SeqCst);
        sh.raws.lock().unwrap().push(String::from_utf8_lossy(&body).into_owned());
        let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
        sh.bodies.lock().unwrap().push(body.clone());
        write_wire(&mut stream, (sh.chat)(n, &body), &model);
    } else {
        write_wire(&mut stream, Wire::Json(404, "{}".into()), &model);
    }
}

// ------------------------------------------------------------------------------- what the fake "model" says

#[derive(Clone, Debug)]
struct Item {
    id: String,
    type_line: String,
    lines: Vec<(usize, String)>,
}
/// The cards of a request, parsed back out of the user message exactly as the prompt printed them.
fn parse_batch(body: &Value) -> Vec<Item> {
    let user = body["messages"].as_array().and_then(|m| m.last()).map_or("", |m| m["content"].as_str().unwrap_or(""));
    let mut items: Vec<Item> = vec![];
    for line in user.lines().skip(1) {
        if let Some(rest) = line.strip_prefix('[') {
            let (id, ty) = rest.split_once("] ").unwrap_or((rest.trim_end_matches(']'), ""));
            items.push(Item { id: id.to_string(), type_line: ty.to_string(), lines: vec![] });
        } else if let Some((n, text)) = line.split_once(": ") {
            if let Ok(n) = n.parse::<usize>() {
                items.last_mut().unwrap().lines.push((n, text.to_string()));
            }
        }
    }
    items
}
fn first(item: &Item) -> &str {
    item.lines.first().map_or("", |l| l.1.as_str())
}
fn entry(id: &str, roles: &[&str], evidence: &[(&str, usize)]) -> Value {
    let roles: Vec<Value> = roles
        .iter()
        .map(|r| json!({"role": r, "line": evidence.iter().find(|(e, _)| e == r).map_or(0, |(_, n)| *n)}))
        .collect();
    json!({"id": id, "roles": roles})
}
/// What a sensible model says about each of the five fixture cards.
fn says5(item: &Item) -> (Vec<&'static str>, Vec<(&'static str, usize)>) {
    let f = first(item);
    if f.starts_with("Destroy target creature with power") {
        (vec!["removal", "selection"], vec![("removal", 0), ("selection", 0)])
    } else if f == "Counter target spell." {
        (vec!["counter"], vec![("counter", 0)])
    } else if f.starts_with("Return from your graveyard") {
        (vec!["reanimate"], vec![("reanimate", 0)])
    } else if f.starts_with("+1: Create three") {
        (vec!["tokens", "wipe"], vec![("tokens", 0), ("wipe", 1)])
    } else {
        (vec![], vec![])
    }
}
fn valid5(items: &[Item]) -> Value {
    json!({"cards": items.iter().map(|i| { let (r, e) = says5(i); entry(&i.id, &r, &e) }).collect::<Vec<_>>()})
}
fn splitmix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
fn text_hash(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3))
}
/// A deterministic pseudo-random but VALID answer for any card: up to three roles, each one whose guard passes on the
/// line it cites (roles are only chosen among those with a guard hit, so the answer is accepted by the validator).
fn pseudo_entry(item: &Item, tax: &Taxonomy, salt: u64) -> Value {
    let seed = text_hash(&format!("{}{}", item.type_line, item.lines.iter().map(|l| l.1.as_str()).collect::<String>()));
    let seed = splitmix(seed ^ salt);
    let mut hits: Vec<(u64, &str, usize)> = vec![];
    for (k, role) in roles::IDS.iter().enumerate() {
        if let Some((n, _)) = item.lines.iter().find(|(_, t)| tax.guard(role).unwrap().is_match(t).unwrap_or(false)) {
            hits.push((splitmix(seed ^ (k as u64 + 1)), role, *n));
        }
    }
    hits.sort();
    let want = (splitmix(seed) % 4) as usize; // 0 to 3 roles; about a quarter of the cards get none
    let chosen: Vec<(&str, usize)> = hits.iter().take(want).map(|(_, r, n)| (*r, *n)).collect();
    let rs: Vec<&str> = chosen.iter().map(|c| c.0).collect();
    entry(&item.id, &rs, &chosen)
}
fn pseudo(items: &[Item], salt: u64) -> Value {
    let tax = taxonomy();
    json!({"cards": items.iter().map(|i| pseudo_entry(i, &tax, salt)).collect::<Vec<_>>()})
}
fn good5() -> ChatFn {
    Box::new(|_, body| Wire::Content(valid5(&parse_batch(body)).to_string()))
}
fn always(content: Value) -> ChatFn {
    Box::new(move |_, _| Wire::Content(content.to_string()))
}

// ---------------------------------------------------------------------------------------------------- environment

static ENV_COUNTER: AtomicUsize = AtomicUsize::new(0);
struct Env(PathBuf);
impl Env {
    fn new() -> Self {
        Self::custom(vec![])
    }
    /// The fixture catalog plus extra card records, in a throw-away project with the repository's roles.toml.
    fn custom(extra: Vec<Value>) -> Self {
        plain_variant();
        let root = std::env::temp_dir().join(format!(
            "roles-llm-adv-{}-{}",
            std::process::id(),
            ENV_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let dir = root.join("data/knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        let mut oracle: Value = serde_json::from_str(&std::fs::read_to_string(FIXTURE).unwrap()).unwrap();
        oracle["cards"].as_array_mut().unwrap().extend(extra);
        let n = oracle["cards"].as_array().unwrap().len();
        oracle["meta"]["count"] = json!(n);
        std::fs::write(dir.join("oracle.json"), oracle.to_string()).unwrap();
        std::fs::write(dir.join("roles.toml"), test_taxonomy()).unwrap();
        std::fs::write(dir.join("roles_overrides.toml"), "").unwrap();
        std::fs::write(dir.join("ids.txt"), FIVE.join("\n")).unwrap();
        std::fs::write(
            root.join("data/decks.json"),
            r#"{"mine":[{"key":"test","commander":"Counterspell","cards":["Counterspell"]}]}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("data/cards.json"),
            r#"{"cards":[{"name":"Counterspell","source":"manual","tags":{"ctr":"any"}}]}"#,
        )
        .unwrap();
        Self(root)
    }
    fn dir(&self) -> PathBuf {
        self.0.join("data/knowledge")
    }
    fn cache(&self) -> PathBuf {
        self.dir().join("llm_roles.jsonl")
    }
    fn cmd(&self, args: &[&str]) -> Result<(String, bool), String> {
        let args: Vec<String> =
            args.iter().map(|s| s.to_string()).chain(["--data-dir".into(), self.dir().display().to_string()]).collect();
        roles::command(&Project::new(&self.0), &args)
    }
    /// `roles --llm --ids ids.txt --endpoint URL` plus `extra`.
    fn llm(&self, srv: &Srv, extra: &[&str]) -> Result<String, String> {
        self.llm_at(&srv.url, extra)
    }
    fn llm_at(&self, url: &str, extra: &[&str]) -> Result<String, String> {
        let ids = self.dir().join("ids.txt");
        let mut args = vec!["--llm", "--ids", ids.to_str().unwrap(), "--endpoint", url];
        args.extend_from_slice(extra);
        self.cmd(&args).map(|(text, _)| text)
    }
    fn rows(&self) -> Vec<Row> {
        llm::read_rows(&self.cache()).unwrap().into_values().collect()
    }
    fn index(&self) -> Index {
        Index::load(&self.dir().join("oracle.json")).unwrap()
    }
    fn merged(&self) -> roles::RoleMap {
        self.cmd(&["--merge"]).unwrap();
        roles::load(&self.dir().join("roles.json")).unwrap()
    }
    /// Every file under the knowledge dir, name to bytes.
    fn files(&self) -> BTreeMap<String, Vec<u8>> {
        let mut out = BTreeMap::new();
        fn walk(dir: &Path, base: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
            for e in std::fs::read_dir(dir).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    walk(&p, base, out);
                } else {
                    out.insert(p.strip_prefix(base).unwrap().display().to_string(), std::fs::read(&p).unwrap());
                }
            }
        }
        walk(&self.dir(), &self.dir(), &mut out);
        out
    }
    fn no_temp_files(&self) {
        let names: Vec<String> = self.files().into_keys().filter(|n| n.ends_with(".tmp")).collect();
        assert!(names.is_empty(), "temp files left behind: {names:?}");
    }
    fn key_of(&self, name: &str, model: &str) -> String {
        let i = self.index();
        let c = &i.cards[i.lookup(name)[0] as usize];
        llm::cache_key(&llm::card_text(c), &llm::prompt_fnv(&taxonomy()), model)
    }
    /// A cache row for a card with the current prompt and the given model.
    fn row(&self, name: &str, model: &str, roles: &[&str], evidence: &[(&str, usize)]) -> Row {
        Row {
            why: None,
            key: self.key_of(name, model),
            name: name.to_string(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            primary: roles.first().map(|r| r.to_string()),
            evidence: evidence.iter().map(|(r, n)| (r.to_string(), *n)).collect(),
            model: model.to_string(),
            taxonomy: taxonomy().hash(),
            prompt: llm::prompt_fnv(&taxonomy()),
        }
    }
    fn write_rows(&self, file: &str, rows: &[Row]) {
        let text: String = rows.iter().map(|r| format!("{}\n", serde_json::to_string(r).unwrap())).collect();
        std::fs::write(self.dir().join(file), text).unwrap();
    }
}
impl Drop for Env {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(self.dir(), std::fs::Permissions::from_mode(0o755));
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn card_of<'a>(index: &'a Index, name: &str) -> &'a knowledge::Card {
    &index.cards[index.lookup(name)[0] as usize]
}
fn assignment_of<'a>(index: &Index, map: &'a roles::RoleMap, name: &str) -> &'a Assignment {
    &map[&card_of(index, name).raw.oracle_id]
}
/// `(classified, failed)` from the one-line run summary.
fn counts(out: &str) -> (usize, usize) {
    let num = |label: &str| -> usize {
        let at = out.find(label).unwrap_or_else(|| panic!("no '{label}' in {out}"));
        out[..at].trim_end().rsplit(' ').next().unwrap().parse().unwrap()
    };
    (num(" classified"), num(" failed"))
}

// ============================================================================ 1. the server misbehaving

fn completion(content: Value, finish: Value) -> String {
    json!({"choices": [{"index": 0, "finish_reason": finish, "message": {"role": "assistant", "content": content}}]})
        .to_string()
}
/// A server that answers every chat request by `make(batch)`; `make` must not capture anything.
fn answering(make: fn(&[Item]) -> Wire) -> Srv {
    Srv::start(Box::new(move |_, body| make(&parse_batch(body))))
}

#[test]
fn http_error_statuses_are_answers_that_fail_the_cards_not_signs_of_a_dead_server() {
    plain_variant();
    for status in [400u16, 404, 500, 502, 503] {
        let e = Env::new();
        let srv = Srv::start(Box::new(move |_, _| Wire::Json(status, "{\"error\":\"no grammar\"}".into())));
        let err = e.llm(&srv, &[]).unwrap_err();
        // the batch, then every card alone once; all fail, so the run exits non-zero without claiming a dead server
        assert!(err.contains("5 of 5 cards ended llm_failed") && !err.contains("unreachable"), "{status}: {err}");
        assert_eq!(srv.chats(), 1 + 5, "{status}");
        assert!(!e.cache().exists(), "{status}: nothing classified, so no cache file");
        // the error names the status and carries the reply body (a rejected grammar says why there)
        let failed = std::fs::read_to_string(e.dir().join("reports/llm-failed.txt")).unwrap();
        assert!(failed.contains(&format!("HTTP {status}: {{\"error\":\"no grammar\"}}")), "{status}: {failed}");
        e.no_temp_files();
    }
    // with `--batch 1` three batches in a row without one valid answer stop the run, naming the last reason
    let e = Env::new();
    let srv = Srv::start(Box::new(|_, _| Wire::Json(400, "{\"error\":\"bad schema\"}".into())));
    let err = e.llm(&srv, &["--batch", "1"]).unwrap_err();
    assert!(err.contains("no valid answer for 3 batches in a row") && err.contains("bad schema"), "{err}");
    assert_eq!(srv.chats(), 3);
    // a dropped connection on every other request is a transport error: the identical batch is resent and succeeds
    let e = Env::new();
    let srv = Srv::start(Box::new(|n, body| {
        if n % 2 == 0 { Wire::Drop } else { Wire::Content(valid5(&parse_batch(body)).to_string()) }
    }));
    let out = e.llm(&srv, &[]).unwrap();
    assert_eq!((srv.chats(), counts(&out)), (2, (5, 0)), "{out}");
}

#[test]
fn a_200_with_a_useless_body_is_an_invalid_answer_never_a_transport_failure() {
    plain_variant();
    let cases: Vec<(&str, fn(&[Item]) -> Wire)> = vec![
        ("html", |_| Wire::Json(200, "<html><body>Bad gateway</body></html>".into())),
        ("empty body", |_| Wire::Json(200, String::new())),
        ("status line only", |_| Wire::Bytes(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n".to_vec())),
        ("open brace", |_| Wire::Json(200, "{".into())),
        ("null", |_| Wire::Json(200, "null".into())),
        ("empty choices", |_| Wire::Json(200, "{\"choices\":[]}".into())),
        ("choices null", |_| Wire::Json(200, "{\"choices\":null}".into())),
        ("choices object", |_| Wire::Json(200, "{\"choices\":{}}".into())),
        ("choice without message", |_| Wire::Json(200, "{\"choices\":[{}]}".into())),
        ("content null", |_| Wire::Json(200, completion(Value::Null, json!("stop")))),
        ("content number", |_| Wire::Json(200, completion(json!(42), json!("stop")))),
        ("content parts array", |_| {
            Wire::Json(200, completion(json!([{"type": "text", "text": "{}"}]), json!("stop")))
        }),
        ("markdown fences", |i| {
            Wire::Json(200, completion(json!(format!("```json\n{}\n```", valid5(i))), json!("stop")))
        }),
        ("byte order mark", |i| Wire::Json(200, completion(json!(format!("\u{feff}{}", valid5(i))), json!("stop")))),
        ("prose before the json", |i| {
            Wire::Json(200, completion(json!(format!("Here you go: {}", valid5(i))), json!("stop")))
        }),
        ("valid json cut off by the token limit", |i| {
            Wire::Json(200, completion(json!(valid5(i).to_string()), json!("length")))
        }),
        ("first choice invalid, second valid (only the first is read)", |i| {
            let ok = json!({"index": 1, "finish_reason": "stop",
                            "message": {"role": "assistant", "content": valid5(i).to_string()}});
            let bad = json!({"index": 0, "finish_reason": "stop", "message": {"role": "assistant", "content": "{}"}});
            Wire::Json(200, json!({"choices": [bad, ok]}).to_string())
        }),
    ];
    for (name, make) in cases {
        let e = Env::new();
        let srv = answering(make);
        let out = e.llm(&srv, &[]).unwrap_err();
        // an answer that is not usable is no transport failure: the batch once, then every card alone once
        assert_eq!((srv.chats(), counts(&out)), (1 + 5, (0, 5)), "{name}: {out}");
        assert!(!e.cache().exists(), "{name}");
        let failed = std::fs::read_to_string(e.dir().join("reports/llm-failed.txt")).unwrap();
        assert_eq!(failed.lines().count(), 5, "{name}: {failed}");
        e.no_temp_files();
    }
}

#[test]
fn harmless_variations_of_a_valid_answer_are_accepted_in_one_request() {
    plain_variant();
    let cases: Vec<(&str, fn(&[Item]) -> Wire)> = vec![
        ("several choices, the first valid", |i| {
            let c = |k: usize, content: String| {
                let message = json!({"role": "assistant", "content": content});
                json!({"index": k, "finish_reason": "stop", "message": message})
            };
            Wire::Json(200, json!({"choices": [c(0, valid5(i).to_string()), c(1, "{}".into())]}).to_string())
        }),
        ("extra response fields", |i| {
            let mut v: Value = serde_json::from_str(&completion(json!(valid5(i).to_string()), json!("stop"))).unwrap();
            v["usage"] = json!({"prompt_tokens": 3, "completion_tokens": 4});
            v["system_fingerprint"] = json!("b1");
            Wire::Json(200, v.to_string())
        }),
        ("finish_reason null", |i| Wire::Json(200, completion(json!(valid5(i).to_string()), Value::Null))),
        ("extra top-level and entry fields", |i| {
            let mut v = valid5(i);
            v["note"] = json!("ignored");
            for c in v["cards"].as_array_mut().unwrap() {
                c["confidence"] = json!(0.9);
            }
            Wire::Content(v.to_string())
        }),
        ("entries in another order", |i| {
            let mut v = valid5(i);
            v["cards"].as_array_mut().unwrap().reverse();
            Wire::Content(v.to_string())
        }),
        ("pretty printed with padding", |i| {
            Wire::Content(format!("\n\n  {}  \n", serde_json::to_string_pretty(&valid5(i)).unwrap()))
        }),
        ("unicode escapes in the ids", |i| {
            let text = valid5(i).to_string().replace("\"a0\"", "\"\\u0061\\u0030\"");
            Wire::Content(text)
        }),
    ];
    for (name, make) in cases {
        let e = Env::new();
        let srv = answering(make);
        let out = e.llm(&srv, &[]).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert_eq!((srv.chats(), counts(&out)), (1, (5, 0)), "{name}: {out}");
        assert_eq!(e.rows().len(), 5, "{name}");
    }
}

#[test]
fn chunked_slow_and_close_delimited_bodies_are_read_whole() {
    plain_variant();
    let body = |i: &[Item]| completion(json!(valid5(i).to_string()), json!("stop"));
    let cases: Vec<(&str, fn(&[Item]) -> Wire)> = vec![
        ("chunked", |i| {
            let b = completion(json!(valid5(i).to_string()), json!("stop")).into_bytes();
            Wire::Chunked(b.chunks(37).map(|c| c.to_vec()).collect())
        }),
        ("slow trickle", |i| {
            let b = completion(json!(valid5(i).to_string()), json!("stop")).into_bytes();
            Wire::Slow(b.chunks(200).map(|c| c.to_vec()).collect(), 40)
        }),
        ("http/1.0 close-delimited", |i| {
            let b = completion(json!(valid5(i).to_string()), json!("stop"));
            Wire::Bytes(format!("HTTP/1.0 200 OK\r\nContent-Type: application/json\r\n\r\n{b}").into_bytes())
        }),
    ];
    let _ = body;
    for (name, make) in cases {
        let e = Env::new();
        let srv = answering(make);
        let started = Instant::now();
        let out = e.llm(&srv, &[]).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert_eq!((srv.chats(), counts(&out)), (1, (5, 0)), "{name}: {out}");
        assert!(started.elapsed() < Duration::from_secs(20), "{name}");
    }
}

#[test]
fn a_body_cut_short_a_garbage_reply_and_a_huge_body_are_transport_failures_not_panics_or_hangs() {
    plain_variant();
    let cases: Vec<(&str, fn(&[Item]) -> Wire)> = vec![
        ("body shorter than Content-Length", |_| Wire::ShortBody("{\"choices\":[{\"mess".into())),
        ("not http at all", |_| Wire::Bytes(b"\x00\x01garbage\r\n\r\n".to_vec())),
        ("a 50 MB body", |_| Wire::Big(50 << 20)),
        ("a body of 11 MB", |_| Wire::Big(11 << 20)),
    ];
    for (name, make) in cases {
        let e = Env::new();
        let srv = answering(make);
        let started = Instant::now();
        let err = e.llm(&srv, &[]).unwrap_err();
        assert!(err.contains("unreachable mid-run"), "{name}: {err}");
        // the batch and its identical resend both got no usable reply: two consecutive failures stop the run
        assert_eq!(srv.chats(), 2, "{name}");
        assert!(started.elapsed() < Duration::from_secs(60), "{name}: took {:?}", started.elapsed());
        e.no_temp_files();
    }
}

fn models_server(models: ModelsFn) -> Srv {
    Srv::build(good5(), Some(models), "/v1", 0)
}
#[test]
fn model_endpoint_variants() {
    plain_variant();
    // a reply with no usable model id stops the run before any file is touched
    let bad: Vec<(&str, fn() -> Wire, &str)> = vec![
        ("no data", || Wire::Json(200, "{\"data\":[]}".into()), "names no model"),
        ("empty object", || Wire::Json(200, "{}".into()), "names no model"),
        ("id is a number", || Wire::Json(200, "{\"data\":[{\"id\":7}]}".into()), "names no model"),
        ("id is null", || Wire::Json(200, "{\"data\":[{\"id\":null}]}".into()), "names no model"),
        ("html", || Wire::Json(200, "<html></html>".into()), "bad /models reply"),
        ("500", || Wire::Json(500, "{}".into()), "LLM unreachable"),
        ("404", || Wire::Json(404, "{}".into()), "LLM unreachable"),
        ("truncated", || Wire::ShortBody("{\"data\":[{\"id\":\"x".into()), "LLM unreachable"),
        ("bom", || Wire::Json(200, "\u{feff}{\"data\":[{\"id\":\"x\"}]}".into()), "bad /models reply"),
    ];
    for (name, reply, needle) in bad {
        let e = Env::new();
        let before = e.files();
        let srv = models_server(Box::new(reply));
        let err = e.llm(&srv, &[]).unwrap_err();
        assert!(err.contains(needle), "{name}: {err}");
        assert_eq!(srv.chats(), 0, "{name}: no chat request after a failed model query");
        assert_eq!(before, e.files(), "{name}: no file written");
    }
    // several models: the first is used (the server serves one; there is nothing better to do)
    let e = Env::new();
    let srv = models_server(Box::new(|| Wire::Json(200, "{\"data\":[{\"id\":\"first\"},{\"id\":\"second\"}]}".into())));
    e.llm(&srv, &[]).unwrap();
    assert!(e.rows().iter().all(|r| r.model == "first"));
    assert!(srv.bodies().iter().all(|b| b["model"] == "first"));
    // the older llama.cpp shape
    let e = Env::new();
    let srv = models_server(Box::new(|| Wire::Json(200, "{\"models\":[{\"model\":\"legacy\"}]}".into())));
    e.llm(&srv, &[]).unwrap();
    assert!(e.rows().iter().all(|r| r.model == "legacy"));
}

#[test]
fn a_model_id_with_quotes_newlines_and_unicode_travels_intact() {
    plain_variant();
    let weird = "org/we\"ird\\model\n\u{e9}\u{1f600}:q4";
    let e = Env::new();
    let srv = good5_srv();
    srv.set_model(weird);
    e.llm(&srv, &[]).unwrap();
    // the cache and the requests use the file name of the model path the server reports
    let key = llm::model_key(weird);
    assert_eq!(key, "model\n\u{e9}\u{1f600}:q4");
    assert!(srv.bodies().iter().all(|b| b["model"] == key.as_str()), "the request names the model exactly");
    let rows = e.rows();
    assert!(rows.len() == 5 && rows.iter().all(|r| r.model == key));
    // and the merge reads it back out of the cache
    let map = e.merged();
    assert_eq!(map.meta.model, key);
}
fn good5_srv() -> Srv {
    Srv::start(good5())
}

#[test]
fn defect_an_empty_model_id_is_accepted_as_a_model() {
    plain_variant();
    // `GET /models` answering {"data":[{"id":""}]} must be "names no model": an empty id is also what a merge with
    // an empty cache records, so the two cannot be told apart.
    let e = Env::new();
    let srv = models_server(Box::new(|| Wire::Json(200, "{\"data\":[{\"id\":\"\"}]}".into())));
    let result = e.llm(&srv, &[]);
    assert!(result.is_err(), "an empty model id was accepted: {result:?}");
}

#[test]
fn endpoint_spellings() {
    plain_variant();
    // a path prefix, trailing slashes and the models/chat paths that are requested
    let e = Env::new();
    let srv = Srv::build(good5(), None, "/proxy/v1", 0);
    e.llm_at(&format!("{}///", srv.url), &[]).unwrap();
    assert_eq!(
        *srv.paths.lock().unwrap(),
        ["GET /proxy/v1/models", "POST /proxy/v1/chat/completions"],
        "double slashes would break a strict proxy"
    );
    // no /v1 on a server that only has /v1: the model query 404s
    let e = Env::new();
    let srv = Srv::start(good5());
    let bare = srv.url.trim_end_matches("/v1").to_string();
    let before = e.files();
    let err = e.llm_at(&bare, &[]).unwrap_err();
    assert!(err.contains("LLM unreachable"), "{err}");
    assert_eq!((srv.chats(), before), (0, e.files()));
    // schemes and shapes that cannot work all fail the same way and touch nothing
    let refused = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    for endpoint in [
        format!("https://127.0.0.1:{refused}/v1"),
        format!("http://127.0.0.1:{refused}/v1"),
        format!("127.0.0.1:{refused}/v1"),
        "ftp://127.0.0.1/v1".to_string(),
        String::new(),
        "http://".to_string(),
        "http://[::1]:1/v1".to_string(),
    ] {
        let e = Env::new();
        let before = e.files();
        let ids = e.dir().join("ids.txt");
        let args = ["--llm", "--ids", ids.to_str().unwrap(), "--endpoint", endpoint.as_str()];
        let result = e.cmd(&args);
        if endpoint.is_empty() {
            // an empty value is rejected as a missing value or accepted as an unreachable endpoint; never a success
            assert!(result.is_err(), "{endpoint:?}");
        } else {
            let err = result.unwrap_err();
            assert!(err.contains("LLM unreachable"), "{endpoint}: {err}");
        }
        assert_eq!(before, e.files(), "{endpoint}");
    }
}

// ============================================================================ 2. output validation

fn ids_of(items: &[Item]) -> BTreeMap<&'static str, String> {
    let mut m = BTreeMap::new();
    for i in items {
        let f = first(i);
        let key = if f.starts_with("Destroy target creature with power") {
            "shot"
        } else if f == "Counter target spell." {
            "ctr"
        } else if f.starts_with("Return from your graveyard") {
            "agadeem"
        } else if f.starts_with("+1: Create three") {
            "elspeth"
        } else {
            "ugin"
        };
        m.insert(key, i.id.clone());
    }
    m
}
type Edit = fn(&mut Value);
/// Per-entry corruptions of the Counterspell answer (`{"id","roles":[{"role":"counter","line":0}]}`): every one makes
/// THAT CARD's entry structurally invalid (the other cards of the batch stay valid).
fn counterspell_classes() -> Vec<(&'static str, Edit)> {
    vec![
        ("id key missing", |e| drop(e.as_object_mut().unwrap().remove("id"))),
        ("id is a number", |e| e["id"] = json!(0)),
        ("id is null", |e| e["id"] = Value::Null),
        ("id in capitals", |e| e["id"] = json!(e["id"].as_str().unwrap().to_uppercase())),
        ("id with a leading space", |e| e["id"] = json!(format!(" {}", e["id"].as_str().unwrap()))),
        ("id with a trailing space", |e| e["id"] = json!(format!("{} ", e["id"].as_str().unwrap()))),
        ("id not in the batch", |e| e["id"] = json!("z9")),
        ("roles is a string", |e| e["roles"] = json!("counter")),
        ("roles is null", |e| e["roles"] = Value::Null),
        ("roles key missing", |e| drop(e.as_object_mut().unwrap().remove("roles"))),
        ("roles is an object", |e| e["roles"] = json!({})),
        ("an element is a number", |e| e["roles"] = json!([7])),
        ("an element is null", |e| e["roles"] = json!([null])),
        ("an element is the old bare string", |e| e["roles"] = json!(["counter"])),
        ("an element without a role", |e| e["roles"] = json!([{"line": 0}])),
        ("a role id that is a number", |e| e["roles"] = json!([{"role": 3, "line": 0}])),
        ("a role in capitals", |e| e["roles"] = json!([{"role": "Counter", "line": 0}])),
        ("a role with a trailing space", |e| e["roles"] = json!([{"role": "counter ", "line": 0}])),
        ("a role with an underscore for the hyphen", |e| e["roles"] = json!([{"role": "sac_outlet", "line": 0}])),
        ("an empty role", |e| e["roles"] = json!([{"role": "", "line": 0}])),
        ("a role outside the vocabulary", |e| e["roles"] = json!([{"role": "banana", "line": 0}])),
        ("a repeated role", |e| e["roles"] = json!([{"role": "counter", "line": 0}, {"role": "counter", "line": 0}])),
        ("five roles", |e| {
            let roles = ["counter", "removal", "draw", "tokens", "ramp"];
            e["roles"] = json!(roles.map(|r| json!({"role": r, "line": 0})));
        }),
        ("line key missing", |e| drop(e["roles"][0].as_object_mut().unwrap().remove("line"))),
        ("line negative", |e| e["roles"][0]["line"] = json!(-1)),
        ("line a float", |e| e["roles"][0]["line"] = json!(0.0)),
        ("line a fraction", |e| e["roles"][0]["line"] = json!(0.5)),
        ("line a string", |e| e["roles"][0]["line"] = json!("0")),
        ("line null", |e| e["roles"][0]["line"] = Value::Null),
        ("line true", |e| e["roles"][0]["line"] = json!(true)),
        ("line an array", |e| e["roles"][0]["line"] = json!([0])),
        ("line an object", |e| e["roles"][0]["line"] = json!({"line": 0})),
        ("line out of range", |e| e["roles"][0]["line"] = json!(99)),
        ("line u64::MAX", |e| e["roles"][0]["line"] = json!(u64::MAX)),
        ("line 1e30", |e| e["roles"][0]["line"] = serde_json::from_str("1e30").unwrap()),
    ]
}
/// Corruptions that are NOT faults: a role whose guard fails on the cited line is kept raw (the merge drops it as a
/// `GuardDrop`), and keys the schema does not have are ignored.
fn soft_classes() -> Vec<(&'static str, Edit)> {
    vec![
        ("a role that fails its guard on the cited line", |e| e["roles"] = json!([{"role": "draw", "line": 0}])),
        ("a leftover primary and evidence of the old shape", |e| {
            e["primary"] = json!("draw");
            e["evidence"] = json!({"draw": 0});
        }),
    ]
}
/// Corruptions of the Ugin's Mastery answer (`roles` empty): invalid too.
fn ugin_classes() -> Vec<(&'static str, Edit)> {
    vec![
        ("roles a string for an empty roles list", |e| e["roles"] = json!("")),
        ("an element without a role for an empty roles list", |e| e["roles"] = json!([{"line": 0}])),
        ("an unknown role for an empty roles list", |e| e["roles"] = json!([{"role": "banana", "line": 0}])),
    ]
}
/// What a corrupted document costs: the whole reply is unusable, `n` cards are faulty, or nothing (harmless).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Doc {
    Whole,
    Cards(usize),
    Harmless,
}
/// Corruptions of the whole document.
fn document_classes() -> Vec<(&'static str, fn(&mut Value), Doc)> {
    vec![
        ("no cards key", |v| *v = json!({"results": v["cards"].clone()}), Doc::Whole),
        ("cards is an object", |v| v["cards"] = json!({}), Doc::Whole),
        ("cards is null", |v| v["cards"] = Value::Null, Doc::Whole),
        ("the root is an array", |v| *v = v["cards"].clone(), Doc::Whole),
        ("the root is a string", |v| *v = json!("cards"), Doc::Whole),
        ("one entry too few", |v| drop(v["cards"].as_array_mut().unwrap().pop()), Doc::Cards(1)),
        (
            "one entry too many",
            |v| {
                let extra = json!({"id": "z9", "roles": []});
                v["cards"].as_array_mut().unwrap().push(extra);
            },
            Doc::Harmless,
        ),
        (
            "a repeated id replacing another",
            |v| {
                let id = v["cards"][0]["id"].clone();
                let last = v["cards"].as_array().unwrap().len() - 1;
                v["cards"][last]["id"] = id;
            },
            Doc::Cards(2),
        ),
        (
            "every entry duplicated",
            |v| {
                let all = v["cards"].as_array().unwrap().clone();
                v["cards"].as_array_mut().unwrap().extend(all);
            },
            Doc::Cards(5),
        ),
    ]
}
/// The five fixture cards as a batch (`a0` .. `a4`) with the lines the model is shown.
fn five_batch(index: &Index) -> Vec<(String, Vec<(usize, String)>)> {
    FIVE.iter().enumerate().map(|(n, name)| (llm::batch_id(n), roles::prompt_lines(card_of(index, name)))).collect()
}

#[test]
fn validate_output_judges_every_card_alone_and_accepts_the_clean_one() {
    plain_variant();
    let e = Env::new();
    let index = e.index();
    let tax = taxonomy();
    let owned = five_batch(&index);
    let cards: Vec<(String, &[(usize, String)])> = owned.iter().map(|(i, l)| (i.clone(), l.as_slice())).collect();
    let items: Vec<Item> =
        owned.iter().map(|(i, l)| Item { id: i.clone(), type_line: String::new(), lines: l.clone() }).collect();
    let clean = valid5(&items);
    let ids = ids_of(&items);
    assert!(llm::validate_output(&clean.to_string(), &cards, &tax).unwrap().iter().all(|r| r.is_ok()));
    let position =
        |key: &str| clean["cards"].as_array().unwrap().iter().position(|c| c["id"] == ids[key].as_str()).unwrap();
    // a structural fault in one entry condemns that card only
    for (key, classes) in [("ctr", counterspell_classes()), ("ugin", ugin_classes())] {
        for (name, edit) in classes {
            let mut v = clean.clone();
            edit(&mut v["cards"][position(key)]);
            let verdicts = llm::validate_output(&v.to_string(), &cards, &tax).unwrap();
            let bad: Vec<usize> = (0..verdicts.len()).filter(|i| verdicts[*i].is_err()).collect();
            assert_eq!(bad, [index_of(&owned, &ids[key])], "{name}");
        }
    }
    // soft corruptions are answers: the raw roles are kept
    for (name, edit) in soft_classes() {
        let mut v = clean.clone();
        edit(&mut v["cards"][position("ctr")]);
        let verdicts = llm::validate_output(&v.to_string(), &cards, &tax).unwrap();
        assert!(verdicts.iter().all(|r| r.is_ok()), "{name}");
    }
    for (name, edit, doc) in document_classes() {
        let mut v = clean.clone();
        edit(&mut v);
        match (doc, llm::validate_output(&v.to_string(), &cards, &tax)) {
            (Doc::Whole, Err(_)) => {}
            (Doc::Cards(n), Ok(verdicts)) => assert_eq!(verdicts.iter().filter(|r| r.is_err()).count(), n, "{name}"),
            (Doc::Harmless, Ok(verdicts)) => assert!(verdicts.iter().all(|r| r.is_ok()), "{name}"),
            (doc, other) => panic!("{name}: expected {doc:?}, got {:?}", other.map(|v| v.len())),
        }
    }
    // not JSON at all
    for text in ["", "{", "null", "[]", "\"x\"", "```json\n{}\n```"] {
        assert!(llm::validate_output(text, &cards, &tax).is_err(), "accepted {text:?}");
    }
}
fn index_of(owned: &[(String, Vec<(usize, String)>)], id: &str) -> usize {
    owned.iter().position(|(i, _)| i == id).unwrap()
}

#[test]
fn every_invalid_class_goes_through_the_retry_ladder_with_the_exact_request_counts() {
    plain_variant();
    // The corruption is applied wherever the card appears, so it is bad in the batch and again when asked alone.
    type Mutate = Box<dyn Fn(&mut Value, &BTreeMap<&'static str, String>) + Send + Sync>;
    // (name, mutation, requests when bad every time, failed then, requests when bad only the first time)
    let mut classes: Vec<(String, Mutate, usize, usize, usize)> = vec![];
    for (key, list) in [("ctr", counterspell_classes()), ("ugin", ugin_classes())] {
        for (name, edit) in list {
            let m: Mutate = Box::new(move |v, ids| {
                if let Some(c) = v["cards"]
                    .as_array_mut()
                    .and_then(|a| a.iter_mut().find(|c| ids.get(key).is_some_and(|i| c["id"] == i.as_str())))
                {
                    edit(c)
                }
            });
            // the batch, then the one faulty card alone (and, if it is bad every time, still faulty)
            classes.push((name.to_string(), m, 2, 1, 2));
        }
    }
    for (name, edit, doc) in document_classes() {
        let m: Mutate = Box::new(move |v, _| edit(v));
        // a whole unusable document or a card missing from it: every such card is asked alone once
        let (always, failed, once) = match (name, doc) {
            (_, Doc::Whole) => (1 + 5, 5, 1 + 5),
            ("one entry too few", _) => (1 + 1, 1, 1 + 1), // the missing card, alone, is again missing
            ("a repeated id replacing another", _) => (1 + 2, 0, 1 + 2), // alone, the cards are fine
            (_, Doc::Cards(_)) => (1 + 5, 5, 1 + 5),
            (_, Doc::Harmless) => (1, 0, 1),
        };
        classes.push((name.to_string(), m, always, failed, once));
    }
    for (name, mutate, always, failed, once) in classes {
        // (a) bad every time
        let e = Env::new();
        let mutate = Arc::new(mutate);
        let m = mutate.clone();
        let srv = Srv::start(Box::new(move |_, body| {
            let items = parse_batch(body);
            let mut v = valid5(&items);
            m(&mut v, &ids_of(&items));
            Wire::Content(v.to_string())
        }));
        let out = e.llm(&srv, &[]).unwrap_or_else(|err| err);
        assert_eq!((srv.chats(), counts(&out)), (always, (5 - failed, failed)), "{name}: {out}");
        assert_eq!(e.rows().len(), 5 - failed, "{name}");
        // (b) bad only the first request: the cards asked alone afterwards recover
        let e = Env::new();
        let m = mutate.clone();
        let srv = Srv::start(Box::new(move |n, body| {
            let items = parse_batch(body);
            let mut v = valid5(&items);
            if n == 0 {
                m(&mut v, &ids_of(&items));
            }
            Wire::Content(v.to_string())
        }));
        let out = e.llm(&srv, &[]).unwrap();
        assert_eq!((srv.chats(), counts(&out)), (once, (5, 0)), "{name}: {out}");
        assert!(out.contains("0 batch retries"), "{name}: {out}");
        assert_eq!(e.rows().len(), 5, "{name}");
    }
}

#[test]
fn a_single_card_batch_is_asked_once_because_a_resend_would_be_byte_identical() {
    plain_variant();
    // `--batch 1` (or a last chunk of one card): a card asked alone is not asked again alone.
    let e = Env::new();
    let srv = Srv::start(Box::new(|_, body| {
        let items = parse_batch(body);
        let mut v = valid5(&items);
        v["cards"][0]["id"] = json!("z9");
        Wire::Content(v.to_string())
    }));
    let out = e.llm(&srv, &["--batch", "1", "--limit", "1"]).unwrap_err();
    assert_eq!((srv.chats(), counts(&out)), (1, (0, 1)), "{out}");
}

#[test]
fn guard_regex_hazards_a_backtracking_guard_fails_closed_and_never_hangs_or_panics() {
    plain_variant();
    // `fancy_regex` reports a backtrack limit as an error; the stage treats an erroring guard as a failed guard.
    let e = Env::new();
    let hazard = "'(?i)^(?=.)(?:a|aa)+b'";
    let toml = TAXONOMY.replacen("guard = '(?i)counter|exile.*spells?'", &format!("guard = {hazard}"), 1);
    assert_ne!(toml, TAXONOMY);
    std::fs::write(e.dir().join("roles.toml"), &toml).unwrap();
    let tax = Taxonomy::parse(&toml).unwrap();
    let long = "a".repeat(2000);
    let started = Instant::now();
    assert!(!tax.guard("counter").unwrap().is_match(&long).unwrap_or(false));
    assert!(started.elapsed() < Duration::from_secs(20), "guard check took {:?}", started.elapsed());
    // a real card: Counterspell cites its line for `counter`, the guard cannot match ("Counter target spell." has no b)
    let srv = Srv::start(good5());
    let out = e.llm(&srv, &[]).unwrap();
    // a guard that never matches is a guard miss: soft, the raw answer is kept and the merge drops the role
    assert_eq!((counts(&out), srv.chats()), ((5, 0), 1), "{out}");
}

#[test]
fn citing_a_line_that_is_not_shown_is_refused_for_every_skipped_or_missing_line_in_the_fixture() {
    plain_variant();
    let e = Env::new();
    let index = e.index();
    let tax = taxonomy();
    let mut checked = 0;
    for card in index.cards.iter().filter(|c| roles::is_deck_card(c)) {
        let shown = roles::prompt_lines(card);
        let n = card.lines().len();
        let cards = [("a0".to_string(), shown.as_slice())];
        // every index that is not shown (skipped, empty after normalising, or beyond the end)
        for i in (0..n + 3).filter(|i| !shown.iter().any(|(s, _)| s == i)) {
            let text = roles::line_text(card, i);
            for role in roles::IDS {
                if !tax.guard(role).unwrap().is_match(&text).unwrap_or(false) {
                    continue; // then it fails on the guard anyway; the point is the cases the guard would pass
                }
                let answer = json!({"cards": [entry("a0", &[role], &[(role, i)])]});
                let result = llm::validate_output(&answer.to_string(), &cards, &tax).unwrap();
                assert!(result[0].is_err(), "{} line {i} ({text:?}) cited for {role} was accepted", card.raw.name);
                checked += 1;
            }
        }
        // every shown line and every role: accepted exactly when the guard matches the shown text
        for (i, text) in &shown {
            for role in roles::IDS {
                let answer = json!({"cards": [entry("a0", &[role], &[(role, *i)])]});
                // a shown line is always citable; whether the guard passes decides only whether the merge keeps the role
                let ok = llm::validate_output(&answer.to_string(), &cards, &tax).unwrap()[0].is_ok();
                assert!(ok, "{} {role} line {i} ({text:?})", card.raw.name);
            }
        }
    }
    assert!(checked > 20, "the sweep must exercise skipped lines (MDFC land backs, ultimates): {checked}");
}

#[test]
fn a_card_whose_only_line_is_over_long_is_never_sent_and_an_over_long_line_cannot_be_cited() {
    plain_variant();
    let long = format!("Draw a card. {}", "x".repeat(2100));
    let record = |name: &str, id: &str, text: &str| {
        let mut c: Value = serde_json::from_str(&std::fs::read_to_string(EXTRA_CARDS).unwrap()).unwrap();
        let mut c = c.as_array_mut().unwrap().remove(3); // a legal Instant record
        c["name"] = json!(name);
        c["oracle_id"] = json!(id);
        c["oracle_text"] = json!(text);
        c
    };
    let e = Env::custom(vec![
        record("Adv Long Only", "adv-long-1", &long),
        record("Adv Long Second", "adv-long-2", &format!("Counter target spell.\n{long}")),
    ]);
    std::fs::write(e.dir().join("ids.txt"), "Adv Long Only\nAdv Long Second\n").unwrap();
    let index = e.index();
    assert!(roles::prompt_lines(card_of(&index, "Adv Long Only")).is_empty());
    assert_eq!(roles::prompt_lines(card_of(&index, "Adv Long Second")).len(), 1);
    let srv = Srv::start(good5());
    let out = e.llm(&srv, &[]).unwrap();
    // only the second card is a candidate; the first has nothing to read or cite
    assert!(out.contains("1 candidate cards"), "{out}");
    let user = srv.bodies()[0]["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap().to_string();
    assert!(user.starts_with("Classify these 1 cards.") && !user.contains("xxxx"), "{user}");
    // citing the over-long line 1 of the second card is refused whatever the guard says
    let tax = taxonomy();
    let shown = roles::prompt_lines(card_of(&index, "Adv Long Second"));
    let cards = [("a0".to_string(), shown.as_slice())];
    let answer = json!({"cards": [entry("a0", &["draw"], &[("draw", 1)])]});
    assert!(llm::validate_output(&answer.to_string(), &cards, &tax).unwrap()[0].is_err());
    // the merge gives the never-sent card an empty answer and does not mark it llm_failed
    let map = e.merged();
    let only = assignment_of(&index, &map, "Adv Long Only");
    assert!(!only.llm_failed, "{only:?}");
}

#[test]
fn citing_a_line_of_the_other_face_is_judged_on_its_text_and_the_merge_records_that_face() {
    plain_variant();
    let e = Env::custom(serde_json::from_str::<Vec<Value>>(&std::fs::read_to_string(EXTRA_CARDS).unwrap()).unwrap());
    let index = e.index();
    let card = card_of(&index, "Adv Dual Face");
    let shown = roles::prompt_lines(card);
    assert_eq!(shown.len(), 2, "{shown:?}");
    // the card's roles live on different faces: draw on line 0 (front), counter on line 1 (back)
    let tax = taxonomy();
    let cards = [("a0".to_string(), shown.as_slice())];
    let answer = json!({"cards": [entry("a0", &["counter", "lifegain"], &[("counter", 1), ("lifegain", 0)])]});
    llm::validate_output(&answer.to_string(), &cards, &tax).unwrap()[0].as_ref().unwrap();
    // a role cited on the wrong face's line fails on that line's text, so it is not a face question but a guard one
    let swapped = json!({"cards": [entry("a0", &["counter", "lifegain"], &[("counter", 0), ("lifegain", 1)])]});
    // (the guard miss is soft: the raw answer is kept and the merge drops it)
    assert!(llm::validate_output(&swapped.to_string(), &cards, &tax).unwrap()[0].is_ok());
    // through the whole stage: the merged evidence carries the face of the line that was cited
    std::fs::write(e.dir().join("ids.txt"), "Adv Dual Face\n").unwrap();
    let srv = Srv::start(Box::new(|_, body| {
        let items = parse_batch(body);
        let answers: Vec<Value> =
            items.iter().map(|i| entry(&i.id, &["counter", "lifegain"], &[("counter", 1), ("lifegain", 0)])).collect();
        Wire::Content(json!({"cards": answers}).to_string())
    }));
    e.llm(&srv, &[]).unwrap();
    let map = e.merged();
    let a = assignment_of(&index, &map, "Adv Dual Face");
    assert!(a.needs_llm || a.roles == ["counter", "lifegain"], "{a:?}");
    if a.needs_llm {
        let faces: BTreeMap<&str, Option<usize>> = a.evidence.iter().map(|(r, e)| (r.as_str(), e.face)).collect();
        assert_eq!(faces.get("counter"), Some(&Some(1)), "{a:?}");
        assert_eq!(faces.get("lifegain"), Some(&Some(0)), "{a:?}");
    }
    let mut attached = e.index();
    attached.attach_roles(&map).unwrap();
}

// ============================================================================ 3. the cache

fn pseudo_srv(salt: u64) -> Srv {
    Srv::start(Box::new(move |_, body| Wire::Content(pseudo(&parse_batch(body), salt).to_string())))
}
/// `roles --llm` over every card of the fixture catalog (no ids file).
fn llm_all(e: &Env, srv: &Srv, extra: &[&str]) -> Result<String, String> {
    let mut args = vec!["--llm", "--endpoint", srv.url.as_str()];
    args.extend_from_slice(extra);
    e.cmd(&args).map(|(t, _)| t)
}

#[test]
fn a_damaged_cache_file_is_an_error_naming_the_line_and_no_command_touches_any_file() {
    plain_variant();
    let e = Env::new();
    let srv = Srv::start(good5());
    e.llm(&srv, &[]).unwrap();
    e.cmd(&["--rules"]).unwrap();
    let good = std::fs::read_to_string(e.cache()).unwrap();
    let lines: Vec<&str> = good.lines().collect();
    assert_eq!(lines.len(), 5);
    let cut = &lines[4][..lines[4].len() / 2];
    let damaged: Vec<(&str, String, &str)> = vec![
        ("garbage line", format!("{}\nthis is not json\n{}\n", lines[0], lines[1]), "line 2"),
        ("truncated last line", format!("{}\n{}\n{}", lines[0], lines[1], cut), "line 3"),
        ("byte order mark", format!("\u{feff}{good}"), "line 1"),
        ("unknown field", format!("{}\n", lines[0].replacen("{\"key\"", "{\"extra\":1,\"key\"", 1)), "line 1"),
        ("roles a string", format!("{}\n", lines[0].replacen("\"roles\":[", "\"roles\":\"x\",\"r2\":[", 1)), "line 1"),
        ("a json array", "[1,2,3]\n".to_string(), "line 1"),
        ("a nul byte", format!("{}\n\0\n", lines[0]), "line 2"),
    ];
    for (name, text, at) in damaged {
        std::fs::write(e.cache(), &text).unwrap();
        let before = e.files();
        for args in [vec!["--merge"], vec!["--llm", "--dry-run", "--model", "m"]] {
            let result = if args[0] == "--merge" { e.cmd(&args).map(|r| r.0) } else { e.llm(&srv, &args[1..]) };
            let err = result.unwrap_err();
            assert!(err.contains("llm_roles.jsonl") && err.contains(at), "{name} {args:?}: {err}");
        }
        let chats = srv.chats();
        let err = e.llm(&srv, &[]).unwrap_err();
        assert!(err.contains("llm_roles.jsonl"), "{name}: {err}");
        assert_eq!(srv.chats(), chats, "{name}: a damaged cache stops the run before any chat request");
        assert_eq!(before, e.files(), "{name}: no file changed by a failing command");
    }
    // the same for the pending file, which the loader reads second
    std::fs::write(e.cache(), &good).unwrap();
    std::fs::write(e.dir().join("llm_roles.pending.jsonl"), "{oops\n").unwrap();
    let err = e.cmd(&["--merge"]).unwrap_err();
    assert!(err.contains("llm_roles.pending.jsonl") && err.contains("line 1"), "{err}");
}

#[test]
fn harmless_cache_file_shapes_are_read_and_rewritten_sorted_without_duplicates() {
    plain_variant();
    let e = Env::new();
    let srv = Srv::start(good5());
    e.llm(&srv, &[]).unwrap();
    let good = std::fs::read_to_string(e.cache()).unwrap();
    let mut lines: Vec<String> = good.lines().map(str::to_string).collect();
    lines.reverse(); // unsorted
    let dup = lines[0].clone();
    lines.push(dup); // a duplicate key
    let shapes: Vec<(&str, String)> = vec![
        ("unsorted with a duplicate", format!("{}\n", lines.join("\n"))),
        ("blank lines and indentation", format!("\n  {}\n\n\n{}\n", lines[0], lines[1..].join("\n"))),
        ("crlf", format!("{}\r\n", lines.join("\r\n"))),
        ("no final newline", lines.join("\n")),
    ];
    for (name, text) in shapes {
        std::fs::write(e.cache(), &text).unwrap();
        let chats = srv.chats();
        let out = e.llm(&srv, &[]).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert_eq!(srv.chats(), chats, "{name}: all five cards are cached: {out}");
        assert!(out.contains("5 cached"), "{name}: {out}");
        assert_eq!(e.merged().meta.model, "fake-model-1", "{name}");
    }
    // a run that has something new to write replaces the file with sorted, de-duplicated rows
    std::fs::write(e.cache(), format!("{}\n", lines.join("\n"))).unwrap();
    std::fs::write(e.dir().join("ids.txt"), format!("{}\nLightning Bolt\n", FIVE.join("\n"))).unwrap();
    e.llm(&srv, &[]).unwrap();
    let text = std::fs::read_to_string(e.cache()).unwrap();
    let keys: Vec<String> =
        text.lines().map(|l| serde_json::from_str::<Value>(l).unwrap()["key"].as_str().unwrap().to_string()).collect();
    assert_eq!(keys.len(), 6);
    assert!(keys.windows(2).all(|w| w[0] < w[1]), "sorted and unique: {keys:?}");
    e.no_temp_files();
}

#[test]
fn of_two_rows_with_one_key_the_later_wins_and_a_pending_row_beats_the_tracked_one() {
    plain_variant();
    let e = Env::new();
    let (tracked, later) = (
        e.row("Skywhaler's Shot", "m1", &["removal"], &[("removal", 0)]),
        e.row("Skywhaler's Shot", "m1", &["selection"], &[("selection", 0)]),
    );
    e.write_rows("llm_roles.jsonl", &[tracked.clone(), later.clone()]);
    let map = e.merged();
    let shot = assignment_of(&e.index(), &map, "Skywhaler's Shot");
    // add-only merge (keep_rule everywhere): the rules' removal stays beside the later row's selection
    assert_eq!(shot.roles, ["selection", "removal"], "the later duplicate wins: {shot:?}");
    // pending beats tracked on the same key
    e.write_rows("llm_roles.jsonl", &[later]);
    e.write_rows("llm_roles.pending.jsonl", &[tracked]);
    let map = e.merged();
    let shot = assignment_of(&e.index(), &map, "Skywhaler's Shot");
    assert_eq!(shot.roles, ["removal"], "the pending row wins: {shot:?}");
    // and a card with a pending row counts as cached for `--llm --pending`
    let srv = Srv::start(good5());
    let out = e.llm(&srv, &["--pending", "--model", "m1", "--dry-run"]).unwrap();
    assert!(out.contains("1 cached") && out.contains("4 to classify"), "{out}");
}

#[test]
fn a_cached_row_for_a_card_whose_text_changed_is_ignored_not_misapplied() {
    plain_variant();
    let e = Env::new();
    let srv = Srv::start(good5());
    e.llm(&srv, &[]).unwrap();
    // edit the card's rules text (not the name): the old row is unreachable
    let path = e.dir().join("oracle.json");
    let mut oracle: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    for c in oracle["cards"].as_array_mut().unwrap() {
        if c["name"] == "Skywhaler's Shot" {
            let text = c["oracle_text"].as_str().unwrap().replace("power 3", "power 4");
            c["oracle_text"] = json!(text);
        }
    }
    std::fs::write(&path, oracle.to_string()).unwrap();
    let before = e.rows().len();
    // before re-running, the merge treats the changed card as unanswered (llm_failed, rule roles), the rest as answered
    let map = e.merged();
    let index = e.index();
    assert!(assignment_of(&index, &map, "Skywhaler's Shot").llm_failed);
    assert!(!assignment_of(&index, &map, "Ugin's Mastery").llm_failed);
    // re-running asks about that one card only, and the old row stays in the file
    let chats = srv.chats();
    let out = e.llm(&srv, &[]).unwrap();
    assert_eq!(srv.chats(), chats + 1, "{out}");
    assert!(out.contains("4 cached") && out.contains("1 classified"), "{out}");
    assert_eq!(e.rows().len(), before + 1);
    assert!(!assignment_of(&index, &e.merged(), "Skywhaler's Shot").llm_failed);
}

#[test]
fn what_changes_the_cache_key_and_what_does_not() {
    plain_variant();
    let e = Env::new();
    let srv = Srv::start(good5());
    e.llm(&srv, &[]).unwrap();
    let mut asked = srv.chats();
    let mut rerun = |label: &str, expect: usize| {
        let out = e.llm(&srv, &[]).unwrap_or_else(|err| panic!("{label}: {err}"));
        asked += expect;
        assert_eq!(srv.chats(), asked, "{label}: {out}");
    };
    // metadata the model never sees
    let path = e.dir().join("oracle.json");
    let mut oracle: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    for c in oracle["cards"].as_array_mut().unwrap() {
        c["usd"] = json!("123.45");
        c["edhrec_rank"] = json!(1);
        c["released_at"] = json!("1993-08-05");
        c["set"] = json!("lea");
        c["rarity"] = json!("mythic");
        c["reserved"] = json!(true);
        c["game_changer"] = json!(true);
        c["keywords"] = json!(["Haste"]);
        if c.get("mana_cost").is_some() {
            c["mana_cost"] = json!("{9}");
        }
        if c["name"] == "Skywhaler's Shot" {
            // reminder text is removed before the model sees the card, so rewording it changes nothing
            let text = c["oracle_text"].as_str().unwrap().replace("Look at the top card", "Peek at the top card");
            c["oracle_text"] = json!(text);
        }
    }
    std::fs::write(&path, oracle.to_string()).unwrap();
    rerun("price, rank, set, rarity, keywords, mana cost and reminder wording", 0);
    // a roles_overrides.toml edit changes rules_fnv, which the key must never contain
    std::fs::write(
        e.dir().join("roles_overrides.toml"),
        "[[card]]\nname='Mogg Fanatic'\nroles=[]\nnote='an unrelated override'\n",
    )
    .unwrap();
    rerun("an unrelated override", 0);
    // comments and blank lines in roles.toml outside the prompt text
    let original = std::fs::read_to_string(e.dir().join("roles.toml")).unwrap();
    std::fs::write(e.dir().join("roles.toml"), format!("# a comment\n\n{original}\n\n# another\n")).unwrap();
    rerun("comments in roles.toml", 0);
    // a trailing space inside a definition is trimmed away; a doubled space is not
    let with = |from: &str, to: &str| {
        let text = original.replacen(from, to, 1);
        assert_ne!(text, original, "{from}");
        std::fs::write(e.dir().join("roles.toml"), text).unwrap();
    };
    with(
        "definition = \"makes opponents discard: each opponent or a target opponent discards, reveal-and-discard.\"",
        "definition = \"makes opponents discard: each opponent or a target opponent discards, reveal-and-discard. \"",
    );
    rerun("trailing space in a definition", 0);
    // every prompt-visible part invalidates all five cards (one batch, one request)
    with(
        "definition = \"makes opponents discard: each opponent or a target opponent discards, reveal-and-discard.\"",
        "definition = \"makes  opponents discard: each opponent or a target opponent discards, reveal-and-discard.\"",
    );
    rerun("a doubled space in a definition", 1);
    with("Planeswalker ultimates have been removed", "Planeswalker ultimates were removed");
    rerun("the shared block", 1);
    with("roles = [\"copy\", \"tokens\"]", "roles = [\"tokens\", \"copy\"]");
    rerun("the role list label of a convention", 1);
    with(
        "A token copy of a creature is copy AND tokens, copy first.",
        "A token copy of a creature is copy AND tokens.",
    );
    rerun("a convention text", 1);
    // back to the original prompt: warm again, the first rows are still there
    std::fs::write(e.dir().join("roles.toml"), &original).unwrap();
    rerun("the original taxonomy again", 0);
    // a card whose shown text changes is asked again, alone
    let mut oracle: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    for c in oracle["cards"].as_array_mut().unwrap() {
        if c["name"] == "Counterspell" {
            c["type_line"] = json!("Sorcery");
        }
    }
    std::fs::write(&path, oracle.to_string()).unwrap();
    rerun("a type line", 1);
}

#[test]
fn two_catalog_cards_with_one_text_share_one_request_and_one_row() {
    plain_variant();
    let extra: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(EXTRA_CARDS).unwrap()).unwrap();
    let e = Env::custom(extra);
    std::fs::write(e.dir().join("ids.txt"), "Adv Counter Twin A\nAdv Counter Twin B\n").unwrap();
    let srv = Srv::start(good5());
    let out = e.llm(&srv, &[]).unwrap();
    assert!(out.contains("2 candidate cards") && out.contains("1 classified") && srv.chats() == 1, "{out}");
    assert_eq!(e.rows().len(), 1);
    let user = srv.bodies()[0]["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap().to_string();
    assert!(user.starts_with("Classify these 1 cards."), "{user}");
    let map = e.merged();
    let index = e.index();
    let (a, b) = (assignment_of(&index, &map, "Adv Counter Twin A"), assignment_of(&index, &map, "Adv Counter Twin B"));
    assert_eq!((&a.roles, &a.evidence), (&b.roles, &b.evidence));
    assert!(!a.llm_failed && !b.llm_failed);
}

#[test]
fn a_read_only_cache_directory_is_an_error_that_leaves_no_partial_file_behind() {
    plain_variant();
    use std::os::unix::fs::PermissionsExt;
    if std::fs::metadata("/proc/self").map(|m| std::os::unix::fs::MetadataExt::uid(&m)).unwrap_or(1) == 0 {
        return; // root ignores directory permissions
    }
    let e = Env::new();
    let srv = Srv::start(good5());
    std::fs::set_permissions(e.dir(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let before = e.files();
    let err = e.llm(&srv, &[]).unwrap_err();
    assert!(err.to_lowercase().contains("permission denied"), "{err}");
    assert_eq!(before, e.files(), "no cache, no report and no temp file");
    // found out before anything is classified: the lock file beside the cache cannot be created
    assert_eq!(srv.chats(), 0, "the writability is probed before the classification");
}

#[test]
fn a_failing_report_write_after_the_cache_write_leaves_a_complete_cache_and_no_temp_file() {
    plain_variant();
    let e = Env::new();
    // `reports` is a file, so the report directory cannot be created
    std::fs::write(e.dir().join("reports"), "in the way").unwrap();
    let srv = Srv::start(good5());
    let err = e.llm(&srv, &[]).unwrap_err();
    assert!(err.contains("reports"), "{err}");
    assert_eq!(e.rows().len(), 5, "the cache was written whole before the report failed");
    e.no_temp_files();
    // `llm-run.json` is a directory, so the rename onto it fails and the temp file is removed again
    let e = Env::new();
    std::fs::create_dir_all(e.dir().join("reports/llm-run.json")).unwrap();
    std::fs::write(e.dir().join("reports/llm-run.json/keep"), "x").unwrap();
    let srv = Srv::start(good5());
    assert!(e.llm(&srv, &[]).is_err());
    e.no_temp_files();
    // a cache path that is a directory is an error before anything is sent
    let e = Env::new();
    std::fs::create_dir_all(e.cache()).unwrap();
    let srv = Srv::start(good5());
    let err = e.llm(&srv, &[]).unwrap_err();
    assert!(err.contains("llm_roles.jsonl"), "{err}");
    assert_eq!(srv.chats(), 0);
}

#[test]
fn defect_two_concurrent_runs_lose_each_others_rows() {
    plain_variant();
    // Each run reads the cache file at the start and rewrites the whole file at the end, so the run that finishes
    // last replaces the other's rows with its own. Two runs on disjoint cards must leave all five rows.
    let e = Env::new();
    let srv = Srv::build(good5(), None, "/v1", 600);
    let bin = env!("CARGO_BIN_EXE_cardgen");
    std::fs::write(e.dir().join("a.txt"), "Counterspell\nSkywhaler's Shot\n").unwrap();
    std::fs::write(e.dir().join("b.txt"), "Ugin's Mastery\nElspeth, Sun's Champion\nAgadeem's Awakening\n").unwrap();
    let spawn = |ids: &str| {
        std::process::Command::new(bin)
            .env_remove("CARDGEN_KNOWLEDGE_DIR")
            .arg("--root")
            .arg(&e.0)
            .args(["roles", "--llm", "--endpoint", &srv.url, "--ids"])
            .arg(e.dir().join(ids))
            .arg("--data-dir")
            .arg(e.dir())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap()
    };
    let mut a = spawn("a.txt");
    std::thread::sleep(Duration::from_millis(150));
    let mut b = spawn("b.txt");
    assert!(a.wait().unwrap().success() && b.wait().unwrap().success());
    assert_eq!(
        e.rows().len(),
        5,
        "rows of both runs must survive: {:?}",
        e.rows().iter().map(|r| &r.name).collect::<Vec<_>>()
    );
}

#[test]
fn defect_a_server_dying_inside_a_batch_throws_away_the_cards_already_answered_in_it() {
    plain_variant();
    // The batch fails twice (invalid), the cards are then asked one by one: three are answered, then the server dies.
    // Those three answers cost three requests and are valid; they must be on disk.
    let e = Env::new();
    let srv = Srv::start(Box::new(|n, body| {
        let items = parse_batch(body);
        match n {
            0 | 1 => Wire::Content("{\"cards\":[]}".into()),
            2..=4 => Wire::Content(valid5(&items).to_string()),
            _ => Wire::Drop,
        }
    }));
    let err = e.llm(&srv, &[]).unwrap_err();
    assert!(err.contains("unreachable mid-run"), "{err}");
    assert_eq!(e.rows().len(), 3, "the answered cards are kept: {err}");
}

#[test]
fn defect_one_card_the_server_cannot_answer_at_the_head_of_a_batch_aborts_the_whole_run() {
    plain_variant();
    // The server errors (HTTP 500) whenever one particular card is in the request, as llama.cpp can for an input it
    // cannot grammar-parse. Two batch failures plus the first per-card failure make three transport failures in a
    // row, so the run stops although four of the five cards are answerable, and every rerun stops at the same place.
    let e = Env::new();
    let index = e.index();
    let mut sorted: Vec<&knowledge::Card> = FIVE.iter().map(|n| card_of(&index, n)).collect();
    sorted.sort_by(|a, b| a.raw.oracle_id.cmp(&b.raw.oracle_id));
    let poison = roles::prompt_lines(sorted[0])[0].1.clone();
    let srv = Srv::start(Box::new(move |_, body| {
        let items = parse_batch(body);
        if items.iter().any(|i| first(i) == poison) {
            Wire::Json(500, "{\"error\":\"cannot parse\"}".into())
        } else {
            Wire::Content(valid5(&items).to_string())
        }
    }));
    let result = e.llm(&srv, &[]);
    let out = result.unwrap_or_else(|err| panic!("the run was aborted: {err}"));
    assert_eq!(counts(&out), (4, 1), "{out}");
}

#[test]
fn a_poison_card_later_in_the_batch_costs_four_requests_and_is_llm_failed_without_stopping_the_run() {
    plain_variant();
    let e = Env::new();
    let index = e.index();
    let mut sorted: Vec<&knowledge::Card> = FIVE.iter().map(|n| card_of(&index, n)).collect();
    sorted.sort_by(|a, b| a.raw.oracle_id.cmp(&b.raw.oracle_id));
    let poison = roles::prompt_lines(sorted[2])[0].1.clone();
    let srv = Srv::start(Box::new(move |_, body| {
        let items = parse_batch(body);
        if items.iter().any(|i| first(i) == poison) {
            Wire::Json(500, "{}".into())
        } else {
            Wire::Content(valid5(&items).to_string())
        }
    }));
    let out = e.llm(&srv, &[]).unwrap();
    // the batch (an error status), then every card alone once
    assert_eq!((srv.chats(), counts(&out)), (1 + 5, (4, 1)), "{out}");
}

#[test]
fn defect_merge_panics_on_a_cached_role_outside_the_vocabulary() {
    plain_variant();
    // A cache row is trusted at merge time except for its guard check. A hand-edited or damaged row naming a role
    // that is not in the taxonomy reaches `Taxonomy::guard`, which panics ("unknown role"). It must be an error or
    // the role must be dropped.
    let e = Env::new();
    e.write_rows("llm_roles.jsonl", &[e.row("Skywhaler's Shot", "m1", &["banana"], &[("banana", 0)])]);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| e.cmd(&["--merge"])));
    match outcome {
        Err(_) => panic!("`roles --merge` panicked on a cache row with an unknown role"),
        Ok(Ok(_)) => {
            let map = roles::load(&e.dir().join("roles.json")).unwrap();
            let shot = assignment_of(&e.index(), &map, "Skywhaler's Shot");
            assert!(!shot.roles.iter().any(|r| r == "banana"));
        }
        Ok(Err(_)) => {}
    }
}

#[test]
fn hostile_but_well_formed_cache_rows_never_break_the_merged_table() {
    plain_variant();
    let e = Env::new();
    let all: Vec<&str> = roles::IDS.to_vec();
    let ev_all: Vec<(&str, usize)> = all.iter().map(|r| (*r, 0)).collect();
    let mut hostile = vec![
        // every role at once, all citing line 0
        e.row("Skywhaler's Shot", "m1", &all, &ev_all),
        // a repeated role and a primary that is not among the roles
        e.row("Ugin's Mastery", "m1", &["tokens", "tokens"], &[("tokens", 0)]),
        // evidence missing for the role, a line number far out of range, and a negative-looking huge number
        e.row("Counterspell", "m1", &["counter", "draw"], &[("counter", usize::MAX)]),
        e.row("Elspeth, Sun's Champion", "m1", &["tokens"], &[("tokens", 2)]),
        e.row("Agadeem's Awakening // Agadeem, the Undercrypt", "m1", &[], &[("ramp", 1)]),
    ];
    hostile[1].primary = Some("banana".into());
    e.write_rows("llm_roles.jsonl", &hostile);
    let map = e.merged();
    let mut index = e.index();
    index.attach_roles(&map).unwrap();
    for (_, a) in map.iter() {
        assert!(a.roles.len() <= 4 && a.primary == a.roles.first().cloned().unwrap_or_default());
        assert_eq!(a.roles.iter().collect::<BTreeSet<_>>().len(), a.roles.len());
    }
    let shot = assignment_of(&index, &map, "Skywhaler's Shot");
    assert!(shot.roles.len() <= 4, "{shot:?}");
    // the all-roles row cites line 0 everywhere, so only roles whose guard matches "Destroy ... Scry 1." can survive
    let tax = taxonomy();
    for r in &shot.roles {
        let line = roles::line_text(card_of(&index, "Skywhaler's Shot"), shot.evidence[r].line.unwrap());
        assert!(tax.guard(r).unwrap().is_match(&line).unwrap(), "{r} on {line:?}");
    }
}

// ============================================================================ 4. the command line

#[test]
fn ids_files_blank_lines_comments_crlf_duplicates_and_case() {
    plain_variant();
    let e = Env::new();
    let srv = Srv::start(good5());
    let index = e.index();
    let id = |n: &str| card_of(&index, n).raw.oracle_id.clone();
    let text = format!(
        "\n# a comment\r\n  counterspell  \r\n{}\n\n\t{}\r\ncounterspell\n{}\n",
        id("Skywhaler's Shot"),
        id("Counterspell"),
        "ugin's mastery"
    );
    std::fs::write(e.dir().join("ids.txt"), text).unwrap();
    let out = e.llm(&srv, &["--dry-run"]).unwrap();
    assert!(out.contains("3 candidate cards"), "duplicates collapse; names fold case: {out}");
    // an empty ids file means no card at all, not every card
    std::fs::write(e.dir().join("ids.txt"), "\n# nothing\n\n").unwrap();
    let out = e.llm(&srv, &[]).unwrap();
    assert!(out.contains("0 candidate cards") && srv.chats() == 0, "{out}");
}

#[test]
fn ids_file_problems_are_errors_before_any_chat_request() {
    plain_variant();
    let e = Env::new();
    let srv = Srv::start(good5());
    for (name, text) in [
        ("an unknown name", "Counterspell\nNo Such Card\n"),
        ("an id with a typo", "cc187110-1148-4090-bbb8-e205694a39f6\n"),
        ("a name with a trailing comment", "Counterspell # the blue one\n"),
        ("a nul byte", "Counterspell\n\0\n"),
    ] {
        std::fs::write(e.dir().join("ids.txt"), text).unwrap();
        let before = e.files();
        let err = e.llm(&srv, &[]).unwrap_err();
        assert!(err.contains("ids.txt") && err.contains("no card"), "{name}: {err}");
        assert_eq!(srv.chats(), 0, "{name}");
        assert_eq!(before, e.files(), "{name}");
    }
    // a missing file and a directory
    for path in [e.dir().join("missing.txt"), e.dir().join("reports")] {
        std::fs::create_dir_all(e.dir().join("reports")).unwrap();
        let err = e.cmd(&["--llm", "--ids", path.to_str().unwrap(), "--endpoint", &srv.url]).unwrap_err();
        assert!(err.contains(path.to_str().unwrap()), "{err}");
    }
    // a relative path is read relative to the working directory of the process (checked with the real binary)
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cardgen"))
        .env_remove("CARDGEN_KNOWLEDGE_DIR")
        .current_dir(e.dir())
        .arg("--root")
        .arg(&e.0)
        .args(["roles", "--llm", "--dry-run", "--ids", "relative-missing.txt", "--endpoint", &srv.url])
        .arg("--data-dir")
        .arg(e.dir())
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("relative-missing.txt"));
    std::fs::write(e.dir().join("relative.txt"), "Counterspell\n").unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cardgen"))
        .env_remove("CARDGEN_KNOWLEDGE_DIR")
        .current_dir(e.dir())
        .arg("--root")
        .arg(&e.0)
        .args(["roles", "--llm", "--dry-run", "--ids", "relative.txt", "--endpoint", &srv.url])
        .arg("--data-dir")
        .arg(e.dir())
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("1 candidate cards"));
}

#[test]
fn excluded_and_override_cards_named_in_the_ids_file_are_silently_not_candidates() {
    plain_variant();
    let extra: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(EXTRA_CARDS).unwrap()).unwrap();
    let e = Env::custom(extra);
    std::fs::write(
        e.dir().join("roles_overrides.toml"),
        "[[card]]\nname='Counterspell'\nroles=['counter']\nnote='by hand'\n",
    )
    .unwrap();
    std::fs::write(e.dir().join("ids.txt"), "Adv Sticker Sheet\nAdv Attraction\nCounterspell\nAdv Reminder Only\n")
        .unwrap();
    let srv = Srv::start(good5());
    let out = e.llm(&srv, &["--dry-run"]).unwrap();
    // the Stickers sheet and the Attraction are not deck cards, Counterspell is fixed by an override, and the
    // reminder-only creature has nothing to read: no candidate is left
    assert!(out.contains("0 candidate cards"), "{out}");
    let out = e.llm(&srv, &[]).unwrap();
    assert!(out.contains("0 candidate cards") && srv.chats() == 0, "{out}");
    // the merge uses the override exactly and gives the sticker sheet no assignment
    let map = e.merged();
    let index = e.index();
    assert_eq!(assignment_of(&index, &map, "Counterspell").roles, ["counter"]);
    assert!(map.get(&card_of(&index, "Adv Sticker Sheet").raw.oracle_id).is_none());
    assert!(map.get(&card_of(&index, "Adv Attraction").raw.oracle_id).is_none());
}

#[test]
fn limit_and_batch_arguments_at_their_edges() {
    plain_variant();
    let e = Env::new();
    let srv = pseudo_srv(1);
    for bad in ["0", "-1", "abc", "1.5", "", "18446744073709551616", "1e3", " 1"] {
        let err = e.llm(&srv, &["--limit", bad]);
        assert!(err.is_err(), "--limit {bad:?} was accepted");
        let err = e.llm(&srv, &["--batch", bad]);
        assert!(err.is_err(), "--batch {bad:?} was accepted");
    }
    assert_eq!(srv.chats(), 0);
    // the largest usize is fine for --limit; for --batch the ceiling is the 260 ids a0..z9
    let out = e.llm(&srv, &["--limit", "18446744073709551615", "--dry-run", "--model", "m"]).unwrap();
    assert!(out.contains("5 candidate cards"), "{out}");
    for ok in ["1", "5", "260"] {
        e.llm(&srv, &["--batch", ok, "--dry-run", "--model", "m"]).unwrap_or_else(|err| panic!("--batch {ok}: {err}"));
    }
    for bad in ["261", "99999999999999999999", "18446744073709551615"] {
        assert!(e.llm(&srv, &["--batch", bad, "--dry-run", "--model", "m"]).is_err(), "--batch {bad}");
    }
    // --batch 1: one request per card; --batch 11 over twelve cards: the second batch starts at b0
    let out = e.llm(&srv, &["--batch", "1"]).unwrap();
    assert_eq!((srv.chats(), counts(&out)), (5, (5, 0)), "{out}");
    let twelve: Vec<&str> = FIVE
        .iter()
        .copied()
        .chain([
            "Lightning Bolt",
            "Brainstorm",
            "Ad Nauseam",
            "Animate Dead",
            "Arrest",
            "Armed Response",
            "Alabaster Wall",
        ])
        .collect();
    std::fs::write(e.dir().join("ids.txt"), twelve.join("\n")).unwrap();
    let before = srv.chats();
    let out = e.llm(&srv, &["--batch", "11"]).unwrap();
    assert_eq!(counts(&out), (7, 0), "five are cached: {out}");
    assert_eq!(srv.chats() - before, 1);
    // the model sees ids a0..a9 then b0 when a batch is larger than ten
    let e = Env::new();
    std::fs::write(e.dir().join("ids.txt"), twelve.join("\n")).unwrap();
    let srv = pseudo_srv(2);
    let out = e.llm(&srv, &["--batch", "11"]).unwrap();
    assert_eq!((srv.chats(), counts(&out)), (2, (12, 0)), "{out}");
    let user = srv.bodies()[0]["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap().to_string();
    assert!(user.starts_with("Classify these 11 cards.") && user.contains("[a9] ") && user.contains("[b0] "), "{user}");
    assert!(!user.contains("[b1] "));
}

#[test]
fn dry_run_sends_nothing_writes_nothing_and_with_a_model_id_needs_no_server_at_all() {
    plain_variant();
    let e = Env::new();
    let srv = Srv::start(good5());
    let before = e.files();
    for extra in [vec!["--dry-run"], vec!["--dry-run", "--pending"], vec!["--dry-run", "--limit", "2", "--batch", "1"]]
    {
        let out = e.llm(&srv, &extra).unwrap();
        assert!(out.starts_with("DRY RUN") && out.contains("0 chat requests"), "{extra:?}: {out}");
        assert_eq!(before, e.files(), "{extra:?}");
    }
    assert_eq!((srv.chats(), srv.models()), (0, 3), "only the model query is made");
    // with --model the server is not asked at all, and a closed port does not matter
    let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let out = e.llm_at(&format!("http://127.0.0.1:{port}/v1"), &["--dry-run", "--model", "whatever"]).unwrap();
    assert!(out.contains("model whatever") && out.contains("5 to classify"), "{out}");
    assert_eq!(before, e.files());
    // a dry run reads the cache with the model named: rows for that model count as cached
    e.write_rows("llm_roles.jsonl", &[e.row("Counterspell", "whatever", &["counter"], &[("counter", 0)])]);
    let out = e.llm_at(&srv.url, &["--dry-run", "--model", "whatever"]).unwrap();
    assert!(out.contains("1 cached") && out.contains("4 to classify"), "{out}");
}

#[test]
fn defect_the_model_flag_is_silently_ignored_outside_a_dry_run() {
    plain_variant();
    // `--model ID` is documented for `--llm` and `--merge`. In a real run the id still comes from GET /models, so the
    // flag changes nothing: no error, no effect. It must be honoured (the id in the request and the key) or refused.
    let e = Env::new();
    let srv = Srv::start(good5());
    let result = e.llm(&srv, &["--model", "custom-m"]);
    if result.is_ok() {
        let rows = e.rows();
        assert!(
            !rows.is_empty() && rows.iter().all(|r| r.model == "custom-m"),
            "rows were cached under '{}'",
            rows[0].model
        );
    }
}

#[test]
fn defect_merging_with_a_model_id_that_has_no_cached_rows_overwrites_roles_json_with_an_empty_merge() {
    plain_variant();
    // A typo in `--model` is not noticed: every routed card becomes `llm_failed` and a good merged table is replaced.
    let e = Env::new();
    let srv = Srv::start(good5());
    e.llm(&srv, &[]).unwrap();
    e.cmd(&["--merge"]).unwrap();
    let good = std::fs::read(e.dir().join("roles.json")).unwrap();
    let result = e.cmd(&["--merge", "--model", "fake-model-l"]);
    assert!(result.is_err(), "merged with a model that has no rows: {:?}", result.map(|r| r.0));
    assert_eq!(good, std::fs::read(e.dir().join("roles.json")).unwrap(), "the good table must survive");
}

#[test]
fn conflicting_modes_repeated_flags_and_unknown_flags_are_all_usage_errors_that_change_nothing() {
    plain_variant();
    let e = Env::new();
    let srv = Srv::start(good5());
    e.llm(&srv, &[]).unwrap();
    let before = e.files();
    let url = srv.url.as_str();
    let cases: Vec<Vec<&str>> = vec![
        vec!["--llm", "--merge"],
        vec!["--merge", "--llm"],
        vec!["--rules", "--llm"],
        vec!["--llm", "--validate"],
        vec!["--llm", "--stats"],
        vec!["--merge", "--limit", "3"],
        vec!["--merge", "--pending"],
        vec!["--merge", "--dry-run"],
        vec!["--merge", "--only-routed"],
        vec!["--merge", "--endpoint", url],
        vec!["--merge", "--ids", "x"],
        vec!["--merge", "--batch", "2"],
        vec!["--rules", "--model", "m"],
        vec!["--validate", "--model", "m"],
        vec!["--validate", "--limit", "2"],
        vec!["--validate", "--stage", "bogus"],
        vec!["--stats", "--pending"],
        vec!["--stats", "--stage", "merged"],
        vec!["--llm", "--stage", "merged"],
        vec!["--llm", "--json"],
        vec!["--llm", "--gold", "x"],
        vec!["--llm", "--seed", "3"],
        vec!["--llm", "--limit", "1", "--limit", "2"],
        vec!["--llm", "--batch", "3", "--batch", "4"],
        vec!["--llm", "--dry-run", "--dry-run"],
        vec!["--llm", "--pending", "--pending"],
        vec!["--llm", "--endpoint", url, "--endpoint", url],
        vec!["--llm", "--model", "a", "--model", "b"],
        vec!["--llm", "--limit"],
        vec!["--llm", "--ids"],
        vec!["--llm", "--endpoint", "--dry-run"],
        vec!["--llm", "--model", "--dry-run"],
        vec!["--llm", "--frobnicate"],
        vec!["--llm", "extra"],
        vec!["--llm", "-n", "3"],
        vec!["--limit", "3"],
        vec!["--dry-run"],
        vec!["--endpoint", url],
        vec![],
    ];
    for args in cases {
        let result = e.cmd(&args);
        assert!(result.is_err(), "accepted {args:?}: {:?}", result.map(|r| r.0));
    }
    assert_eq!(before, e.files());
    assert_eq!(srv.chats(), 1, "no usage error sent a request");
}

#[test]
fn a_run_where_most_cards_fail_exits_non_zero_and_says_so_in_the_reports_too() {
    plain_variant();
    // more than 20% of the cards asked about ending llm_failed is an error exit, so a scheduled job sees a dead model
    let e = Env::new();
    let srv = Srv::start(always(json!({"cards": []})));
    let err =
        e.cmd(&["--llm", "--ids", e.dir().join("ids.txt").to_str().unwrap(), "--endpoint", &srv.url]).unwrap_err();
    assert!(err.contains("5 of 5 cards ended llm_failed") && counts(&err) == (0, 5), "{err}");
    let report: Value =
        serde_json::from_str(&std::fs::read_to_string(e.dir().join("reports/llm-run.json")).unwrap()).unwrap();
    assert_eq!((report["classified"].as_u64(), report["failed"].as_u64()), (Some(0), Some(5)));
}

// ============================================================================ 5. `merge` properties

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = splitmix(self.0);
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }
}
fn rule_evidence(certain: bool, line: usize) -> Evidence {
    Evidence { source: Source::Rule, certain, rule: "r".into(), face: None, line: Some(line) }
}
fn random_rule(rng: &mut Rng) -> Assignment {
    let mut roles: Vec<String> = vec![];
    for _ in 0..rng.below(5) {
        let r = roles::IDS[rng.below(26)].to_string();
        if !roles.contains(&r) {
            roles.push(r);
        }
    }
    let needs_llm = rng.chance(65);
    let route = if needs_llm {
        let all = [
            RouteReason::UnknownLine,
            RouteReason::NoRole,
            RouteReason::RejectedHit,
            RouteReason::SameLine,
            RouteReason::RiskyRole,
            RouteReason::Overflow,
            RouteReason::RegexError,
        ];
        let mut route: Vec<RouteReason> = vec![all[rng.below(7)]];
        if rng.chance(30) {
            route.push(all[rng.below(7)]);
        }
        route.sort();
        route.dedup();
        route
    } else {
        vec![]
    };
    let evidence: BTreeMap<String, Evidence> =
        roles.iter().map(|r| (r.clone(), rule_evidence(rng.chance(30), rng.below(8)))).collect();
    Assignment {
        primary: roles.first().cloned().unwrap_or_default(),
        roles,
        evidence,
        needs_llm,
        route,
        llm_failed: false,
    }
}
/// `hostile` allows what a damaged cache row could hold: more than four roles, repeated roles, a primary outside the
/// roles and missing evidence. Validated answers (`hostile == false`) never have those.
fn random_llm(rng: &mut Rng, rule: &Assignment, hostile: bool) -> LlmCard {
    let n = if hostile { rng.below(8) } else { rng.below(5) };
    let mut roles: Vec<String> = vec![];
    for _ in 0..n {
        let r = if !rule.roles.is_empty() && rng.chance(50) {
            rule.roles[rng.below(rule.roles.len())].clone()
        } else {
            roles::IDS[rng.below(26)].to_string()
        };
        if hostile || !roles.contains(&r) {
            roles.push(r);
        }
    }
    let primary = if roles.is_empty() {
        if hostile && rng.chance(20) { Some("banana".into()) } else { None }
    } else if hostile && rng.chance(25) {
        Some(roles::IDS[rng.below(26)].to_string())
    } else {
        Some(roles[0].clone())
    };
    let mut evidence = BTreeMap::new();
    for r in &roles {
        if !hostile || rng.chance(90) {
            let line = rng.below(9);
            evidence.insert(r.clone(), line);
        }
    }
    LlmCard { roles, primary, evidence, why: None }
}
/// A deterministic stand-in for the guard check and for `keep_rule`: lines 0 to 5 exist, a third of the
/// (role, line) pairs fail their guard, and a role's face is `None` on even lines.
fn guard_ok(role: &str, line: usize) -> Option<Option<usize>> {
    (line < 6 && text_hash(&format!("{role}/{line}")) % 3 != 0).then_some(if line % 2 == 0 {
        None
    } else {
        Some(line / 2)
    })
}
fn keeps(role: &str) -> bool {
    text_hash(&format!("keep/{role}")) % 4 == 0
}
fn sorted(mut v: Vec<&String>) -> Vec<&String> {
    v.sort();
    v
}
fn do_merge(
    rule: &Assignment,
    llm: Option<&LlmCard>,
    ovr: Option<&Assignment>,
) -> (Assignment, Vec<llm::Disagreement>) {
    llm::merge(rule, llm, ovr, &MergeEnv { keep: &keeps, check: &guard_ok })
}

#[test]
fn merge_invariants_hold_over_many_random_inputs() {
    plain_variant();
    let mut rng = Rng(0xC0FFEE);
    let (mut routed_answered, mut overflows, mut vetoes, mut guard_drops, mut omitted) = (0, 0, 0, 0, 0);
    for case in 0..30_000 {
        let rule = random_rule(&mut rng);
        let llm_card = if rng.chance(15) { None } else { Some(random_llm(&mut rng, &rule, false)) };
        let ovr = rng.chance(6).then(|| {
            let mut roles: Vec<String> = vec![];
            for _ in 0..rng.below(5) {
                let r = roles::IDS[rng.below(26)].to_string();
                if !roles.contains(&r) {
                    roles.push(r);
                }
            }
            Assignment {
                primary: roles.first().cloned().unwrap_or_default(),
                evidence: roles
                    .iter()
                    .map(|r| {
                        let ev = Evidence {
                            source: Source::Override,
                            certain: false,
                            rule: "n".into(),
                            face: None,
                            line: None,
                        };
                        (r.clone(), ev)
                    })
                    .collect(),
                roles,
                needs_llm: false,
                route: vec![],
                llm_failed: false,
            }
        });
        let (a, dis) = do_merge(&rule, llm_card.as_ref(), ovr.as_ref());
        let ctx = format!("case {case}: rule {rule:?}\nllm {llm_card:?}\nmerged {a:?}\ndis {dis:?}");
        // deterministic
        let (a2, dis2) = do_merge(&rule, llm_card.as_ref(), ovr.as_ref());
        assert!(a == a2 && dis == dis2, "not deterministic: {ctx}");
        // an override is used exactly, with nothing logged
        if let Some(o) = &ovr {
            assert!(&a == o && dis.is_empty(), "override: {ctx}");
            continue;
        }
        // the shape of every result
        assert!(a.roles.len() <= 4, "more than four: {ctx}");
        assert_eq!(a.roles.iter().collect::<BTreeSet<_>>().len(), a.roles.len(), "repeated role: {ctx}");
        assert_eq!(a.primary, a.roles.first().cloned().unwrap_or_default(), "primary: {ctx}");
        assert_eq!(
            a.evidence.keys().collect::<BTreeSet<_>>(),
            a.roles.iter().collect::<BTreeSet<_>>(),
            "evidence: {ctx}"
        );
        assert_eq!((a.needs_llm, &a.route), (rule.needs_llm, &rule.route), "{ctx}");
        let certain: Vec<&String> = rule.roles.iter().filter(|r| rule.evidence[*r].certain).collect();
        assert!(certain.iter().all(|r| a.roles.contains(r)), "a certain role was dropped: {ctx}");
        // a card the rules decided is never changed by the LLM
        if !rule.needs_llm {
            assert!(a == rule, "rules-final card changed: {ctx}");
            assert!(dis.iter().all(|d| d.kind == Kind::Shadow), "{ctx}");
            assert_eq!(
                dis.is_empty(),
                llm_card.as_ref().is_none_or(
                    |l| l.roles.iter().collect::<BTreeSet<_>>() == rule.roles.iter().collect::<BTreeSet<_>>()
                ),
                "{ctx}"
            );
            continue;
        }
        // llm_failed exactly when routed and unanswered
        assert_eq!(a.llm_failed, llm_card.is_none(), "llm_failed: {ctx}");
        let Some(l) = &llm_card else {
            assert!(
                a.roles == rule.roles && a.evidence == rule.evidence && a.primary == rule.primary && dis.is_empty(),
                "{ctx}"
            );
            continue;
        };
        routed_answered += 1;
        // the policy, derived again from the documented rules
        let certain_set: BTreeSet<&String> = certain.iter().copied().collect();
        let passes = |r: &String| l.evidence.get(r).and_then(|n| guard_ok(r, *n)).is_some();
        let guarded: Vec<&String> = l.roles.iter().filter(|r| passes(r)).collect();
        let kept: Vec<&String> =
            rule.roles.iter().filter(|r| !certain_set.contains(r) && (l.roles.contains(r) || keeps(r))).collect();
        let mut expect: Vec<String> = vec![];
        let mut push = |r: &String| {
            if !expect.contains(r) {
                expect.push(r.clone());
            }
        };
        let pre = |r: &String| certain_set.contains(r) || guarded.contains(&r) || kept.contains(&r);
        if let Some(p) = l.primary.as_ref().filter(|p| pre(p)) {
            push(p);
        }
        for r in l.roles.iter().filter(|r| pre(r)) {
            push(r);
        }
        for r in &certain {
            push(r);
        }
        for r in &kept {
            push(r);
        }
        let pre_len = expect.len();
        let mut cut: Vec<String> = vec![];
        while expect.len() > 4 {
            let at = expect.iter().rposition(|r| !certain_set.contains(r)).expect("certain roles are at most four");
            cut.push(expect.remove(at));
        }
        assert_eq!(a.roles, expect, "{ctx}");
        assert_eq!(pre_len - a.roles.len(), cut.len(), "{ctx}");
        // evidence: the rule's own for a rule role, else LLM evidence on the cited line
        for r in &a.roles {
            let e = &a.evidence[r];
            match rule.evidence.get(r) {
                Some(re) => assert_eq!(e, re, "rule evidence replaced: {ctx}"),
                None => {
                    let line = l.evidence[r];
                    assert_eq!(
                        (e.source, e.certain, e.rule.as_str(), e.line),
                        (Source::Llm, false, "llm", Some(line)),
                        "{ctx}"
                    );
                    assert_eq!(Some(e.face), guard_ok(r, line), "{ctx}");
                }
            }
        }
        // every veto, guard drop, omitted certain role and overflow is logged exactly once, and nothing else
        let logged = |k: Kind| -> Vec<&String> { dis.iter().filter(|d| d.kind == k).map(|d| &d.role).collect() };
        let want_veto = sorted(
            rule.roles.iter().filter(|r| !certain_set.contains(r) && !l.roles.contains(r) && !keeps(r)).collect(),
        );
        let want_drop = sorted(l.roles.iter().filter(|r| !passes(r) && !rule.roles.contains(r)).collect());
        let want_omit = sorted(certain.iter().copied().filter(|r| !l.roles.contains(r)).collect());
        let cut_refs: Vec<&String> = cut.iter().collect();
        assert_eq!(sorted(logged(Kind::Veto)), want_veto, "veto: {ctx}");
        assert_eq!(sorted(logged(Kind::GuardDrop)), want_drop, "guard drop: {ctx}");
        assert_eq!(sorted(logged(Kind::CertainOmitted)), want_omit, "omitted: {ctx}");
        assert_eq!(sorted(logged(Kind::Overflow)), sorted(cut_refs), "overflow: {ctx}");
        assert!(logged(Kind::Shadow).is_empty(), "{ctx}");
        let mut pairs: Vec<(Kind, &String)> = dis.iter().map(|d| (d.kind, &d.role)).collect();
        let n = pairs.len();
        pairs.sort();
        pairs.dedup();
        assert_eq!(pairs.len(), n, "a disagreement was logged twice: {ctx}");
        overflows += cut.len();
        vetoes += want_veto.len();
        guard_drops += want_drop.len();
        omitted += want_omit.len();
        // idempotent: feeding the merged result back in with the same answer changes nothing
        let (again, _) = do_merge(&a, Some(l), None);
        assert_eq!(
            (&again.roles, &again.primary, &again.evidence),
            (&a.roles, &a.primary, &a.evidence),
            "idempotence: {ctx}"
        );
    }
    // the generator really reaches every branch
    assert!(
        routed_answered > 5000 && overflows > 100 && vetoes > 1000 && guard_drops > 1000 && omitted > 100,
        "{routed_answered} {overflows} {vetoes} {guard_drops} {omitted}"
    );
}

#[test]
fn merge_survives_hostile_llm_cards_with_core_invariants() {
    plain_variant();
    // A damaged cache row can hold what validation never lets through. `merge` must not panic and must still emit a
    // table `attach_roles` accepts: at most four unique roles, certain roles kept, primary first, evidence aligned.
    let mut rng = Rng(0xBAD5EED);
    for case in 0..30_000 {
        let rule = random_rule(&mut rng);
        let l = random_llm(&mut rng, &rule, true);
        // the guard environment would panic on an unknown role in the real merge; those are covered by the defect test
        let (a, _dis) = do_merge(&rule, Some(&l), None);
        let ctx = format!("case {case}: rule {rule:?}\nllm {l:?}\nmerged {a:?}");
        if !rule.needs_llm {
            assert!(a == rule, "{ctx}");
            continue;
        }
        assert!(a.roles.len() <= 4, "{ctx}");
        assert_eq!(a.roles.iter().collect::<BTreeSet<_>>().len(), a.roles.len(), "repeated role: {ctx}");
        assert_eq!(a.primary, a.roles.first().cloned().unwrap_or_default(), "{ctx}");
        assert_eq!(a.evidence.keys().collect::<BTreeSet<_>>(), a.roles.iter().collect::<BTreeSet<_>>(), "{ctx}");
        for r in rule.roles.iter().filter(|r| rule.evidence[*r].certain) {
            assert!(a.roles.contains(r), "certain role dropped: {ctx}");
        }
    }
}

#[test]
fn merge_edge_cases_by_hand() {
    plain_variant();
    let rule = |roles: &[(&str, bool)], routed: bool| {
        let evidence: BTreeMap<String, Evidence> =
            roles.iter().map(|(r, c)| (r.to_string(), rule_evidence(*c, 0))).collect();
        Assignment {
            roles: roles.iter().map(|(r, _)| r.to_string()).collect(),
            primary: roles.first().map(|(r, _)| r.to_string()).unwrap_or_default(),
            evidence,
            needs_llm: routed,
            route: if routed { vec![RouteReason::RiskyRole] } else { vec![] },
            llm_failed: false,
        }
    };
    let said = |roles: &[&str]| LlmCard {
        why: None,
        roles: roles.iter().map(|r| r.to_string()).collect(),
        primary: roles.first().map(|r| r.to_string()),
        evidence: roles.iter().map(|r| (r.to_string(), 0)).collect(),
    };
    // four certain roles and an LLM that wants a fifth: the fifth is the one cut, the certain four stay in rule order
    let four: Vec<(&str, bool)> = vec![("ramp", true), ("draw", true), ("tutor", true), ("removal", true)];
    let (a, dis) = do_merge(&rule(&four, true), Some(&said(&["tokens", "removal"])), None);
    // tokens is the LLM's primary and in the merge first; with five roles the last non-certain role is cut (tokens)
    assert_eq!(a.roles, ["removal", "ramp", "draw", "tutor"], "{a:?}");
    assert!(dis.iter().any(|d| d.kind == Kind::Overflow && d.role == "tokens"));
    // an empty LLM answer on a routed card with only non-certain rule roles vetoes them all
    let loose: Vec<&str> = roles::IDS.iter().copied().filter(|r| !keeps(r)).take(2).collect();
    let (a, dis) = do_merge(&rule(&[(loose[0], false), (loose[1], false)], true), Some(&said(&[])), None);
    assert!(a.roles.is_empty() && a.primary.is_empty() && dis.len() == 2 && !a.llm_failed, "{a:?}");
    // keep_rule protects a non-certain rule role from the same empty answer
    let keeper = roles::IDS.iter().copied().find(|r| keeps(r)).unwrap();
    let (a, dis) = do_merge(&rule(&[(keeper, false)], true), Some(&said(&[])), None);
    assert!(a.roles == [keeper] && dis.is_empty(), "{a:?} {dis:?}");
    // a card with nothing from the rules and an LLM role whose guard fails stays empty and logs a guard drop
    let failing_role = roles::IDS.iter().find(|r| guard_ok(r, 0).is_none()).copied().unwrap();
    let (a, dis) = do_merge(&rule(&[], true), Some(&said(&[failing_role])), None);
    assert!(a.roles.is_empty() && dis.len() == 1 && dis[0].kind == Kind::GuardDrop, "{a:?} {dis:?}");
    // an LLM primary that is not in the final set is ignored
    let mut l = said(&["removal", failing_role]);
    l.primary = Some(failing_role.to_string());
    let (a, _) = do_merge(&rule(&[], true), Some(&l), None);
    assert_eq!(a.primary, "removal", "{a:?}");
    // rules-final cards and the LLM: shadow differences only, both directions
    let (a, dis) = do_merge(&rule(&[("removal", false)], false), Some(&said(&["wipe"])), None);
    assert_eq!(a.roles, ["removal"]);
    assert_eq!(
        dis.iter().map(|d| (d.kind, d.role.as_str())).collect::<Vec<_>>(),
        [(Kind::Shadow, "removal"), (Kind::Shadow, "wipe")]
    );
}

// ============================================================================ whole pipelines on the fixture

/// What `--merge` must satisfy for every card of `index`, given the rule map (no overrides), the overrides, the cache
/// and the model. Returns the number of LLM-evidenced roles it saw.
fn check_merged(
    index: &Index,
    tax: &Taxonomy,
    rules: &roles::RoleMap,
    ovr: &BTreeMap<String, Assignment>,
    cache: &BTreeMap<String, Row>,
    model: &str,
    merged: &roles::RoleMap,
) -> usize {
    let prompt = llm::prompt_fnv(tax);
    assert_eq!(merged.len(), index.cards.iter().filter(|c| roles::is_deck_card(c)).count());
    let mut llm_roles = 0;
    for c in index.cards.iter().filter(|c| roles::is_deck_card(c)) {
        let id = &c.raw.oracle_id;
        let (a, rule) = (&merged[id], &rules[id]);
        let ctx = format!("{} {a:?} (rules {rule:?})", c.raw.name);
        assert!(a.roles.len() <= 4, "{ctx}");
        assert_eq!(a.roles.iter().collect::<BTreeSet<_>>().len(), a.roles.len(), "{ctx}");
        assert_eq!(a.primary, a.roles.first().cloned().unwrap_or_default(), "{ctx}");
        assert_eq!(a.evidence.keys().collect::<BTreeSet<_>>(), a.roles.iter().collect::<BTreeSet<_>>(), "{ctx}");
        if let Some(o) = ovr.get(id) {
            assert_eq!(a, o, "{ctx}");
            continue;
        }
        if !rule.needs_llm {
            assert_eq!(a, rule, "a rules-final card changed: {ctx}");
            continue;
        }
        assert!(a.needs_llm && a.route == rule.route, "{ctx}");
        let empty = Row {
            why: None,
            key: String::new(),
            name: String::new(),
            roles: vec![],
            primary: None,
            evidence: BTreeMap::new(),
            model: String::new(),
            taxonomy: String::new(),
            prompt: String::new(),
        };
        let answer = if roles::prompt_lines(c).is_empty() {
            Some(&empty)
        } else {
            cache.get(&llm::cache_key(&llm::card_text(c), &prompt, model))
        };
        let Some(row) = answer else {
            // only Commander-legal cards are ever sent, so only they are flagged as waiting for an answer
            assert!(a.llm_failed == (c.legal == Legal::Legal), "{ctx}");
            assert!(a.roles == rule.roles && a.evidence == rule.evidence, "{ctx}");
            continue;
        };
        assert!(!a.llm_failed, "{ctx}");
        for r in rule.roles.iter().filter(|r| rule.evidence[*r].certain) {
            assert!(a.roles.contains(r), "certain role dropped: {ctx}");
        }
        for r in &a.roles {
            let e = &a.evidence[r];
            if e.source == Source::Llm {
                llm_roles += 1;
                let line = e.line.expect("a cited line");
                assert!(row.roles.contains(r) && row.evidence.get(r) == Some(&line), "role not in the answer: {ctx}");
                assert!(!roles::skipped(c, line) && !e.certain && e.rule == "llm", "{ctx}");
                assert_eq!(e.face, c.lines()[line].0, "{ctx}");
                assert!(
                    tax.guard(r).unwrap().is_match(&roles::line_text(c, line)).unwrap(),
                    "guard fails on the cited line: {ctx}"
                );
                assert!(!rule.roles.contains(r), "a role both stages found keeps the rule's evidence: {ctx}");
            } else {
                assert_eq!(Some(e), rule.evidence.get(r), "{ctx}");
                assert!(
                    rule.evidence[r].certain || row.roles.contains(r) || tax.keep_rule(r),
                    "unconfirmed rule role kept: {ctx}"
                );
            }
        }
    }
    assert_eq!(merged.meta.prompt_fnv, prompt);
    assert_eq!(merged.meta.model, model);
    llm_roles
}

#[test]
fn the_whole_fixture_catalog_with_pseudo_random_valid_answers_keeps_every_invariant() {
    plain_variant();
    let tax = taxonomy();
    for salt in [1u64, 2, 3] {
        for only_routed in [false, true] {
            let e = Env::new();
            let srv = pseudo_srv(salt);
            let out = if only_routed { llm_all(&e, &srv, &["--only-routed"]) } else { llm_all(&e, &srv, &[]) }.unwrap();
            assert_eq!(counts(&out).1, 0, "{out}");
            let index = e.index();
            let map = e.merged();
            let rules = roles::generate(&index, "").unwrap();
            let cache = llm::load_cache(&e.dir()).unwrap();
            let seen = check_merged(&index, &tax, &rules, &BTreeMap::new(), &cache, "fake-model-1", &map);
            assert!(seen > 20, "salt {salt}: the pseudo answers must add LLM roles: {seen}");
            let mut attached = e.index();
            attached.attach_roles(&map).unwrap();
            // idempotent and byte-deterministic
            let first = std::fs::read(e.dir().join("roles.json")).unwrap();
            e.cmd(&["--merge"]).unwrap();
            assert_eq!(first, std::fs::read(e.dir().join("roles.json")).unwrap());
            // the disagreement report names every veto: the totals in merge.json add up to the report's detail lines
            let report = std::fs::read_to_string(e.dir().join("reports/roles-disagreements.txt")).unwrap();
            let details = report.split("DETAILS").nth(1).unwrap().lines().skip(1).filter(|l| !l.is_empty()).count();
            let stats: llm::MergeStats =
                serde_json::from_slice(&std::fs::read(e.dir().join("reports/merge.json")).unwrap()).unwrap();
            let total: usize = stats.disagreements.values().flat_map(|k| k.values()).sum();
            assert_eq!(details, total, "salt {salt}: report lines {details} vs counts {total}");
            assert_eq!(stats.routed, stats.with_answer + stats.llm_failed);
            if only_routed {
                assert_eq!(stats.llm_failed, 0, "every routed card was answered");
                assert!(!report.contains("| Shadow |"), "no rules-final card was sent, so no shadow rows");
            } else {
                assert!(
                    report.contains("RulesFinal | Shadow |"),
                    "rules-final cards were sent: the shadow audit logs them"
                );
            }
        }
    }
}

#[test]
fn an_empty_cache_merge_is_the_rule_table_with_llm_failed_on_every_routed_card() {
    plain_variant();
    let e = Env::new();
    let index = e.index();
    let rules = roles::generate(&index, "").unwrap();
    let map = e.merged();
    assert_eq!((map.meta.model.as_str(), map.meta.prompt_fnv.as_str()), ("", llm::prompt_fnv(&taxonomy()).as_str()));
    for c in index.cards.iter().filter(|c| roles::is_deck_card(c)) {
        let (a, r) = (&map[&c.raw.oracle_id], &rules[&c.raw.oracle_id]);
        assert_eq!((&a.roles, &a.evidence, &a.route, a.needs_llm), (&r.roles, &r.evidence, &r.route, r.needs_llm));
        // a routed card with nothing readable gets an empty answer, not llm_failed
        let nothing = roles::prompt_lines(c).is_empty();
        assert_eq!(a.llm_failed, r.needs_llm && !nothing, "{}", c.raw.name);
    }
    let mut attached = e.index();
    attached.attach_roles(&map).unwrap();
}

#[test]
fn a_routed_card_with_nothing_to_show_the_model_keeps_its_rule_roles() {
    plain_variant();
    // `merge_table` gives a routed card with no prompt lines an EMPTY answer, which vetoes every non-certain rule
    // role. If the rules found roles on such a card (from something other than a shown line), nothing ever asked
    // the model; the roles must stay. Checked on every fixture card, empty-cache merge, so nothing else interferes.
    let e = Env::new();
    let index = e.index();
    let rules = roles::generate(&index, "").unwrap();
    let map = e.merged();
    for c in index.cards.iter().filter(|c| roles::is_deck_card(c)) {
        let (a, r) = (&map[&c.raw.oracle_id], &rules[&c.raw.oracle_id]);
        if r.needs_llm && roles::prompt_lines(c).is_empty() {
            assert!(
                r.roles.is_empty() || a.roles == r.roles,
                "{}: rules {:?} merged {:?}",
                c.raw.name,
                r.roles,
                a.roles
            );
        }
    }
}

#[test]
fn guards_edited_after_caching_turn_old_rows_into_logged_guard_drops_not_errors() {
    plain_variant();
    let e = Env::new();
    let srv = Srv::start(good5());
    e.llm(&srv, &[]).unwrap();
    let index = e.index();
    let before = e.merged();
    assert!(assignment_of(&index, &before, "Skywhaler's Shot").evidence["selection"].source == Source::Llm);
    // the selection guard now matches nothing: the cached `selection` can no longer be defended
    let toml = std::fs::read_to_string(e.dir().join("roles.toml")).unwrap();
    let start = toml.find("[[role]]\nid = \"selection\"").unwrap();
    let g = start + toml[start..].find("guard = ").unwrap();
    let end = g + toml[g..].find("\n[[role]]").unwrap_or(toml.len() - g);
    let edited = format!("{}guard = 'zzzz-never-matches'\n{}", &toml[..g], &toml[end..]);
    std::fs::write(e.dir().join("roles.toml"), &edited).unwrap();
    let chats = srv.chats();
    let out = e.llm(&srv, &[]).unwrap();
    assert_eq!(srv.chats(), chats, "a guard edit leaves the cache warm: {out}");
    let after = e.merged();
    let shot = assignment_of(&index, &after, "Skywhaler's Shot");
    assert_eq!(shot.roles, ["removal"], "{shot:?}");
    let report = std::fs::read_to_string(e.dir().join("reports/roles-disagreements.txt")).unwrap();
    assert!(report.contains("GuardDrop") && report.contains("Skywhaler's Shot | selection"), "{report}");
    let mut attached = e.index();
    attached.attach_roles(&after).unwrap();
}

#[test]
fn defect_merge_and_attach_read_roles_toml_from_different_places() {
    plain_variant();
    // `roles --llm/--merge` build the prompt and check guards with <project>/data/knowledge/roles.toml, while
    // `Index::load` (staleness hash, the guards `attach_roles` applies to LLM evidence) reads the roles.toml beside
    // oracle.json. With --data-dir (or CARDGEN_KNOWLEDGE_DIR, or a worktree borrowing the main checkout's catalog)
    // these are different files. The merged table must attach in the very dir it was written to.
    let e = Env::new();
    let alt = e.0.join("alt/knowledge");
    std::fs::create_dir_all(&alt).unwrap();
    std::fs::copy(e.dir().join("oracle.json"), alt.join("oracle.json")).unwrap();
    // the catalog's own roles.toml has a selection guard that matches nothing the model could cite
    let toml = TAXONOMY.replacen("guard = '(?i)scry|surveil|", "guard = '(?i)zzzz-no-such-word|", 1);
    assert_ne!(toml, TAXONOMY);
    std::fs::write(alt.join("roles.toml"), &toml).unwrap();
    let srv = Srv::start(good5());
    let ids = e.dir().join("ids.txt");
    let run = |args: &[&str]| {
        let args: Vec<String> =
            args.iter().map(|s| s.to_string()).chain(["--data-dir".into(), alt.display().to_string()]).collect();
        roles::command(&Project::new(&e.0), &args)
    };
    run(&["--llm", "--ids", ids.to_str().unwrap(), "--endpoint", &srv.url]).unwrap();
    run(&["--merge"]).unwrap();
    let map = roles::load(&alt.join("roles.json")).unwrap();
    let mut index = Index::load(&alt.join("oracle.json")).unwrap();
    let attached = index.attach_roles(&map);
    assert!(attached.is_ok(), "the table `--merge` wrote is refused in its own directory: {attached:?}");
}

// ============================================================================ 7. the prompt

#[test]
fn the_user_message_holds_only_the_batch_cards_text_in_prompt_lines_form() {
    plain_variant();
    let e = Env::new();
    let index = e.index();
    let twelve: Vec<&str> = FIVE
        .iter()
        .copied()
        .chain([
            "Lightning Bolt",
            "Brainstorm",
            "Ad Nauseam",
            "Animate Dead",
            "Arrest",
            "Armed Response",
            "Alabaster Wall",
        ])
        .collect();
    std::fs::write(e.dir().join("ids.txt"), twelve.join("\n")).unwrap();
    let srv = pseudo_srv(5);
    e.llm(&srv, &["--batch", "5"]).unwrap();
    // the candidates in the order they are sent: by oracle id
    let mut cards: Vec<&knowledge::Card> = twelve.iter().map(|n| card_of(&index, n)).collect();
    cards.sort_by(|a, b| a.raw.oracle_id.cmp(&b.raw.oracle_id));
    let bodies = srv.bodies();
    assert_eq!(bodies.len(), 3);
    for (n, chunk) in cards.chunks(5).enumerate() {
        let shown: Vec<(String, Vec<(usize, String)>)> =
            chunk.iter().map(|c| (c.raw.type_line.clone(), roles::prompt_lines(c))).collect();
        let batch: Vec<(String, &str, &[(usize, String)])> =
            shown.iter().enumerate().map(|(i, (t, l))| (llm::batch_id(i), t.as_str(), l.as_slice())).collect();
        let expected = llm::user_message(&batch);
        let user = bodies[n]["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap();
        assert_eq!(user, expected, "batch {n}");
        // nothing but the batch: no card name, no rule role, no rule id, no gold label
        for c in chunk {
            assert!(!user.contains(&c.raw.name), "the card's own name is in its text: {}", c.raw.name);
        }
        for other in cards.iter().filter(|c| !chunk.iter().any(|x| x.raw.oracle_id == c.raw.oracle_id)) {
            let first_line = roles::prompt_lines(other).first().map(|l| l.1.clone()).unwrap_or_default();
            if first_line.len() > 25 && !chunk.iter().any(|c| roles::prompt_lines(c).iter().any(|l| l.1 == first_line))
            {
                assert!(!user.contains(&first_line), "text of a card from another batch: {first_line}");
            }
        }
        for banned in ["needs_llm", "certain", "route", "RiskyRole", "NoRole", "primary", "roles:", "evidence"] {
            assert!(!user.contains(banned), "{banned} in the user message: {user}");
        }
        for r in roles::RULES.iter().filter(|r| r.id.len() > 8) {
            assert!(!user.contains(r.id), "rule id {} in the user message", r.id);
        }
        // system prompt, the worked example as a user/assistant turn, then the real user message
        let messages = bodies[n]["messages"].as_array().unwrap();
        let roles_of: Vec<&str> = messages.iter().map(|m| m["role"].as_str().unwrap()).collect();
        assert_eq!(roles_of, ["system", "user", "assistant", "user"]);
        assert_eq!(messages[0]["content"], llm::system_prompt(&taxonomy()));
        let (example_user, example_answer) = llm::example_turn();
        assert_eq!((&messages[1]["content"], &messages[2]["content"]), (&json!(example_user), &json!(example_answer)));
    }
    // a card's own name is `~` in what the model reads
    let bolt = roles::prompt_lines(card_of(&index, "Lightning Bolt"));
    assert_eq!(bolt, [(0, "~ deals 3 damage to any target.".to_string())]);
}

#[test]
fn requests_are_byte_deterministic_across_runs_environments_and_batch_boundaries() {
    plain_variant();
    let twelve: Vec<&str> = FIVE
        .iter()
        .copied()
        .chain([
            "Lightning Bolt",
            "Brainstorm",
            "Ad Nauseam",
            "Animate Dead",
            "Arrest",
            "Armed Response",
            "Alabaster Wall",
        ])
        .collect();
    let mut runs: Vec<Vec<String>> = vec![];
    for _ in 0..2 {
        let e = Env::new();
        std::fs::write(e.dir().join("ids.txt"), twelve.join("\n")).unwrap();
        let srv = pseudo_srv(9);
        e.llm(&srv, &["--batch", "4"]).unwrap();
        runs.push(srv.raws());
    }
    assert_eq!(runs[0].len(), 3);
    assert_eq!(runs[0], runs[1], "two environments, two runs, identical request bytes");
    let tax = taxonomy();
    assert_eq!(llm::system_prompt(&tax), llm::system_prompt(&taxonomy()));
    assert_eq!(llm::prompt_fnv(&tax), llm::prompt_fnv(&taxonomy()));
    let body = |n: usize| llm::chat_body("m", &llm::system_prompt(&tax), "u", &llm::response_schema_text(&tax, n), n);
    assert_eq!(body(3), body(3));
    assert_ne!(body(3), body(4));
    // the user message and the answer examples do not depend on the order cards were requested in
    let a = llm::user_message(&[("a0".into(), "Instant", &[(0usize, "x".to_string())][..])]);
    assert_eq!(a, llm::user_message(&[("a0".into(), "Instant", &[(0usize, "x".to_string())][..])]));
}

/// Every key of every JSON object in `text`, to find a repeated key (JSON allows it; a schema consumer must not).
fn repeated_keys(text: &str) -> Vec<String> {
    let mut stack: Vec<BTreeSet<String>> = vec![];
    let mut repeated = vec![];
    let (b, mut i) = (text.as_bytes(), 0);
    while i < b.len() {
        match b[i] {
            b'{' => {
                stack.push(BTreeSet::new());
            }
            b'}' => {
                stack.pop();
            }
            b'"' => {
                let start = i + 1;
                i += 1;
                while b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                let key = &text[start..i];
                let is_key = b.get(i + 1) == Some(&b':');
                if is_key {
                    if let Some(set) = stack.last_mut() {
                        if !set.insert(key.to_string()) {
                            repeated.push(key.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    repeated
}
/// The JSON Schema subset the stage uses, checked for well-formedness.
fn schema_wellformed(s: &Value, path: &str) -> Result<(), String> {
    let obj = s.as_object().ok_or(format!("{path}: a schema must be an object"))?;
    const KNOWN: [&str; 11] = [
        "type",
        "properties",
        "required",
        "additionalProperties",
        "enum",
        "pattern",
        "minItems",
        "maxItems",
        "items",
        "name",
        "minimum",
    ];
    if let Some(k) = obj.keys().find(|k| !KNOWN.contains(&k.as_str())) {
        return Err(format!("{path}: unknown keyword {k}"));
    }
    if let Some(t) = obj.get("type") {
        if !matches!(t.as_str(), Some("object" | "array" | "string" | "integer")) {
            return Err(format!("{path}: bad type {t}"));
        }
    }
    if let Some(e) = obj.get("enum") {
        let a = e.as_array().filter(|a| !a.is_empty()).ok_or(format!("{path}: empty enum"))?;
        if a.iter().map(|v| v.to_string()).collect::<BTreeSet<_>>().len() != a.len() {
            return Err(format!("{path}: repeated enum value"));
        }
    }
    if let Some(p) = obj.get("pattern") {
        fancy_regex::Regex::new(p.as_str().ok_or("pattern is not a string")?).map_err(|e| format!("{path}: {e}"))?;
    }
    if let (Some(lo), Some(hi)) =
        (obj.get("minItems").and_then(Value::as_u64), obj.get("maxItems").and_then(Value::as_u64))
    {
        if lo > hi {
            return Err(format!("{path}: minItems above maxItems"));
        }
    }
    if let Some(props) = obj.get("properties") {
        for (k, v) in props.as_object().ok_or(format!("{path}: properties is not an object"))? {
            schema_wellformed(v, &format!("{path}.{k}"))?;
        }
        if let Some(req) = obj.get("required") {
            for r in req.as_array().ok_or(format!("{path}: required is not an array"))? {
                if !props.as_object().unwrap().contains_key(r.as_str().unwrap_or("")) {
                    return Err(format!("{path}: required {r} is not a property"));
                }
            }
        }
    }
    if let Some(items) = obj.get("items") {
        schema_wellformed(items, &format!("{path}[]"))?;
    }
    Ok(())
}
/// Validate `v` against the schema subset.
fn schema_accepts(s: &Value, v: &Value, path: &str) -> Result<(), String> {
    if let Some(e) = s.get("enum") {
        return if e.as_array().unwrap().contains(v) { Ok(()) } else { Err(format!("{path}: {v} not in the enum")) };
    }
    match s.get("type").and_then(Value::as_str) {
        Some("object") => {
            let o = v.as_object().ok_or(format!("{path}: not an object"))?;
            let props = s["properties"].as_object().unwrap();
            for r in s.get("required").and_then(Value::as_array).into_iter().flatten() {
                if !o.contains_key(r.as_str().unwrap()) {
                    return Err(format!("{path}: missing {r}"));
                }
            }
            for (k, x) in o {
                match props.get(k) {
                    Some(sub) => schema_accepts(sub, x, &format!("{path}.{k}"))?,
                    None if s.get("additionalProperties") == Some(&json!(false)) => {
                        return Err(format!("{path}: extra {k}"));
                    }
                    None => {}
                }
            }
            Ok(())
        }
        Some("array") => {
            let a = v.as_array().ok_or(format!("{path}: not an array"))?;
            if s.get("minItems").and_then(Value::as_u64).is_some_and(|n| (a.len() as u64) < n)
                || s.get("maxItems").and_then(Value::as_u64).is_some_and(|n| (a.len() as u64) > n)
            {
                return Err(format!("{path}: {} items", a.len()));
            }
            a.iter().enumerate().try_for_each(|(i, x)| schema_accepts(&s["items"], x, &format!("{path}[{i}]")))
        }
        Some("string") => {
            let t = v.as_str().ok_or(format!("{path}: not a string"))?;
            match s.get("pattern").and_then(Value::as_str) {
                Some(p) if !fancy_regex::Regex::new(p).unwrap().is_match(t).unwrap() => {
                    Err(format!("{path}: {t:?} !~ {p}"))
                }
                _ => Ok(()),
            }
        }
        Some("integer") => {
            if !(v.is_i64() || v.is_u64()) {
                return Err(format!("{path}: not an integer"));
            }
            match s.get("minimum").and_then(Value::as_i64) {
                Some(m) if v.as_i64().is_some_and(|x| x < m) => Err(format!("{path}: below the minimum {m}")),
                _ => Ok(()),
            }
        }
        _ => Ok(()),
    }
}

#[test]
fn the_json_schema_is_well_formed_has_no_repeated_keys_and_accepts_exactly_the_shape_of_an_answer() {
    plain_variant();
    let tax = taxonomy();
    for n in [1usize, 2, 6, 10, 11, 260] {
        let text = llm::response_schema_text(&tax, n);
        assert!(repeated_keys(&text).is_empty(), "repeated keys in {text}");
        let schema: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(schema, llm::response_schema(&tax, n));
        schema_wellformed(&schema, "$").unwrap();
        let cards = &schema["properties"]["cards"];
        assert_eq!((&cards["minItems"], &cards["maxItems"]), (&json!(n), &json!(n)));
        assert_eq!(
            cards["items"]["properties"]["id"]["enum"],
            json!((0..n).map(llm::batch_id).collect::<Vec<_>>()),
            "ids are an enum of the batch's ids"
        );
        let enumerated: Vec<&str> = cards["items"]["properties"]["roles"]["items"]["properties"]["role"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(enumerated, roles::IDS, "the grammar's role enum is the taxonomy, in order");
        let item = &cards["items"];
        assert_eq!(item["required"], json!(["id", "roles"]));
        assert_eq!(item["properties"]["roles"]["maxItems"], 4);
        let pair = &item["properties"]["roles"]["items"];
        assert_eq!(pair["required"], json!(["role", "line"]));
        assert_eq!(pair["properties"]["line"], json!({"type": "integer", "minimum": 0}));
        assert!(item["properties"].get("primary").is_none() && item["properties"].get("evidence").is_none());
        // a valid answer of n entries passes; so does one with several roles (any role may be cited)
        let both = |i: usize| entry(&llm::batch_id(i), &["removal", "draw"], &[("removal", 3), ("draw", 0)]);
        let answer = json!({"cards": (0..n).map(both).collect::<Vec<_>>()});
        schema_accepts(&schema, &answer, "$").unwrap();
        let none = json!({"cards": (0..n).map(|i| entry(&llm::batch_id(i), &[], &[])).collect::<Vec<_>>()});
        schema_accepts(&schema, &none, "$").unwrap();
        // and what the grammar must refuse
        let mut bad: Vec<(&str, Value)> = vec![];
        let mut one_less = answer.clone();
        one_less["cards"].as_array_mut().unwrap().pop();
        bad.push(("one entry fewer", one_less));
        let mut one_more = answer.clone();
        let extra = one_more["cards"][0].clone();
        one_more["cards"].as_array_mut().unwrap().push(extra);
        bad.push(("one entry more", one_more));
        let schema_edits: Vec<(&str, fn(&mut Value))> = vec![
            ("id outside the enum", |c: &mut Value| c["id"] = json!("A0")),
            ("id too long", |c: &mut Value| c["id"] = json!("a00")),
            ("id not in the batch", |c: &mut Value| c["id"] = json!("?!")),
            ("role outside the enum", |c: &mut Value| c["roles"] = json!([{"role": "banana", "line": 0}])),
            ("bare role strings", |c: &mut Value| c["roles"] = json!(["removal"])),
            ("five roles", |c: &mut Value| {
                let roles = ["ramp", "draw", "tutor", "removal", "wipe"];
                c["roles"] = json!(roles.map(|r| json!({"role": r, "line": 0})));
            }),
            ("missing roles", |c: &mut Value| drop(c.as_object_mut().unwrap().remove("roles"))),
            ("extra key", |c: &mut Value| c["extra"] = json!(1)),
            ("the old primary key", |c: &mut Value| c["primary"] = json!("removal")),
            ("the old evidence key", |c: &mut Value| c["evidence"] = json!({"removal": 1})),
            ("line missing", |c: &mut Value| drop(c["roles"][0].as_object_mut().unwrap().remove("line"))),
            ("line negative", |c: &mut Value| c["roles"][0]["line"] = json!(-1)),
            ("line a float", |c: &mut Value| c["roles"][0]["line"] = json!(1.5)),
            ("line a string", |c: &mut Value| c["roles"][0]["line"] = json!("1")),
            ("extra key in a pair", |c: &mut Value| c["roles"][0]["why"] = json!("because")),
        ];
        for (name, edit) in schema_edits {
            let mut v = answer.clone();
            edit(&mut v["cards"][0]);
            bad.push((name, v));
        }
        for (name, v) in bad {
            assert!(schema_accepts(&schema, &v, "$").is_err(), "n={n}: the grammar would allow: {name}");
        }
    }
    // the examples' own answer under the schema for six cards, and with the exact text the system prompt carries
    let lines: Vec<Vec<(usize, String)>> =
        llm::EXAMPLES.iter().map(|e| e.lines.iter().map(|(i, t)| (*i, t.to_string())).collect()).collect();
    let entries: Vec<String> = llm::EXAMPLES
        .iter()
        .enumerate()
        .map(|(i, e)| llm::answer_entry(&llm::batch_id(i), e.roles, e.evidence))
        .collect();
    let answer: Value = serde_json::from_str(&format!("{{\"cards\":[{}]}}", entries.join(","))).unwrap();
    schema_accepts(&llm::response_schema(&tax, llm::EXAMPLES.len()), &answer, "$").unwrap();
    let _ = lines;
}

#[test]
fn the_worked_examples_teach_the_exact_answer_format_and_nothing_else() {
    plain_variant();
    let tax = taxonomy();
    let system = llm::system_prompt(&tax);
    // the examples are a real user/assistant turn pair, no longer text inside the system prompt
    assert!(!system.contains("WORKED EXAMPLE") && !system.contains("Classify these"));
    let (example_user, answer_line) = llm::example_turn();
    let parsed: Value = serde_json::from_str(&answer_line).unwrap();
    let entries = parsed["cards"].as_array().unwrap();
    assert_eq!(entries.len(), llm::EXAMPLES.len());
    assert_eq!(llm::EXAMPLES.len(), 10);
    // keys in the schema's order, ids a0.., roles as {role, line} pairs, `[]` when empty
    for (i, (e, ex)) in entries.iter().zip(llm::EXAMPLES.iter()).enumerate() {
        assert_eq!(e["id"], llm::batch_id(i).as_str());
        assert_eq!(e.as_object().unwrap().len(), 2, "{}: only id and roles", ex.name);
        let pairs: Vec<(&str, usize)> = e["roles"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (r["role"].as_str().unwrap(), r["line"].as_u64().unwrap() as usize))
            .collect();
        assert_eq!(pairs.iter().map(|p| p.0).collect::<Vec<_>>(), ex.roles);
        for (r, line) in &pairs {
            let (_, text) = ex.lines.iter().find(|(n, _)| n == line).expect("the cited line exists in the example");
            assert!(tax.guard(r).unwrap().is_match(text).unwrap(), "{}: {r} on {text:?}", ex.name);
        }
    }
    assert!(answer_line.starts_with("{\"cards\":[{\"id\":\"a0\",\"roles\":["));
    for i in 0..entries.len() {
        let at = answer_line.find(&format!("{{\"id\":\"{}\"", llm::batch_id(i))).unwrap();
        assert!(answer_line[at..].find("\"roles\"").unwrap() < 20);
    }
    // two examples carry two roles each, so the format and the order conventions (copy first, tokens and ramp) are seen
    let two: Vec<Vec<&str>> = llm::EXAMPLES.iter().filter(|e| e.roles.len() == 2).map(|e| e.roles.to_vec()).collect();
    assert_eq!(two, [vec!["tokens", "ramp"], vec!["copy", "tokens"]]);
    // one example has three roles: the model must not stop after the first
    assert!(llm::EXAMPLES.iter().any(|e| e.roles == ["tokens", "draw", "counters"]));
    // the example user message says "10 cards" and numbers them like a real batch does
    assert!(
        example_user.starts_with("Classify these 10 cards.\n[a0] Instant\n0: Destroy target attacking creature."),
        "{example_user}"
    );
    // the examples stay one-line answers: no markdown, no prose around the JSON, no card names
    assert!(!answer_line.contains("```") && !answer_line.contains('\n'));
    for ex in &llm::EXAMPLES {
        assert!(!system.contains(ex.name), "the example card name {} is in the prompt", ex.name);
        assert!(!example_user.contains(ex.name) && !answer_line.contains(ex.name), "{}", ex.name);
    }
    // the examples include both a card with a role and cards without one, a cost-side line and a rider
    let empties = entries.iter().filter(|e| e["roles"].as_array().unwrap().is_empty()).count();
    assert!(empties >= 2 && empties < entries.len());
    // a 10-card real batch uses the same layout as the example (header, [id] type, numbered lines)
    let user = llm::user_message(&[("a0".into(), "Instant", &[(0usize, "Counter target spell.".to_string())][..])]);
    assert_eq!(user, "Classify these 1 cards.\n[a0] Instant\n0: Counter target spell.\n");
}

#[test]
fn the_system_prompt_stays_well_under_the_context_and_the_request_has_room_for_the_answer() {
    plain_variant();
    let tax = taxonomy();
    let system = llm::system_prompt(&tax);
    let tokens = llm::estimate_tokens(&system);
    assert!(tokens < 5000, "{tokens}");
    // ten cards at the 2048-byte line cap, four lines each, is the worst case a batch can carry
    let worst_user = 10 * 4 * (2048 / 4 + 8);
    assert!(tokens + worst_user + 160 * 10 + 64 < 32_000, "{tokens} + {worst_user}");
    let body = llm::chat_body("m", &system, "u", &llm::response_schema_text(&tax, 10), 10);
    let v: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["max_tokens"], 160 * 10 + 64);
    assert!(v["response_format"]["json_schema"]["strict"] == json!(true));
    // the answer for ten cards with four roles each fits the token limit by the same chars / 4 estimate
    let entry_text = llm::answer_entry(
        "a0",
        &["removal", "wipe", "tokens", "anthem"],
        &[("removal", 3), ("wipe", 3), ("tokens", 0), ("anthem", 12)],
    );
    assert!(llm::estimate_tokens(&entry_text) * 10 < 160 * 10 + 64, "{}", entry_text.len());
}

// ============================================================================ 8. staleness and metadata

fn tampered(e: &Env, edit: impl Fn(&mut roles::RoleMap, &Index)) -> Result<(), String> {
    let map = e.merged();
    let index = e.index();
    let mut map = map;
    edit(&mut map, &index);
    // written through `bytes`, so the table hash is right and only the semantics are wrong
    let path = e.dir().join("roles-tampered.json");
    std::fs::write(&path, roles::bytes(&map).unwrap()).unwrap();
    let loaded = roles::load(&path)?;
    let mut index = e.index();
    index.attach_roles(&loaded)
}

#[test]
fn the_merged_table_records_its_prompt_model_and_hashes_and_a_hand_edit_is_refused() {
    plain_variant();
    let e = Env::new();
    let srv = Srv::start(good5());
    e.llm(&srv, &[]).unwrap();
    let map = e.merged();
    let index = e.index();
    let tax = taxonomy();
    assert_eq!(map.meta.prompt_fnv, llm::prompt_fnv(&tax));
    assert_eq!(map.meta.model, "fake-model-1");
    assert_eq!((map.meta.taxonomy.as_str(), map.meta.count), (index.taxonomy_fnv.as_str(), map.len()));
    assert_eq!(map.meta.table_fnv.len(), 16);
    roles::check_file(&e.dir().join("roles.json"), &index).unwrap();
    // a hand edit of one role, one flag, one line number: damaged for `load`, `check_file`, `--show`, `--stats`, find
    let path = e.dir().join("roles.json");
    let good = std::fs::read_to_string(&path).unwrap();
    for (name, from, to) in [
        ("a role", "\"selection\"", "\"removal\""),
        ("a flag", "\"needs_llm\": true", "\"needs_llm\": false"),
        ("a source", "\"source\": \"Llm\"", "\"source\": \"Rule\""),
        ("a certain flag", "\"certain\": false", "\"certain\": true"),
        ("a line number", "\"line\": 0", "\"line\": 1"),
    ] {
        let edited = good.replacen(from, to, 1);
        assert_ne!(edited, good, "{name}");
        std::fs::write(&path, &edited).unwrap();
        assert!(roles::load(&path).unwrap_err().contains("damaged"), "{name}");
        assert!(roles::check_file(&path, &index).is_err(), "{name}");
        assert!(e.cmd(&["--show", "Counterspell"]).is_err(), "{name}");
        assert!(e.cmd(&["--stats"]).is_err(), "{name}");
        let found = knowledge::find(
            &Project::new(&e.0),
            &["--role".into(), "counter".into(), "--data-dir".into(), e.dir().display().to_string()],
        );
        assert!(found.is_err(), "{name}: find --role must refuse a damaged table");
    }
    // a truncated table, an empty file and a leading BOM
    for text in [&good[..good.len() / 2], "", &format!("\u{feff}{good}")] {
        std::fs::write(&path, text).unwrap();
        assert!(roles::load(&path).is_err());
        assert!(roles::check_file(&path, &index).is_err());
    }
    // a stale table: a changed definition changes the taxonomy hash, so the old merged table no longer attaches
    std::fs::write(&path, &good).unwrap();
    let toml = std::fs::read_to_string(e.dir().join("roles.toml")).unwrap();
    std::fs::write(
        e.dir().join("roles.toml"),
        toml.replacen("counters a spell or an ability on the stack.", "counters a spell.", 1),
    )
    .unwrap();
    let mut stale = e.index();
    let err = stale.attach_roles(&roles::load(&path).unwrap()).unwrap_err();
    assert!(err.contains("stale"), "{err}");
    assert!(e.cmd(&["--stats"]).is_err());
    std::fs::write(e.dir().join("roles.toml"), &toml).unwrap();
    // a changed Oracle text makes it stale too
    let mut oracle: Value =
        serde_json::from_str(&std::fs::read_to_string(e.dir().join("oracle.json")).unwrap()).unwrap();
    oracle["cards"][0]["oracle_text"] = json!("Draw seven cards.");
    std::fs::write(e.dir().join("oracle.json"), oracle.to_string()).unwrap();
    assert!(e.cmd(&["--stats"]).is_err());
}

#[test]
fn llm_evidence_that_lies_is_refused_by_attach_roles_even_with_a_correct_table_hash() {
    plain_variant();
    let e = Env::new();
    let srv = Srv::start(good5());
    e.llm(&srv, &[]).unwrap();
    assert!(tampered(&e, |_, _| {}).is_ok(), "the untouched merged table attaches");
    type Edit = fn(&mut Assignment);
    let edits: Vec<(&str, Edit)> = vec![
        ("a skipped line", |_| {}),
        ("certain", |a| a.evidence.get_mut("selection").unwrap().certain = true),
        ("a rule that is not llm", |a| a.evidence.get_mut("selection").unwrap().rule = "single-removal".into()),
        ("no line", |a| a.evidence.get_mut("selection").unwrap().line = None),
        ("a line out of range", |a| a.evidence.get_mut("selection").unwrap().line = Some(40)),
        ("the wrong face", |a| a.evidence.get_mut("selection").unwrap().face = Some(1)),
        ("a role whose guard fails on the cited line", |a| {
            let ev = a.evidence.remove("selection").unwrap();
            a.evidence.insert("counter".into(), ev);
            a.roles[1] = "counter".into();
        }),
        ("a role outside the vocabulary", |a| {
            let ev = a.evidence.remove("selection").unwrap();
            a.evidence.insert("banana".into(), ev);
            a.roles[1] = "banana".into();
        }),
        ("override source with a line", |a| a.evidence.get_mut("selection").unwrap().source = Source::Override),
        ("five roles", |a| {
            for r in ["ramp", "draw", "tutor"] {
                a.roles.push(r.into());
                a.evidence.insert(r.into(), a.evidence["removal"].clone());
            }
        }),
        ("a repeated role", |a| {
            a.roles.push("selection".into());
        }),
        ("a primary that is not the first role", |a| a.primary = "selection".into()),
        ("evidence missing for a role", |a| drop(a.evidence.remove("selection"))),
        ("evidence for a role not listed", |a| {
            a.evidence.insert("draw".into(), a.evidence["selection"].clone());
        }),
    ];
    for (name, edit) in edits {
        let result = tampered(&e, |m, i| {
            let id = card_of(i, "Skywhaler's Shot").raw.oracle_id.clone();
            let a = m.get_mut(&id).unwrap();
            assert_eq!(a.roles, ["removal", "selection"], "{a:?}");
            edit(a);
        });
        if name == "a skipped line" {
            assert!(result.is_ok());
            continue;
        }
        assert!(result.is_err(), "attach_roles accepted: {name}");
    }
    // the same on a card whose cited line is a skipped one (Elspeth's -7 ultimate is line 2)
    let result = tampered(&e, |m, i| {
        let c = card_of(i, "Elspeth, Sun's Champion");
        assert!(roles::skipped(c, 2));
        let a = m.get_mut(&c.raw.oracle_id).unwrap();
        a.roles = vec!["anthem".into()];
        a.primary = "anthem".into();
        a.evidence = [(
            "anthem".to_string(),
            Evidence { source: Source::Llm, certain: false, rule: "llm".into(), face: None, line: Some(2) },
        )]
        .into();
    });
    assert!(result.is_err(), "an LLM role cited on a planeswalker ultimate was attached");
    // ... and on an MDFC land back
    let result = tampered(&e, |m, i| {
        let c = card_of(i, "Agadeem's Awakening // Agadeem, the Undercrypt");
        let tax = taxonomy();
        // a skipped line (the land back) and a role whose guard would pass on it: only the "not shown" check can refuse
        let (back, role) = (0..c.lines().len())
            .filter(|n| roles::skipped(c, *n))
            .find_map(|n| {
                let text = roles::line_text(c, n);
                roles::IDS.iter().find(|r| tax.guard(r).unwrap().is_match(&text).unwrap()).map(|r| (n, *r))
            })
            .expect("a skipped line of the MDFC that a guard matches");
        let a = m.get_mut(&c.raw.oracle_id).unwrap();
        a.roles = vec![role.into()];
        a.primary = role.into();
        a.evidence = [(
            role.to_string(),
            Evidence {
                source: Source::Llm,
                certain: false,
                rule: "llm".into(),
                face: c.lines()[back].0,
                line: Some(back),
            },
        )]
        .into();
    });
    assert!(result.is_err(), "an LLM role cited on an MDFC land back was attached");
}

#[test]
fn defect_hand_edited_prompt_and_model_in_the_table_metadata_are_not_noticed() {
    plain_variant();
    // `table_fnv` covers the card table only, and `attach_roles` / `--validate --stage merged` never compare
    // `meta.prompt_fnv` or `meta.model` with anything. Editing the two metadata lines (no hash to repair) turns a
    // rules-only table into a "merged" one, and a merged table built with another prompt is still current.
    let e = Env::new();
    e.cmd(&["--rules"]).unwrap();
    let path = e.dir().join("roles.json");
    let rules_only = std::fs::read_to_string(&path).unwrap();
    assert!(rules_only.contains("\"prompt_fnv\": \"\""));
    let gold = {
        let i = e.index();
        format!(
            "[[card]]\nname='Counterspell'\noracle_id='{}'\nroles=['counter']\nprimary='counter'\n",
            card_of(&i, "Counterspell").raw.oracle_id
        )
    };
    std::fs::write(e.0.join("gold.toml"), gold).unwrap();
    let gold_path = e.0.join("gold.toml");
    let args = ["--validate", "--gold", gold_path.to_str().unwrap(), "--stage", "merged"];
    assert!(e.cmd(&args).unwrap_err().contains("--merge"), "a rules-only table is not a merged one");
    std::fs::write(&path, rules_only.replacen("\"prompt_fnv\": \"\"", "\"prompt_fnv\": \"0123456789abcdef\"", 1))
        .unwrap();
    let result = e.cmd(&args);
    assert!(result.is_err(), "a rules-only table passed as merged after editing one metadata line");
}

// ============================================================================ validate --stage merged arithmetic

fn f1(tp: usize, fp: usize, missed: usize) -> f64 {
    let denom = 2 * tp + fp + missed;
    if denom == 0 { 1.0 } else { 2.0 * tp as f64 / denom as f64 }
}
fn gold_toml(index: &Index, cards: &[(&str, &[&str])]) -> String {
    cards
        .iter()
        .map(|(name, roles)| {
            let list = roles.iter().map(|r| format!("'{r}'")).collect::<Vec<_>>().join(",");
            format!(
                "[[card]]\nname=\"{name}\"\noracle_id='{}'\nroles=[{list}]\nprimary='{}'\n\n",
                card_of(index, name).raw.oracle_id,
                roles.first().unwrap_or(&"")
            )
        })
        .collect()
}
/// Run the whole stage on crafted gold and return the JSON report.
fn staged(e: &Env, srv: &Srv, gold: &[(&str, &[&str])]) -> Value {
    e.llm(srv, &[]).unwrap();
    e.cmd(&["--merge"]).unwrap();
    let path = e.0.join("gold.toml");
    std::fs::write(&path, gold_toml(&e.index(), gold)).unwrap();
    let (text, _) = e.cmd(&["--validate", "--gold", path.to_str().unwrap(), "--stage", "merged", "--json"]).unwrap();
    serde_json::from_str(&text).unwrap()
}
fn score(v: &Value, key: &str) -> (usize, usize, usize) {
    let n = |f: &str| v[key][f].as_u64().unwrap() as usize;
    (n("tp"), n("fp"), n("missed"))
}

#[test]
fn validate_stage_merged_arithmetic_on_crafted_gold_files() {
    plain_variant();
    // gold labels chosen against the fake model's answers (Skywhaler's Shot: removal + selection; Counterspell:
    // counter; Agadeem's: reanimate; Elspeth: tokens + wipe; Ugin's Mastery: none)
    let e = Env::new();
    let srv = Srv::start(good5());
    let gold: &[(&str, &[&str])] = &[
        ("Skywhaler's Shot", &["removal"]), // merged adds `selection`: one false positive
        ("Counterspell", &["counter"]),     // exact
        ("Agadeem's Awakening // Agadeem, the Undercrypt", &["reanimate", "recursion"]), // `recursion` missed
        ("Elspeth, Sun's Champion", &["tokens", "wipe", "anthem"]), // `anthem` missed (its -7 is an ultimate)
        ("Ugin's Mastery", &[]),
    ];
    let v = staged(&e, &srv, gold);
    let index = e.index();
    let rules = roles::generate(&index, "").unwrap();
    let merged = roles::load(&e.dir().join("roles.json")).unwrap();
    // recompute everything independently from the two maps and the gold labels
    let (mut all, mut routed_rules, mut routed_merged, mut routed_n, mut primary_ok) =
        ((0, 0, 0), (0, 0, 0), (0, 0, 0), 0usize, 0usize);
    for (name, labels) in gold {
        let id = &card_of(&index, name).raw.oracle_id;
        let routed = rules[id].needs_llm;
        routed_n += usize::from(routed);
        primary_ok += usize::from(merged[id].primary == labels.first().copied().unwrap_or(""));
        for role in roles::IDS {
            let want = labels.contains(&role);
            let got = merged[id].roles.iter().any(|r| r == role);
            let tally = |t: &mut (usize, usize, usize), got: bool| match (want, got) {
                (true, true) => t.0 += 1,
                (false, true) => t.1 += 1,
                (true, false) => t.2 += 1,
                _ => {}
            };
            tally(&mut all, got);
            if routed {
                tally(&mut routed_merged, got);
                tally(&mut routed_rules, rules[id].roles.iter().any(|r| r == role));
            }
        }
    }
    assert_eq!(score(&v, "gold_micro"), all, "{v}");
    assert_eq!(score(&v, "routed_scored_micro"), routed_merged);
    assert_eq!(score(&v, "routed_rules_micro"), routed_rules);
    assert_eq!(v["routed_cards"].as_u64().unwrap() as usize, routed_n);
    assert_eq!(v["primary_total"], 5);
    assert_eq!(v["primary_correct"].as_u64().unwrap() as usize, primary_ok);
    assert_eq!(v["stage"], "merged");
    // hand-derived: the fake model's answers add exactly one false positive (selection) and miss recursion and anthem
    assert_eq!(all, (5, 1, 2), "tp removal, counter, reanimate, tokens, wipe; fp selection; missed recursion, anthem");
    // the gate: merged F1 on the routed gold cards below the rules alone fails the stage
    let below =
        f1(routed_merged.0, routed_merged.1, routed_merged.2) < f1(routed_rules.0, routed_rules.1, routed_rules.2);
    let failures: Vec<String> =
        v["failures"].as_array().unwrap().iter().map(|f| f.as_str().unwrap().to_string()).collect();
    assert_eq!(failures.iter().any(|f| f.contains("merged F1")), below, "{failures:?}");
    assert!(below, "F1 falls from 1.0 to 0.667 when a false positive is added: the gate must fire");
    // a model that only ever says "no roles" vetoes every non-certain rule role: with certain gold roles lost the
    // merged F1 on the routed cards cannot beat the rules, and a model that agrees with the rules cannot be below
    let e2 = Env::new();
    let srv2 = Srv::start(Box::new(|_, body| {
        let items = parse_batch(body);
        let answers: Vec<Value> = items.iter().map(|i| entry(&i.id, &[], &[])).collect();
        Wire::Content(json!({"cards": answers}).to_string())
    }));
    let v2 = staged(&e2, &srv2, gold);
    let (rr, rm) = (score(&v2, "routed_rules_micro"), score(&v2, "routed_scored_micro"));
    // hand-derived: the rules find Skywhaler's Shot's removal; an empty answer vetoes nothing now (add-only merge)
    assert_eq!((rr, rm), ((1, 0, 0), (1, 0, 0)));
    assert_eq!((routed_rules, routed_merged), ((1, 0, 0), (1, 1, 0)), "the sensible model adds only `selection`");
    assert_eq!(rm.1, 0, "an empty answer adds no false positive");
    assert!(rm.0 <= rr.0, "an empty answer cannot find more than the rules: {rm:?} vs {rr:?}");
    let failures: Vec<String> =
        v2["failures"].as_array().unwrap().iter().map(|f| f.as_str().unwrap().to_string()).collect();
    assert_eq!(
        failures.iter().any(|f| f.contains("merged F1")),
        f1(rm.0, rm.1, rm.2) < f1(rr.0, rr.1, rr.2),
        "{failures:?}"
    );
    // gold with a card that is not in the catalog, a duplicate card, and inconsistent labels are errors, not scores
    let path = e.0.join("gold2.toml");
    for (name, text) in [
        ("an absent card", "[[card]]\nname='X'\noracle_id='nope'\nroles=['draw']\nprimary='draw'\n".to_string()),
        ("an empty set", "card = []\n".to_string()),
        (
            "primary not first",
            gold_toml(&index, &[("Counterspell", &["counter"])]).replace("primary='counter'", "primary='draw'"),
        ),
        ("a duplicate card", gold_toml(&index, &[("Counterspell", &["counter"]), ("Counterspell", &["counter"])])),
        ("five roles", gold_toml(&index, &[("Counterspell", &["counter", "draw", "ramp", "tutor", "wipe"])])),
    ] {
        std::fs::write(&path, text).unwrap();
        assert!(e.cmd(&["--validate", "--gold", path.to_str().unwrap(), "--stage", "merged"]).is_err(), "{name}");
    }
}

#[test]
fn validate_stage_merged_leaves_override_cards_out_and_never_counts_them() {
    plain_variant();
    let e = Env::new();
    let srv = Srv::start(good5());
    e.llm(&srv, &[]).unwrap();
    std::fs::write(
        e.dir().join("roles_overrides.toml"),
        "[[card]]\nname=\"Skywhaler's Shot\"\nroles=['removal','selection']\nnote='n'\n",
    )
    .unwrap();
    e.cmd(&["--merge"]).unwrap();
    let gold = e.0.join("gold.toml");
    std::fs::write(
        &gold,
        gold_toml(&e.index(), &[("Skywhaler's Shot", &["removal", "selection"]), ("Counterspell", &["counter"])]),
    )
    .unwrap();
    let (text, _) = e.cmd(&["--validate", "--gold", gold.to_str().unwrap(), "--stage", "merged", "--json"]).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["primary_total"], 1, "the overridden gold card is not scored");
    assert_eq!(score(&v, "gold_micro"), (1, 0, 0));
}

// =========================================================================== 6. the pipeline on the real catalog

const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../..");
const NEEDS_CATALOG: &str = "needs data/knowledge/oracle.json (run cardgen bulk-sync)";

/// A scratch copy of the real knowledge directory (catalog, taxonomy, overrides, lock): everything the stage writes
/// goes there, never into the repository.
struct RealEnv(PathBuf);
impl RealEnv {
    fn new() -> Self {
        plain_variant();
        let real = Path::new(REPO).join("data/knowledge");
        assert!(real.join("oracle.json").exists(), "{NEEDS_CATALOG}");
        let root = std::env::temp_dir().join(format!(
            "roles-llm-real-{}-{}",
            std::process::id(),
            ENV_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let dir = root.join("knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        for f in ["oracle.json", "roles.toml", "roles_overrides.toml", "sync.lock"] {
            if real.join(f).exists() {
                std::fs::copy(real.join(f), dir.join(f)).unwrap();
            }
        }
        Self(root)
    }
    fn dir(&self) -> PathBuf {
        self.0.join("knowledge")
    }
    fn cmd(&self, args: &[&str]) -> Result<(String, bool), String> {
        let args: Vec<String> =
            args.iter().map(|s| s.to_string()).chain(["--data-dir".into(), self.dir().display().to_string()]).collect();
        roles::command(&Project::new(REPO), &args)
    }
}
impl Drop for RealEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn real_gold() -> Vec<(String, String, Vec<String>, String)> {
    let text = std::fs::read_to_string(Path::new(REPO).join("data/knowledge/roles_gold.toml")).unwrap();
    let v: toml::Value = toml::from_str(&text).unwrap();
    v["card"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["name"].as_str().unwrap().to_string(),
                c["oracle_id"].as_str().unwrap().to_string(),
                c["roles"].as_array().unwrap().iter().map(|r| r.as_str().unwrap().to_string()).collect(),
                c["primary"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_catalog_pipeline_with_2000_cards_of_pseudo_random_valid_answers_keeps_every_invariant() {
    plain_variant();
    let started = Instant::now();
    let e = RealEnv::new();
    let tax = taxonomy();
    let index = Index::load(&e.dir().join("oracle.json")).unwrap();
    let ovr_text = roles::read_overrides(&e.dir().join("oracle.json")).unwrap();
    let rules = roles::generate(&index, "").unwrap();
    let ovr = roles::overrides(&index, &ovr_text).unwrap();
    // a sample: 1,500 routed cards, 500 rules-final cards (the shadow audit) and every tuning gold card
    let ok = |c: &&knowledge::Card| {
        roles::is_deck_card(c)
            && c.legal == Legal::Legal
            && !ovr.contains_key(&c.raw.oracle_id)
            && !roles::prompt_lines(c).is_empty()
    };
    let by_hash = |c: &&knowledge::Card| text_hash(&c.raw.oracle_id);
    let mut routed: Vec<&knowledge::Card> =
        index.cards.iter().filter(ok).filter(|c| rules[&c.raw.oracle_id].needs_llm).collect();
    let mut final_: Vec<&knowledge::Card> =
        index.cards.iter().filter(ok).filter(|c| !rules[&c.raw.oracle_id].needs_llm).collect();
    routed.sort_by_key(by_hash);
    final_.sort_by_key(by_hash);
    let mut ids: BTreeSet<String> =
        routed.iter().take(1500).chain(final_.iter().take(500)).map(|c| c.raw.oracle_id.clone()).collect();
    let gold = real_gold();
    for (_, id, _, _) in &gold {
        ids.insert(id.clone());
    }
    let ids_file = e.0.join("ids.txt");
    std::fs::write(&ids_file, ids.iter().cloned().collect::<Vec<_>>().join("\n")).unwrap();
    let ids_arg = ids_file.to_str().unwrap();
    // the dry run says how many requests the run must make; the model is named so no server is needed for it
    let dry = e.cmd(&["--llm", "--ids", ids_arg, "--dry-run", "--model", "fake-model-1"]).unwrap().0;
    let to_classify: usize = dry
        .split(", ")
        .find(|p| p.ends_with("to classify in") || p.contains(" to classify in "))
        .map(|p| p.split_whitespace().next().unwrap().parse().unwrap())
        .unwrap();
    let batches = to_classify.div_ceil(10);
    println!("DRY RUN: {dry}");
    let srv = pseudo_srv(11);
    let out = e.cmd(&["--llm", "--ids", ids_arg, "--endpoint", &srv.url]).unwrap().0;
    assert_eq!(counts(&out), (to_classify, 0), "{out}");
    assert_eq!(srv.chats(), batches, "valid answers: one request per batch, no retry: {out}");
    let rows = llm::load_cache(&e.dir()).unwrap();
    assert_eq!(rows.len(), to_classify);
    // a warm rerun asks nothing
    let chats = srv.chats();
    let again = e.cmd(&["--llm", "--ids", ids_arg, "--endpoint", &srv.url]).unwrap().0;
    assert_eq!(srv.chats(), chats, "{again}");
    // the merge over all of the catalog
    let out = e.cmd(&["--merge"]).unwrap().0;
    println!("{out}");
    let merged = roles::load(&e.dir().join("roles.json")).unwrap();
    let seen = check_merged(&index, &tax, &rules, &ovr, &rows, "fake-model-1", &merged);
    assert!(seen > 500, "LLM-evidenced roles in the merged table: {seen}");
    // routed cards that were not in the sample are exactly the llm_failed ones
    let failed = merged.iter().filter(|(_, a)| a.llm_failed).count();
    let routed_all = index
        .cards
        .iter()
        .filter(|c| roles::is_deck_card(c) && !ovr.contains_key(&c.raw.oracle_id) && rules[&c.raw.oracle_id].needs_llm)
        .filter(|c| !roles::prompt_lines(c).is_empty())
        // only Commander-legal cards are ever sent, so only they can be flagged as waiting
        .filter(|c| c.legal == Legal::Legal)
        .count();
    let answered = index
        .cards
        .iter()
        .filter(|c| roles::is_deck_card(c) && !ovr.contains_key(&c.raw.oracle_id) && rules[&c.raw.oracle_id].needs_llm)
        .filter(|c| c.legal == Legal::Legal)
        .filter(|c| rows.contains_key(&llm::cache_key(&llm::card_text(c), &llm::prompt_fnv(&tax), "fake-model-1")))
        .count();
    assert_eq!(failed, routed_all - answered, "llm_failed is exactly the routed cards without an answer");
    // attach, byte determinism and idempotence
    let mut fresh = Index::load(&e.dir().join("oracle.json")).unwrap();
    fresh.attach_roles(&merged).unwrap();
    let first = std::fs::read(e.dir().join("roles.json")).unwrap();
    e.cmd(&["--merge"]).unwrap();
    assert_eq!(first, std::fs::read(e.dir().join("roles.json")).unwrap());
    // `find --role` works on the merged table and agrees with it, for every role
    for role in roles::IDS {
        let expected = fresh
            .cards
            .iter()
            .filter(|c| {
                roles::is_deck_card(c) && !c.is_meld() && merged[&c.raw.oracle_id].roles.iter().any(|r| r == role)
            })
            .count();
        let args: Vec<String> = ["--role", role, "--legal", "any", "--count", "--data-dir"]
            .iter()
            .map(|s| s.to_string())
            .chain([e.dir().display().to_string()])
            .collect();
        let got = knowledge::find(&Project::new(REPO), &args).unwrap();
        assert_eq!(got.trim().parse::<usize>().unwrap(), expected, "find --role {role}");
    }
    // validate --stage merged against the real gold file: the report's counts equal an independent recount
    let (text, _) = e.cmd(&["--validate", "--stage", "merged", "--json"]).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    let mut routed_cards = 0usize;
    let (mut all, mut r_rules, mut r_merged) =
        ((0usize, 0usize, 0usize), (0usize, 0usize, 0usize), (0usize, 0usize, 0usize));
    let mut primary_ok = 0;
    let mut total = 0;
    for (_, id, labels, primary) in &gold {
        if ovr.contains_key(id) {
            continue;
        }
        total += 1;
        let (m, r) = (&merged[id], &rules[id]);
        primary_ok += usize::from(&m.primary == primary);
        routed_cards += usize::from(r.needs_llm);
        for role in roles::IDS {
            let want = labels.iter().any(|l| l == role);
            let tally = |t: &mut (usize, usize, usize), got: bool| match (want, got) {
                (true, true) => t.0 += 1,
                (false, true) => t.1 += 1,
                (true, false) => t.2 += 1,
                _ => {}
            };
            tally(&mut all, m.roles.iter().any(|x| x == role));
            if r.needs_llm {
                tally(&mut r_merged, m.roles.iter().any(|x| x == role));
                tally(&mut r_rules, r.roles.iter().any(|x| x == role));
            }
        }
    }
    assert_eq!(score(&v, "gold_micro"), all);
    assert_eq!(score(&v, "routed_scored_micro"), r_merged);
    assert_eq!(score(&v, "routed_rules_micro"), r_rules);
    assert_eq!(v["routed_cards"].as_u64().unwrap() as usize, routed_cards);
    assert_eq!(
        (v["primary_total"].as_u64().unwrap() as usize, v["primary_correct"].as_u64().unwrap() as usize),
        (total, primary_ok)
    );
    println!(
        "real pipeline: {to_classify} classified in {batches} batches, {seen} LLM roles merged, \
         {failed} llm_failed, {:?}",
        started.elapsed()
    );
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_catalog_empty_cache_merge_is_the_rule_table_and_no_routed_card_with_roles_is_silently_vetoed() {
    plain_variant();
    let e = RealEnv::new();
    let index = Index::load(&e.dir().join("oracle.json")).unwrap();
    let ovr_text = roles::read_overrides(&e.dir().join("oracle.json")).unwrap();
    let rules = roles::generate(&index, "").unwrap();
    let ovr = roles::overrides(&index, &ovr_text).unwrap();
    e.cmd(&["--merge"]).unwrap();
    let merged = roles::load(&e.dir().join("roles.json")).unwrap();
    let (mut routed_empty_lines_with_roles, mut failed) = (vec![], 0usize);
    for c in index.cards.iter().filter(|c| roles::is_deck_card(c)) {
        let id = &c.raw.oracle_id;
        let (m, r) = (&merged[id], &rules[id]);
        if let Some(o) = ovr.get(id) {
            assert_eq!(m, o);
            continue;
        }
        assert_eq!(
            (&m.roles, &m.evidence, &m.route, m.needs_llm),
            (&r.roles, &r.evidence, &r.route, r.needs_llm),
            "{}",
            c.raw.name
        );
        failed += usize::from(m.llm_failed);
        if r.needs_llm && roles::prompt_lines(c).is_empty() && !r.roles.is_empty() {
            routed_empty_lines_with_roles.push(c.raw.name.clone());
        }
    }
    assert!(failed > 20_000, "about 23,000 routed cards, none answered: {failed}");
    assert!(
        routed_empty_lines_with_roles.is_empty(),
        "routed cards with rule roles and nothing to show the model (their roles would be vetoed by the empty \
         answer the merge invents): {routed_empty_lines_with_roles:?}"
    );
    let mut fresh = Index::load(&e.dir().join("oracle.json")).unwrap();
    fresh.attach_roles(&merged).unwrap();
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_catalog_the_worked_examples_are_the_real_oracle_text_and_the_worst_batch_fits_the_context() {
    plain_variant();
    let real = Path::new(REPO).join("data/knowledge/oracle.json");
    assert!(real.exists(), "{NEEDS_CATALOG}");
    let index = Index::load(&real).unwrap();
    let tax = taxonomy();
    let rules = roles::generate(&index, "").unwrap();
    for ex in &llm::EXAMPLES {
        let found = index.lookup(ex.name);
        assert_eq!(found.len(), 1, "{} must be exactly one catalog card", ex.name);
        let c = &index.cards[found[0] as usize];
        assert_eq!(c.raw.type_line, ex.type_line, "{}", ex.name);
        let shown = roles::prompt_lines(c);
        let want: Vec<(usize, String)> = ex.lines.iter().map(|(i, t)| (*i, t.to_string())).collect();
        assert_eq!(shown, want, "{}: the example is not what the model would be shown for the real card", ex.name);
        println!("example {:<20} routed by the rules: {}", ex.name, rules[&c.raw.oracle_id].needs_llm);
    }
    // the ten longest cards a real batch could hold
    let mut sizes: Vec<usize> = index
        .cards
        .iter()
        .filter(|c| roles::is_deck_card(c) && c.legal == Legal::Legal)
        .map(|c| llm::card_text(c).chars().count() + 12)
        .collect();
    sizes.sort_unstable();
    let worst: usize = sizes.iter().rev().take(10).sum();
    let system = llm::estimate_tokens(&llm::system_prompt(&tax));
    let total = system + worst.div_ceil(4) + 160 * 10 + 64;
    println!(
        "worst batch: system {system} tokens + ten longest cards {} tokens, answer budget 1,664: {total}",
        worst.div_ceil(4)
    );
    assert!(total < 8000, "{total}");
    assert!(*sizes.last().unwrap() < 4000, "the longest card text is {} chars", sizes.last().unwrap());
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_catalog_validation_accepts_a_citation_exactly_when_the_line_is_shown_and_the_guard_matches() {
    plain_variant();
    let real = Path::new(REPO).join("data/knowledge/oracle.json");
    assert!(real.exists(), "{NEEDS_CATALOG}");
    let index = Index::load(&real).unwrap();
    let tax = taxonomy();
    let mut cards: Vec<&knowledge::Card> = index.cards.iter().filter(|c| roles::is_deck_card(c)).collect();
    cards.sort_by_key(|c| text_hash(&c.raw.oracle_id));
    let (mut accepted, mut refused_skipped) = (0usize, 0usize);
    for card in cards.iter().take(3000) {
        let shown = roles::prompt_lines(card);
        let n = card.lines().len();
        let batch = [("a0".to_string(), shown.as_slice())];
        for i in 0..n + 2 {
            let text = roles::line_text(card, i);
            let is_shown = shown.iter().any(|(s, _)| *s == i);
            for role in roles::IDS {
                let guard = tax.guard(role).unwrap().is_match(&text).unwrap_or(false);
                let answer = json!({"cards": [entry("a0", &[role], &[(role, i)])]});
                // a shown line is citable whatever the guard says (a guard miss is dropped later by the merge)
                let ok = llm::validate_output(&answer.to_string(), &batch, &tax).unwrap()[0].is_ok();
                assert_eq!(ok, is_shown, "{} line {i} {role}: shown {is_shown} guard {guard}", card.raw.name);
                accepted += usize::from(ok);
                refused_skipped += usize::from(!is_shown && guard && !ok);
            }
        }
    }
    assert!(
        accepted > 1000 && refused_skipped > 50,
        "{accepted} accepted, {refused_skipped} skipped-line citations refused"
    );
}
