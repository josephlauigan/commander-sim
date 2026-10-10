//! The Rust half of the differential harness (rust/tools/difftest.py): build each position of a positions file and
//! print what the AI reads from it as JSON, for comparison with the Python engine.
//!     cargo run --release -p sim-core --example difftest <positions.json>
//!     DIFFTEST_LOG=1 ... also prints the game log of each position's building
//!
//! A position is a recipe both engines follow with their test helpers (tests/table.py, testkit):
//!     {"name": ..., "decks": ["sauron", ...], "seed": 1, "me": 0, "post": false,
//!      "seats": [{"life": 40, "turns": 3, "lands": [["Island", 2, false]], "hand": [...], "perms": [...],
//!                 "tokens": [3], "gy": [...], "treasures": 0}, ...]}

use serde_json::{Map, Value, json};
use sim_core::ai::{brain, decks, search};
use sim_core::engine::values::{epow, pval};
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::tag::Tag;
use sim_core::testkit::*;

fn names(v: &Value, k: &str) -> Vec<String> {
    v.get(k)
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|x| x.as_str().unwrap().to_string()).collect())
        .unwrap_or_default()
}

fn build(pos: &Value) -> Game {
    let keys = names(pos, "decks");
    let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
    let seed = pos.get("seed").and_then(Value::as_u64).unwrap_or(1);
    let mut g = table_with(&keys, seed, 40);
    if std::env::var_os("DIFFTEST_LOG").is_some() {
        g.log = Some(vec![]); // DIFFTEST_LOG=1: print what happened while building each position (stderr)
    }
    for (i, s) in pos["seats"].as_array().unwrap().iter().enumerate() {
        let p = PlayerId(i as u8);
        if s.get("library").is_some() {
            // the same order in both engines (the shuffles differ)
            let order = names(s, "library");
            let at = |c: &sim_core::ids::CardId| {
                let n = &g.db.get(*c).name;
                order.iter().position(|x| x.as_str() == &**n).unwrap_or(order.len())
            };
            let mut lib = g.player(p).library.clone();
            lib.sort_by_key(at);
            g.player_mut(p).library = lib;
        }
        if let Some(l) = s.get("life").and_then(Value::as_i64) {
            g.player_mut(p).life = l as i32;
        }
        if let Some(t) = s.get("turns").and_then(Value::as_u64) {
            g.player_mut(p).turns = t as u32;
        }
        if let Some(t) = s.get("treasures").and_then(Value::as_u64) {
            g.player_mut(p).treasures = t as u32;
        }
        for l in s.get("lands").and_then(Value::as_array).into_iter().flatten() {
            let l = l.as_array().unwrap();
            lands(
                &mut g,
                p,
                l[0].as_str().unwrap(),
                l[1].as_u64().unwrap() as usize,
                l.get(2).and_then(Value::as_bool).unwrap_or(false),
            );
        }
        for n in names(s, "gy") {
            let c = take(&mut g, p, &n);
            g.player_mut(p).gy.push(c);
        }
        let h = names(s, "hand");
        hand(&mut g, p, &h.iter().map(String::as_str).collect::<Vec<_>>());
        for n in names(s, "perms") {
            perm(&mut g, p, &n);
        }
        for t in s.get("tokens").and_then(Value::as_array).into_iter().flatten() {
            token(&mut g, p, t.as_i64().unwrap() as i32);
        }
    }
    g
}

fn name(g: &Game, c: sim_core::ids::CardId) -> String {
    g.db.get(c).name.to_string()
}

/// the position as built, comparable across engines
fn built(g: &Game) -> Value {
    let sorted = |mut v: Vec<String>| {
        v.sort();
        v
    };
    json!(
        g.players
            .iter()
            .map(|p| json!({
                "life": p.life,
                "hand": sorted(p.hand.iter().map(|&c| name(g, c)).collect()),
                "perms": sorted(p.perms.iter().map(|&m| g.perm(m).name.to_string()).collect()),
                "lands": sorted(p.lands.iter().map(|&l| name(g, g.land(l).cd)).collect()),
                "gy": sorted(p.gy.iter().map(|&c| name(g, c)).collect()),
                "library": p.library.len(),
        "ends": p.library.iter().take(3).chain(p.library.iter().skip(p.library.len().saturating_sub(3))).map(|&c| name(g, c)).collect::<Vec<_>>(),
                "treasures": p.treasures,
            }))
            .collect::<Vec<_>>()
    )
}

fn report(pos: &Value) -> Value {
    let mut g = build(pos);
    if let Some(log) = g.log.take() {
        eprintln!("== {}\n{}", pos["name"], log.join("\n"));
    }
    let me = PlayerId(pos.get("me").and_then(Value::as_u64).unwrap_or(0) as u8);
    let post = pos.get("post").and_then(Value::as_bool).unwrap_or(false);
    let seats: Vec<PlayerId> = (0..g.players.len()).map(|i| PlayerId(i as u8)).collect();
    let s = brain::Situation::new(&g, me);
    let (hold_c, hold_v) = brain::hold_value(&g, me, &s);
    let mut cands = g.player(me).hand.clone();
    if g.player(me).cmd_in_zone {
        cands.push(g.player(me).cmd);
    }
    let mut util = Map::new();
    let mut prio = Map::new();
    for &c in &cands {
        util.insert(name(&g, c), json!(brain::card_utility(&g, me, &s, c)));
        prio.insert(name(&g, c), json!(decks::deck_prio(&g, me, c)));
    }
    let threat: Map<String, Value> = s.threat.iter().map(|(q, v)| (q.0.to_string(), json!(v))).collect();
    let opts = match brain::main_options(&mut g, me, post) {
        Ok(o) => json!(o.iter().map(|o| json!([o.label, o.utility])).collect::<Vec<_>>()),
        Err(e) => json!(format!("error: {e:?}")),
    };
    json!({
        "name": pos["name"],
        "built": built(&g),
        "evaluate": seats.iter().map(|&q| search::evaluate(&g, q)).collect::<Vec<_>>(),
        "strength": seats.iter().map(|&q| search::strength(&g, q)).collect::<Vec<_>>(),
        "situation": {
            "mana": s.mana, "hand": s.hand, "lands_in_hand": s.lands_in_hand, "threat": threat,
            "leader": s.leader.map(|q| q.0), "max_threat": s.max_threat, "incoming": s.incoming,
            "danger": s.danger, "combo_near": s.combo_near, "ctr_risk": s.ctr_risk,
        },
        "removal_risk": brain::removal_risk(&g, me),
        "detail": seats.iter().map(|&q| json!({
            "board": g.player(q).perms.iter().map(|&m| json!([g.perm(m).name.to_string(), pval(&g, m), epow(&g, m)])).collect::<Vec<_>>(),
            "mana": brain::open_mana(&g, q, None),
            "open_u": brain::open_mana(&g, q, Some('U')),
            "p_counter": brain::prob_holding(&g, q, |c| g.db.get(c).tag(Tag::Ctr)),
        })).collect::<Vec<_>>(),
        "hold": [hold_c.map(|c| name(&g, c)), hold_v],
        "card_utility": util,
        "deck_prio": prio,
        "tutor_pick": decks::tutor_pick(&g, me, "any").map(|c| name(&g, c)),
        "main_options": opts,
    })
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: difftest <positions.json>");
    let positions: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let out: Vec<Value> = positions.iter().map(report).collect();
    println!("{}", serde_json::to_string(&out).unwrap());
}
