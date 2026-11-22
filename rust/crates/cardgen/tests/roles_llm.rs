//! The local-LLM stage against a FAKE server (a `TcpListener` serving canned `/v1/models` and `/v1/chat/completions`
//! replies): no real model, no network beyond localhost. Cards are real Oracle text from the tracked fixture.
use cardgen::data::Project;
use cardgen::knowledge::{Index, Legal};
use cardgen::llm::{self, Kind, LlmCard, MergeEnv};
use cardgen::roles::{self, Assignment, Evidence, RouteReason, Source, Taxonomy};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const TAXONOMY: &str = include_str!("../../../../data/knowledge/roles.toml");
const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/roles_oracle.json");
const FIVE: [&str; 5] = [
    "Skywhaler's Shot",
    "Ugin's Mastery",
    "Counterspell",
    "Agadeem's Awakening // Agadeem, the Undercrypt",
    "Elspeth, Sun's Champion",
];

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

enum Reply {
    /// a chat completion carrying this content, finish_reason stop
    Content(String),
    /// a chat completion cut off by the token limit
    Truncated(String),
    /// a body sent as is
    Raw(String),
    Status(u16),
    /// close the connection without answering
    Drop,
}
type Handler = Box<dyn Fn(usize, &Value) -> Reply + Send + Sync>;

struct Fake {
    url: String,
    addr: SocketAddr,
    model: Arc<Mutex<String>>,
    chats: Arc<AtomicUsize>,
    models: Arc<AtomicUsize>,
    bodies: Arc<Mutex<Vec<Value>>>,
    /// the request bodies as the exact text that came over the wire
    raw: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}
impl Fake {
    fn start(handler: Handler) -> Fake {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let model = Arc::new(Mutex::new("fake-model-1".to_string()));
        let (chats, models) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
        let bodies = Arc::new(Mutex::new(vec![]));
        let raw = Arc::new(Mutex::new(vec![]));
        let stop = Arc::new(AtomicBool::new(false));
        let (m, c, mo, b, st) = (model.clone(), chats.clone(), models.clone(), bodies.clone(), stop.clone());
        let rw = raw.clone();
        let handle = std::thread::spawn(move || {
            for stream in listener.incoming() {
                if st.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                serve(&mut stream, &handler, &m, &c, &mo, &b, &rw);
            }
        });
        Fake { url: format!("http://{addr}/v1"), addr, model, chats, models, bodies, raw, stop, handle: Some(handle) }
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
}
impl Drop for Fake {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.addr);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}
fn serve(
    stream: &mut TcpStream,
    handler: &Handler,
    model: &Mutex<String>,
    chats: &AtomicUsize,
    models: &AtomicUsize,
    bodies: &Mutex<Vec<Value>>,
    raw: &Mutex<Vec<String>>,
) {
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
    let respond = |stream: &mut TcpStream, status: u16, text: &str| {
        let _ = write!(
            stream,
            concat!(
                "HTTP/1.1 {} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\n",
                "Connection: close\r\n\r\n{}"
            ),
            status,
            text.len(),
            text
        );
    };
    let id = model.lock().unwrap().clone();
    if request.starts_with("GET /v1/models") {
        models.fetch_add(1, Ordering::SeqCst);
        respond(stream, 200, &json!({"object": "list", "data": [{"id": id, "object": "model"}]}).to_string());
    } else if request.starts_with("POST /v1/chat/completions") {
        let n = chats.fetch_add(1, Ordering::SeqCst);
        raw.lock().unwrap().push(String::from_utf8_lossy(&body).to_string());
        let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
        bodies.lock().unwrap().push(body.clone());
        let completion = |content: &str, finish: &str| {
            json!({"id": "c", "object": "chat.completion", "model": id,
                   "choices": [{"index": 0, "finish_reason": finish,
                                "message": {"role": "assistant", "content": content}}]})
            .to_string()
        };
        match handler(n, &body) {
            Reply::Content(c) => respond(stream, 200, &completion(&c, "stop")),
            Reply::Truncated(c) => respond(stream, 200, &completion(&c, "length")),
            Reply::Raw(t) => respond(stream, 200, &t),
            Reply::Status(s) => respond(stream, s, "{\"error\":\"boom\"}"),
            Reply::Drop => {}
        }
    } else {
        respond(stream, 404, "{}");
    }
}

// ------------------------------------------------------------------------------- what the fake "model" says

#[derive(Clone)]
struct Item {
    id: String,
    lines: Vec<(usize, String)>,
}
/// The cards of a request, parsed back out of the user message exactly as the prompt printed them.
fn parse_batch(body: &Value) -> Vec<Item> {
    let user = body["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap();
    let mut items: Vec<Item> = vec![];
    for line in user.lines().skip(1) {
        if let Some(rest) = line.strip_prefix('[') {
            items.push(Item { id: rest.split(']').next().unwrap().to_string(), lines: vec![] });
        } else if let Some((n, text)) = line.split_once(": ") {
            if let Ok(n) = n.parse::<usize>() {
                items.last_mut().unwrap().lines.push((n, text.to_string()));
            }
        }
    }
    items
}
fn first(item: &Item) -> &str {
    &item.lines[0].1
}
/// What a sensible model says about each of the five fixture cards.
fn says(item: &Item) -> (Vec<&'static str>, Vec<(&'static str, usize)>) {
    let f = first(item);
    if f.starts_with("Destroy target creature with power") {
        (vec!["removal", "selection"], vec![("removal", 0), ("selection", 0)])
    } else if f.starts_with("Whenever you cast a colorless") {
        (vec![], vec![])
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
fn entry(id: &str, roles: &[&str], evidence: &[(&str, usize)]) -> Value {
    let roles: Vec<Value> = roles
        .iter()
        .map(|r| json!({"role": r, "line": evidence.iter().find(|(e, _)| e == r).map_or(0, |(_, n)| *n)}))
        .collect();
    json!({"id": id, "roles": roles})
}
fn valid(items: &[Item]) -> Value {
    json!({"cards": items.iter().map(|i| { let (r, e) = says(i); entry(&i.id, &r, &e) }).collect::<Vec<_>>()})
}
fn find<'a>(v: &'a mut Value, starts: &str, items: &[Item]) -> Option<&'a mut Value> {
    let id = items.iter().find(|i| first(i).starts_with(starts))?.id.clone();
    v["cards"].as_array_mut()?.iter_mut().find(|c| c["id"] == id.as_str())
}
/// One class of invalid model output for ONE card (the first removal card), applied to an otherwise valid answer.
fn corrupt_card(class: &str, items: &[Item]) -> Value {
    let mut v = valid(items);
    match class {
        "ids" => v["cards"][0]["id"] = json!("z9"),
        "vocabulary" => {
            let id = v["cards"][0]["id"].as_str().unwrap().to_string();
            v["cards"][0] = entry(&id, &["banana"], &[("banana", 0)]);
        }
        "five roles" => {
            let id = v["cards"][0]["id"].as_str().unwrap().to_string();
            let roles = ["removal", "wipe", "draw", "tokens", "ramp"];
            v["cards"][0] = entry(&id, &roles, &roles.map(|r| (r, 0)));
        }
        "repeated role" => {
            let id = v["cards"][0]["id"].as_str().unwrap().to_string();
            v["cards"][0] = entry(&id, &["removal", "removal"], &[("removal", 0)]);
        }
        "out of range" => {
            let c = find(&mut v, "Destroy target creature with power", items).unwrap();
            c["roles"][0]["line"] = json!(99);
        }
        "skipped line" => {
            // Elspeth's ultimate (line 2) is never shown, so it cannot be cited
            let c = find(&mut v, "+1: Create three", items).unwrap();
            c["roles"][1]["line"] = json!(2);
        }
        "no citation" => {
            let c = find(&mut v, "Destroy target creature with power", items).unwrap();
            c["roles"][0].as_object_mut().unwrap().remove("line");
        }
        other => panic!("unknown class {other}"),
    }
    v
}
/// One class of unusable reply (no document, or an error status) for the whole batch.
fn corrupt_document(class: &str, items: &[Item]) -> Reply {
    match class {
        "malformed" => Reply::Content("{\"cards\": [{\"id\": \"a0\", ".into()),
        "truncated" => Reply::Truncated(valid(items).to_string()[..40].to_string()),
        "no choices" => Reply::Raw("{}".into()),
        "http 500" => Reply::Status(500),
        other => panic!("unknown class {other}"),
    }
}
fn good() -> Handler {
    Box::new(|_, body| Reply::Content(valid(&parse_batch(body)).to_string()))
}

// ---------------------------------------------------------------------------------------------------- environment

struct Env(PathBuf);
impl Env {
    fn new() -> Self {
        plain_variant();
        static N: AtomicUsize = AtomicUsize::new(0);
        let root =
            std::env::temp_dir().join(format!("roles-llm-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        let dir = root.join("data/knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(FIXTURE, dir.join("oracle.json")).unwrap();
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
    fn command(&self, args: &[&str]) -> Result<(String, bool), String> {
        let args: Vec<String> =
            args.iter().map(|s| s.to_string()).chain(["--data-dir".into(), self.dir().display().to_string()]).collect();
        roles::command(&Project::new(&self.0), &args)
    }
    fn llm(&self, fake: &Fake, extra: &[&str]) -> Result<String, String> {
        let ids = self.dir().join("ids.txt");
        let mut args = vec!["--llm", "--ids", ids.to_str().unwrap(), "--endpoint", &fake.url];
        args.extend_from_slice(extra);
        self.command(&args).map(|(text, _)| text)
    }
    fn rows(&self) -> Vec<llm::Row> {
        llm::read_rows(&self.dir().join("llm_roles.jsonl")).unwrap().into_values().collect()
    }
    fn index(&self) -> Index {
        Index::load(&self.dir().join("oracle.json")).unwrap()
    }
    fn merged(&self) -> roles::RoleMap {
        self.command(&["--merge"]).unwrap();
        roles::load(&self.dir().join("roles.json")).unwrap()
    }
    fn no_temp_files_beside_cache(&self) {
        let tmp: Vec<_> = std::fs::read_dir(self.dir())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(tmp.is_empty(), "{tmp:?}");
    }
    fn files(&self) -> BTreeMap<String, Vec<u8>> {
        let mut out = BTreeMap::new();
        fn walk(dir: &std::path::Path, base: &std::path::Path, out: &mut BTreeMap<String, Vec<u8>>) {
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
}
impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn assignment_of<'a>(index: &Index, map: &'a roles::RoleMap, name: &str) -> &'a Assignment {
    &map[&index.cards[index.lookup(name)[0] as usize].raw.oracle_id]
}

// ----------------------------------------------------------------------------------------- the client against the fake

#[test]
fn a_valid_batch_is_requested_validated_cached_and_merged() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    let out = e.llm(&fake, &[]).unwrap();
    assert_eq!((fake.chats(), fake.models()), (1, 1), "{out}");
    assert!(out.contains("1 chat requests") && out.contains("5 classified") && out.contains("fake-model-1"), "{out}");
    // the request: deterministic settings, a grammar-constrained schema and the user message without any rule roles
    let body = fake.bodies.lock().unwrap()[0].clone();
    assert_eq!((&body["temperature"], &body["seed"], &body["cache_prompt"]), (&json!(0), &json!(1), &json!(true)));
    assert_eq!(body["model"], "fake-model-1");
    let schema = &body["response_format"]["json_schema"]["schema"];
    assert_eq!(body["response_format"]["type"], "json_schema");
    assert_eq!(
        (&schema["properties"]["cards"]["minItems"], &schema["properties"]["cards"]["maxItems"]),
        (&json!(5), &json!(5))
    );
    let items = &schema["properties"]["cards"]["items"]["properties"];
    // one array of {role, line} pairs: the roles and their citations cannot drift apart or be written out of order
    let pair = &items["roles"]["items"]["properties"];
    assert_eq!(pair["role"]["enum"].as_array().unwrap().len(), 26);
    assert_eq!((&pair["line"]["type"], &pair["line"]["minimum"]), (&json!("integer"), &json!(0)));
    assert_eq!(items["roles"]["maxItems"], 4);
    assert_eq!(items["id"]["enum"], json!(["a0", "a1", "a2", "a3", "a4"]));
    assert!(items.get("primary").is_none() && items.get("evidence").is_none());
    let user = body["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap();
    assert!(user.starts_with("Classify these 5 cards.\n[a0] ") && user.contains("[a4] "), "{user}");
    assert!(!user.contains("needs_llm") && !user.contains("roles"), "the model must not see rule roles: {user}");
    assert_eq!(body["messages"][0]["role"], "system");
    // the cache: one sorted row per card with the documented fields
    let text = std::fs::read_to_string(e.dir().join("llm_roles.jsonl")).unwrap();
    let keys: Vec<String> =
        text.lines().map(|l| serde_json::from_str::<Value>(l).unwrap()["key"].as_str().unwrap().to_string()).collect();
    assert_eq!(keys.len(), 5);
    assert!(keys.windows(2).all(|w| w[0] < w[1]), "rows sorted by key");
    let rows = e.rows();
    let row = rows.iter().find(|r| r.name == "Skywhaler's Shot").unwrap();
    assert_eq!(row.roles, ["removal", "selection"]);
    assert_eq!(row.primary.as_deref(), Some("removal"));
    assert_eq!(row.evidence["selection"], 0);
    assert_eq!(row.model, "fake-model-1");
    let tax = Taxonomy::parse(TAXONOMY).unwrap();
    assert_eq!((row.taxonomy.clone(), row.prompt.clone()), (tax.hash(), llm::prompt_fnv(&tax)));
    assert!(rows.iter().find(|r| r.name == "Ugin's Mastery").unwrap().primary.is_none());
    // the run report and merge
    assert!(e.dir().join("reports/llm-run.json").exists());
    let map = e.merged();
    let i = e.index();
    assert_eq!(map.meta.model, "fake-model-1");
    assert_eq!(map.meta.prompt_fnv, llm::prompt_fnv(&tax));
    let shot = assignment_of(&i, &map, "Skywhaler's Shot");
    assert_eq!(shot.roles, ["removal", "selection"], "{shot:?}");
    assert_eq!(shot.evidence["selection"].source, Source::Llm);
    assert_eq!(shot.evidence["removal"].source, Source::Rule, "a role the rules also found keeps the rule's evidence");
    assert!(shot.needs_llm && !shot.llm_failed);
    // the merged table attaches (LLM evidence is guard-checked), searches and shows
    let (shown, _) = e.command(&["--show", "Skywhaler's Shot"]).unwrap();
    assert!(shown.contains("selection: Llm llm") && shown.contains("removal: Rule"), "{shown}");
    let found = cardgen::knowledge::find(
        &Project::new(&e.0),
        &[
            "--role".into(),
            "selection".into(),
            "--name".into(),
            "Skywhaler's Shot".into(),
            "--data-dir".into(),
            e.dir().display().to_string(),
        ],
    )
    .unwrap();
    assert!(found.contains("Skywhaler's Shot"), "find --role works on the merged table: {found}");
    assert!(e.dir().join("reports/roles-disagreements.txt").exists());
}

#[test]
fn a_structurally_invalid_card_is_asked_alone_once_and_never_costs_the_batch() {
    plain_variant();
    for class in ["ids", "vocabulary", "five roles", "repeated role", "out of range", "skipped line", "no citation"] {
        let e = Env::new();
        let fake = Fake::start(Box::new(move |n, body| {
            let items = parse_batch(body);
            if n == 0 {
                Reply::Content(corrupt_card(class, &items).to_string())
            } else {
                Reply::Content(valid(&items).to_string())
            }
        }));
        let out = e.llm(&fake, &[]).unwrap_or_else(|err| panic!("{class}: {err}"));
        assert_eq!(fake.chats(), 2, "{class}: the batch, then only the bad card alone: {out}");
        assert!(
            out.contains("5 classified") && out.contains("0 failed") && out.contains("1 per-card retries"),
            "{class}: {out}"
        );
        let bodies = fake.bodies.lock().unwrap().clone();
        assert_eq!(parse_batch(&bodies[1]).len(), 1, "{class}: the resend carries one card");
        assert_eq!(e.rows().len(), 5, "{class}");
    }
}

#[test]
fn an_unusable_reply_document_sends_every_card_alone_once_and_a_transport_error_resends_the_batch() {
    plain_variant();
    for class in ["malformed", "truncated", "no choices", "http 500"] {
        let e = Env::new();
        let fake = Fake::start(Box::new(move |n, body| {
            let items = parse_batch(body);
            if n == 0 { corrupt_document(class, &items) } else { Reply::Content(valid(&items).to_string()) }
        }));
        let out = e.llm(&fake, &[]).unwrap_or_else(|err| panic!("{class}: {err}"));
        assert_eq!(fake.chats(), 1 + 5, "{class}: the batch once, then five cards alone: {out}");
        assert!(out.contains("5 classified") && out.contains("5 per-card retries"), "{class}: {out}");
        let bodies = fake.bodies.lock().unwrap().clone();
        assert!(bodies[1..].iter().all(|b| parse_batch(b).len() == 1), "{class}");
        assert_eq!(bodies[1]["response_format"]["json_schema"]["schema"]["properties"]["cards"]["minItems"], 1);
        assert_eq!(e.rows().len(), 5, "{class}");
    }
    // only a transport error (no answer at all) resends the identical batch
    let e = Env::new();
    let fake =
        Fake::start(Box::new(
            |n, body| {
                if n == 0 { Reply::Drop } else { Reply::Content(valid(&parse_batch(body)).to_string()) }
            },
        ));
    let out = e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 2, "{out}");
    assert!(out.contains("1 batch retries") && out.contains("0 per-card retries"), "{out}");
    let bodies = fake.bodies.lock().unwrap().clone();
    assert_eq!(bodies[0], bodies[1], "the resend is byte-identical");
}

#[test]
fn a_guard_miss_is_soft_the_raw_answer_is_cached_and_the_merge_logs_the_drop() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(Box::new(|_, body| {
        let items = parse_batch(body);
        let mut v = valid(&items);
        // Counterspell is said to be a draw card, citing its only line: the guard for draw refuses it
        let id = items.iter().find(|i| first(i) == "Counter target spell.").unwrap().id.clone();
        let c = v["cards"].as_array_mut().unwrap().iter_mut().find(|c| c["id"] == id.as_str()).unwrap();
        *c = entry(&id, &["draw"], &[("draw", 0)]);
        Reply::Content(v.to_string())
    }));
    let out = e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 1, "{out}");
    assert!(out.contains("5 classified") && out.contains("0 failed"), "{out}");
    let row = e.rows().into_iter().find(|r| r.name == "Counterspell").unwrap();
    assert_eq!((row.roles.as_slice(), row.evidence["draw"]), (&["draw".to_string()][..], 0), "the raw answer is kept");
    e.command(&["--merge"]).unwrap();
    let report = std::fs::read_to_string(e.dir().join("reports/roles-disagreements.txt")).unwrap();
    assert!(report.contains("Counterspell"), "{report}");
    let map = roles::load(&e.dir().join("roles.json")).unwrap();
    assert_eq!(assignment_of(&e.index(), &map, "Counterspell").roles, ["counter"], "a rules-final card never changes");
}

#[test]
fn a_card_failing_alone_is_llm_failed_and_keeps_its_rule_roles() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(Box::new(|_, body| {
        let items = parse_batch(body);
        let mut v = valid(&items);
        if let Some(c) = find(&mut v, "Destroy target creature with power", &items) {
            let id = c["id"].clone();
            *c = entry(id.as_str().unwrap(), &["banana"], &[("banana", 0)]);
        }
        Reply::Content(v.to_string())
    }));
    let out = e.llm(&fake, &[]).unwrap();
    // the batch, then the bad card alone: the same request is not repeated
    assert_eq!(fake.chats(), 2, "{out}");
    assert!(out.contains("4 classified") && out.contains("1 failed") && out.contains("1 per-card retries"), "{out}");
    assert_eq!(e.rows().len(), 4);
    let failed = std::fs::read_to_string(e.dir().join("reports/llm-failed.txt")).unwrap();
    assert!(failed.contains("Skywhaler's Shot"), "{failed}");
    let i = e.index();
    let rules = roles::generate(&i, "").unwrap();
    let map = e.merged();
    let (rule, merged) = (assignment_of(&i, &rules, "Skywhaler's Shot"), assignment_of(&i, &map, "Skywhaler's Shot"));
    assert!(merged.llm_failed && merged.needs_llm, "{merged:?}");
    assert_eq!((&merged.roles, &merged.evidence), (&rule.roles, &rule.evidence), "the rule roles stand");
    assert!(!assignment_of(&i, &map, "Ugin's Mastery").llm_failed, "a routed card with an answer is not llm_failed");
    let stats: llm::MergeStats =
        serde_json::from_slice(&std::fs::read(e.dir().join("reports/merge.json")).unwrap()).unwrap();
    assert_eq!((stats.failed, stats.not_run), (1, stats.llm_failed - 1), "{stats:?}");
}

#[test]
fn a_warm_cache_makes_no_chat_requests() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    e.llm(&fake, &[]).unwrap();
    let before = std::fs::read(e.dir().join("llm_roles.jsonl")).unwrap();
    let out = e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 1, "the second run asked nothing: {out}");
    assert!(out.contains("0 chat requests") && out.contains("5 cached"), "{out}");
    assert_eq!(before, std::fs::read(e.dir().join("llm_roles.jsonl")).unwrap());
    assert_eq!(fake.models(), 2, "only the model id is read again");
}

#[test]
fn taxonomy_and_model_changes_invalidate_the_cache_but_guards_overrides_and_rules_do_not() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 1);
    // a guard, keep_rule or overrides edit changes no prompt text, no model and no key: still warm
    let toml = std::fs::read_to_string(e.dir().join("roles.toml")).unwrap();
    let tweaked = toml.replacen("guard = '(?i)counter|exile.*spells?'", "guard = '(?i)counter|exile.*spells?|x'", 1);
    assert_ne!(toml, tweaked);
    let tweaked =
        tweaked.replacen("guard = '(?i)discard'\nkeep_rule = true", "guard = '(?i)discard'\nkeep_rule = false", 1);
    std::fs::write(e.dir().join("roles.toml"), &tweaked).unwrap();
    std::fs::write(
        e.dir().join("roles_overrides.toml"),
        "[[card]]\nname='Mogg Fanatic'\nroles=[]\nnote='changes the rules fingerprint, not the cache key'\n",
    )
    .unwrap();
    let out = e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 1, "guards, keep_rule and overrides leave the cache warm: {out}");
    // a definition edit changes what the model is told: every row is stale
    let edited = tweaked.replacen(
        "counters a spell or an ability on the stack.",
        "counters a spell or an activated ability.",
        1,
    );
    assert_ne!(edited, tweaked);
    std::fs::write(e.dir().join("roles.toml"), &edited).unwrap();
    e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 2);
    assert_eq!(e.rows().len(), 10, "the old rows stay (append-merge) beside the new ones");
    // another model id: stale again
    fake.set_model("fake-model-2");
    e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 3);
    assert_eq!(e.rows().len(), 15);
    let models: std::collections::BTreeSet<_> = e.rows().into_iter().map(|r| r.model).collect();
    assert_eq!(models.len(), 2);
    // and back: the first model's rows for the first prompt are not lost, but the prompt has changed, so still cold
    fake.set_model("fake-model-1");
    std::fs::write(e.dir().join("roles.toml"), &tweaked).unwrap();
    e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 3, "the original prompt and model are warm again");
    // the cache now holds this prompt for one model only, so a merge knows whose answers to read
    e.command(&["--merge"]).unwrap();
    // with two models cached for one prompt it has to be told
    std::fs::write(e.dir().join("roles.toml"), &edited).unwrap();
    let err = e.command(&["--merge"]).unwrap_err();
    assert!(err.contains("several models"), "{err}");
    e.command(&["--merge", "--model", "fake-model-2"]).unwrap();
    assert_eq!(roles::load(&e.dir().join("roles.json")).unwrap().meta.model, "fake-model-2");
}

#[test]
fn an_unreachable_endpoint_exits_with_llm_unreachable_and_changes_no_file() {
    plain_variant();
    let e = Env::new();
    let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port(); // freed again at once
    let before = e.files();
    let ids = e.dir().join("ids.txt");
    let err = e
        .command(&["--llm", "--ids", ids.to_str().unwrap(), "--endpoint", &format!("http://127.0.0.1:{port}/v1")])
        .unwrap_err();
    assert!(err.contains("LLM unreachable"), "{err}");
    assert_eq!(before, e.files(), "no file written, no report, no temp file");
    // a dry run needs the model id too, so it fails the same way
    assert!(
        e.command(&["--llm", "--dry-run", "--endpoint", &format!("http://127.0.0.1:{port}/v1")])
            .unwrap_err()
            .contains("LLM unreachable")
    );
    assert_eq!(before, e.files());
}

#[test]
fn a_server_that_dies_mid_run_keeps_what_was_classified_and_stops() {
    plain_variant();
    let e = Env::new();
    let fake =
        Fake::start(Box::new(
            |n, body| {
                if n == 0 { Reply::Content(valid(&parse_batch(body)).to_string()) } else { Reply::Drop }
            },
        ));
    let err = e.llm(&fake, &["--batch", "2"]).unwrap_err();
    assert!(err.contains("LLM unreachable mid-run") && err.contains("saved 2 classified"), "{err}");
    assert_eq!(e.rows().len(), 2, "the finished batch is on disk");
    assert_eq!(fake.chats(), 1 + 2, "the batch and its resend both unanswered: the run stops");
}

#[test]
fn limit_only_routed_dry_run_batch_size_and_pending_file() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    let out = e.llm(&fake, &["--dry-run"]).unwrap();
    assert!(
        out.starts_with("DRY RUN") && out.contains("5 to classify in 1 batches") && out.contains("0 chat requests"),
        "{out}"
    );
    assert_eq!(fake.chats(), 0);
    assert!(!e.dir().join("llm_roles.jsonl").exists() && !e.dir().join("reports").exists(), "a dry run writes nothing");
    e.llm(&fake, &["--limit", "3", "--batch", "2"]).unwrap();
    assert_eq!(fake.chats(), 2, "three cards in batches of two");
    assert_eq!(e.rows().len(), 3);
    // the scheduled job writes the untracked pending file, which the loader reads after the tracked one
    let out = e.llm(&fake, &["--pending"]).unwrap();
    assert!(out.contains("3 cached") && out.contains("2 classified"), "{out}");
    assert_eq!(e.rows().len(), 3, "the tracked file is untouched");
    let pending = llm::read_rows(&e.dir().join("llm_roles.pending.jsonl")).unwrap();
    assert_eq!(pending.len(), 2);
    assert_eq!(llm::load_cache(&e.dir()).unwrap().len(), 5);
    // --only-routed drops the card the rules decided alone
    let i = e.index();
    let rules = roles::generate(&i, "").unwrap();
    let routed = FIVE.iter().filter(|n| assignment_of(&i, &rules, n).needs_llm).count();
    assert!(routed < 5 && !assignment_of(&i, &rules, "Counterspell").needs_llm);
    let ids = e.dir().join("ids.txt");
    let out = e
        .command(&["--llm", "--ids", ids.to_str().unwrap(), "--endpoint", &fake.url, "--only-routed", "--dry-run"])
        .unwrap()
        .0;
    assert!(out.contains(&format!("{routed} candidate cards")), "{out}");
    for bad in [
        vec!["--rules", "--limit", "3"],
        vec!["--llm", "--batch", "0"],
        vec!["--llm", "--stage", "merged"],
        vec!["--validate", "--dry-run"],
    ] {
        assert!(e.command(&bad).is_err(), "{bad:?}");
    }
}

#[test]
fn cards_fixed_by_an_override_are_never_sent_even_when_the_override_clears_their_roles() {
    plain_variant();
    let e = Env::new();
    std::fs::write(
        e.dir().join("roles_overrides.toml"),
        "[[card]]\nname=\"Skywhaler's Shot\"\nroles=[]\nnote='cleared by hand'\n",
    )
    .unwrap();
    let fake = Fake::start(good());
    let out = e.llm(&fake, &[]).unwrap();
    assert!(out.contains("4 classified"), "{out}");
    let user = fake.bodies.lock().unwrap()[0]["messages"].as_array().unwrap().last().unwrap()["content"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        user.starts_with("Classify these 4 cards.") && !user.contains("Destroy target creature with power"),
        "{user}"
    );
    // and the merge uses the override exactly, whatever the cache holds
    let map = e.merged();
    let shot = assignment_of(&e.index(), &map, "Skywhaler's Shot");
    assert!(shot.roles.is_empty() && !shot.needs_llm && !shot.llm_failed, "{shot:?}");
}

// ------------------------------------------------------------------------------------------------ determinism

#[test]
fn the_cache_file_and_the_merged_table_are_deterministic_whatever_the_batching() {
    plain_variant();
    let (a, b) = (Env::new(), Env::new());
    let (fa, fb) = (Fake::start(good()), Fake::start(good()));
    a.llm(&fa, &["--batch", "10"]).unwrap();
    b.llm(&fb, &["--batch", "2"]).unwrap();
    assert_eq!(
        std::fs::read(a.dir().join("llm_roles.jsonl")).unwrap(),
        std::fs::read(b.dir().join("llm_roles.jsonl")).unwrap()
    );
    let (ma, mb) = (a.merged(), b.merged());
    assert_eq!(roles::bytes(&ma).unwrap(), roles::bytes(&mb).unwrap());
    let first = std::fs::read(a.dir().join("roles.json")).unwrap();
    let report = std::fs::read(a.dir().join("reports/roles-disagreements.txt")).unwrap();
    a.command(&["--merge"]).unwrap();
    assert_eq!(first, std::fs::read(a.dir().join("roles.json")).unwrap());
    assert_eq!(report, std::fs::read(a.dir().join("reports/roles-disagreements.txt")).unwrap());
}

// --------------------------------------------------------------------------------------------- the merge policy

fn ev(source: Source, certain: bool) -> Evidence {
    Evidence { source, certain, rule: "r".into(), face: None, line: Some(0) }
}
/// A rule assignment: roles with their certainty, routed or not.
fn rule(roles: &[(&str, bool)], needs_llm: bool, route: &[RouteReason]) -> Assignment {
    Assignment {
        roles: roles.iter().map(|(r, _)| r.to_string()).collect(),
        primary: roles.first().map(|(r, _)| r.to_string()).unwrap_or_default(),
        evidence: roles.iter().map(|(r, c)| (r.to_string(), ev(Source::Rule, *c))).collect(),
        needs_llm,
        route: route.to_vec(),
        llm_failed: false,
    }
}
fn said(roles: &[&str]) -> LlmCard {
    LlmCard {
        why: None,
        roles: roles.iter().map(|r| r.to_string()).collect(),
        primary: roles.first().map(|r| r.to_string()),
        evidence: roles.iter().enumerate().map(|(i, r)| (r.to_string(), i)).collect(),
    }
}
/// Run `merge` with `keep` roles protected and the (role, line) pairs in `failing` failing their guard; lines
/// below 9 exist.
fn run(
    rule: &Assignment,
    llm: Option<&LlmCard>,
    keep: &[&str],
    failing: &[(&str, usize)],
) -> (Assignment, Vec<llm::Disagreement>) {
    let keep_fn = |r: &str| keep.contains(&r);
    let check = |role: &str, line: usize| (line < 9 && !failing.contains(&(role, line))).then_some(Some(line + 100));
    llm::merge(rule, llm, None, &MergeEnv { keep: &keep_fn, check: &check })
}
fn kinds(d: &[llm::Disagreement]) -> Vec<(Kind, String)> {
    d.iter().map(|d| (d.kind, d.role.clone())).collect()
}

#[test]
fn merge_uses_an_override_exactly() {
    plain_variant();
    let r = rule(&[("counter", true), ("draw", false)], true, &[RouteReason::RiskyRole]);
    let mut o = rule(&[("tutor", false)], false, &[]);
    o.evidence.get_mut("tutor").unwrap().source = Source::Override;
    let keep = |_: &str| true;
    let check = |_: &str, _: usize| Some(None);
    let (a, d) = llm::merge(&r, Some(&said(&["removal"])), Some(&o), &MergeEnv { keep: &keep, check: &check });
    assert_eq!(a, o);
    assert!(d.is_empty());
}

#[test]
fn merge_never_lets_the_llm_change_a_card_the_rules_decided() {
    plain_variant();
    let r = rule(&[("counter", false)], false, &[]);
    let (a, d) = run(&r, Some(&said(&["removal"])), &[], &[]);
    assert_eq!(a, r);
    assert_eq!(kinds(&d), [(Kind::Shadow, "counter".into()), (Kind::Shadow, "removal".into())], "{d:?}");
    let (a, d) = run(&r, Some(&said(&["counter"])), &[], &[]);
    assert!(a == r && d.is_empty());
}

#[test]
fn merge_vetoes_unconfirmed_rule_roles_unless_keep_rule_or_certain() {
    plain_variant();
    let r = rule(&[("removal", false), ("draw", false)], true, &[RouteReason::RiskyRole]);
    let (a, d) = run(&r, Some(&said(&["draw"])), &[], &[]);
    assert_eq!(a.roles, ["draw"], "{a:?}");
    assert_eq!(a.primary, "draw");
    assert_eq!(kinds(&d), [(Kind::Veto, "removal".into())]);
    assert_eq!(d[0].rule, ["removal", "draw"]);
    assert_eq!(d[0].llm, ["draw"]);
    assert_eq!(d[0].route, [RouteReason::RiskyRole]);
    assert!(a.needs_llm && !a.llm_failed);
    // keep_rule protects the role, listed after the LLM's own
    let (a, d) = run(&r, Some(&said(&["draw"])), &["removal"], &[]);
    assert_eq!(a.roles, ["draw", "removal"], "{a:?}");
    assert!(d.is_empty());
    assert_eq!(a.evidence["removal"].source, Source::Rule);
}

#[test]
fn merge_keeps_a_certain_role_the_llm_omitted_and_logs_it() {
    plain_variant();
    let r = rule(&[("counter", true), ("draw", false)], true, &[RouteReason::RiskyRole]);
    let (a, d) = run(&r, Some(&said(&[])), &[], &[]);
    assert_eq!(a.roles, ["counter"], "{a:?}");
    assert_eq!(a.primary, "counter");
    assert_eq!(kinds(&d), [(Kind::Veto, "draw".into()), (Kind::CertainOmitted, "counter".into())], "{d:?}");
    // a certain role the LLM also proposes is simply kept once, in the LLM's order
    let (a, d) = run(&r, Some(&said(&["draw", "counter"])), &[], &[]);
    assert_eq!(a.roles, ["draw", "counter"], "{a:?}");
    assert!(d.is_empty());
}

#[test]
fn merge_drops_an_llm_role_whose_guard_fails_and_logs_it() {
    plain_variant();
    // no rule roles at all (NoRole): the LLM's guarded roles are all there is
    let r = rule(&[], true, &[RouteReason::NoRole]);
    let (a, d) = run(&r, Some(&said(&["draw", "wipe"])), &[], &[("wipe", 1)]);
    assert_eq!(a.roles, ["draw"], "{a:?}");
    assert_eq!(kinds(&d), [(Kind::GuardDrop, "wipe".into())]);
    assert_eq!(d[0].route, [RouteReason::NoRole]);
    // an unciteable line (out of range) is the same
    let mut l = said(&["draw"]);
    l.evidence.insert("draw".into(), 40);
    let (a, d) = run(&r, Some(&l), &[], &[]);
    assert!(a.roles.is_empty() && a.primary.is_empty(), "{a:?}");
    assert_eq!(kinds(&d), [(Kind::GuardDrop, "draw".into())]);
    // when the rules found the same role themselves it survives through their own evidence
    let r = rule(&[("wipe", false)], true, &[RouteReason::RiskyRole]);
    let (a, d) = run(&r, Some(&said(&["wipe"])), &[], &[("wipe", 0)]);
    assert_eq!(a.roles, ["wipe"]);
    assert_eq!(a.evidence["wipe"].source, Source::Rule);
    assert!(d.is_empty());
}

#[test]
fn merge_takes_guarded_llm_roles_for_a_card_with_no_rule_roles_with_llm_evidence() {
    plain_variant();
    let r = rule(&[], true, &[RouteReason::NoRole]);
    let (a, d) = run(&r, Some(&said(&["tokens", "draw"])), &[], &[]);
    assert_eq!(a.roles, ["tokens", "draw"]);
    assert_eq!(a.primary, "tokens");
    let e = &a.evidence["draw"];
    assert_eq!((e.source, e.certain, e.rule.as_str(), e.line, e.face), (Source::Llm, false, "llm", Some(1), Some(101)));
    assert!(d.is_empty());
}

#[test]
fn merge_orders_the_llm_primary_first_then_llm_order_then_certain_then_kept_rule_roles() {
    plain_variant();
    let r = rule(&[("counter", true), ("removal", false), ("draw", false)], true, &[RouteReason::RiskyRole]);
    let mut l = said(&["wipe", "tokens"]);
    l.primary = Some("tokens".into());
    let (a, d) = run(&r, Some(&l), &["removal", "draw"], &[]);
    // tokens (LLM primary), wipe (LLM order), counter (certain), removal and draw (kept rule roles): one too many
    assert_eq!(a.roles, ["tokens", "wipe", "counter", "removal"], "{a:?}");
    assert_eq!(a.primary, "tokens");
    assert_eq!(
        kinds(&d).iter().filter(|(k, _)| *k == Kind::Overflow).collect::<Vec<_>>(),
        [&(Kind::Overflow, "draw".to_string())]
    );
    // a primary that did not survive (its guard failed) falls back to the LLM's order
    let (a, _) = run(&rule(&[], true, &[RouteReason::NoRole]), Some(&l), &[], &[("tokens", 1)]);
    assert_eq!((a.roles.clone(), a.primary.as_str()), (vec!["wipe".to_string()], "wipe"));
}

#[test]
fn merge_truncates_to_four_logs_the_overflow_and_never_cuts_a_certain_role() {
    plain_variant();
    let r = rule(&[("counter", true), ("tutor", true)], true, &[RouteReason::Overflow]);
    let (a, d) = run(&r, Some(&said(&["removal", "wipe", "draw", "tokens"])), &[], &[]);
    assert_eq!(a.roles, ["removal", "wipe", "counter", "tutor"], "{a:?}");
    assert_eq!(a.evidence.len(), 4);
    let overflow: Vec<_> = d.iter().filter(|d| d.kind == Kind::Overflow).map(|d| d.role.as_str()).collect();
    assert_eq!(overflow, ["tokens", "draw"]);
    assert!(d.iter().any(|d| d.kind == Kind::CertainOmitted && d.role == "counter"));
}

#[test]
fn merge_marks_llm_failed_when_there_is_no_answer_and_keeps_the_rule_roles() {
    plain_variant();
    let r = rule(&[("removal", false)], true, &[RouteReason::RiskyRole]);
    let (a, d) = run(&r, None, &[], &[]);
    assert!(a.llm_failed && a.needs_llm);
    assert_eq!((&a.roles, &a.evidence, &a.route), (&r.roles, &r.evidence, &r.route));
    assert!(d.is_empty());
    // a card the rules decided is never marked, whatever the LLM did
    let (a, _) = run(&rule(&[("removal", false)], false, &[]), None, &[], &[]);
    assert!(!a.llm_failed);
}

// ------------------------------------------------------------------------------------ the shared line functions

#[test]
fn line_text_and_prompt_lines_are_what_the_rules_and_guards_read() {
    plain_variant();
    let i = Index::load(&PathBuf::from(FIXTURE)).unwrap();
    let card = |n: &str| &i.cards[i.lookup(n)[0] as usize];
    let bolt = card("Lightning Bolt");
    assert_eq!(roles::line_text(bolt, 0), "~ deals 3 damage to any target.");
    assert_eq!(roles::prompt_lines(bolt), [(0, "~ deals 3 damage to any target.".to_string())]);
    let shot = card("Skywhaler's Shot");
    assert_eq!(
        roles::line_text(shot, 0),
        "Destroy target creature with power 3 or greater. Scry 1.",
        "reminder text is stripped"
    );
    // an MDFC's land back and a planeswalker's ultimate are skipped; original indices are kept
    let agadeem = card("Agadeem's Awakening // Agadeem, the Undercrypt");
    assert_eq!(roles::prompt_lines(agadeem).iter().map(|(n, _)| *n).collect::<Vec<_>>(), [0]);
    assert!(!roles::skipped(agadeem, 0) && roles::skipped(agadeem, 1) && roles::skipped(agadeem, 2));
    let elspeth = card("Elspeth, Sun's Champion");
    assert_eq!(roles::prompt_lines(elspeth).iter().map(|(n, _)| *n).collect::<Vec<_>>(), [0, 1]);
    assert!(roles::skipped(elspeth, 2) && !roles::skipped(elspeth, 1));
    assert!(roles::skipped(elspeth, 99), "out of range counts as skipped");
    assert_eq!(roles::line_text(elspeth, 99), "");
    // the classifier agrees: it never cites a skipped line
    let classifier = roles::Classifier::new().unwrap();
    for c in i.cards.iter().filter(|c| roles::is_deck_card(c) && c.legal == Legal::Legal) {
        for e in classifier.classify(c).unwrap().evidence.values() {
            assert!(!roles::skipped(c, e.line.unwrap()), "{} cites a skipped line", c.raw.name);
        }
    }
}

#[test]
fn llm_evidence_is_attached_only_when_it_cites_a_readable_line_that_passes_the_guard() {
    plain_variant();
    let mut i = Index::load(&PathBuf::from(FIXTURE)).unwrap();
    let rules = roles::generate(&i, "").unwrap();
    let id = |n: &str| i.cards[i.lookup(n)[0] as usize].raw.oracle_id.clone();
    let (shot, agadeem) = (id("Skywhaler's Shot"), id("Agadeem's Awakening // Agadeem, the Undercrypt"));
    let llm_evidence = |face: Option<usize>, line: usize| Evidence {
        source: Source::Llm,
        certain: false,
        rule: "llm".into(),
        face,
        line: Some(line),
    };
    let with = |card: &str, role: &str, e: Evidence| {
        let mut m = rules.clone();
        // a table with LLM evidence names the prompt it was merged under
        m.meta.prompt_fnv = llm::prompt_fnv(&Taxonomy::parse(TAXONOMY).unwrap());
        let a = m.get_mut(card).unwrap();
        a.roles = vec![role.into()];
        a.primary = role.into();
        a.evidence = [(role.to_string(), e)].into();
        m
    };
    // valid: scry on a Skywhaler's Shot line
    i.attach_roles(&with(&shot, "selection", llm_evidence(None, 0))).unwrap();
    // guard failure, out-of-range line, wrong face, skipped line, certain flag and a foreign rule name are all refused
    assert!(
        i.attach_roles(&with(&shot, "counter", llm_evidence(None, 0))).is_err(),
        "counter's guard fails on the line"
    );
    assert!(i.attach_roles(&with(&shot, "selection", llm_evidence(None, 9))).is_err());
    assert!(i.attach_roles(&with(&shot, "selection", llm_evidence(Some(1), 0))).is_err());
    let land_back = llm_evidence(Some(1), 2);
    assert!(i.attach_roles(&with(&agadeem, "ramp", land_back)).is_err(), "an MDFC land back is skipped");
    let mut certain = llm_evidence(None, 0);
    certain.certain = true;
    assert!(i.attach_roles(&with(&shot, "selection", certain)).is_err());
    let mut named = llm_evidence(None, 0);
    named.rule = "single-removal".into();
    assert!(i.attach_roles(&with(&shot, "selection", named)).is_err());
}

#[test]
fn load_verifies_the_table_hash_and_the_overrides_path_is_one_resolver() {
    plain_variant();
    let e = Env::new();
    e.command(&["--rules"]).unwrap();
    let path = e.dir().join("roles.json");
    roles::load(&path).unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    let at = bytes.len() - 20;
    bytes[at] = if bytes[at] == b'x' { b'y' } else { b'x' };
    std::fs::write(&path, &bytes).unwrap();
    assert!(roles::load(&path).unwrap_err().contains("damaged"));
    assert!(e.command(&["--stats"]).is_err());
    // one resolver for the overrides file: `--rules` and `Index::load` read the one beside `--data-dir`'s oracle.json
    std::fs::write(e.dir().join("roles_overrides.toml"), "[[card]]\nname='Counterspell'\nroles=['draw']\nnote='n'\n")
        .unwrap();
    e.command(&["--rules"]).unwrap();
    let map = roles::load(&path).unwrap();
    assert_eq!(map.meta.rules_fnv, e.index().rules_fnv, "no false stale from two readers of two paths");
    assert_eq!(roles::overrides_path(&e.dir().join("oracle.json")), e.dir().join("roles_overrides.toml"));
    e.index().attach_roles(&map).unwrap();
}

// ------------------------------------------------------------------------------------------- the taxonomy prompt

#[test]
fn taxonomy_prompt_text_hash_and_guards() {
    plain_variant();
    let t = Taxonomy::parse(TAXONOMY).unwrap();
    let text = t.prompt_text();
    for id in roles::IDS {
        assert_eq!(text.matches(&format!("\n- {id}: ")).count(), 1, "{id} appears once as a role line");
        assert!(t.guard(id).unwrap().is_match("anything").is_ok());
    }
    // the frozen configuration: add-only (keep_rule everywhere), the LLM off for the 8 weak roles
    let kept: Vec<&str> = t.role.iter().filter(|r| r.keep_rule).map(|r| r.id.as_str()).collect();
    assert_eq!(kept, roles::IDS);
    let off: Vec<&str> = t.role.iter().filter(|r| !r.llm_add).map(|r| r.id.as_str()).collect();
    assert_eq!(off, ["ramp", "fixing", "draw", "selection", "sac-outlet", "blink", "finisher", "evasion"]);
    assert!(t.keep_rule("removal") && t.keep_rule("tutor") && !t.llm_add("ramp") && t.llm_add("removal"));
    // the boilerplate that used to be repeated in every definition is said once
    assert_eq!(text.matches("modal double-faced").count(), 1, "{text}");
    assert_eq!(text.matches("Planeswalker ultimates have been removed").count(), 1);
    for clause in &t.clause {
        assert_eq!(text.matches(clause.text.trim()).count(), 1, "{}", clause.id);
    }
    // hash: stable, tied to the prompt-visible text, blind to guards and keep_rule
    assert_eq!(t.hash(), Taxonomy::parse(TAXONOMY).unwrap().hash());
    let edited = TAXONOMY.replacen("counters a spell or an ability on the stack.", "counters a spell.", 1);
    assert_ne!(edited, TAXONOMY);
    assert_ne!(Taxonomy::parse(&edited).unwrap().hash(), t.hash());
    let guard = TAXONOMY.replacen("(?i)counter|exile.*spells?", "(?i)counter|exile.*spells? ", 1);
    assert_eq!(Taxonomy::parse(&guard).unwrap().hash(), t.hash());
    let keep =
        TAXONOMY.replacen("guard = '(?i)discard'\nkeep_rule = true", "guard = '(?i)discard'\nkeep_rule = false", 1);
    let kept = Taxonomy::parse(&keep).unwrap();
    assert!(!kept.keep_rule("discard") && kept.hash() == t.hash());
    // malformed taxonomies are errors
    assert!(
        Taxonomy::parse(&TAXONOMY.replacen(
            "definition = \"makes opponents discard: each opponent or a target opponent discards, reveal-and-discard.\"",
            "definition = \"makes opponents\\ndiscard.\"",
            1
        ))
        .is_err()
    );
    assert!(Taxonomy::parse(&TAXONOMY.replacen("roles = [\"ramp\", \"tokens\"]", "roles = [\"nope\"]", 1)).is_err());
}

#[test]
fn no_definition_names_a_card_because_the_model_has_no_internet() {
    plain_variant();
    let t = Taxonomy::parse(TAXONOMY).unwrap();
    let text = t.prompt_text();
    let mut names: Vec<String> = vec![];
    for (path, key) in [
        (concat!(env!("CARGO_MANIFEST_DIR"), "/../../../data/knowledge/roles_gold.toml"), "name = \""),
        (concat!(env!("CARGO_MANIFEST_DIR"), "/../../../data/knowledge/roles_gold_regression.toml"), "name = \""),
    ] {
        for line in std::fs::read_to_string(path).unwrap().lines() {
            if let Some(n) = line.strip_prefix(key) {
                names.push(n.trim_end_matches('"').to_string());
            }
        }
    }
    let fixture: Value = serde_json::from_str(&std::fs::read_to_string(FIXTURE).unwrap()).unwrap();
    names.extend(fixture["cards"].as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap().to_string()));
    // a card named like an ordinary word cannot be told from the word; every longer name must be absent
    let hits: Vec<_> = names.iter().filter(|n| n.chars().count() >= 8 && text.contains(n.as_str())).collect();
    assert!(hits.is_empty(), "a card name inside a definition: {hits:?}");
    for name in [
        "Mercy Killing",
        "Y'shtola",
        "Entomb",
        "Intuition",
        "Rule of Law",
        "Drannith",
        "Esper Sentinel",
        "Throne of Eldraine",
        "Scroll Rack",
        "Boots",
    ] {
        assert!(!text.contains(name), "{name} is a card name");
    }
}

#[test]
fn the_system_prompt_fits_its_token_budget_and_its_examples_are_valid_and_not_from_a_gold_set() {
    plain_variant();
    let t = Taxonomy::parse(TAXONOMY).unwrap();
    let system = llm::system_prompt(&t);
    let preface = llm::preface_text(&t);
    let (chars, tokens) = (system.chars().count(), llm::estimate_tokens(&system));
    let taxonomy_chars = t.prompt_text().chars().count();
    println!(
        "SYSTEM PROMPT: {chars} chars, about {tokens} tokens (chars/4); the taxonomy part is {taxonomy_chars} chars; \
         with the example turn {} chars, about {} tokens",
        preface.chars().count(),
        llm::estimate_tokens(&preface)
    );
    println!("per-role definitions (definition + role clauses), chars:");
    for r in &t.role {
        println!("  {:<12} {}", r.id, r.definition.len() + r.clauses.iter().map(|c| c.len() + 1).sum::<usize>());
    }
    assert!(llm::estimate_tokens(&preface) < 5000, "system prompt and example turn stay under 5k tokens");
    assert!(taxonomy_chars < 17_000, "the YES/NO taxonomy is {taxonomy_chars} chars");
    for v in [llm::Variant::Plain, llm::Variant::Think] {
        let (eu, ea) = llm::example_turn_v(v);
        let example_tokens = llm::estimate_tokens(&eu) + llm::estimate_tokens(&ea);
        let sys = llm::system_prompt_v(&t, v);
        println!(
            "VARIANT {}: system {} chars = {} tokens; example turn {} chars = {} tokens; system + example {} tokens",
            v.name(),
            sys.chars().count(),
            llm::estimate_tokens(&sys),
            eu.chars().count() + ea.chars().count(),
            example_tokens,
            llm::estimate_tokens(&llm::preface_text_v(&t, v)),
        );
        assert!(example_tokens < 2500, "{v:?}: the example turn is {example_tokens} tokens");
        assert!(llm::estimate_tokens(&llm::preface_text_v(&t, v)) < 5000, "{v:?}");
    }
    // every worked example is a valid answer under the very validation the real answers face
    let lines: Vec<Vec<(usize, String)>> =
        llm::EXAMPLES.iter().map(|e| e.lines.iter().map(|(i, t)| (*i, t.to_string())).collect()).collect();
    let (example_user, content) = llm::example_turn();
    let cards: Vec<(String, &[(usize, String)])> =
        lines.iter().enumerate().map(|(i, l)| (llm::batch_id(i), l.as_slice())).collect();
    let parsed = llm::validate_output(&content, &cards, &t).unwrap();
    assert!(parsed.len() == 10 && parsed.iter().all(|r| r.is_ok()), "{parsed:?}");
    // the guards pass on every cited example line, so the examples never show a role the guard would drop
    for (ex, got) in llm::EXAMPLES.iter().zip(&parsed) {
        for (r, n) in &got.as_ref().unwrap().evidence {
            let text = &ex.lines.iter().find(|(i, _)| i == n).unwrap().1;
            assert!(t.guard(r).unwrap().is_match(text).unwrap(), "{}: {r}", ex.name);
        }
    }
    assert!(content.starts_with("{\"cards\":[{\"id\":\"a0\",\"roles\":[{\"role\":\"removal\",\"line\":0}]}"));
    let schema = llm::response_schema_text(&t, 3);
    let at = |key: &str| schema.find(&format!("\"{key}\":{{")).unwrap();
    assert!(at("id") < at("roles") && at("roles") < at("role") && at("role") < at("line"), "{schema}");
    let body = llm::chat_body("m", "sys", "usr", &schema, 3);
    let parsed: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["max_tokens"], 160 * 3 + 64);
    assert!(body.contains(&schema), "the schema text goes into the body untouched");
    // system, example user, example assistant, real user
    let messages: Vec<(&str, &str)> = parsed["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| (m["role"].as_str().unwrap(), m["content"].as_str().unwrap()))
        .collect();
    assert_eq!(
        messages,
        [("system", "sys"), ("user", example_user.as_str()), ("assistant", content.as_str()), ("user", "usr")]
    );
    // the fingerprint covers the example turn too
    assert_eq!(llm::prompt_fnv(&t), roles::fingerprint(llm::preface_text(&t).as_bytes()));
    // none of the example cards is in the tuning or regression gold sets
    for path in ["roles_gold.toml", "roles_gold_regression.toml"] {
        let gold = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data/knowledge").join(path),
        )
        .unwrap();
        for e in &llm::EXAMPLES {
            assert!(!gold.contains(&format!("name = \"{}\"", e.name)), "{} is in {path}", e.name);
        }
    }
    // a sample user message
    let bolt = [(0, "~ deals 3 damage to any target.".to_string())];
    let items: Vec<(String, &str, &[(usize, String)])> = vec![("a0".into(), "Instant", &bolt[..])];
    let sample = llm::user_message(&items);
    assert_eq!(sample, "Classify these 1 cards.\n[a0] Instant\n0: ~ deals 3 damage to any target.\n");
    if std::env::var_os("CARDGEN_PRINT_PROMPT").is_some() {
        // a real batch from the fixture catalog (five cards and a two-faced one), as the request would carry it
        let i = Index::load(&PathBuf::from(FIXTURE)).unwrap();
        let names: Vec<&str> = FIVE.iter().copied().chain(["Emeritus of Conflict // Lightning Bolt"]).collect();
        let cards: Vec<_> = names.iter().map(|n| &i.cards[i.lookup(n)[0] as usize]).collect();
        let owned: Vec<(Vec<(usize, String)>, std::collections::BTreeMap<usize, String>)> =
            cards.iter().map(|c| (roles::prompt_lines(c), roles::prompt_headers(c))).collect();
        let shown: Vec<llm::Shown> = cards
            .iter()
            .zip(&owned)
            .enumerate()
            .map(|(n, (c, (l, h)))| llm::Shown {
                id: llm::batch_id(n),
                type_line: c.raw.type_line.as_str(),
                lines: l.as_slice(),
                headers: h,
            })
            .collect();
        let user = llm::user_message_cards(&shown);
        println!("=== SAMPLE USER MESSAGE ===\n{user}");
        for v in [llm::Variant::Plain, llm::Variant::Think] {
            let body = llm::chat_body_v(
                "Qwen3-Coder-30B-A3B-Instruct-UD-Q4_K_XL.gguf",
                &llm::system_prompt_v(&t, v),
                &user,
                &llm::response_schema_text_v(&t, shown.len(), v),
                shown.len(),
                v,
            );
            println!("=== FULL REQUEST BODY, variant {} ({} chars) ===\n{body}", v.name(), body.len());
        }
    }
    assert_eq!(
        (llm::batch_id(0), llm::batch_id(9), llm::batch_id(10), llm::batch_id(25)),
        ("a0".into(), "a9".into(), "b0".into(), "c5".into())
    );
    let schema = llm::response_schema(&t, 3);
    assert_eq!(schema["properties"]["cards"]["minItems"], 3);
}

#[test]
fn a_later_face_is_announced_by_an_unnumbered_header_and_the_key_text_carries_it() {
    plain_variant();
    let i = Index::load(&PathBuf::from(FIXTURE)).unwrap();
    let card = |n: &str| &i.cards[i.lookup(n)[0] as usize];
    let two = card("Emeritus of Conflict // Lightning Bolt");
    let headers = roles::prompt_headers(two);
    assert_eq!(headers.len(), 1, "{headers:?}");
    let (at, header) = headers.iter().next().unwrap();
    assert_eq!(header, "(second face: Instant)");
    let lines = roles::prompt_lines(two);
    assert!(lines.iter().any(|(n, _)| n == at) && *at > 0);
    let shown = [llm::Shown { id: "a0".into(), type_line: &two.raw.type_line, lines: &lines, headers: &headers }];
    let user = llm::user_message_cards(&shown);
    let expect = format!("(second face: Instant)\n{at}: ");
    assert!(user.contains(&expect), "{user}");
    assert!(llm::card_text(two).contains(&expect), "the face header is part of what the cache key hashes");
    // a one-faced card, and an MDFC whose land back is skipped, get no header
    assert!(roles::prompt_headers(card("Lightning Bolt")).is_empty());
    assert!(roles::prompt_headers(card("Agadeem's Awakening // Agadeem, the Undercrypt")).is_empty());
}

#[test]
fn the_cache_key_is_the_card_text_the_prompt_and_the_model_and_nothing_else() {
    plain_variant();
    let k = llm::cache_key("Instant\n0: Counter target spell.", "p1", "m1");
    assert_eq!(k.len(), 16);
    assert_eq!(k, llm::cache_key("Instant\n0: Counter target spell.", "p1", "m1"));
    for other in [
        llm::cache_key("Instant\n0: Counter target spell!", "p1", "m1"),
        llm::cache_key("Instant\n0: Counter target spell.", "p2", "m1"),
        llm::cache_key("Instant\n0: Counter target spell.", "p1", "m2"),
    ] {
        assert_ne!(k, other);
    }
}

// ------------------------------------------------------------------------------------------------ validation stage

#[test]
fn validate_stage_merged_scores_the_saved_merged_map_without_override_cards() {
    plain_variant();
    let e = Env::new();
    let i = e.index();
    let id = |n: &str| i.cards[i.lookup(n)[0] as usize].raw.oracle_id.clone();
    std::fs::write(
        e.0.join("gold.toml"),
        format!(
            "[[card]]\nname='Counterspell'\noracle_id='{}'\nroles=['counter']\nprimary='counter'\n\n\
             [[card]]\nname=\"Skywhaler's Shot\"\noracle_id='{}'\nroles=['removal']\nprimary='removal'\n",
            id("Counterspell"),
            id("Skywhaler's Shot")
        ),
    )
    .unwrap();
    let gold = e.0.join("gold.toml");
    let gold = gold.to_str().unwrap();
    e.command(&["--rules"]).unwrap();
    // a rules-only table cannot be scored as merged
    assert!(e.command(&["--validate", "--gold", gold, "--stage", "merged"]).unwrap_err().contains("--merge"));
    let fake = Fake::start(good());
    e.llm(&fake, &[]).unwrap();
    e.command(&["--merge"]).unwrap();
    let (text, _) = e.command(&["--validate", "--gold", gold, "--stage", "merged"]).unwrap();
    for needle in [
        "STAGE: merged",
        "ROUTED GOLD CARDS",
        "rules alone",
        "merged stage",
        "LLM STAGE",
        "last run: model fake-model-1",
        "last merge:",
    ] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
    let (json_text, passed) = e.command(&["--validate", "--gold", gold, "--stage", "merged", "--json"]).unwrap();
    let v: Value = serde_json::from_str(&json_text).unwrap();
    assert_eq!(v["stage"], "merged");
    assert_eq!(v["primary_total"], 2);
    assert!(v["routed_cards"].as_u64().unwrap() >= 1);
    assert_eq!(v["llm"]["run"]["classified"], 5);
    assert!(v["routed_scored_micro"]["tp"].is_number());
    let _ = passed;
    // the rules stage still scores the rules alone
    let (rules_text, _) = e.command(&["--validate", "--gold", gold]).unwrap();
    assert!(rules_text.starts_with("STAGE: rules"));
    // an override card is left out of the merged score, so an override can never improve validation
    std::fs::write(
        e.dir().join("roles_overrides.toml"),
        "[[card]]\nname=\"Skywhaler's Shot\"\nroles=['removal']\nnote='n'\n",
    )
    .unwrap();
    e.command(&["--rules"]).unwrap();
    e.command(&["--merge"]).unwrap();
    let (json_text, _) = e.command(&["--validate", "--gold", gold, "--stage", "merged", "--json"]).unwrap();
    let v: Value = serde_json::from_str(&json_text).unwrap();
    assert_eq!(v["primary_total"], 1, "the overridden gold card is excluded");
}

// ------------------------------------------------------------------------------------------- round 2: review fixes

#[test]
fn a_second_run_waits_for_the_lock_and_a_stale_lock_is_named() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    // a lock left behind (a crashed run): the run gives up with the file's name once its wait is over
    std::fs::write(e.dir().join("llm_roles.lock"), format!("{}\n", std::process::id())).unwrap();
    // SAFETY of the env var: only this test makes a run meet an existing lock
    unsafe { std::env::set_var("CARDGEN_LLM_LOCK_WAIT_SECS", "1") };
    let err = e.llm(&fake, &[]).unwrap_err();
    assert!(err.contains("llm_roles.lock") && err.contains("another `roles --llm`"), "{err}");
    assert_eq!(fake.chats(), 0, "nothing is classified without the lock");
    // the lock file of a finished run is gone, and the pending target has its own lock
    std::fs::remove_file(e.dir().join("llm_roles.lock")).unwrap();
    e.llm(&fake, &["--pending"]).unwrap();
    assert!(!e.dir().join("llm_roles.lock").exists() && !e.dir().join("llm_roles.pending.lock").exists());
}

#[test]
fn llm_add_false_switches_the_llm_off_for_a_role_without_touching_the_cache() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    e.llm(&fake, &[]).unwrap();
    let on = e.merged();
    let shot = |m: &roles::RoleMap| assignment_of(&e.index(), m, "Skywhaler's Shot").roles.clone();
    assert_eq!(shot(&on), ["removal", "selection"]);
    // `selection` off: the LLM may not add it (its answers stay cached and the prompt, hence the key, is unchanged)
    let toml = std::fs::read_to_string(e.dir().join("roles.toml")).unwrap();
    let off = toml.replacen(
        "keep_rule = true\n\n[[role]]\nid = \"tutor\"",
        "keep_rule = true\nllm_add = false\n\n[[role]]\nid = \"tutor\"",
        1,
    );
    assert_ne!(toml, off);
    std::fs::write(e.dir().join("roles.toml"), off).unwrap();
    let out = e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 1, "still warm: {out}");
    assert_eq!(shot(&e.merged()), ["removal"]);
    let report = std::fs::read_to_string(e.dir().join("reports/roles-disagreements.txt")).unwrap();
    assert!(report.contains("GuardDrop | Skywhaler's Shot | selection"), "{report}");
}

#[test]
fn no_pending_leaves_the_scheduled_jobs_rows_out_of_a_merge() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    e.llm(&fake, &["--pending"]).unwrap();
    assert!(!e.dir().join("llm_roles.jsonl").exists());
    let stats = |args: &[&str]| -> llm::MergeStats {
        e.command(args).unwrap();
        serde_json::from_slice(&std::fs::read(e.dir().join("reports/merge.json")).unwrap()).unwrap()
    };
    assert!(stats(&["--merge"]).with_answer > 0);
    // with no tracked rows there is no model to merge, and nothing is answered
    assert_eq!(stats(&["--merge", "--no-pending"]).with_answer, 0);
    assert!(e.command(&["--rules", "--no-pending"]).is_err(), "--no-pending belongs to --merge");
}

#[test]
fn the_model_key_is_the_file_name_a_warm_run_keeps_the_last_real_report_and_the_cache_is_probed_first() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    fake.set_model("/models/qwen/Qwen3-Coder-30B.gguf");
    e.llm(&fake, &[]).unwrap();
    assert!(e.rows().iter().all(|r| r.model == "Qwen3-Coder-30B.gguf"));
    assert!(fake.bodies.lock().unwrap().iter().all(|b| b["model"] == "Qwen3-Coder-30B.gguf"));
    // another path to the same file name is the same model: warm
    fake.set_model("/elsewhere/Qwen3-Coder-30B.gguf");
    let before: Value = serde_json::from_slice(&std::fs::read(e.dir().join("reports/llm-run.json")).unwrap()).unwrap();
    e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 1);
    // the warm run did not overwrite the report of the last real run; it wrote its own
    let after: Value = serde_json::from_slice(&std::fs::read(e.dir().join("reports/llm-run.json")).unwrap()).unwrap();
    assert_eq!(before, after);
    assert_eq!(before["classified"], 5);
    let last: Value =
        serde_json::from_slice(&std::fs::read(e.dir().join("reports/llm-run-last.json")).unwrap()).unwrap();
    assert_eq!((last["classified"].clone(), last["chat_requests"].clone()), (json!(0), json!(0)));
    // `--model` is for a dry run only, and an empty one is refused
    assert!(e.llm(&fake, &["--model", "x"]).unwrap_err().contains("--dry-run"));
    assert!(e.llm(&fake, &["--dry-run", "--model", ""]).is_err());
}

#[test]
fn only_legal_cards_are_flagged_and_the_merge_reports_failed_not_run_and_not_legal_apart() {
    plain_variant();
    let e = Env::new();
    let path = e.dir().join("oracle.json");
    let mut doc: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for c in doc["cards"].as_array_mut().unwrap() {
        if c["name"] == "Skywhaler's Shot" {
            c["legalities"]["commander"] = json!("not_legal");
        }
    }
    std::fs::write(&path, doc.to_string()).unwrap();
    let fake = Fake::start(good());
    let out = e.llm(&fake, &[]).unwrap();
    assert!(out.contains("4 classified"), "the non-legal card is never sent: {out}");
    let map = e.merged();
    let i = e.index();
    let shot = assignment_of(&i, &map, "Skywhaler's Shot");
    assert!(shot.needs_llm && !shot.llm_failed, "a non-legal routed card is not llm_failed: {shot:?}");
    let stats: llm::MergeStats =
        serde_json::from_slice(&std::fs::read(e.dir().join("reports/merge.json")).unwrap()).unwrap();
    assert_eq!((stats.not_legal, stats.failed), (1, 0), "{stats:?}");
    assert_eq!(stats.routed, stats.with_answer + stats.llm_failed);
    assert_eq!(stats.llm_failed, stats.failed + stats.not_run);
}

#[test]
fn a_damaged_cache_row_with_an_unknown_role_is_an_error_naming_file_and_line() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    e.llm(&fake, &[]).unwrap();
    let text = std::fs::read_to_string(e.dir().join("llm_roles.jsonl")).unwrap();
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    let mut row: Value = serde_json::from_str(&lines[1]).unwrap();
    row["roles"] = json!(["banana"]);
    row["evidence"] = json!({"banana": 0});
    lines[1] = row.to_string();
    std::fs::write(e.dir().join("llm_roles.jsonl"), lines.join("\n") + "\n").unwrap();
    let err = e.command(&["--merge"]).unwrap_err();
    assert!(err.contains("llm_roles.jsonl line 2") && err.contains("banana"), "{err}");
}

#[test]
fn a_merged_table_is_tied_to_the_prompt_and_model_it_was_built_with() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    e.llm(&fake, &[]).unwrap();
    let map = e.merged();
    let path = e.dir().join("roles.json");
    // editing the prompt fingerprint or the model by hand (even re-writing the table through `bytes`) is refused
    let mut other = map.clone();
    other.meta.prompt_fnv = "0123456789abcdef".into();
    std::fs::write(&path, roles::bytes(&other).unwrap()).unwrap();
    let err = e.index().attach_roles(&roles::load(&path).unwrap()).unwrap_err();
    assert!(err.contains("another LLM prompt"), "{err}");
    assert!(e.command(&["--validate", "--stage", "merged"]).unwrap_err().contains("another LLM prompt"));
    // a hand edit of the metadata line alone breaks the table hash
    std::fs::write(&path, roles::bytes(&map).unwrap()).unwrap();
    let text = std::fs::read_to_string(&path).unwrap().replacen(&map.meta.model, "x-model", 1);
    std::fs::write(&path, text).unwrap();
    assert!(roles::load(&path).unwrap_err().contains("damaged"));
    // a rules-only table cannot claim LLM evidence
    let mut claim = e.merged();
    claim.meta.prompt_fnv.clear();
    assert!(e.index().attach_roles(&claim).is_err());
}

#[test]
fn the_taxonomy_for_llm_and_merge_comes_from_the_bytes_the_index_checks_guards_against() {
    plain_variant();
    let e = Env::new();
    let i = e.index();
    assert_eq!(i.taxonomy_text(), test_taxonomy());
    // an unreadable roles.toml beside the catalog is an error for the LLM commands, not a silent fallback
    std::fs::write(e.dir().join("roles.toml"), "version = 1\n").unwrap();
    assert!(e.command(&["--merge"]).is_err());
}

// ------------------------------------------------------------------------------------------------ prompt round 1

#[test]
fn every_role_has_a_yes_line_and_a_no_line_and_the_procedure_is_in_the_answer_rules() {
    plain_variant();
    let t = Taxonomy::parse(TAXONOMY).unwrap();
    let text = t.prompt_text();
    for r in &t.role {
        assert!(!r.no.trim().is_empty(), "{} has no NO line", r.id);
        let at = text.find(&format!("\n- {}: YES: ", r.id)).unwrap_or_else(|| panic!("{} YES", r.id));
        let line_end = text[at + 1..].find("\n- ").map_or(text.len(), |n| at + 1 + n);
        assert!(text[at..line_end].contains("\n  NO: "), "{}: NO follows YES", r.id);
    }
    // the incidental-effect exclusions the pilot showed the model breaking are said in the NO lines
    for (role, needle) in [
        ("protection", "keywords printed on the card"),
        ("anthem", "this creature only"),
        ("ramp", "taps for one mana"),
        ("draw", "cycling"),
        ("draw", "scry"),
        ("sac-outlet", "the card itself"),
        ("removal", "temporary steal"),
        ("fixing", "fetchland"),
        ("selection", "scry 1"),
    ] {
        let r = t.role.iter().find(|r| r.id == role).unwrap();
        assert!(
            r.no.contains(needle) || r.definition.contains(needle) || r.clauses.iter().any(|c| c.contains(needle)),
            "{role}: {needle}"
        );
    }
    for v in [llm::Variant::Plain, llm::Variant::Think] {
        let system = llm::system_prompt_v(&t, v);
        assert!(system.contains("Work through each card line by line"), "{v:?}");
        assert!(system.contains("do not stop after the first one"), "{v:?}");
    }
}

#[test]
fn the_think_variant_has_its_own_schema_answer_validation_prompt_and_fingerprint() {
    plain_variant();
    let t = Taxonomy::parse(TAXONOMY).unwrap();
    let (plain, think) = (llm::Variant::Plain, llm::Variant::Think);
    // the switch: plain by default, think by name, anything else is an error
    assert_eq!(llm::Variant::parse("").unwrap(), think, "think is the default");
    assert_eq!(llm::Variant::default(), think);
    assert_eq!(llm::Variant::parse("plain").unwrap(), plain);
    assert_eq!(llm::Variant::parse("think").unwrap(), think);
    assert!(llm::Variant::parse("Think").is_err() && llm::Variant::parse("deep").is_err());
    // different prompts and fingerprints, so cache rows never mix
    assert_ne!(llm::prompt_fnv_v(&t, plain), llm::prompt_fnv_v(&t, think));
    assert_ne!(llm::system_prompt_v(&t, plain), llm::system_prompt_v(&t, think));
    assert!(llm::system_prompt_v(&t, think).contains("\"why\": one short sentence (at most 160 characters)"));
    assert!(!llm::system_prompt_v(&t, plain).contains("why"));
    // the schema: `why` between id and roles, required, at most 160 characters; plain is unchanged
    let text = llm::response_schema_text_v(&t, 3, think);
    let at = |k: &str| text.find(&format!("\"{k}\":{{")).unwrap();
    assert!(at("id") < at("why") && at("why") < at("roles"), "{text}");
    let schema: Value = serde_json::from_str(&text).unwrap();
    let item = &schema["properties"]["cards"]["items"];
    assert_eq!(item["required"], json!(["id", "why", "roles"]));
    assert_eq!(item["properties"]["why"], json!({"type": "string", "maxLength": 160}));
    assert!(!llm::response_schema_text_v(&t, 3, plain).contains("why"));
    // the example answer carries a why for every example, in that order, and validates under think
    let (user, answer) = llm::example_turn_v(think);
    assert!(user.starts_with("Classify these 10 cards."));
    assert!(answer.starts_with("{\"cards\":[{\"id\":\"a0\",\"why\":\"Line 0 destroys one creature"), "{answer}");
    let lines: Vec<Vec<(usize, String)>> =
        llm::EXAMPLES.iter().map(|e| e.lines.iter().map(|(i, t)| (*i, t.to_string())).collect()).collect();
    let cards: Vec<(String, &[(usize, String)])> =
        lines.iter().enumerate().map(|(i, l)| (llm::batch_id(i), l.as_slice())).collect();
    let got = llm::validate_output_v(&answer, &cards, &t, think).unwrap();
    assert!(got.iter().all(|r| r.is_ok()), "{got:?}");
    assert!(got.iter().all(|r| r.as_ref().unwrap().why.as_deref().is_some_and(|w| w.chars().count() <= llm::WHY_MAX)));
    for e in &llm::EXAMPLES {
        assert!(e.why.chars().count() <= llm::WHY_MAX && !e.why.is_empty(), "{}", e.name);
    }
    // plain answers (no why) are refused under think, and why too long or not a string is a structural fault
    let (_, plain_answer) = llm::example_turn_v(plain);
    let refused = llm::validate_output_v(&plain_answer, &cards, &t, think).unwrap();
    assert!(refused.iter().all(|r| r.is_err()));
    let mut v: Value = serde_json::from_str(&answer).unwrap();
    v["cards"][0]["why"] = json!("x".repeat(161));
    v["cards"][1]["why"] = json!(7);
    let mixed = llm::validate_output_v(&v.to_string(), &cards, &t, think).unwrap();
    assert!(mixed[0].is_err() && mixed[1].is_err() && mixed[2].is_ok());
    // plain ignores a why it did not ask for
    assert!(llm::validate_output_v(&answer, &cards, &t, plain).unwrap().iter().all(|r| r.is_ok()));
    // the request body has the think schema and the larger token limit
    let body = llm::chat_body_v("m", "sys", "usr", &text, 3, think);
    let parsed: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["max_tokens"], 210 * 3 + 64);
    assert_eq!(parsed["messages"][2]["content"], answer.as_str());
}

#[test]
fn a_think_run_caches_its_why_under_its_own_fingerprint_and_plain_rows_stay_apart() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(Box::new(|_, body| {
        // answers carry a why when the request's schema asks for one
        let items = parse_batch(body);
        let think = body["response_format"]["json_schema"]["schema"].to_string().contains("\"why\"");
        let mut v = valid(&items);
        if think {
            for c in v["cards"].as_array_mut().unwrap() {
                let id = c["id"].clone();
                let roles = c["roles"].clone();
                *c = json!({"id": id, "why": "Line 0 decides it.", "roles": roles});
            }
        }
        Reply::Content(v.to_string())
    }));
    e.llm(&fake, &[]).unwrap();
    assert!(e.rows().iter().all(|r| r.why.is_none()));
    // the same cache under the think variant is cold, asks again and stores the why (a child process: the switch is
    // an environment variable, and the other tests of this process must not see it)
    let cardgen = |variant: &str, args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_cardgen"))
            .env_remove("CARDGEN_KNOWLEDGE_DIR")
            .env("CARDGEN_LLM_VARIANT", variant)
            .arg("--root")
            .arg(&e.0)
            .arg("roles")
            .args(args)
            .arg("--data-dir")
            .arg(e.dir())
            .output()
            .unwrap()
    };
    let ids = e.dir().join("ids.txt");
    let run = cardgen("think", &["--llm", "--ids", ids.to_str().unwrap(), "--endpoint", &fake.url]);
    let out = String::from_utf8_lossy(&run.stdout).to_string();
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(out.contains("prompt variant think") && out.contains("5 classified"), "{out}");
    assert_eq!(fake.chats(), 2);
    let rows = e.rows();
    assert_eq!(rows.len(), 10, "both variants' rows live side by side");
    assert_eq!(rows.iter().filter(|r| r.why.as_deref() == Some("Line 0 decides it.")).count(), 5);
    let merged = cardgen("think", &["--merge"]);
    assert!(String::from_utf8_lossy(&merged.stdout).contains("prompt variant think"));
    // the merged table carries the think prompt, and attaches whichever variant is set
    let map = roles::load(&e.dir().join("roles.json")).unwrap();
    assert_eq!(map.meta.prompt_fnv, llm::prompt_fnv_v(&Taxonomy::parse(TAXONOMY).unwrap(), llm::Variant::Think));
    e.index().attach_roles(&map).unwrap();
    // a bad variant name is an error before anything happens
    let bad = cardgen("deep", &["--llm", "--ids", ids.to_str().unwrap(), "--endpoint", &fake.url]);
    assert!(!bad.status.success() && String::from_utf8_lossy(&bad.stderr).contains("CARDGEN_LLM_VARIANT"));
    assert_eq!(fake.chats(), 2);
}

#[test]
fn the_merge_says_per_role_how_many_proposals_llm_add_false_dropped() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    e.llm(&fake, &[]).unwrap();
    let (out, _) = e.command(&["--merge"]).unwrap();
    assert!(!out.contains("llm_add = false"), "{out}");
    let toml = std::fs::read_to_string(e.dir().join("roles.toml")).unwrap();
    let off = toml.replacen(
        "keep_rule = true\n\n[[role]]\nid = \"tutor\"",
        "keep_rule = true\nllm_add = false\n\n[[role]]\nid = \"tutor\"",
        1,
    );
    assert_ne!(toml, off);
    std::fs::write(e.dir().join("roles.toml"), off).unwrap();
    let (out, _) = e.command(&["--merge"]).unwrap();
    assert!(out.contains("llm_add = false for selection: 1 LLM proposals dropped"), "{out}");
    let report = std::fs::read_to_string(e.dir().join("reports/roles-disagreements.txt")).unwrap();
    assert!(report.contains("LLM proposals dropped because llm_add = false for selection: 1"), "{report}");
    let stats: llm::MergeStats =
        serde_json::from_slice(&std::fs::read(e.dir().join("reports/merge.json")).unwrap()).unwrap();
    assert_eq!(stats.llm_add_dropped.get("selection"), Some(&1));
}

// ------------------------------------------------------------------------------------------- final configuration

#[test]
fn the_frozen_configuration_is_in_the_real_roles_toml_and_think_is_the_default() {
    plain_variant();
    let t = Taxonomy::parse(TAXONOMY).unwrap();
    assert_eq!(t.role.iter().filter(|r| r.keep_rule).count(), 26, "add-only: the LLM never removes a rule role");
    let off: Vec<&str> = t.role.iter().filter(|r| !r.llm_add).map(|r| r.id.as_str()).collect();
    assert_eq!(off, ["ramp", "fixing", "draw", "selection", "sac-outlet", "blink", "finisher", "evasion"]);
    assert!(TAXONOMY.contains("FROZEN CONFIGURATION") && TAXONOMY.contains("26 versus 27 errors"));
    // a bare environment means `think`; `plain` selects the old prompt
    assert_eq!(llm::Variant::default(), llm::Variant::Think);
    assert_eq!(llm::Variant::parse("plain").unwrap(), llm::Variant::Plain);
}

fn body_of(fake: &Fake, n: usize) -> Value {
    fake.bodies.lock().unwrap()[n].clone()
}

#[test]
fn no_think_adds_the_template_kwarg_to_every_request_and_keeps_its_rows_apart() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    e.llm(&fake, &[]).unwrap();
    assert!(body_of(&fake, 0).get("chat_template_kwargs").is_none(), "absent unless asked for");
    let out = e.llm(&fake, &["--no-think"]).unwrap();
    assert_eq!(fake.chats(), 2, "a no-think run does not reuse thinking rows: {out}");
    let b = body_of(&fake, 1);
    assert_eq!(b["chat_template_kwargs"], json!({"enable_thinking": false}));
    let raw = fake.raw.lock().unwrap()[1].clone();
    assert!(raw.contains("\"chat_template_kwargs\":{\"enable_thinking\":false}"), "exactly this text");
    assert_eq!(e.rows().len(), 10, "distinct keys");
    // recorded in the reports
    let report: Value = serde_json::from_slice(&std::fs::read(e.dir().join("reports/llm-run.json")).unwrap()).unwrap();
    assert_eq!(report["no_think"], true);
    let last: Value =
        serde_json::from_slice(&std::fs::read(e.dir().join("reports/llm-run-last.json")).unwrap()).unwrap();
    assert_eq!(last["no_think"], true);
    // a warm no-think run asks nothing; the fingerprints differ
    e.llm(&fake, &["--no-think"]).unwrap();
    assert_eq!(fake.chats(), 2);
    let t = Taxonomy::parse(TAXONOMY).unwrap();
    let v = llm::Variant::Plain;
    assert_ne!(llm::prompt_fnv_opts(&t, v, true), llm::prompt_fnv_opts(&t, v, false));
    assert_eq!(llm::prompt_fnv_opts(&t, v, false), llm::prompt_fnv_v(&t, v));
    let with = llm::chat_body_opts("m", "s", "u", "{}", 1, v, true);
    assert!(with.ends_with(",\"chat_template_kwargs\":{\"enable_thinking\":false}}"), "{with}");
    assert!(!llm::chat_body_opts("m", "s", "u", "{}", 1, v, false).contains("chat_template_kwargs"));
    // the merge needs the flag too, to find those rows; a merged table built either way attaches
    let (only_think, _) = e.command(&["--merge"]).unwrap();
    assert!(only_think.contains("5 with an LLM answer") || only_think.contains("with an LLM answer"), "{only_think}");
    let e2 = Env::new();
    let fake2 = Fake::start(good());
    e2.llm(&fake2, &["--no-think"]).unwrap();
    let stats = |args: &[&str]| -> llm::MergeStats {
        e2.command(args).unwrap();
        serde_json::from_slice(&std::fs::read(e2.dir().join("reports/merge.json")).unwrap()).unwrap()
    };
    assert_eq!(stats(&["--merge"]).with_answer, 0, "thinking rows are not these rows");
    assert!(stats(&["--merge", "--no-think"]).with_answer > 0);
    e2.index().attach_roles(&roles::load(&e2.dir().join("roles.json")).unwrap()).unwrap();
    assert!(e2.command(&["--rules", "--no-think"]).is_err() && e2.command(&["--validate", "--no-think"]).is_err());
}

#[test]
fn routes_legal_only_and_the_dry_run_count() {
    plain_variant();
    let e = Env::new();
    let i = e.index();
    let rules = roles::generate(&i, "").unwrap();
    let ids: Vec<&roles::RouteReason> = roles::RouteReason::ALL.iter().collect();
    // a reason carried by some but not all of the five cards
    let has = |n: &str, r: &roles::RouteReason| assignment_of(&i, &rules, n).route.contains(r);
    let (reason, count) = ids
        .iter()
        .map(|r| (r, FIVE.iter().filter(|n| has(n, r)).count()))
        .find(|(_, c)| *c > 0 && *c < 5)
        .expect("the five cards split on some route reason");
    let fake = Fake::start(good());
    let name = format!("{reason:?}");
    let out = e.llm(&fake, &["--dry-run", "--routes", &name]).unwrap();
    assert!(out.contains(&format!("{count} candidate cards selected")) && out.contains("1 batches"), "{out}");
    // several reasons are an any-of; a bad name is a usage error listing the valid ones; --only-routed combines
    let all: Vec<String> = roles::RouteReason::ALL.iter().map(|r| format!("{r:?}")).collect();
    let any = e.llm(&fake, &["--dry-run", "--routes", &all.join(",")]).unwrap();
    assert!(any.contains("candidate cards selected"), "{any}");
    let err = e.llm(&fake, &["--dry-run", "--routes", "NoRole,Nope"]).unwrap_err();
    assert!(err.contains("unknown route reason 'Nope'") && err.contains("UnknownLine"), "{err}");
    assert!(e.command(&["--rules", "--routes", "NoRole"]).is_err());
    let routed = e.llm(&fake, &["--dry-run", "--only-routed", "--routes", &name]).unwrap();
    assert!(routed.contains(&format!("{count} candidate cards selected")), "{routed}");
    let limited = e.llm(&fake, &["--dry-run", "--routes", &name, "--limit", "1"]).unwrap();
    assert!(limited.contains("1 to classify in 1 batches"), "{limited}");
    assert_eq!(fake.chats(), 0);
    // the route filter works for a real run too: only the selected cards are sent
    e.llm(&fake, &["--routes", &name]).unwrap();
    let user =
        body_of(&fake, 0)["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap().to_string();
    assert!(user.starts_with(&format!("Classify these {count} cards.")), "{user}");
    // legal cards only by default
    let path = e.dir().join("oracle.json");
    let mut doc: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for c in doc["cards"].as_array_mut().unwrap() {
        if c["name"] == "Skywhaler's Shot" {
            c["legalities"]["commander"] = json!("not_legal");
        }
    }
    std::fs::write(&path, doc.to_string()).unwrap();
    let e_default = e.llm(&fake, &["--dry-run"]).unwrap();
    assert!(e_default.contains("4 candidate cards selected"), "{e_default}");
    assert!(e.llm(&fake, &["--dry-run", "--legal-only"]).unwrap().contains("4 candidate cards selected"));
    assert!(e.llm(&fake, &["--dry-run", "--include-not-legal"]).unwrap().contains("5 candidate cards selected"));
    assert!(e.llm(&fake, &["--dry-run", "--legal-only", "--include-not-legal"]).is_err());
}

#[test]
fn a_killed_run_leaves_a_valid_cache_and_a_stale_lock_and_the_rerun_continues() {
    plain_variant();
    let e = Env::new();
    // the server answers slowly, so the child is mid-run when it is killed
    let fake = Fake::start(Box::new(|_, body| {
        std::thread::sleep(std::time::Duration::from_millis(500));
        Reply::Content(valid(&parse_batch(body)).to_string())
    }));
    let ids = e.dir().join("ids.txt");
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_cardgen"))
        .env_remove("CARDGEN_KNOWLEDGE_DIR")
        .env("CARDGEN_LLM_VARIANT", "plain")
        .arg("--root")
        .arg(&e.0)
        .args(["roles", "--llm", "--batch", "2", "--endpoint", &fake.url, "--ids"])
        .arg(&ids)
        .arg("--data-dir")
        .arg(e.dir())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let started = std::time::Instant::now();
    while e.dir().join("llm_roles.jsonl").exists().then(|| e.rows().len()).unwrap_or(0) < 2 {
        assert!(started.elapsed() < std::time::Duration::from_secs(30), "the first batch never arrived");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    child.kill().unwrap();
    let output = child.wait_with_output().unwrap();
    let progress = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(progress.contains("batch 1/3: 2 cards, ") && progress.contains(", cache rows 2"), "{progress}");
    // the cache parses (never half written) and holds whole batches only
    let done = e.rows().len();
    assert!(done == 2 || done == 4, "{done}");
    assert!(e.dir().join("llm_roles.jsonl").exists());
    let before = fake.chats();
    // the killed run's lock names a dead process: the rerun takes it over, skips the cached cards and asks for the rest
    assert!(e.dir().join("llm_roles.lock").exists(), "a killed run leaves its lock");
    let out = e.llm(&fake, &["--batch", "2"]).unwrap();
    assert_eq!(e.rows().len(), 5, "{out}");
    let asked = fake.chats() - before;
    assert_eq!(asked, (5 - done).div_ceil(2), "only the missing cards are asked: {out}");
    assert!(out.contains(&format!("{done} cached")), "{out}");
    assert!(!e.dir().join("llm_roles.lock").exists());
    e.no_temp_files_beside_cache();
}

#[test]
fn a_live_lock_is_waited_for_but_a_dead_owners_lock_is_taken_over() {
    plain_variant();
    let e = Env::new();
    let fake = Fake::start(good());
    if !std::path::Path::new("/proc/self").exists() || std::path::Path::new("/proc/4194303").exists() {
        return; // the takeover relies on /proc
    }
    std::fs::write(e.dir().join("llm_roles.lock"), "4194303\n").unwrap();
    e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 1);
    assert!(!e.dir().join("llm_roles.lock").exists());
}
