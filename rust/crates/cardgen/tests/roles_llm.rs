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
        let stop = Arc::new(AtomicBool::new(false));
        let (m, c, mo, b, st) = (model.clone(), chats.clone(), models.clone(), bodies.clone(), stop.clone());
        let handle = std::thread::spawn(move || {
            for stream in listener.incoming() {
                if st.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                serve(&mut stream, &handler, &m, &c, &mo, &b);
            }
        });
        Fake { url: format!("http://{addr}/v1"), addr, model, chats, models, bodies, stop, handle: Some(handle) }
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
    let user = body["messages"][1]["content"].as_str().unwrap();
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
    json!({"id": id, "roles": roles, "primary": roles.first(),
           "evidence": evidence.iter().map(|(r, n)| (r.to_string(), json!(n))).collect::<serde_json::Map<_, _>>()})
}
fn valid(items: &[Item]) -> Value {
    json!({"cards": items.iter().map(|i| { let (r, e) = says(i); entry(&i.id, &r, &e) }).collect::<Vec<_>>()})
}
fn find<'a>(v: &'a mut Value, starts: &str, items: &[Item]) -> Option<&'a mut Value> {
    let id = items.iter().find(|i| first(i).starts_with(starts))?.id.clone();
    v["cards"].as_array_mut()?.iter_mut().find(|c| c["id"] == id.as_str())
}
/// One class of invalid model output, applied to an otherwise valid answer for `items`.
fn corrupt(class: &str, items: &[Item]) -> Reply {
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
        "primary" => {
            let c = find(&mut v, "Destroy target creature with power", items).unwrap();
            c["primary"] = json!("wipe");
        }
        "out of range" => {
            let c = find(&mut v, "Destroy target creature with power", items).unwrap();
            c["evidence"]["removal"] = json!(99);
        }
        "skipped line" => {
            // Elspeth's ultimate (line 2) is never shown, so it cannot be cited
            let c = find(&mut v, "+1: Create three", items).unwrap();
            c["evidence"]["wipe"] = json!(2);
        }
        "guard" => {
            let id = items.iter().find(|i| first(i) == "Counter target spell.").unwrap().id.clone();
            let c = v["cards"].as_array_mut().unwrap().iter_mut().find(|c| c["id"] == id.as_str()).unwrap();
            *c = entry(&id, &["draw"], &[("draw", 0)]);
        }
        "no citation" => {
            let c = find(&mut v, "Destroy target creature with power", items).unwrap();
            c["evidence"] = json!({});
        }
        "malformed" => return Reply::Content("{\"cards\": [{\"id\": \"a0\", ".into()),
        "truncated" => return Reply::Truncated(v.to_string()[..40].to_string()),
        "no choices" => return Reply::Raw("{}".into()),
        "http 500" => return Reply::Status(500),
        other => panic!("unknown class {other}"),
    }
    Reply::Content(v.to_string())
}
fn good() -> Handler {
    Box::new(|_, body| Reply::Content(valid(&parse_batch(body)).to_string()))
}

// ---------------------------------------------------------------------------------------------------- environment

struct Env(PathBuf);
impl Env {
    fn new() -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let root =
            std::env::temp_dir().join(format!("roles-llm-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        let dir = root.join("data/knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(FIXTURE, dir.join("oracle.json")).unwrap();
        std::fs::write(dir.join("roles.toml"), TAXONOMY).unwrap();
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
    assert_eq!(items["roles"]["items"]["enum"].as_array().unwrap().len(), 26);
    assert_eq!(items["roles"]["maxItems"], 4);
    assert!(items["primary"]["enum"].as_array().unwrap().contains(&Value::Null));
    assert_eq!(items["id"]["pattern"], "^[a-z][0-9]$");
    let user = body["messages"][1]["content"].as_str().unwrap();
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
fn every_invalid_output_class_triggers_the_retry_ladder_and_recovers_on_the_resend() {
    for class in [
        "ids",
        "vocabulary",
        "five roles",
        "primary",
        "out of range",
        "skipped line",
        "guard",
        "no citation",
        "malformed",
        "truncated",
        "no choices",
        "http 500",
    ] {
        let e = Env::new();
        let fake = Fake::start(Box::new(move |n, body| {
            let items = parse_batch(body);
            if n == 0 { corrupt(class, &items) } else { Reply::Content(valid(&items).to_string()) }
        }));
        let out = e.llm(&fake, &[]).unwrap_or_else(|err| panic!("{class}: {err}"));
        assert_eq!(fake.chats(), 2, "{class}: the same batch is resent once: {out}");
        assert!(
            out.contains("5 classified") && out.contains("0 failed") && out.contains("1 batch retries"),
            "{class}: {out}"
        );
        assert_eq!(e.rows().len(), 5, "{class}");
    }
}

#[test]
fn a_batch_failing_twice_is_sent_card_by_card() {
    let e = Env::new();
    let fake = Fake::start(Box::new(|n, body| {
        let items = parse_batch(body);
        if n < 2 { corrupt("vocabulary", &items) } else { Reply::Content(valid(&items).to_string()) }
    }));
    let out = e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 2 + 5, "{out}");
    let bodies = fake.bodies.lock().unwrap().clone();
    assert!(bodies[2..].iter().all(|b| parse_batch(b).len() == 1), "per-card requests carry one card");
    assert_eq!(bodies[2]["response_format"]["json_schema"]["schema"]["properties"]["cards"]["minItems"], 1);
    assert!(out.contains("5 classified") && out.contains("0 failed") && out.contains("0 per-card retries"), "{out}");
    assert_eq!(e.rows().len(), 5);
}

#[test]
fn a_card_failing_alone_twice_is_llm_failed_and_keeps_its_rule_roles() {
    let e = Env::new();
    let fake = Fake::start(Box::new(|n, body| {
        let items = parse_batch(body);
        let bad_card = items.len() == 1 && first(&items[0]).starts_with("Destroy target creature with power");
        if items.len() > 1 && n < 2 || bad_card {
            corrupt("vocabulary", &items)
        } else {
            Reply::Content(valid(&items).to_string())
        }
    }));
    let out = e.llm(&fake, &[]).unwrap();
    // 2 batch attempts, then five cards alone, the failing one twice
    assert_eq!(fake.chats(), 2 + 5 + 1, "{out}");
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
}

#[test]
fn a_warm_cache_makes_no_chat_requests() {
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
    let e = Env::new();
    let fake = Fake::start(good());
    e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 1);
    // a guard, keep_rule or overrides edit changes no prompt text, no model and no key: still warm
    let toml = std::fs::read_to_string(e.dir().join("roles.toml")).unwrap();
    let tweaked = toml.replacen("guard = '(?i)counter|exile.*spells?'", "guard = '(?i)counter|exile.*spells?|x'", 1);
    assert_ne!(toml, tweaked);
    let tweaked = tweaked.replacen(
        "src = \"rules\"\nguard = '(?i)discard'",
        "src = \"rules\"\nguard = '(?i)discard'\nkeep_rule = true",
        1,
    );
    std::fs::write(e.dir().join("roles.toml"), &tweaked).unwrap();
    std::fs::write(
        e.dir().join("roles_overrides.toml"),
        "[[card]]\nname='Mogg Fanatic'\nroles=[]\nnote='changes the rules fingerprint, not the cache key'\n",
    )
    .unwrap();
    let out = e.llm(&fake, &[]).unwrap();
    assert_eq!(fake.chats(), 1, "guards, keep_rule and overrides leave the cache warm: {out}");
    // a definition edit changes what the model is told: every row is stale
    let edited = tweaked.replacen("counters a spell or ability.", "counters a spell or an activated ability.", 1);
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
    assert_eq!(fake.chats(), 1 + 3, "three consecutive failures stop the run");
}

#[test]
fn limit_only_routed_dry_run_batch_size_and_pending_file() {
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
    let e = Env::new();
    std::fs::write(
        e.dir().join("roles_overrides.toml"),
        "[[card]]\nname=\"Skywhaler's Shot\"\nroles=[]\nnote='cleared by hand'\n",
    )
    .unwrap();
    let fake = Fake::start(good());
    let out = e.llm(&fake, &[]).unwrap();
    assert!(out.contains("4 classified"), "{out}");
    let user = fake.bodies.lock().unwrap()[0]["messages"][1]["content"].as_str().unwrap().to_string();
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
    let r = rule(&[("counter", false)], false, &[]);
    let (a, d) = run(&r, Some(&said(&["removal"])), &[], &[]);
    assert_eq!(a, r);
    assert_eq!(kinds(&d), [(Kind::Shadow, "counter".into()), (Kind::Shadow, "removal".into())], "{d:?}");
    let (a, d) = run(&r, Some(&said(&["counter"])), &[], &[]);
    assert!(a == r && d.is_empty());
}

#[test]
fn merge_vetoes_unconfirmed_rule_roles_unless_keep_rule_or_certain() {
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
    let t = Taxonomy::parse(TAXONOMY).unwrap();
    let text = t.prompt_text();
    for id in roles::IDS {
        assert_eq!(text.matches(&format!("\n- {id}: ")).count(), 1, "{id} appears once as a role line");
        assert!(t.guard(id).is_match("anything").is_ok());
    }
    assert!(t.role.iter().all(|r| !r.keep_rule), "no role keeps rule hits by default");
    assert!(!t.keep_rule("removal"));
    // the boilerplate that used to be repeated in every definition is said once
    assert_eq!(text.matches("MDFC").count(), 1, "{text}");
    assert_eq!(text.matches("planeswalker ability costing").count(), 1);
    for clause in &t.clause {
        assert_eq!(text.matches(clause.text.trim()).count(), 1, "{}", clause.id);
    }
    // hash: stable, tied to the prompt-visible text, blind to guards and keep_rule
    assert_eq!(t.hash(), Taxonomy::parse(TAXONOMY).unwrap().hash());
    let edited = TAXONOMY.replacen("counters a spell or ability.", "counters a spell.", 1);
    assert_ne!(edited, TAXONOMY);
    assert_ne!(Taxonomy::parse(&edited).unwrap().hash(), t.hash());
    let guard = TAXONOMY.replacen("(?i)counter|exile.*spells?", "(?i)counter|exile.*spells? ", 1);
    assert_eq!(Taxonomy::parse(&guard).unwrap().hash(), t.hash());
    let keep = TAXONOMY.replacen(
        "src = \"rules\"\nguard = '(?i)discard'",
        "src = \"rules\"\nguard = '(?i)discard'\nkeep_rule = true",
        1,
    );
    let kept = Taxonomy::parse(&keep).unwrap();
    assert!(kept.keep_rule("discard") && kept.hash() == t.hash());
    // malformed taxonomies are errors
    assert!(
        Taxonomy::parse(&TAXONOMY.replacen(
            "definition = \"makes opponents discard.\"",
            "definition = \"makes opponents\\ndiscard.\"",
            1
        ))
        .is_err()
    );
    assert!(Taxonomy::parse(&TAXONOMY.replacen("roles = [\"ramp\", \"tokens\"]", "roles = [\"nope\"]", 1)).is_err());
}

#[test]
fn no_definition_names_a_card_because_the_model_has_no_internet() {
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
    let t = Taxonomy::parse(TAXONOMY).unwrap();
    let system = llm::system_prompt(&t);
    let (chars, tokens) = (system.chars().count(), llm::estimate_tokens(&system));
    let taxonomy_chars = t.prompt_text().chars().count();
    println!(
        "SYSTEM PROMPT: {chars} chars, about {tokens} tokens (chars/4); the taxonomy part is {taxonomy_chars} chars"
    );
    println!("per-role definitions (definition + role clauses), chars:");
    for r in &t.role {
        println!("  {:<12} {}", r.id, r.definition.len() + r.clauses.iter().map(|c| c.len() + 1).sum::<usize>());
    }
    assert!(tokens < 5000, "the system prompt must stay well under 5k tokens: {tokens}");
    assert!(taxonomy_chars < 13_967, "deduplicated: the old definitions were 13,967 chars, now {taxonomy_chars}");
    // every worked example is a valid answer under the very validation the real answers face
    let lines: Vec<Vec<(usize, String)>> =
        llm::EXAMPLES.iter().map(|e| e.lines.iter().map(|(i, t)| (*i, t.to_string())).collect()).collect();
    let entries: Vec<String> = llm::EXAMPLES
        .iter()
        .enumerate()
        .map(|(i, e)| llm::answer_entry(&llm::batch_id(i), e.roles, e.evidence))
        .collect();
    let content = format!("{{\"cards\":[{}]}}", entries.join(","));
    let cards: Vec<(String, &[(usize, String)])> =
        lines.iter().enumerate().map(|(i, l)| (llm::batch_id(i), l.as_slice())).collect();
    let parsed = llm::validate_output(&content, &cards, &t).unwrap();
    assert_eq!(parsed.len(), 6);
    assert!(system.contains("WORKED EXAMPLE") && system.contains(&content));
    // the examples and the grammar write each entry in the same order: roles are decided before they are cited
    assert!(content.starts_with(concat!(
        "{\"cards\":[{\"id\":\"a0\",\"roles\":[\"removal\"],",
        "\"primary\":\"removal\",\"evidence\":{\"removal\":0}}"
    )));
    let schema = llm::response_schema_text(&t, 3);
    let at = |key: &str| schema.find(&format!("\"{key}\":{{")).unwrap();
    assert!(at("id") < at("roles") && at("roles") < at("primary") && at("primary") < at("evidence"), "{schema}");
    let body = llm::chat_body("m", "sys", "usr", &schema, 3);
    let parsed: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["max_tokens"], 160 * 3 + 64);
    assert!(body.contains(&schema), "the schema text goes into the body untouched");
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
    println!("SAMPLE USER MESSAGE:\n{sample}");
    if std::env::var_os("CARDGEN_PRINT_PROMPT").is_some() {
        // a real five-card batch from the fixture catalog, as the request would carry it
        let i = Index::load(&PathBuf::from(FIXTURE)).unwrap();
        let cards: Vec<_> = FIVE.iter().map(|n| &i.cards[i.lookup(n)[0] as usize]).collect();
        let shown: Vec<(String, Vec<(usize, String)>)> =
            cards.iter().map(|c| (c.raw.type_line.clone(), roles::prompt_lines(c))).collect();
        let batch: Vec<(String, &str, &[(usize, String)])> =
            shown.iter().enumerate().map(|(n, (t, l))| (llm::batch_id(n), t.as_str(), l.as_slice())).collect();
        println!("=== SYSTEM PROMPT ===\n{system}\n=== SAMPLE BATCH (user message) ===\n{}", llm::user_message(&batch));
    }
    assert_eq!(sample, "Classify these 1 cards.\n[a0] Instant\n0: ~ deals 3 damage to any target.\n");
    assert_eq!(
        (llm::batch_id(0), llm::batch_id(9), llm::batch_id(10), llm::batch_id(25)),
        ("a0".into(), "a9".into(), "b0".into(), "c5".into())
    );
    let schema = llm::response_schema(&t, 3);
    assert_eq!(schema["properties"]["cards"]["minItems"], 3);
}

#[test]
fn the_cache_key_is_the_card_text_the_prompt_and_the_model_and_nothing_else() {
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
