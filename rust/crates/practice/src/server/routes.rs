//! The routes (Python's `Handler.do_GET` / `do_POST`): status codes, validation messages and JSON shapes as the
//! page expects them.

use super::api::AUTOPASS;
use super::http::{Request, Response, set_cookie, static_file};
use super::hub::{Hub, new_token, truthy};
use super::lan::lan_addresses;
use crate::catalog::{self, MY_DECKS, TIERS};
use crate::images::Images;
use serde_json::{Map, Value, json};
use std::cell::Cell;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub const HOST_ONLY: [&str; 5] = ["/api/new", "/api/load", "/api/quit", "/api/save", "/api/saves"];

/// What the connection should do for a request.
pub enum Reply {
    Respond(Response),
    /// stream events: (token, local, since, a new cookie to set)
    Events {
        token: String,
        local: bool,
        since: u64,
        new_token: Option<String>,
    },
}

pub struct Site {
    pub hub: Arc<Hub>,
    pub data: PathBuf,
    pub static_dir: PathBuf,
    pub images: Images,
    catalog: Mutex<Option<Value>>,
}

impl Site {
    pub fn new(hub: Arc<Hub>, data: PathBuf, static_dir: PathBuf) -> Site {
        let images = hub.images.clone();
        Site { hub, data, static_dir, images, catalog: Mutex::new(None) }
    }

    /// the setup screen's data: your decks, the tiers, and the commanders' images already cached
    pub fn options(&self) -> Result<Value, String> {
        let cat = self.catalog()?;
        let names = catalog::commanders(&cat);
        let mut c = cat.as_object().cloned().unwrap_or_default();
        let mut images = self.images.cached(&names);
        for m in self.images.cached_printed(&catalog::commander_printings(&self.data, &cat)).into_values() {
            images.extend(m.into_iter().filter_map(|(n, copies)| copies.into_iter().next().map(|f| (n, f))));
        }
        // pictures not downloaded yet (decks never played here): one background download, the page asks again
        let missing = names.iter().any(|n| !images.contains_key(n));
        if missing && std::env::var_os("PRACTICE_NO_FETCH").is_none() {
            self.images.fetch_online_in_background(names, catalog::commander_printings(&self.data, &cat));
        }
        c.insert("images".into(), json!(images));
        Ok(Value::Object(c))
    }

    /// the game format of a new-game request: the given `format`, else the deck's own; `Err` is the 400's text.
    /// Read from the catalog (formats, and which formats admit each deck: `Format::admits` computed there).
    fn request_format(&self, deck: &str, given: Option<&Value>) -> Result<RequestFormat, String> {
        let cat = match self.catalog() {
            Ok(c) => c,
            Err(e) if given.is_some() => return Err(format!("format: the format list is not available ({e})")),
            Err(_) => return Ok(RequestFormat::default()),
        };
        let formats = cat["formats"].as_array().cloned().unwrap_or_default();
        let keys: Vec<&str> = formats.iter().filter_map(|f| f["key"].as_str()).collect();
        let d = cat["decks"].as_array().and_then(|a| a.iter().find(|d| d["key"] == deck));
        let key = match given {
            Some(v) => {
                v.as_str().filter(|k| keys.contains(k)).ok_or_else(|| format!("format: one of {}", keys.join(", ")))?
            }
            None => d.and_then(|d| d["format"].as_str()).unwrap_or("commander"),
        };
        let f =
            formats.iter().find(|f| f["key"] == key).ok_or_else(|| format!("format: one of {}", keys.join(", ")))?;
        let name = f["name"].as_str().unwrap_or(key).to_string();
        let admitted = d.and_then(|d| d["formats"].as_array()).is_none_or(|a| a.iter().any(|k| k == key));
        if !admitted {
            return Err(format!("{deck} cannot be played at a {name} table"));
        }
        Ok(RequestFormat { key: key.to_string(), seats: f["seats"].as_u64().unwrap_or(4) as usize })
    }

    pub fn catalog(&self) -> Result<Value, String> {
        let mut g = self.catalog.lock().unwrap_or_else(|e| e.into_inner());
        if g.is_none() {
            *g = Some(catalog::catalog(&self.data)?);
        }
        Ok(g.clone().unwrap_or(Value::Null))
    }
}

/// the format of a new game, as the routes check it
struct RequestFormat {
    key: String,
    seats: usize,
}

impl Default for RequestFormat {
    fn default() -> Self {
        RequestFormat { key: "commander".into(), seats: 4 }
    }
}

/// "one", "two", ... for the messages
fn seat_word(n: usize) -> String {
    ["zero", "one", "two", "three", "four"].get(n).map_or(n.to_string(), |w| w.to_string())
}

/// This request's browser cookie (a new one is set with the response if it has none, and only if it was used).
struct Token {
    value: String,
    fresh: bool,
    used: Cell<bool>,
}

impl Token {
    fn of(req: &Request) -> Token {
        match req.cookie(super::http::COOKIE) {
            Some(v) => Token { value: v, fresh: false, used: Cell::new(false) },
            None => Token { value: new_token(), fresh: true, used: Cell::new(false) },
        }
    }
    fn get(&self) -> &str {
        self.used.set(true);
        &self.value
    }
    fn cookie(&self) -> Option<String> {
        (self.used.get() && self.fresh).then(|| set_cookie(&self.value))
    }
}

fn kv(pairs: &[(&str, Value)]) -> Map<String, Value> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

fn err(code: u16, msg: impl Into<String>) -> Response {
    Response::json(code, &json!({"error": msg.into()}))
}

/// 409 with why, or 200 with ok
fn reply(why: Option<String>, ok: Option<Map<String, Value>>) -> Response {
    match why {
        Some(w) => err(409, w),
        None => {
            let mut m = Map::new();
            m.insert("ok".into(), json!(true));
            m.extend(ok.unwrap_or_default());
            Response::json(200, &Value::Object(m))
        }
    }
}

fn result(r: Result<Value, String>) -> Response {
    match r {
        Ok(v) => Response::json(200, &v),
        Err(e) => err(409, e),
    }
}

pub fn dispatch(site: &Site, req: &Request) -> Reply {
    let tok = Token::of(req);
    let mut reply = match req.method.as_str() {
        "GET" => match get(site, req, &tok) {
            Ok(r) => return r,
            Err(r) => r,
        },
        "POST" => post(site, req, &tok),
        _ => err(404, "not found"),
    };
    reply.cookie = tok.cookie();
    Reply::Respond(reply)
}

/// `Ok(reply)` for a stream, `Err(response)` for a plain response (so the caller can add the cookie)
fn get(site: &Site, req: &Request, tok: &Token) -> Result<Reply, Response> {
    let hub = &site.hub;
    let seat = |tok: &Token| hub.seat_of(tok.get(), req.local());
    let path = req.path.as_str();
    if path == "/" || path == "/index.html" {
        tok.get();
        return Err(static_file(&site.static_dir, "index.html"));
    }
    if let Some(name) = path.strip_prefix("/static/") {
        return Err(static_file(&site.static_dir, name));
    }
    if let Some(name) = path.strip_prefix("/images/") {
        return Err(static_file(&site.images.dir, name));
    }
    if HOST_ONLY.contains(&path) && !req.local() {
        return Err(err(403, "Only the host computer can do that."));
    }
    Err(match path {
        "/api/state" => Response::json(200, &hub.state(seat(tok).as_deref(), req.local())),
        "/api/options" => match site.options() {
            Ok(Value::Object(mut o)) => {
                o.insert("lan".into(), json!(hub.lan));
                o.insert("host".into(), json!(req.local()));
                o.insert("app".into(), json!(hub.app));
                Response::json(200, &Value::Object(o))
            }
            Ok(_) => err(500, "bad catalog"),
            Err(e) => err(500, e),
        },
        "/api/saves" => Response::json(200, &json!({"saves": hub.saves_list(false), "autosave": hub.autosaved()})),
        "/api/review" => result(hub.review(seat(tok).as_deref())),
        "/api/events" => {
            let since = req
                .header("last-event-id")
                .map(String::from)
                .or_else(|| req.query_param("since"))
                .unwrap_or_else(|| "0".into());
            let since = if !since.is_empty() && since.bytes().all(|b| b.is_ascii_digit()) {
                since.parse().unwrap_or(0)
            } else {
                0
            };
            let token = tok.get().to_string();
            return Ok(Reply::Events {
                token,
                local: req.local(),
                since,
                new_token: tok.fresh.then(|| tok.value.clone()),
            });
        }
        _ => err(404, "not found"),
    })
}

fn str_list(v: &Value) -> bool {
    v.as_array().is_some_and(|a| a.iter().all(|x| x.is_string()))
}

/// `int(x)` of a count, or 1 when it isn't digits (Python: `int(n) if str(n).isdigit() else 1`)
fn count(v: Option<&Value>) -> i64 {
    match v {
        None => 1,
        Some(Value::Number(n)) => n.as_u64().map(|x| x as i64).unwrap_or(1),
        Some(Value::String(s)) if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) => s.parse().unwrap_or(1),
        _ => 1,
    }
}

fn post(site: &Site, req: &Request, tok: &Token) -> Response {
    let hub = &site.hub;
    let body: Value = if req.body.is_empty() {
        json!({})
    } else {
        match serde_json::from_slice(&req.body) {
            Ok(v @ Value::Object(_)) => v,
            _ => return err(400, "the request body is not JSON"),
        }
    };
    let path = req.path.as_str();
    if HOST_ONLY.contains(&path) && !req.local() {
        return err(403, "Only the host computer can do that.");
    }
    let seat = |tok: &Token| hub.seat_of(tok.get(), req.local());
    let get = |k: &str| body.get(k).filter(|v| !v.is_null());
    match path {
        "/api/new" => {
            let in_list =
                |v: Option<&Value>, list: &[&str]| v.and_then(|v| v.as_str()).is_some_and(|s| list.contains(&s));
            if !in_list(get("deck"), &MY_DECKS) {
                return err(400, format!("deck: one of {}", MY_DECKS.join(", ")));
            }
            if !in_list(get("tier"), &TIERS) {
                return err(400, format!("tier: one of {}", TIERS.join(", ")));
            }
            if get("format").is_some_and(|v| !v.is_string()) {
                return err(400, "format: a format key");
            }
            let fmt = match site.request_format(body["deck"].as_str().unwrap_or(""), get("format")) {
                Ok(f) => f,
                Err(e) => return err(400, e),
            };
            if get("opponents").is_some_and(|v| !str_list(v)) {
                return err(400, "opponents: a list of deck keys");
            }
            if get("seats").is_some_and(|v| !str_list(v)) {
                return err(400, format!("seats: {} deck keys in turn order", seat_word(fmt.seats)));
            }
            if get("seat").is_some_and(|v| !(v.is_i64() || v.is_u64() || v.is_boolean())) {
                return err(400, format!("seat: 1 to {}", fmt.seats));
            }
            if get("autopass").is_some_and(|v| !v.as_str().is_some_and(|s| AUTOPASS.contains(&s))) {
                return err(400, format!("autopass: one of {}", AUTOPASS.join(", ")));
            }
            if get("two").is_some_and(truthy) {
                if !hub.lan {
                    return err(
                        409,
                        "To play with a friend, start the server with --lan: python3 -m commander_sim.play --lan",
                    );
                }
                let mut o = body.as_object().cloned().unwrap_or_default();
                for k in ["two", "seat", "seats"] {
                    o.remove(k);
                }
                // two people at a two-seat table: nobody else plays, so there is no tier (the page still sends one)
                if fmt.seats <= 2 {
                    o.remove("tier");
                }
                // the lobby carries the table's format (a Commander request is as it always was)
                if fmt.key != "commander" {
                    o.insert("format".into(), json!(fmt.key));
                }
                return match hub.open_lobby(&Value::Object(o), tok.get(), None) {
                    Ok(code) => Response::json(200, &json!({"ok": true, "code": code, "addresses": lan_addresses()})),
                    Err(e) => err(400, e),
                };
            }
            // a friend joins the table with its code: a partner key here would seat a second person nobody plays
            // (still allowed at the four-seat table, as it always was)
            if get("partner").is_some() && fmt.seats <= 2 {
                return err(400, "partner: a friend joins with the table's code");
            }
            let deck = body["deck"].as_str().unwrap_or("").to_string();
            match hub.new_game(&body, None, [(tok.get().to_string(), deck)].into()) {
                Ok(s) => {
                    let i = s.info();
                    Response::json(200, &json!({"ok": true, "seed": i.seed, "seats": i.seats}))
                }
                Err(e) => err(400, e),
            }
        }
        "/api/join" => reply(
            hub.join(tok.get(), body.get("code").unwrap_or(&Value::Null), body.get("deck").unwrap_or(&Value::Null)),
            None,
        ),
        "/api/answer" => {
            let s = seat(tok);
            reply(
                hub.answer(
                    s.as_deref(),
                    body.get("id").unwrap_or(&Value::Null),
                    body.get("answer").cloned().unwrap_or(Value::Null),
                ),
                None,
            )
        }
        "/api/undo" | "/api/tryit" => {
            let Some(s) = seat(tok) else { return reply(Some("You have no seat in this game.".into()), None) };
            let (why, asked) = if path == "/api/undo" {
                hub.request_rewind(&s, "undo", &json!(count(body.get("n"))))
            } else {
                hub.request_rewind(&s, "tryit", body.get("n").unwrap_or(&Value::Null))
            };
            reply(why, asked.map(|a| kv(&[("asked", json!(a))])))
        }
        "/api/approve" => {
            let s = seat(tok);
            reply(
                hub.approve(s.as_deref(), body.get("id").unwrap_or(&Value::Null), body.get("yes").is_some_and(truthy)),
                None,
            )
        }
        "/api/save" => match hub.save() {
            Ok(name) => reply(None, Some(kv(&[("name", json!(name))]))),
            Err(why) => reply(Some(why), None),
        },
        "/api/load" => {
            let images = body.get("images").is_none_or(|v| *v != Value::Bool(false));
            match hub.load(body.get("name").unwrap_or(&Value::Null), images, tok.get()) {
                Ok(Some(code)) => {
                    reply(None, Some(kv(&[("code", json!(code)), ("addresses", json!(lan_addresses()))])))
                }
                Ok(None) => reply(None, None),
                Err(why) => reply(Some(why), None),
            }
        }
        "/api/autopass" => {
            let s = seat(tok);
            reply(hub.set_autopass(s.as_deref(), body.get("mode").unwrap_or(&Value::Null)), None)
        }
        "/api/hint" => {
            let s = seat(tok);
            result(hub.hint(s.as_deref()))
        }
        "/api/quit" => {
            hub.quit();
            hub.forget_autosave();
            Response::json(200, &json!({"ok": true}))
        }
        _ => err(404, "not found"),
    }
}
