//! The text client (Python's `play/text.py`): play a practice game in the terminal. It drives a `Session` through
//! its event stream and answers with the same JSON the browser sends.

use crate::session::Session;
use serde_json::{Value, json};
use std::io::{BufRead, Write};
use std::sync::Arc;
use std::time::Duration;

/// unusable answers in a row before a prompt takes its default (pass / none / keep / cancel)
const TRIES: usize = 20;

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

fn n(v: &Value, k: &str) -> i64 {
    v.get(k).and_then(Value::as_i64).unwrap_or(0)
}

fn arr<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v.get(k).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn truthy(v: &Value, k: &str) -> bool {
    match v.get(k) {
        None | Some(Value::Null) | Some(Value::Bool(false)) => false,
        Some(Value::Number(x)) => x.as_f64() != Some(0.0),
        Some(Value::String(t)) => !t.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        _ => true,
    }
}

/// a JSON number or string as text (Python's `str()` of a counters / pt value)
fn txt(v: &Value) -> String {
    match v {
        Value::String(t) => t.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn line_of(p: &Value) -> String {
    let tag = if truthy(p, "you") { " (you)" } else { "" };
    let dead = if truthy(p, "alive") { "" } else { "  [out]" };
    format!(
        "  {}{}: {} life, hand {}, library {}, graveyard {}{}",
        s(p, "name"),
        tag,
        txt(&p["life"]),
        n(p, "hand_count"),
        n(p, "library"),
        arr(p, "graveyard").len(),
        dead
    )
}

/// identical lines once, with a count, in first-seen order
fn grouped(lines: Vec<String>) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = vec![];
    for x in lines {
        match out.iter_mut().find(|(t, _)| *t == x) {
            Some(e) => e.1 += 1,
            None => out.push((x, 1)),
        }
    }
    out
}

fn colours(x: &Value) -> String {
    match &x["colours"] {
        Value::Array(a) => a.iter().map(txt).collect::<Vec<_>>().join(""),
        v => txt(v),
    }
}

/// a compact picture of the table from the human seat
pub fn show_table(view: &Value, out: &mut dyn Write) {
    let _ = writeln!(out, "\n=== Round {} ===", txt(&view["round"]));
    for p in arr(view, "players") {
        let _ = writeln!(out, "{}", line_of(p));
        let lands = arr(p, "lands");
        if !lands.is_empty() {
            let untapped = lands.iter().filter(|l| !truthy(l, "tapped")).count();
            let _ = writeln!(out, "      lands: {} ({} untapped)", lands.len(), untapped);
        }
        let mut lines = vec![];
        for m in arr(p, "battlefield") {
            let mut bits = vec![s(m, "name").to_string()];
            if m.get("pt").is_some() {
                bits.push(txt(&m["pt"]));
            }
            if truthy(m, "counters") {
                bits.push(format!("+{} counters", txt(&m["counters"])));
            }
            if !m["loyalty"].is_null() && m.get("loyalty").is_some() {
                bits.push(format!("loyalty {}", txt(&m["loyalty"])));
            }
            if truthy(m, "attached_to") {
                bits.push(format!("on {}", txt(&m["attached_to"])));
            }
            if truthy(m, "tapped") {
                bits.push("tapped".into());
            }
            lines.push(bits.join(", "));
        }
        for (t, c) in grouped(lines) {
            let _ = writeln!(out, "      {}{}", t, if c > 1 { format!("  x{c}") } else { String::new() });
        }
        if truthy(p, "you") && p.get("hand").is_some() {
            let hand: Vec<&str> = arr(p, "hand").iter().filter_map(Value::as_str).collect();
            let _ = writeln!(out, "      hand: {}", if hand.is_empty() { "(empty)".into() } else { hand.join("; ") });
            let srcs = arr(p, "mana_sources");
            let pool = if truthy(p, "mana_pool") { txt(&p["mana_pool"]) } else { "empty".into() };
            let list: Vec<String> = srcs.iter().map(|x| format!("{} ({})", s(x, "name"), colours(x))).collect();
            let _ = writeln!(
                out,
                "      mana pool: {}; untapped sources: {}",
                pool,
                if list.is_empty() { "none".into() } else { list.join(", ") }
            );
        }
    }
}

const HELP: &str = "Commands:
  show              the table again
  tap N [C]         tap mana source N (C: the colour, for sources that make several)
  land N            play card N from your hand as your land drop
  cast N            cast card N from your hand
  cast cmd          cast your commander from the command zone
  cast gN / land gN cast or play card N of your graveyard (Yawgmoth's Will, flashback, Underworld Breach)
  use N             an ability of your permanent N (Equip, loyalty abilities, Jace's Archivist ...)
  use LN            an ability of your land N (Rogue's Passage, Scavenger Grounds)
  pass              pass priority (move on)
When asked to choose (a target, an ability, a blocker), type its number, or cancel / the last choice.
Attacking: type the attackers' numbers (1,3), all, or none.
  quit              end the game";

fn me(view: &Value) -> &Value {
    arr(view, "players").iter().find(|p| truthy(p, "you")).unwrap_or(&Value::Null)
}

fn priority_prompt(req: &Value, out: &mut dyn Write) {
    let view = &req["data"]["view"];
    let me = me(view);
    let _ = writeln!(out, "\n{}. Mana pool: {}", s(req, "prompt"), txt(&me["mana_pool"]));
    for x in arr(&req["data"], "stack") {
        let _ = writeln!(out, "  On the stack: {}", s(x, "name"));
    }
    let hand: Vec<String> = arr(me, "hand").iter().enumerate().map(|(i, h)| format!("{}) {}", i + 1, txt(h))).collect();
    let _ = writeln!(out, "  Hand: {}", if hand.is_empty() { "(empty)".into() } else { hand.join("  ") });
    let srcs: Vec<String> = arr(me, "mana_sources")
        .iter()
        .enumerate()
        .map(|(i, x)| format!("{}) {} ({})", i + 1, s(x, "name"), colours(x)))
        .collect();
    let _ =
        writeln!(out, "  Mana sources: {}", if srcs.is_empty() { "(none untapped)".into() } else { srcs.join("  ") });
    let perms: Vec<String> = arr(me, "battlefield")
        .iter()
        .map(|m| format!("{}) {}{}", n(m, "i") + 1, s(m, "name"), if truthy(m, "tapped") { " (tapped)" } else { "" }))
        .collect();
    if !perms.is_empty() {
        let _ = writeln!(out, "  Your permanents: {}", perms.join("  "));
    }
    let gyp: Vec<String> = arr(me, "graveyard_playable")
        .iter()
        .map(|x| format!("G{}) {} ({})", n(x, "i") + 1, s(x, "name"), s(x, "how")))
        .collect();
    if !gyp.is_empty() {
        let _ = writeln!(out, "  From your graveyard: {}", gyp.join("  "));
    }
    let act: Vec<String> = arr(me, "lands")
        .iter()
        .filter(|l| truthy(l, "ability") && !truthy(l, "tapped"))
        .map(|l| format!("L{}) {}", n(l, "i") + 1, s(l, "name")))
        .collect();
    if !act.is_empty() {
        let _ = writeln!(out, "  Lands with abilities: {}", act.join("  "));
    }
    if truthy(me, "commander_in_zone") {
        let _ = writeln!(out, "  Commander in the command zone: {} (tax {})", s(me, "commander"), txt(&me["tax"]));
    }
}

/// what a typed command means
#[derive(Debug, PartialEq)]
pub enum Cmd {
    Act(Value),
    Show,
    Help,
    Quit,
    Err(String),
}

fn digits(t: &str) -> Option<i64> {
    if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) { t.parse().ok() } else { None }
}

pub fn parse(line: &str, view: &Value) -> Cmd {
    let w: Vec<&str> = line.split_whitespace().collect();
    let Some(first) = w.first() else { return Cmd::Err("Type a command (help for the list).".into()) };
    let cmd = first.to_lowercase();
    let w1 = w.get(1).map(|x| x.to_lowercase());
    match cmd.as_str() {
        "pass" | "p" => return Cmd::Act(json!({"do": "pass"})),
        "show" | "s" => return Cmd::Show,
        "help" | "h" | "?" => return Cmd::Help,
        "quit" | "q" => return Cmd::Quit,
        _ => {}
    }
    if cmd == "cast" && matches!(w1.as_deref(), Some("cmd" | "commander")) {
        return Cmd::Act(json!({"do": "cast", "zone": "cmd"}));
    }
    if cmd == "use" {
        if let Some(t) = &w1
            && t.starts_with('l')
            && let Some(k) = digits(&t[1..])
        {
            return Cmd::Act(json!({"do": "use", "land": k - 1}));
        }
        return match w.get(1).and_then(|t| digits(t)) {
            Some(k) => Cmd::Act(json!({"do": "use", "perm": k - 1})),
            None => Cmd::Err("Which permanent? e.g. use 1 (or use L1 for a land)".into()),
        };
    }
    if matches!(cmd.as_str(), "land" | "cast")
        && let Some(t) = &w1
        && t.starts_with('g')
        && let Some(k) = digits(&t[1..])
    {
        return Cmd::Act(json!({"do": cmd, "zone": "gy", "card": k - 1}));
    }
    if matches!(cmd.as_str(), "tap" | "land" | "cast") {
        let Some(k) = w.get(1).and_then(|t| digits(t)) else {
            return Cmd::Err(format!("Which one? e.g. {cmd} 1"));
        };
        let k = k - 1;
        if cmd == "tap" {
            let srcs = arr(me(view), "mana_sources");
            if k < 0 || k as usize >= srcs.len() {
                return Cmd::Err("No such mana source.".into());
            }
            let mut act = json!({"do": "tap", "source": srcs[k as usize]["id"]});
            if let Some(c) = w.get(2).and_then(|c| c.chars().next()) {
                act["colour"] = c.to_uppercase().to_string().into();
            }
            return Cmd::Act(act);
        }
        return Cmd::Act(json!({"do": cmd, "card": k}));
    }
    Cmd::Err(format!("Unknown command '{}' (help for the list).", first))
}

struct Io<'a> {
    inp: &'a mut dyn BufRead,
    out: &'a mut dyn Write,
}

impl Io<'_> {
    /// one line after the prompt, or None at end of input
    fn ask(&mut self, prompt: &str) -> Option<String> {
        let _ = write!(self.out, "{prompt}");
        let _ = self.out.flush();
        let mut l = String::new();
        match self.inp.read_line(&mut l) {
            Ok(0) | Err(_) => {
                let _ = writeln!(self.out);
                None
            }
            Ok(_) => Some(l.trim_end_matches(['\n', '\r']).to_string()),
        }
    }
}

fn list_choices(req: &Value, io: &mut Io) -> usize {
    let _ = writeln!(io.out, "\n{}", s(req, "prompt"));
    let ch = arr(req, "choices");
    for (i, c) in ch.iter().enumerate() {
        let _ = writeln!(io.out, "  {}) {}", i + 1, txt(c));
    }
    ch.len()
}

/// Drive a session from the terminal until the game ends; returns the final `over` / `error` event.
pub fn run(session: &Arc<Session>, inp: &mut dyn BufRead, out: &mut dyn Write) -> Value {
    session.start();
    let _ = writeln!(
        out,
        "Practice game: {} vs {}, seed {}. Seats: {}",
        session.deck,
        session.tier,
        session.seed,
        session.seats.join(", ")
    );
    let me_key = session.deck.clone();
    let mut io = Io { inp, out };
    let mut cursor = 0u64;
    loop {
        let (c, evs) = session.events_after(cursor, Duration::from_millis(500));
        cursor = c;
        if evs.is_empty() && !session.threads_running() {
            // the game thread ended without a last event (closed by 'quit' or end of input)
            let _ = writeln!(io.out, "\nGame over: no winner (closed)");
            return json!({"kind": "over", "winner": null, "how": "closed", "view": null});
        }
        for ev in evs {
            match s(&ev, "kind") {
                "log" => {
                    // a line meant for another seat only is not for this terminal
                    if ev.get("seat").is_none_or(|x| x.as_str() == Some(&me_key))
                        && ev.get("hide").and_then(Value::as_str) != Some(&me_key)
                    {
                        let _ = writeln!(io.out, "{}", s(&ev, "text"));
                    }
                }
                "turn" => {
                    if s(&ev, "player") == me_key && ev.get("view").is_some_and(|v| !v.is_null()) {
                        show_table(&ev["view"], io.out);
                    }
                }
                "invalid" | "auto" => {
                    let _ = writeln!(
                        io.out,
                        "{}{}",
                        if s(&ev, "kind") == "invalid" { "  Not allowed: " } else { "  (automatic) " },
                        s(&ev, "text")
                    );
                }
                "request" => {
                    if s(&ev, "seat") == me_key {
                        answer_request(session, &me_key, &ev["request"], &mut io);
                    }
                }
                "over" => {
                    if !ev["view"].is_null() {
                        show_table(&ev["view"], io.out);
                    }
                    let w = ev["winner"].as_str().unwrap_or("no winner");
                    let _ = writeln!(io.out, "\nGame over: {} ({})", w, s(&ev, "how"));
                    session.close(Duration::from_secs(5));
                    return ev;
                }
                "error" => {
                    let _ = writeln!(io.out, "{}", s(&ev, "text"));
                    session.close(Duration::from_secs(5));
                    return ev;
                }
                _ => {
                    if ev.get("text").is_some() && ev.get("seat").is_none_or(|x| x.as_str() == Some(&me_key)) {
                        let _ = writeln!(io.out, "  {}", s(&ev, "text"));
                    }
                }
            }
        }
    }
}

fn answer_request(session: &Arc<Session>, seat: &str, req: &Value, io: &mut Io) {
    let ans = |v: Value| session.answer(seat, &v);
    match s(req, "kind") {
        "priority" => {
            priority_prompt(req, io.out);
            let view = &req["data"]["view"];
            for _ in 0..TRIES {
                let line = io.ask("> ").unwrap_or_else(|| "quit".into());
                match parse(&line, view) {
                    Cmd::Show => show_table(view, io.out),
                    Cmd::Help => {
                        let _ = writeln!(io.out, "{HELP}");
                    }
                    Cmd::Quit => {
                        session.close(Duration::from_secs(5));
                        return;
                    }
                    Cmd::Err(e) => {
                        let _ = writeln!(io.out, "  {e}");
                    }
                    Cmd::Act(a) => return ans(a),
                }
            }
            ans(json!({"do": "pass"}));
        }
        "attack" => {
            let len = list_choices(req, io);
            for _ in 0..TRIES {
                let line =
                    io.ask("attackers (e.g. 1,3 / all / none)> ").map_or("none".into(), |l| l.trim().to_lowercase());
                if matches!(line.as_str(), "none" | "n" | "") {
                    return ans(json!([]));
                }
                if line == "all" {
                    return ans(json!((0..len).collect::<Vec<_>>()));
                }
                let parts: Vec<&str> = line.split([' ', ',']).filter(|x| !x.is_empty()).collect();
                let nums: Vec<Option<i64>> = parts.iter().map(|x| digits(x)).collect();
                if !parts.is_empty() && nums.iter().all(|k| k.is_some_and(|k| 1 <= k && k as usize <= len)) {
                    return ans(json!(nums.iter().map(|k| k.unwrap() - 1).collect::<Vec<_>>()));
                }
                let _ = writeln!(io.out, "  Type numbers separated by commas, all, or none.");
            }
            ans(json!([]));
        }
        "mulligan" => {
            let _ = writeln!(io.out, "\n{}", s(req, "prompt"));
            for (i, h) in arr(&req["data"], "hand").iter().enumerate() {
                let _ = writeln!(io.out, "  {}) {}", i + 1, txt(h));
            }
            for _ in 0..TRIES {
                let line = io.ask("keep or mulligan? ").map_or("keep".into(), |l| l.trim().to_lowercase());
                if matches!(line.as_str(), "keep" | "k" | "") {
                    return ans(json!("keep"));
                }
                if matches!(line.as_str(), "mulligan" | "mull" | "m") {
                    return ans(json!("mulligan"));
                }
                let _ = writeln!(io.out, "  Type keep or mulligan.");
            }
            ans(json!("keep"));
        }
        "choose" | "target" | "block" | "search" => {
            let len = list_choices(req, io);
            for _ in 0..TRIES {
                let line = io.ask("> ").map_or("cancel".into(), |l| l.trim().to_lowercase());
                if matches!(line.as_str(), "cancel" | "c" | "x") {
                    return ans(json!("cancel"));
                }
                if let Some(k) = digits(&line)
                    && 1 <= k
                    && k as usize <= len
                {
                    return ans(json!(k - 1));
                }
                let _ = writeln!(io.out, "  Type a number from 1 to {len}, or cancel.");
            }
            ans(json!("cancel"));
        }
        "continue" => {
            let line = io.ask(&format!("[{}] Enter to continue, q to quit: ", s(req, "prompt")));
            match line {
                None => session.close(Duration::from_secs(5)),
                Some(l) if l.trim().eq_ignore_ascii_case("q") => session.close(Duration::from_secs(5)),
                _ => {}
            }
            ans(json!(true));
        }
        _ => {}
    }
}
